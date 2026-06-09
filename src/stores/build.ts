import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export type BuildStatus = 'idle' | 'building' | 'success' | 'failed' | 'cancelled'
export type BuildKind = 'package' | 'generateAndroidProject' | 'generateIosProject' | 'generateHarmonyProject'

export interface BuildLog {
  id: string
  timestamp: string
  level: 'info' | 'warn' | 'error' | 'success'
  message: string
}

export interface BuildTask {
  id: string
  projectId: string
  platform: 'android' | 'ios' | 'harmony'
  kind: BuildKind
  status: BuildStatus
  startTime: string | null
  endTime: string | null
  progress: number
  logs: BuildLog[]
  artifactPath: string | null
  artifactSizeBytes: number | null
  generatedProjectPath: string | null
  logPath: string | null
  lastError: string | null
}

interface BufferedLog {
  buildId: string
  level: BuildLog['level']
  message: string
  progress?: number
}

interface StopBuildResult {
  artifactPath?: string | null
  artifactSizeBytes?: number | null
  generatedProjectPath?: string | null
}

const STACK_FRAME_RE = /^\s*(at\s+|\.\.\. \d+ more)/
const D8_METADATA_RE = /WARNING:\s*D8:.*kotlin metadata/i
const D8_REWRITE_RE = /WARNING:\s*D8:\s*Unexpected error during rewriting of Kotlin metadata for class '([^']+)'/i
const D8_INTERNAL_RE = /^\s*(com\.android\.tools\.r8\.internal\.|at\s+com\.android\.tools\.r8\.|at\s+com\.android\.builder\.dexing\.|at\s+org\.gradle\.|at\s+java\.|at\s+com\.google\.common\.|at\s+org\.jetbrains\.)/
const KOTLIN_METADATA_MISMATCH_RE = /Module was compiled with an incompatible version of Kotlin\..*binary version of its metadata is .*expected version is/i
const SETTINGS_REPOSITORY_WARNING_RE = /Build was configured to prefer settings repositories over project repositories but repository '([^']+)' was added by build file 'build\.gradle'/i

export const useBuildStore = defineStore('build', () => {
  const builds = ref<Record<string, BuildTask>>({})
  const activeBuildIds = ref<string[]>([])
  const activeEventBuildId = ref<string | null>(null)

  const MAX_LOG_ENTRIES = 5000
  const LOG_FLUSH_INTERVAL_MS = 2000
  const PROGRESS_TICK_INTERVAL_MS = 350
  const MAX_PROGRESS_WHILE_BUILDING = 98

  let unlistenBuildLog: UnlistenFn | null = null
  let listenerPromise: Promise<void> | null = null
  const logBuffers = new Map<string, BufferedLog[]>()
  const logFlushTimers = new Map<string, ReturnType<typeof setTimeout>>()
  const emittedLogLines = new Map<string, Set<string>>()
  const progressTargets = new Map<string, number>()
  const progressTimers = new Map<string, ReturnType<typeof setInterval>>()
  const progressStartedAt = new Map<string, number>()
  const progressLastActivityAt = new Map<string, number>()
  const progressLongStageStartedAt = new Map<string, number>()

  const currentBuilds = computed(() => {
    return activeBuildIds.value.map(id => builds.value[id]).filter(Boolean)
  })

  const hasActiveBuilds = computed(() => {
    return !!activeEventBuildId.value || Object.values(builds.value).some(build => build.status === 'building')
  })

  function generateBuildId(): string {
    return `build_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  function generateLogId(): string {
    return `log_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  function startBuild(
    projectId: string,
    platform: BuildTask['platform'],
    kind: BuildKind = 'package'
  ): string {
    const buildId = generateBuildId()
    const buildTask: BuildTask = {
      id: buildId,
      projectId,
      platform,
      kind,
      status: 'building',
      startTime: new Date().toISOString(),
      endTime: null,
      progress: 1,
      logs: [],
      artifactPath: null,
      artifactSizeBytes: null,
      generatedProjectPath: null,
      logPath: null,
      lastError: null
    }

    builds.value[buildId] = buildTask
    if (!activeBuildIds.value.includes(buildId)) activeBuildIds.value.push(buildId)
    emittedLogLines.set(buildId, new Set())
    logBuffers.set(buildId, [])
    initProgressRuntime(buildId)

    return buildId
  }

  function stopBuild(buildId: string, success: boolean = false, result: StopBuildResult = {}) {
    const build = builds.value[buildId]
    if (!build) return

    build.status = success ? 'success' : 'cancelled'
    build.endTime = new Date().toISOString()
    clearProgressRuntime(buildId)
    build.progress = success ? 100 : build.progress
    if (result.artifactPath != null) build.artifactPath = result.artifactPath
    if (result.artifactSizeBytes != null) build.artifactSizeBytes = result.artifactSizeBytes
    if (result.generatedProjectPath != null) build.generatedProjectPath = result.generatedProjectPath
    removeActiveBuildId(buildId)
  }

  function failBuild(buildId: string, error: string) {
    const build = builds.value[buildId]
    if (!build) return

    build.status = 'failed'
    build.endTime = new Date().toISOString()
    build.lastError = error
    clearProgressRuntime(buildId)
    removeActiveBuildId(buildId)
  }

  function addLog(buildId: string, level: BuildLog['level'], message: string) {
    const build = builds.value[buildId]
    if (!build) return

    const log: BuildLog = {
      id: generateLogId(),
      timestamp: new Date().toISOString(),
      level,
      message
    }

    build.logs.push(log)
    touchProgressActivity(buildId)

    if (build.logs.length > MAX_LOG_ENTRIES) {
      build.logs = build.logs.slice(-MAX_LOG_ENTRIES)
    }
  }

  function updateProgress(buildId: string, progress: number) {
    const build = builds.value[buildId]
    if (!build) return

    const nextProgress = normalizeProgress(progress)
    if (build.status !== 'building') {
      build.progress = nextProgress
      progressTargets.set(buildId, nextProgress)
      return
    }

    touchProgressActivity(buildId)
    const nextTarget = Math.min(nextProgress, MAX_PROGRESS_WHILE_BUILDING)
    const currentTarget = progressTargets.get(buildId) ?? build.progress
    progressTargets.set(buildId, Math.max(currentTarget, nextTarget, build.progress))

    const longStageThreshold = getLongStageThreshold(build)
    if (
      build.kind === 'package'
      && nextTarget >= longStageThreshold
      && !progressLongStageStartedAt.has(buildId)
    ) {
      progressLongStageStartedAt.set(buildId, Date.now())
    }

    ensureProgressTimer(buildId)
  }

  function getBuild(buildId: string): BuildTask | undefined {
    return builds.value[buildId]
  }

  function getProjectBuilds(projectId: string): BuildTask[] {
    return Object.values(builds.value).filter(b => b.projectId === projectId)
  }

  function getActiveBuildForProject(projectId: string): BuildTask | undefined {
    return Object.values(builds.value).find(build => build.projectId === projectId && build.status === 'building')
  }

  function clearBuild(buildId: string) {
    delete builds.value[buildId]
    removeActiveBuildId(buildId)
    clearBuildRuntime(buildId)
  }

  function clearAllBuilds() {
    for (const buildId of Object.keys(builds.value)) {
      clearBuildRuntime(buildId)
    }
    builds.value = {}
    activeBuildIds.value = []
    activeEventBuildId.value = null
  }

  async function setupGlobalListener() {
    if (unlistenBuildLog) return
    if (listenerPromise) return listenerPromise

    listenerPromise = listen<any>('build-log', (event) => {
      const normalized = normalizeBuildLogPayload(event.payload)
      if (!normalized) return

      const buildId = resolveEventBuildId(normalized.buildId)
      if (!buildId) return

      const entry: BufferedLog = {
        buildId,
        level: normalized.level,
        message: normalized.message,
        progress: normalized.progress
      }

      const buffer = logBuffers.get(buildId) || []
      buffer.push(entry)
      logBuffers.set(buildId, buffer)
      touchProgressActivity(buildId)
      if (normalized.progress != null) updateProgress(buildId, normalized.progress)
      scheduleLogFlush(buildId)
    }).then((unlisten) => {
      unlistenBuildLog = unlisten
      listenerPromise = null
    }).catch((error) => {
      listenerPromise = null
      console.warn('Failed to register build-log listener:', error)
    })

    return listenerPromise
  }

  function teardownGlobalListener() {
    unlistenBuildLog?.()
    unlistenBuildLog = null
    listenerPromise = null
    for (const timer of logFlushTimers.values()) clearTimeout(timer)
    logFlushTimers.clear()
    for (const timer of progressTimers.values()) clearInterval(timer)
    progressTimers.clear()
    progressTargets.clear()
    progressStartedAt.clear()
    progressLastActivityAt.clear()
    progressLongStageStartedAt.clear()
  }

  function setActiveEventBuildId(buildId: string | null) {
    activeEventBuildId.value = buildId
  }

  async function appendBuildLogLines(buildId: string, lines: string[]) {
    const build = builds.value[buildId]
    if (!build || !lines.length) return null

    for (const line of lines) {
      const parsed = parseLogLine(line)
      addLog(buildId, parsed.level, parsed.message)
      rememberEmittedLine(buildId, line)
    }

    return writeBuildLogLines(buildId, lines)
  }

  async function flushBuildLogs(buildId?: string) {
    if (buildId) {
      await flushOneBuildLogs(buildId)
      return
    }

    const ids = Array.from(logBuffers.keys())
    for (const id of ids) {
      await flushOneBuildLogs(id)
    }
  }

  function normalizeBuildLogPayload(payload: any): {
    buildId?: string
    level: BuildLog['level']
    message: string
    progress?: number
  } | null {
    if (typeof payload === 'string') {
      return { level: 'info', message: payload }
    }

    if (!payload || typeof payload !== 'object') return null

    const line = payload.message || payload.line
    if (!line) return null

    const level: BuildLog['level'] = payload.level === 'error' || payload.type === 'stderr'
      ? 'error'
      : payload.level === 'warn'
        ? 'warn'
        : payload.level === 'success'
          ? 'success'
          : 'info'

    const buildId = payload.buildId || payload.build_id
    const progress = typeof payload.progress === 'number' ? payload.progress : undefined

    return {
      buildId: typeof buildId === 'string' && buildId.trim() ? buildId : undefined,
      level,
      message: String(line),
      progress
    }
  }

  function resolveEventBuildId(payloadBuildId?: string): string | null {
    if (payloadBuildId && builds.value[payloadBuildId]) return payloadBuildId

    if (activeEventBuildId.value && builds.value[activeEventBuildId.value]?.status === 'building') {
      return activeEventBuildId.value
    }

    const active = Object.values(builds.value).filter(build => build.status === 'building')
    return active.length === 1 ? active[0].id : null
  }

  function scheduleLogFlush(buildId: string) {
    if (logFlushTimers.has(buildId)) return
    const timer = setTimeout(() => {
      logFlushTimers.delete(buildId)
      void flushOneBuildLogs(buildId)
    }, LOG_FLUSH_INTERVAL_MS)
    logFlushTimers.set(buildId, timer)
  }

  async function flushOneBuildLogs(buildId: string) {
    const timer = logFlushTimers.get(buildId)
    if (timer) {
      clearTimeout(timer)
      logFlushTimers.delete(buildId)
    }

    const buffer = logBuffers.get(buildId)
    if (!buffer?.length) return

    const entries = buffer.splice(0)
    const compactedEntries = compactBufferedLogs(buildId, entries)

    for (const entry of entries) {
      if (entry.progress != null) updateProgress(entry.buildId, entry.progress)
    }

    const build = builds.value[buildId]
    let lastMsg = build?.logs?.length ? build.logs[build.logs.length - 1] : null

    for (const entry of compactedEntries) {
      if (lastMsg && lastMsg.level === entry.level && lastMsg.message === entry.message) {
        continue
      }
      addLog(entry.buildId, entry.level, entry.message)
      lastMsg = { level: entry.level, message: entry.message } as BuildLog
    }

    const rawLines: string[] = []
    for (let i = 0; i < compactedEntries.length; i++) {
      const entry = compactedEntries[i]
      if (i > 0) {
        const prev = compactedEntries[i - 1]
        if (prev.level === entry.level && prev.message === entry.message) continue
      }
      rawLines.push(`[${entry.level}] ${entry.message}`)
    }

    const uniqueLines = rawLines.filter(line => rememberEmittedLine(buildId, line))
    if (uniqueLines.length) {
      await writeBuildLogLines(buildId, uniqueLines)
    }
  }

  async function writeBuildLogLines(buildId: string, lines: string[]) {
    const build = builds.value[buildId]
    if (!build || !lines.length) return null

    try {
      const logPath = await invoke<string>('append_build_log', {
        projectId: build.projectId,
        buildId,
        lines
      })

      if (logPath && !build.logPath) {
        build.logPath = logPath
        await invoke('update_build_record', {
          id: buildId,
          update: { log_path: logPath }
        }).catch((error) => {
          console.warn('Failed to update build log path:', error)
        })
      }

      return logPath
    } catch (error) {
      console.warn('Failed to append build log:', error)
      return null
    }
  }

  function rememberEmittedLine(buildId: string, line: string) {
    const lines = emittedLogLines.get(buildId) || new Set<string>()
    emittedLogLines.set(buildId, lines)
    if (lines.has(line)) return false
    lines.add(line)
    return true
  }

  function parseLogLine(line: string): { level: BuildLog['level']; message: string } {
    const match = line.match(/^\[(info|warn|error|success)\]\s*(.*)$/i)
    if (!match) return { level: 'info', message: line }
    return {
      level: match[1].toLowerCase() as BuildLog['level'],
      message: match[2]
    }
  }

  function compactBufferedLogs(buildId: string, entries: BufferedLog[]): BufferedLog[] {
    const result: BufferedLog[] = []
    const stackBuffer: BufferedLog[] = []
    const d8Classes = new Set<string>()
    const kotlinModules = new Set<string>()
    const settingsRepositoryWarnings = new Set<string>()
    let d8WarningCount = 0
    let d8SuppressedLines = 0
    let kotlinMismatchCount = 0
    let settingsRepositoryWarningCount = 0
    let suppressingD8Details = false

    function pushEntry(entry: BufferedLog) {
      flushStack()
      flushD8Summary()
      flushKotlinMismatchSummary()
      flushSettingsRepositorySummary()
      result.push(entry)
    }

    function flushStack() {
      if (!stackBuffer.length) return
      if (stackBuffer.length > 3) {
        result.push({
          ...stackBuffer[0],
          level: stackBuffer.some(entry => entry.level === 'error') ? 'warn' : stackBuffer[0].level,
          message: `已折叠 ${stackBuffer.length} 行 Gradle/Java 内部堆栈`,
        })
      } else {
        result.push(...stackBuffer)
      }
      stackBuffer.length = 0
    }

    function flushD8Summary() {
      if (!d8WarningCount && d8Classes.size === 0 && !d8SuppressedLines) return
      const shownClasses = Array.from(d8Classes).slice(0, 5)
      const classText = shownClasses.length
        ? `，示例: ${shownClasses.join('、')}${d8Classes.size > shownClasses.length ? ` 等 ${d8Classes.size} 个类` : ''}`
        : ''
      result.push({
        buildId,
        level: 'warn',
        message: `D8 Kotlin metadata 警告已折叠${classText}；省略 ${d8SuppressedLines} 行 R8/D8 内部堆栈。建议对齐 Kotlin 与 AGP/R8 版本。`,
      })
      d8Classes.clear()
      d8WarningCount = 0
      d8SuppressedLines = 0
      suppressingD8Details = false
    }

    function flushKotlinMismatchSummary() {
      if (!kotlinMismatchCount) return
      const modules = Array.from(kotlinModules)
      const shownModules = modules.slice(0, 5)
      result.push({
        buildId,
        level: 'warn',
        message: `Kotlin 元数据版本不兼容提示已折叠: ${kotlinMismatchCount} 条，涉及 ${shownModules.join('、')}${modules.length > shownModules.length ? ` 等 ${modules.length} 个依赖` : ''}。`,
      })
      kotlinModules.clear()
      kotlinMismatchCount = 0
    }

    function flushSettingsRepositorySummary() {
      if (!settingsRepositoryWarningCount) return
      const repositories = Array.from(settingsRepositoryWarnings)
      const shown = repositories.slice(0, 6).join('、')
      result.push({
        buildId,
        level: 'info',
        message: `Gradle 项目级仓库提示已折叠: ${settingsRepositoryWarningCount} 条；settings.gradle 已接管依赖仓库${shown ? `，被忽略仓库: ${shown}` : ''}。`,
      })
      settingsRepositoryWarnings.clear()
      settingsRepositoryWarningCount = 0
    }

    for (const entry of entries) {
      const message = entry.message.trimEnd()
      const d8Class = message.match(D8_REWRITE_RE)?.[1]
      if (D8_METADATA_RE.test(message) || d8Class) {
        flushStack()
        flushKotlinMismatchSummary()
        d8WarningCount += 1
        if (d8Class) d8Classes.add(d8Class)
        suppressingD8Details = true
        continue
      }

      if (suppressingD8Details && (D8_INTERNAL_RE.test(message) || STACK_FRAME_RE.test(message))) {
        d8SuppressedLines += 1
        continue
      }

      if (KOTLIN_METADATA_MISMATCH_RE.test(message)) {
        flushStack()
        flushD8Summary()
        kotlinMismatchCount += 1
        kotlinModules.add(extractKotlinMismatchModule(message))
        continue
      }

      const settingsRepository = message.match(SETTINGS_REPOSITORY_WARNING_RE)?.[1]
      if (settingsRepository) {
        flushStack()
        flushD8Summary()
        flushKotlinMismatchSummary()
        settingsRepositoryWarnings.add(settingsRepository)
        settingsRepositoryWarningCount += 1
        continue
      }

      if (STACK_FRAME_RE.test(message)) {
        flushD8Summary()
        flushKotlinMismatchSummary()
        flushSettingsRepositorySummary()
        stackBuffer.push({ ...entry, message })
        continue
      }

      pushEntry({ ...entry, message })
    }

    flushStack()
    flushD8Summary()
    flushKotlinMismatchSummary()
    flushSettingsRepositorySummary()
    return result
  }

  function extractKotlinMismatchModule(message: string): string {
    const kotlinModule = message.match(/org\.jetbrains\.kotlin\/([^/]+)\/([^/]+)/)
    if (kotlinModule) return `${kotlinModule[1]}:${kotlinModule[2]}`
    if (message.includes('utsplugin_release.kotlin_module')) return 'utsplugin-release'
    const jar = message.match(/\/([^/!]+\.jar)!\/META-INF/)
    if (jar) return jar[1]
    const metadata = message.match(/META-INF\/([^/\s]+\.kotlin_module)/)
    return metadata?.[1] || 'unknown'
  }

  function initProgressRuntime(buildId: string) {
    const now = Date.now()
    progressTargets.set(buildId, 2)
    progressStartedAt.set(buildId, now)
    progressLastActivityAt.set(buildId, now)
    ensureProgressTimer(buildId)
  }

  function normalizeProgress(progress: number) {
    return Math.min(100, Math.max(0, Number.isFinite(progress) ? progress : 0))
  }

  function roundProgress(progress: number) {
    return Number(progress.toFixed(1))
  }

  function touchProgressActivity(buildId: string) {
    const build = builds.value[buildId]
    if (build?.status === 'building') {
      progressLastActivityAt.set(buildId, Date.now())
    }
  }

  function ensureProgressTimer(buildId: string) {
    if (progressTimers.has(buildId)) return

    const timer = setInterval(() => {
      tickProgress(buildId)
    }, PROGRESS_TICK_INTERVAL_MS)
    progressTimers.set(buildId, timer)
    tickProgress(buildId)
  }

  function tickProgress(buildId: string) {
    const build = builds.value[buildId]
    if (!build || build.status !== 'building') {
      clearProgressRuntime(buildId)
      return
    }

    const target = getVisibleProgressTarget(buildId, build)
    if (build.progress >= target) return

    const gap = target - build.progress
    const delta = Math.min(4.5, Math.max(0.25, gap * 0.16))
    build.progress = roundProgress(Math.min(target, build.progress + delta))
  }

  function getVisibleProgressTarget(buildId: string, build: BuildTask) {
    const explicitTarget = progressTargets.get(buildId) ?? build.progress
    const longStageTarget = getLongStageProgressTarget(buildId, build, explicitTarget)
    return Math.min(MAX_PROGRESS_WHILE_BUILDING, Math.max(explicitTarget, longStageTarget))
  }

  function getLongStageProgressTarget(buildId: string, build: BuildTask, explicitTarget: number) {
    const threshold = getLongStageThreshold(build)
    if (build.kind !== 'package' || explicitTarget < threshold) return explicitTarget

    const now = Date.now()
    const stageStartedAt = progressLongStageStartedAt.get(buildId) ?? progressStartedAt.get(buildId) ?? now
    const lastActivityAt = progressLastActivityAt.get(buildId) ?? stageStartedAt
    const hasRecentActivity = now - lastActivityAt < 10000
    const phaseElapsed = Math.max(0, now - stageStartedAt)
    const expectedDuration = getLongStageExpectedDuration(build) * (hasRecentActivity ? 1 : 1.8)
    const ratio = Math.min(1, phaseElapsed / expectedDuration)
    const easedRatio = 1 - Math.pow(1 - ratio, 2.2)
    const inferred = threshold + (MAX_PROGRESS_WHILE_BUILDING - threshold) * easedRatio

    return Math.max(explicitTarget, inferred)
  }

  function getLongStageThreshold(build: BuildTask) {
    if (build.platform === 'ios') return 55
    if (build.platform === 'harmony') return 65
    return 70
  }

  function getLongStageExpectedDuration(build: BuildTask) {
    if (build.platform === 'ios') return 240000
    if (build.platform === 'harmony') return 180000
    return 180000
  }

  function clearProgressRuntime(buildId: string) {
    const timer = progressTimers.get(buildId)
    if (timer) clearInterval(timer)
    progressTimers.delete(buildId)
    progressTargets.delete(buildId)
    progressStartedAt.delete(buildId)
    progressLastActivityAt.delete(buildId)
    progressLongStageStartedAt.delete(buildId)
  }

  function removeActiveBuildId(buildId: string) {
    const index = activeBuildIds.value.indexOf(buildId)
    if (index > -1) activeBuildIds.value.splice(index, 1)
  }

  function clearBuildRuntime(buildId: string) {
    const timer = logFlushTimers.get(buildId)
    if (timer) clearTimeout(timer)
    logFlushTimers.delete(buildId)
    logBuffers.delete(buildId)
    emittedLogLines.delete(buildId)
    clearProgressRuntime(buildId)
    if (activeEventBuildId.value === buildId) activeEventBuildId.value = null
  }

  return {
    builds,
    activeBuildIds,
    activeEventBuildId,
    currentBuilds,
    hasActiveBuilds,
    startBuild,
    stopBuild,
    failBuild,
    addLog,
    updateProgress,
    getBuild,
    getProjectBuilds,
    getActiveBuildForProject,
    clearBuild,
    clearAllBuilds,
    setupGlobalListener,
    teardownGlobalListener,
    setActiveEventBuildId,
    appendBuildLogLines,
    flushBuildLogs
  }
})
