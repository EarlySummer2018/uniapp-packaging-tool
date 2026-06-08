<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NAlert,
  NButton,
  NCard,
  NDivider,
  NFormItem,
  NGi,
  NGrid,
  NIcon,
  NInput,
  NProgress,
  NSpace,
  NTag,
  NText,
  useMessage
} from 'naive-ui'
import { ArrowBackOutline, FolderOpenOutline, PlayOutline } from '@vicons/ionicons5'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { LogoAndroid, LogoApple, PhonePortraitOutline } from '@vicons/ionicons5'
import PlatformCard from '../components/PlatformCard.vue'
import LogPanel from '../components/LogPanel.vue'
import { useBuildStore, type BuildLog } from '../stores/build'
import { useProjectsStore } from '../stores/projects'

type Platform = 'android' | 'ios' | 'harmony'

interface UtsBuiltinModule {
  name: string
  localAar: string
  onlineDeps: string[]
  dependsOn: string[]
}

interface UtsCustomPlugin {
  id: string
  androidDir?: string | null
  iosDir?: string | null
  androidDeps: string[]
  iosFrameworks: string[]
}

interface DetectedModule {
  name: string
  category: string
  platforms: string[]
  configured: boolean
  requiredKeys: string[]
  source: string
}

interface AndroidManifestConfig {
  packageName?: string | null
  minSdkVersion?: number | null
  targetSdkVersion?: number | null
  compileSdkVersion?: number | null
}

interface PlatformPackages {
  androidPackage?: string | null
  iosBundleId?: string | null
  harmonyBundle?: string | null
}

interface SplashscreenConfig {
  androidStyle?: string | null
  android: Record<string, string>
  useOriginalMsgbox?: boolean | null
}

interface UniappManifestInfo {
  appName?: string | null
  appId?: string | null
  versionName?: string | null
  versionCode?: number | null
  hbuilderxVersion?: string | null
  androidIcons?: { android: Record<string, string> } | null
  iosIcons?: { ios: Record<string, string> } | null
  splashscreen?: SplashscreenConfig | null
  manifestValue?: Record<string, any> | null
  manifestPath: string
  projectRoot: string
  android: AndroidManifestConfig
  packageNames: PlatformPackages
  detectedModules: DetectedModule[]
  warnings: string[]
}

interface AndroidModuleConfigField {
  key: string
  label: string
  required: boolean
  secret: boolean
  value?: string | null
  valueSource?: string | null
  placeholder: string
  field_type?: string
}

interface AndroidModuleConfigModule {
  name: string
  templateKey: string
  category: string
  platforms: string[]
  source: string
  fields: AndroidModuleConfigField[]
}

interface AndroidModuleMissingConfig {
  moduleName: string
  key: string
  label: string
}

interface AndroidModuleConfigReport {
  modules: AndroidModuleConfigModule[]
  missingRequired: AndroidModuleMissingConfig[]
  allConfigured: boolean
}

interface ResourceScanResult {
  appId: string
  appName?: string | null
  versionName?: string | null
  versionCode?: number | null
  hbuilderxVersion?: string | null
  sourcePath: string
  importedPath: string
  appResourcePath: string
  isZip: boolean
  manifestPath?: string | null
  splashscreen?: SplashscreenConfig | null
  detectedModules: DetectedModule[]
  uts: {
    hasUtsPlugins: boolean
    builtinModules: UtsBuiltinModule[]
    customPlugins: UtsCustomPlugin[]
  }
  warnings: string[]
}

interface BuildArtifact {
  platform: Platform
  path: string
  fileName: string
  sizeBytes: number
  buildId: string
}

interface BuildRecord {
  id: string
  project_id: string
  project_name: string
  platform: Platform
  status: string
  artifact_path?: string | null
  artifact_size_mb?: number | null
  version_name: string
  version_code: number
  build_mode: string
  duration_secs: number
  started_at: string
  finished_at?: string | null
  error_message?: string | null
  log_path?: string | null,
  resource_path?: string | null
}

const route = useRoute()
const router = useRouter()
const message = useMessage()
const buildStore = useBuildStore()
const projectsStore = useProjectsStore()

const projectId = computed(() => route.params.id as string)
const selectedPlatforms = ref<Platform[]>([])
const importing = ref(false)
const isBuilding = ref(false)
const isGenerating = ref(false)
const generatedProjectPath = ref<string | null>(null)
const currentBuildId = ref<string | null>(null)
const scanResult = ref<ResourceScanResult | null>(null)
const latestManifestInfo = ref<UniappManifestInfo | null>(null)
const manifestReadWarning = ref('')
const androidModuleConfigReport = ref<AndroidModuleConfigReport | null>(null)
const androidModuleConfigValues = ref<Record<string, string>>({})
const androidModuleConfigLoading = ref(false)
const artifacts = ref<BuildArtifact[]>([])

let unlistenBuildLog: UnlistenFn | null = null
let androidModuleConfigSaveTimer: ReturnType<typeof setTimeout> | null = null

// ===== 日志节流调度系统（2 秒刷新，最大限度让出主线程给打包编译）=====

interface BufferedLog {
  buildId: string
  level: 'info' | 'warn' | 'error' | 'success'
  message: string
  progress?: number
}

/** 原始事件缓冲区：回调只做轻量 push，不做任何响应式/IPC 操作 */
const logBuffer: BufferedLog[] = []

/** 全局去重集合：记录本构建已写入文件的日志行，防止 Gradle --stacktrace 等重复输出 */
const emittedLogLines = new Set<string>()

/** UI 刷新定时器句柄 */
let logFlushTimer: ReturnType<typeof setTimeout> | null = null

/** 是否已有定时器在等待 */
let flushPending = false

/** UI 刷新间隔（毫秒） */
const LOG_FLUSH_INTERVAL_MS = 2000

/**
 * 将缓冲区日志批量写入 store（触发 UI 更新）和文件。
 * 由 2 秒定时器驱动，也可在 catch/finally 中主动调用以排空缓冲区。
 */
async function flushToStoreAndUI() {
  flushPending = false
  logFlushTimer = null

  if (logBuffer.length === 0 || !currentBuildId.value) return

  // 取出所有缓冲数据
  const entries = logBuffer.splice(0)
  const compactedEntries = compactBufferedLogs(entries)

  // 进度事件不一定都有可展示日志，先从原始事件里同步进度
  for (const entry of entries) {
    if (entry.progress != null) {
      buildStore.updateProgress(entry.buildId, entry.progress)
    }
  }

  // 获取当前构建的最后一条日志，用于连续去重
  const build = buildStore.getBuild(currentBuildId.value)
  let lastMsg = build?.logs?.length ? build.logs[build.logs.length - 1] : null

  // 批量推入 store（连续去重：跳过与前一条 level+message 完全相同的日志）
  for (const entry of compactedEntries) {
    if (lastMsg && lastMsg.level === entry.level && lastMsg.message === entry.message) {
      continue // Gradle 重复输出常见，跳过连续重复
    }
    buildStore.addLog(entry.buildId, entry.level, entry.message)
    lastMsg = { level: entry.level, message: entry.message } as BuildLog
  }

  // 文件写入：写入与实时 UI 一致的精简日志
  const rawLines: string[] = []
  for (let i = 0; i < compactedEntries.length; i++) {
    const e = compactedEntries[i]
    if (i > 0) {
      const prev = compactedEntries[i - 1]
      if (prev.level === e.level && prev.message === e.message) continue
    }
    rawLines.push(`[${e.level}] ${e.message}`)
  }

  // 全局去重：跳过本次构建中已写入的行
  const uniqueLines: string[] = []
  for (const line of rawLines) {
    if (!emittedLogLines.has(line)) {
      emittedLogLines.add(line)
      uniqueLines.push(line)
    }
  }

  if (uniqueLines.length > 0) {
    void appendLog(currentBuildId.value!, uniqueLines).catch(() => {})
  }
}

const STACK_FRAME_RE = /^\s*(at\s+|\.\.\. \d+ more)/
const D8_METADATA_RE = /WARNING:\s*D8:.*kotlin metadata/i
const D8_REWRITE_RE = /WARNING:\s*D8:\s*Unexpected error during rewriting of Kotlin metadata for class '([^']+)'/i
const D8_INTERNAL_RE = /^\s*(com\.android\.tools\.r8\.internal\.|at\s+com\.android\.tools\.r8\.|at\s+com\.android\.builder\.dexing\.|at\s+org\.gradle\.|at\s+java\.|at\s+com\.google\.common\.|at\s+org\.jetbrains\.)/
const KOTLIN_METADATA_MISMATCH_RE = /Module was compiled with an incompatible version of Kotlin\..*binary version of its metadata is .*expected version is/i
const SETTINGS_REPOSITORY_WARNING_RE = /Build was configured to prefer settings repositories over project repositories but repository '([^']+)' was added by build file 'build\.gradle'/i

function compactBufferedLogs(entries: BufferedLog[]): BufferedLog[] {
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
      buildId: currentBuildId.value!,
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
      buildId: currentBuildId.value!,
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
      buildId: currentBuildId.value!,
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

/** 调度下一次 UI 刷新（2 秒后，不重复调度） */
function scheduleLogFlush(): Promise<void> {
  if (flushPending) return Promise.resolve()
  flushPending = true
  return new Promise(resolve => {
    logFlushTimer = setTimeout(async () => {
      await flushToStoreAndUI()
      resolve()
    }, LOG_FLUSH_INTERVAL_MS)
  })
}

const platforms = [
  { key: 'android' as const, label: 'Android', icon: LogoAndroid, description: '生成 APK 安装包', color: '#2f9e44', bgColor: '#e8f5e9' },
  { key: 'ios' as const, label: 'iOS', icon: LogoApple, description: '生成 IPA 安装包', color: '#1c7ed6', bgColor: '#e7f5ff' },
  { key: 'harmony' as const, label: '鸿蒙', icon: PhonePortraitOutline, description: '生成 HAP 安装包', color: '#d6336c', bgColor: '#fff0f6' }
]

const selectedNeedsAndroidConfig = computed(() => selectedPlatforms.value.includes('android'))
const androidMissingRequired = computed(() => {
  const report = androidModuleConfigReport.value
  if (!report) return []
  return report.modules.flatMap(mod => mod.fields
    .filter(field => field.required && !androidFieldValue(field).trim())
    .map(field => ({ moduleName: mod.name, key: field.key, label: field.label })))
})
const canBuild = computed(() => {
  if (!scanResult.value || selectedPlatforms.value.length === 0 || isBuilding.value) return false
  if (selectedNeedsAndroidConfig.value && !androidModulesReady.value) return false
  return true
})
const canGenerateAndroid = computed(() => {
  if (!scanResult.value || !selectedPlatforms.value.includes('android') || isBuilding.value || isGenerating.value) return false
  if (selectedNeedsAndroidConfig.value && !androidModulesReady.value) return false
  return true
})
const androidModulesReady = computed(() => {
  if (!selectedNeedsAndroidConfig.value) return true
  if (!latestManifestInfo.value) return !!currentProject.value?.localPath
  if (androidModuleConfigLoading.value) return false
  const report = androidModuleConfigReport.value
  if (!report) return true
  return androidMissingRequired.value.length === 0
})
const buildDisabledReason = computed(() => {
  if (!scanResult.value) return '请先导入 UniApp 资源'
  if (!selectedPlatforms.value.length) return '请选择至少一个平台'
  if (isBuilding.value) return '正在构建中'
  if (selectedNeedsAndroidConfig.value) {
    if (!latestManifestInfo.value && !currentProject.value?.localPath) {
      return manifestReadWarning.value || '请先在项目配置中选择包含 manifest.json 的本地项目路径'
    }
    if (!latestManifestInfo.value) return ''
    if (androidModuleConfigLoading.value) return '正在分析 Android 模块配置'
    if (androidMissingRequired.value.length) return `还有 ${androidMissingRequired.value.length} 个 Android 必填配置未填写`
  }
  return ''
})
const currentBuild = computed(() => currentBuildId.value ? buildStore.getBuild(currentBuildId.value) : null)
const currentProject = computed(() => projectsStore.projects.find(p => p.id === projectId.value) || null)
const utsDependencyCount = computed(() => {
  const result = scanResult.value
  if (!result) return 0
  const deps = new Set<string>()
  for (const mod of result.uts.builtinModules) {
    for (const dep of mod.onlineDeps) deps.add(dep)
  }
  for (const plugin of result.uts.customPlugins) {
    for (const dep of plugin.androidDeps) deps.add(dep)
  }
  return deps.size
})
const manifestModules = computed(() => latestManifestInfo.value?.detectedModules || scanResult.value?.detectedModules || [])
const manifestModuleLabels = computed(() => manifestModules.value.map(mod => `${mod.name} · ${formatPlatforms(mod.platforms)}`))
const insightAppName = computed(() => latestManifestInfo.value?.appName || scanResult.value?.appName || currentProject.value?.app.name || scanResult.value?.appId || '-')
const insightAppId = computed(() => latestManifestInfo.value?.appId || scanResult.value?.appId || currentProject.value?.app.appId || '-')
const insightVersionName = computed(() => latestManifestInfo.value?.versionName || scanResult.value?.versionName || currentProject.value?.app.version || '-')
const insightVersionCode = computed(() => latestManifestInfo.value?.versionCode ?? scanResult.value?.versionCode ?? currentProject.value?.app.versionCode ?? '-')
const insightManifestPath = computed(() => latestManifestInfo.value?.manifestPath || scanResult.value?.manifestPath || '-')
const utsPluginLabels = computed(() => {
  const result = scanResult.value
  if (!result) return []
  return [
    ...result.uts.builtinModules.map(mod => mod.name),
    ...result.uts.customPlugins.map(plugin => plugin.id)
  ]
})
const androidModuleConfigSummary = computed(() => {
  const report = androidModuleConfigReport.value
  if (!report) return []
  return report.modules.map(mod => `${mod.name} · ${mod.fields.length ? `${mod.fields.length} 项配置` : '无需配置'}`)
})
const androidConfiguredModuleNames = computed(() => androidModuleConfigReport.value?.modules.map(mod => mod.name) || [])

onMounted(async () => {
  if (!projectsStore.projects.length) await projectsStore.loadProjects()
  projectsStore.setCurrentProject(projectId.value)
  unlistenBuildLog = await listen<any>('build-log', (event) => {
    if (!currentBuildId.value) return
    const payload = event.payload

    // 只做轻量解析 + 推入缓冲区，零响应式、零 IPC
    if (typeof payload === 'string') {
      logBuffer.push({ buildId: currentBuildId.value, level: 'info', message: payload })
    } else {
      const line = payload?.message || payload?.line
      if (line) {
        const level = payload.level === 'error' || payload.type === 'stderr'
          ? 'error'
          : payload.level === 'warn'
            ? 'warn'
            : payload.level === 'success'
              ? 'success'
              : 'info'
        logBuffer.push({
          buildId: currentBuildId.value,
          level,
          message: String(line),
          progress: typeof payload?.progress === 'number' ? payload.progress : undefined,
        })
      }
    }

    // 调度 2 秒后的 UI 刷新
    scheduleLogFlush()
  })
})

onUnmounted(() => {
  if (androidModuleConfigSaveTimer) {
    clearTimeout(androidModuleConfigSaveTimer)
    androidModuleConfigSaveTimer = null
  }
  void persistAndroidModuleConfigCache()
  // 立即刷新剩余缓冲区（同步版本，供 onUnmounted 等无法 await 的场景使用）
  flushLogBufferImmediately()
  if (logFlushTimer) { clearTimeout(logFlushTimer); logFlushTimer = null }
  unlistenBuildLog?.()
})

/** 同步版本：立即排空 logBuffer，不经过定时器 */
function flushLogBufferImmediately() {
  if (logFlushTimer) {
    clearTimeout(logFlushTimer)
    logFlushTimer = null
  }
  flushPending = false
  if (logBuffer.length === 0 || !currentBuildId.value) return
  const entries = logBuffer.splice(0)
  const compactedEntries = compactBufferedLogs(entries)
  for (const entry of entries) {
    if (entry.progress != null) buildStore.updateProgress(entry.buildId, entry.progress)
  }
  const build = buildStore.getBuild(currentBuildId.value)
  let lastMsg = build?.logs?.length ? build.logs[build.logs.length - 1] : null
  for (const entry of compactedEntries) {
    if (lastMsg && lastMsg.level === entry.level && lastMsg.message === entry.message) continue
    buildStore.addLog(entry.buildId, entry.level, entry.message)
    lastMsg = { level: entry.level, message: entry.message } as BuildLog
  }
  const rawLines: string[] = []
  for (let i = 0; i < compactedEntries.length; i++) {
    const e = compactedEntries[i]
    if (i > 0) {
      const prev = compactedEntries[i - 1]
      if (prev.level === e.level && prev.message === e.message) continue
    }
    rawLines.push(`[${e.level}] ${e.message}`)
  }
  // 全局去重
  const uniqueLines: string[] = []
  for (const line of rawLines) {
    if (!emittedLogLines.has(line)) {
      emittedLogLines.add(line)
      uniqueLines.push(line)
    }
  }
  if (uniqueLines.length > 0) void appendLog(currentBuildId.value!, uniqueLines).catch(() => {})
}

async function chooseResource() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择 HBuilderX 导出的 resources 或 __UNI__ 资源目录'
  })
  if (typeof selected === 'string') {
    await importResource(selected)
  }
}

async function importResource(path: string) {
  importing.value = true
  scanResult.value = null
  latestManifestInfo.value = null
  manifestReadWarning.value = ''
  androidModuleConfigReport.value = null
  androidModuleConfigValues.value = {}
  try {
    scanResult.value = await invoke<ResourceScanResult>('import_uniapp_resource', {
      projectId: projectId.value,
      resourcePath: path
    })
    await refreshManifestFromLocalProject({ required: false, persist: true })
    await refreshAndroidModuleConfig()
    message.success(`已导入 ${insightAppId.value}`)
  } catch (e: any) {
    message.error(String(e))
  } finally {
    importing.value = false
  }
}

function togglePlatform(platform: Platform) {
  const index = selectedPlatforms.value.indexOf(platform)
  if (index >= 0) selectedPlatforms.value.splice(index, 1)
  else selectedPlatforms.value.push(platform)
}

async function startBuild() {
  if (!scanResult.value || selectedPlatforms.value.length === 0 || isBuilding.value) return
  // 重置全局日志去重集合（每次构建独立）
  emittedLogLines.clear()
  let manifestInfo: UniappManifestInfo
  try {
    manifestInfo = await ensureManifestInfoLoaded({ persist: true })
  } catch (e: any) {
    message.error(String(e))
    return
  }
  if (!(await ensureAndroidModuleConfigReadyForBuild())) {
    return
  }
  const importedResourcePath = scanResult.value.importedPath
  const androidModuleConfig = buildAndroidModuleConfigPayload()
  await persistAndroidModuleConfigCache()
  artifacts.value = []
  isBuilding.value = true
  let lastBuildId: string | null = null
  const buildIds: string[] = []
  for (const platform of selectedPlatforms.value) {
    const buildId = buildStore.startBuild(projectId.value, platform)
    lastBuildId = buildId
    buildIds.push(buildId)
    currentBuildId.value = buildId
    const startedAt = new Date()
    await createBuildRecord(buildId, platform, startedAt)
    await appendManifestLog(buildId, manifestInfo)
    try {
      const command = platform === 'android'
        ? 'build_android_apk'
        : platform === 'ios'
          ? 'build_ios_ipa'
          : 'build_harmony_hap'
      const artifact = await invoke<BuildArtifact>(command, {
        projectId: projectId.value,
        resourcePath: scanResult.value.importedPath,
        buildId,
        manifestInfo,
        moduleConfig: platform === 'android' ? androidModuleConfig : undefined
      })
      artifacts.value.push(artifact)
      const build = buildStore.getBuild(buildId)
      if (build) build.artifactPath = artifact.path
      buildStore.stopBuild(buildId, true)
      await finalizeBuildRecord(buildId, 'success', startedAt, artifact)
      message.success(`${platform} 构建完成`)
    } catch (e: any) {
      // 先排空 logBuffer 中的残留 Gradle 输出，确保错误日志在清理日志之前写入
      if (logBuffer.length > 0 || flushPending) await flushToStoreAndUI()
      buildStore.failBuild(buildId, String(e))
      await finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
      message.error(`${platform} 构建失败: ${String(e)}`)
    } finally {
      // 同样先排空缓冲区，确保所有构建日志在清理日志之前完成写入
      if (logBuffer.length > 0 || flushPending) await flushToStoreAndUI()
      await cleanupBuildTemporaryFiles(buildId, buildId, null)
    }
  }
  if (lastBuildId) {
    const resourceCleanupLines = await cleanupBuildTemporaryFiles(lastBuildId, null, importedResourcePath)
    for (const buildId of buildIds) {
      if (buildId !== lastBuildId) {
        await appendCleanupLines(buildId, resourceCleanupLines)
      }
    }
  }
  isBuilding.value = false
  scanResult.value = null
}

async function generateAndroidProject() {
  if (!scanResult.value || !selectedPlatforms.value.includes('android') || isBuilding.value || isGenerating.value) return
  let manifestInfo: UniappManifestInfo
  try {
    manifestInfo = await ensureManifestInfoLoaded({ persist: true })
  } catch (e: any) {
    message.error(String(e))
    return
  }
  if (!(await ensureAndroidModuleConfigReadyForBuild())) {
    return
  }
  const androidModuleConfig = buildAndroidModuleConfigPayload()
  await persistAndroidModuleConfigCache()
  isGenerating.value = true
  generatedProjectPath.value = null
  // 重置全局日志去重集合
  emittedLogLines.clear()
  const buildId = buildStore.startBuild(projectId.value, 'android')
  currentBuildId.value = buildId
  const startedAt = new Date()
  await createBuildRecord(buildId, 'android', startedAt)
  await appendManifestLog(buildId, manifestInfo)
  try {
    const projectPath = await invoke<string>('generate_android_project', {
      projectId: projectId.value,
      resourcePath: scanResult.value!.importedPath,
      buildId,
      manifestInfo,
      moduleConfig: androidModuleConfig,
    })
    generatedProjectPath.value = projectPath
    buildStore.stopBuild(buildId, true)
    await finalizeBuildRecord(buildId, 'success', startedAt, null)
    message.success(`安卓工程已生成: ${projectPath}`)
  } catch (e: any) {
    if (logBuffer.length > 0 || flushPending) await flushToStoreAndUI()
    buildStore.failBuild(buildId, String(e))
    await finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
    message.error(`生成安卓工程失败: ${String(e)}`)
  } finally {
    if (logBuffer.length > 0 || flushPending) await flushToStoreAndUI()
    isGenerating.value = false
  }
}

async function persistAndroidModuleConfigCache() {
  if (androidModuleConfigSaveTimer) {
    clearTimeout(androidModuleConfigSaveTimer)
    androidModuleConfigSaveTimer = null
  }
  const project = currentProject.value
  if (!project) return
  if (!syncAndroidModuleConfigCache()) return
  await projectsStore.saveProject(project)
}

function scheduleAndroidModuleConfigCacheSave() {
  if (androidModuleConfigSaveTimer) {
    clearTimeout(androidModuleConfigSaveTimer)
  }
  androidModuleConfigSaveTimer = setTimeout(() => {
    androidModuleConfigSaveTimer = null
    void persistAndroidModuleConfigCache()
  }, 300)
}

function getProjectName() {
  return currentProject.value?.name || currentProject.value?.app.name || projectId.value
}

function formatPlatforms(platforms: string[]) {
  if (!platforms.length) return 'all'
  return platforms.join(' / ')
}

async function refreshManifestFromLocalProject(options: { required: boolean; persist: boolean }) {
  const project = currentProject.value
  if (!project?.localPath) {
    const warning = '请先在项目配置中选择包含 manifest.json 的本地项目路径'
    latestManifestInfo.value = null
    manifestReadWarning.value = warning
    if (options.required) throw new Error(warning)
    return null
  }
  try {
    const info = await invoke<UniappManifestInfo>('read_uniapp_manifest', {
      projectPath: project.localPath
    })
    applyManifestInfoToProject(info)
    latestManifestInfo.value = info
    applyManifestInfoToScanResult(info)
    manifestReadWarning.value = ''
    if (scanResult.value && info.appId && info.appId !== scanResult.value.appId) {
      manifestReadWarning.value = `本地 manifest AppId (${info.appId}) 与导入资源 AppId (${scanResult.value.appId}) 不一致，请确认 resources 是否来自同一项目`
    }
    if (options.persist) {
      await projectsStore.saveProject(project)
    }
    return info
  } catch (e: any) {
    const warning = String(e)
    latestManifestInfo.value = null
    manifestReadWarning.value = warning
    if (options.required) throw new Error(warning)
    return null
  }
}

function applyManifestInfoToProject(info: UniappManifestInfo) {
  const project = currentProject.value
  if (!project) return
  if (info.appName) project.app.name = info.appName
  if (info.appId) project.app.appId = info.appId
  if (info.versionName) project.app.version = info.versionName
  if (typeof info.versionCode === 'number') project.app.versionCode = info.versionCode
  if (info.android.packageName) project.android.packageName = info.android.packageName
  if (typeof info.android.minSdkVersion === 'number') project.android.minSdkVersion = info.android.minSdkVersion
  if (typeof info.android.targetSdkVersion === 'number') project.android.targetSdkVersion = info.android.targetSdkVersion
  if (typeof info.android.compileSdkVersion === 'number') project.android.compileSdkVersion = info.android.compileSdkVersion
  if (!project.androidModuleConfig) project.androidModuleConfig = {}
}

function applyManifestInfoToScanResult(info: UniappManifestInfo) {
  if (!scanResult.value) return
  scanResult.value = {
    ...scanResult.value,
    appName: info.appName || scanResult.value.appName,
    versionName: info.versionName || scanResult.value.versionName,
    versionCode: info.versionCode ?? scanResult.value.versionCode,
    hbuilderxVersion: info.hbuilderxVersion || scanResult.value.hbuilderxVersion,
    manifestPath: info.manifestPath,
    detectedModules: info.detectedModules
  }
}

async function refreshAndroidModuleConfig() {
  if (!latestManifestInfo.value) {
    androidModuleConfigReport.value = null
    return
  }
  androidModuleConfigLoading.value = true
  try {
    const report = await invoke<AndroidModuleConfigReport>('analyze_android_module_config', {
      manifestInfo: latestManifestInfo.value,
      userConfig: cachedAndroidModuleConfig()
    })
    androidModuleConfigReport.value = report
    mergeAndroidModuleConfigDefaults(report)
  } catch (e: any) {
    androidModuleConfigReport.value = null
    manifestReadWarning.value = String(e)
  } finally {
    androidModuleConfigLoading.value = false
  }
}

async function ensureManifestInfoLoaded(options: { persist: boolean } = { persist: true }): Promise<UniappManifestInfo> {
  if (latestManifestInfo.value) return latestManifestInfo.value
  const info = await refreshManifestFromLocalProject({ required: true, persist: options.persist })
  if (!info) {
    const warning = manifestReadWarning.value || '请先导入资源，并确保已从本地项目路径读取 manifest.json'
    throw new Error(warning)
  }
  return info
}

async function ensureAndroidModuleConfigReadyForBuild() {
  if (!selectedNeedsAndroidConfig.value) return true
  if (!androidModuleConfigReport.value) {
    await refreshAndroidModuleConfig()
  }
  if (!androidModuleConfigReport.value) {
    message.error(manifestReadWarning.value || 'Android 模块配置分析失败')
    return false
  }
  if (androidMissingRequired.value.length) {
    message.error(`请先填写 Android 模块配置: ${androidMissingRequired.value.map(item => `${item.moduleName}-${item.label}`).join('、')}`)
    return false
  }
  return true
}

function mergeAndroidModuleConfigDefaults(report: AndroidModuleConfigReport) {
  const next: Record<string, string> = {}
  for (const mod of report.modules) {
    for (const field of mod.fields) {
      next[field.key] = field.value || ''
    }
  }
  androidModuleConfigValues.value = next
  syncAndroidModuleConfigCache()
  scheduleAndroidModuleConfigCacheSave()
}

function androidFieldValue(field: AndroidModuleConfigField) {
  return androidModuleConfigValues.value[field.key] ?? field.value ?? ''
}

function updateAndroidField(field: AndroidModuleConfigField, value: string) {
  androidModuleConfigValues.value = {
    ...androidModuleConfigValues.value,
    [field.key]: value
  }
  syncAndroidModuleConfigCache()
  scheduleAndroidModuleConfigCacheSave()
}

async function pickAndroidFileField(field: AndroidModuleConfigField) {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'JSON', extensions: ['json'] }],
    title: `选择 ${field.label}`,
  })
  if (typeof selected !== 'string') return
  try {
    const content = await readFile(selected)
    // 转为 base64 存储
    let binary = ''
    for (let i = 0; i < content.length; i++) binary += String.fromCharCode(content[i])
    const base64 = btoa(binary)
    updateAndroidField(field, base64)
  } catch (e) {
    message.error('读取文件失败: ' + String(e))
  }
}

function clearAndroidFileField(field: AndroidModuleConfigField) {
  updateAndroidField(field, '')
}

function isFileField(field: AndroidModuleConfigField): boolean {
  return field.field_type === 'file'
}

function formatFileSize(base64Value: string): string {
  if (!base64Value) return ''
  const kb = Math.ceil((base64Value.length * 3 / 4) / 1024)
  return `${kb}KB`
}

function buildAndroidModuleConfigPayload() {
  const payload: Record<string, string> = {}
  for (const [key, value] of Object.entries(androidModuleConfigValues.value)) {
    if (value.trim()) payload[key] = value.trim()
  }
  return payload
}

function cachedAndroidModuleConfig() {
  return currentProject.value?.androidModuleConfig || {}
}

function syncAndroidModuleConfigCache() {
  const project = currentProject.value
  const report = androidModuleConfigReport.value
  if (!project || !report) return false
  const next: Record<string, string> = {}
  for (const mod of report.modules) {
    for (const field of mod.fields) {
      const value = androidModuleConfigValues.value[field.key]?.trim()
      if (value && field.valueSource !== 'manifest') {
        next[field.key] = value
      }
    }
  }
  project.androidModuleConfig = next
  return true
}

function fieldStatusType(field: AndroidModuleConfigField) {
  const value = androidFieldValue(field).trim()
  if (!value && field.required) return 'error'
  if (field.valueSource === 'manifest' && value) return 'success'
  if (value) return 'info'
  return 'default'
}

function fieldStatusLabel(field: AndroidModuleConfigField) {
  const value = androidFieldValue(field).trim()
  if (!value && field.required) return '必填'
  if (!value) return '可选'
  if (field.valueSource === 'manifest') return 'manifest'
  return '已填写'
}

function manifestLogLines(info: UniappManifestInfo) {
  const moduleNames = info.detectedModules.map(mod => `${mod.name}(${formatPlatforms(mod.platforms)})`)
  const lines = [
    `[info] 已读取 manifest.json: ${info.manifestPath}`,
    `[info] manifest 应用名称: ${info.appName || '-'}`,
    `[info] manifest UniApp AppId: ${info.appId || '-'}`,
    `[info] manifest 版本: ${info.versionName || '-'} / ${info.versionCode ?? '-'}`,
    `[info] manifest Android 图标: ${info.androidIcons ? Object.keys(info.androidIcons.android || {}).join(', ') : '-'}`,
    // `[info] manifest Android 包名: ${info.android.packageName || '-'}`,
    `[info] manifest Android SDK: min ${info.android.minSdkVersion ?? '-'}, target ${info.android.targetSdkVersion ?? '-'}, compile ${info.android.compileSdkVersion ?? '-'}`,
    `[info] manifest 模块: ${moduleNames.length ? moduleNames.join(', ') : '无'}`
  ]
  const report = androidModuleConfigReport.value
  if (report?.modules.length) {
    lines.push(`[info] Android 模块配置清单: ${report.modules.map(mod => `${mod.name}(${mod.fields.length})`).join(', ')}`)
    for (const mod of report.modules) {
      for (const field of mod.fields) {
        const value = androidFieldValue(field).trim()
        const source = field.valueSource === 'manifest' ? 'manifest' : value ? '构建中心' : field.required ? '缺失' : '可选未填'
        lines.push(`[info] Android 模块配置 ${mod.name} / ${field.label}: ${source}`)
      }
    }
  }
  return lines
}

async function appendManifestLog(buildId: string, info: UniappManifestInfo) {
  const lines = manifestLogLines(info)
  await appendLog(buildId, lines)
  for (const line of lines) {
    buildStore.addLog(buildId, 'info', line.replace(/^\[info\]\s*/, ''))
  }
}

async function createBuildRecord(buildId: string, platform: Platform, startedAt: Date) {
  const record: BuildRecord = {
    id: buildId,
    project_id: projectId.value,
    project_name: getProjectName(),
    platform,
    status: 'building',
    artifact_path: null,
    artifact_size_mb: null,
    version_name: latestManifestInfo.value?.versionName || currentProject.value?.app.version || scanResult.value?.versionName || '-',
    version_code: latestManifestInfo.value?.versionCode || currentProject.value?.app.versionCode || scanResult.value?.versionCode || 1,
    build_mode: 'release',
    duration_secs: 0,
    started_at: startedAt.toISOString(),
    finished_at: null,
    error_message: null,
    log_path: null,
    resource_path: scanResult.value?.importedPath || null
  }
  try {
    await invoke('add_build_record', { record })
    await appendLog(buildId, [`[info] 开始构建 ${platform} 版本...`])
  } catch (e) {
    console.warn('Failed to create build history:', e)
  }
}

async function finalizeBuildRecord(
  buildId: string,
  status: 'success' | 'failed',
  startedAt: Date,
  artifact: BuildArtifact | null,
  errorMessage?: string
) {
  const finishedAt = new Date()
  try {
    const logPath = await appendLog(buildId, [
      status === 'success' ? '[success] 构建完成！' : `[error] 构建失败: ${errorMessage || '未知错误'}`
    ])
    await invoke('update_build_record', {
      id: buildId,
      update: {
        status,
        artifact_path: artifact?.path || null,
        artifact_size_mb: artifact ? artifact.sizeBytes / 1024 / 1024 : null,
        finished_at: finishedAt.toISOString(),
        error_message: errorMessage || null,
        log_path: logPath,
        duration_secs: Math.max(1, Math.round((finishedAt.getTime() - startedAt.getTime()) / 1000))
      }
    })
  } catch (e) {
    console.warn('Failed to update build history:', e)
  }
}

async function appendLog(buildId: string, lines: string[]) {
  return invoke<string>('append_build_log', {
    projectId: projectId.value,
    buildId,
    lines
  })
}

async function cleanupBuildTemporaryFiles(
  logBuildId: string,
  cleanupBuildId: string | null,
  resourcePath: string | null
): Promise<string[]> {
  try {
    const result = await invoke<{ items: Array<{ label: string; path: string; status: string; message: string }> }>(
      'cleanup_build_temporary_files',
      {
        projectId: projectId.value,
        buildId: cleanupBuildId,
        resourcePath
      }
    )
    const lines = result.items.map((item) => {
      const level = item.status === 'failed' ? 'warn' : 'info'
      return `[${level}] 清理${item.label}: ${item.message} (${item.path})`
    })
    if (lines.length) {
      await appendCleanupLines(logBuildId, lines)
    }
    return lines
  } catch (e: any) {
    const line = `清理临时文件失败: ${String(e)}`
    const lines = [`[warn] ${line}`]
    await appendCleanupLines(logBuildId, lines)
    return lines
  }
}

async function appendCleanupLines(buildId: string, lines: string[]) {
  if (!lines.length) return
  await appendLog(buildId, lines).catch(() => undefined)
  for (const line of lines) {
    const match = line.match(/^\[(\w+)\]\s*(.*)$/)
    buildStore.addLog(buildId, match?.[1] === 'warn' ? 'warn' : 'info', match?.[2] || line)
  }
}

function goBack() {
  router.push(`/project/${projectId.value}`)
}
</script>

<template>
  <div class="build-center">
    <div class="page-header">
      <n-space align="center">
        <n-button quaternary circle @click="goBack">
          <template #icon><n-icon><ArrowBackOutline /></n-icon></template>
        </n-button>
        <n-text strong class="title">构建中心</n-text>
      </n-space>
    </div>
    <n-grid :cols="2" :x-gap="20" responsive="screen">
      <n-gi>
        <n-card title="1. 导入 UniApp 资源" style="min-height: 507px;">
          <n-space>
            <n-button type="primary" :loading="importing" @click="chooseResource">
              <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
              选择 resources 目录
            </n-button>
          </n-space>
          <div v-if="scanResult" class="scan-result">
            <n-alert type="success" title="资源扫描完成">
              <n-space vertical :size="8">
                <n-text>AppId: <n-text code>{{ insightAppId }}</n-text></n-text>
                <n-text>版本: {{ insightVersionName }} / {{ insightVersionCode }}</n-text>
                <n-text>资源包根目录: <n-text code>{{ scanResult.importedPath }}</n-text></n-text>
                <n-text>应用资源目录: <n-text code>{{ scanResult.appResourcePath }}</n-text></n-text>
                <n-text>manifest 路径: <n-text code>{{ insightManifestPath }}</n-text></n-text>
              </n-space>
            </n-alert>
            <n-alert v-for="warning in scanResult.warnings" :key="warning" type="warning" style="margin-top: 8px;">
              {{ warning }}
            </n-alert>
            <n-alert v-if="manifestReadWarning" type="warning" style="margin-top: 8px;">
              {{ manifestReadWarning }}
            </n-alert>
          </div>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card title="2. 选择平台">
          <PlatformCard
            :platforms="platforms"
            :selected-platforms="selectedPlatforms"
            @toggle="togglePlatform"
          />
          <n-space justify="end" style="margin-top: 18px;">
            <n-text v-if="buildDisabledReason && !canBuild" depth="3">{{ buildDisabledReason }}</n-text>
            <n-button type="primary" :disabled="!canGenerateAndroid" :loading="isGenerating" @click="generateAndroidProject">
              <template #icon><n-icon><LogoAndroid /></n-icon></template>
              生成安卓项目
            </n-button>
            <n-button type="success" :disabled="!canBuild" :loading="isBuilding" @click="startBuild">
              <template #icon><n-icon><PlayOutline /></n-icon></template>
              开始打包
            </n-button>
          </n-space>
        </n-card>
      </n-gi>
    </n-grid>
    <n-card v-if="scanResult && selectedPlatforms.includes('android')" title="Android 模块配置" style="margin-top: 16px;">
      <n-space vertical :size="14">
        <n-alert v-if="androidModuleConfigLoading" type="info">正在从 manifest 解析 Android 模块配置...</n-alert>
        <n-alert v-else-if="!latestManifestInfo" type="warning">
          {{ manifestReadWarning || '请先在项目配置中设置本地项目路径，以便读取 manifest.json' }}
        </n-alert>
        <n-alert v-else-if="!androidModuleConfigReport || !androidModuleConfigReport.modules.length" type="success">
          未检测到需要额外配置项的 Android 模块。
        </n-alert>
        <n-alert v-else :type="androidMissingRequired.length ? 'warning' : 'success'">
          <n-space vertical :size="6">
            <n-text>
              已检测到 {{ androidModuleConfigReport.modules.length }} 个 Android 模块将参与打包：
              {{ androidConfiguredModuleNames.join('、') }}
            </n-text>
            <n-text v-if="androidMissingRequired.length">
              还有 {{ androidMissingRequired.length }} 个必填项未填写，填写完成后才能开始打包。
            </n-text>
            <n-text v-else>模块配置已就绪，可以开始 Android 打包。</n-text>
          </n-space>
        </n-alert>

        <div v-if="androidModuleConfigReport?.modules.length" class="android-config-list">
          <div v-for="mod in androidModuleConfigReport.modules" :key="mod.templateKey + mod.name" class="android-config-module">
            <div class="android-config-head">
              <n-space align="center" :size="8">
                <n-text strong>{{ mod.name }}</n-text>
                <n-tag size="small" type="info">{{ mod.category }}</n-tag>
                <n-tag size="small" :type="mod.platforms.includes('android') ? 'success' : 'default'">{{ formatPlatforms(mod.platforms) }}</n-tag>
              </n-space>
              <n-text depth="3">{{ mod.fields.length ? `${mod.fields.length} 项配置` : '无需配置' }}</n-text>
            </div>
            <n-grid :cols="2" :x-gap="14" :y-gap="10" responsive="screen">
              <n-gi v-for="field in mod.fields" :key="mod.templateKey + field.key">
                <n-form-item :label="field.label" :feedback="field.required && !androidFieldValue(field).trim() ? '必填项，未填写时不能开始打包' : undefined">
                  <template #label>
                    <n-space align="center" :size="6">
                      <n-text>{{ field.label }}</n-text>
                      <n-tag size="tiny" :type="fieldStatusType(field)">{{ fieldStatusLabel(field) }}</n-tag>
                    </n-space>
                  </template>
                  <!-- 文件类型字段：渲染文件选择器 -->
                  <template v-if="isFileField(field)">
                    <n-space :size="8" align="center" style="width: 100%;">
                      <n-button size="small" @click="pickAndroidFileField(field)">选择文件</n-button>
                      <n-text v-if="androidFieldValue(field)" depth="3" style="font-size: 12px;">
                        已选择 ({{ formatFileSize(androidFieldValue(field)) }})
                      </n-text>
                      <n-button v-if="androidFieldValue(field)" size="small" quaternary type="error" @click="clearAndroidFileField(field)">清除</n-button>
                      <n-text v-else depth="3" style="font-size: 12px;">{{ field.placeholder }}</n-text>
                    </n-space>
                  </template>
                  <!-- 文本类型字段：渲染输入框 -->
                  <n-input
                    v-else
                    :value="androidFieldValue(field)"
                    :placeholder="field.placeholder"
                    :type="field.secret ? 'password' : 'text'"
                    :show-password-on="field.secret ? 'click' : undefined"
                    @update:value="(value: string) => updateAndroidField(field, value)"
                  />
                </n-form-item>
              </n-gi>
            </n-grid>
            <n-divider style="margin: 4px 0 0;" />
          </div>
        </div>
      </n-space>
    </n-card>

    <n-card v-if="scanResult" title="识别到的资源与模块" style="margin-top: 16px;">
      <div class="insight-panel">
        <div class="insight-head">
          <div>
            <n-text strong class="insight-title">{{ insightAppName }}</n-text>
            <n-text depth="3" class="insight-subtitle">{{ insightAppId }} · {{ insightVersionName }} / {{ insightVersionCode }}</n-text>
          </div>
          <n-tag :type="scanResult.isZip ? 'warning' : 'success'">{{ scanResult.isZip ? 'ZIP 导入' : '目录导入' }}</n-tag>
        </div>
        <n-grid :cols="4" :x-gap="12" :y-gap="12" responsive="screen" class="insight-grid">
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">Manifest 模块</n-text>
              <n-text strong class="summary-value">{{ manifestModules.length }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">UTS 内置模块</n-text>
              <n-text strong class="summary-value">{{ scanResult.uts.builtinModules.length }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">UTS 自定义插件</n-text>
              <n-text strong class="summary-value">{{ scanResult.uts.customPlugins.length }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">远程依赖</n-text>
              <n-text strong class="summary-value">{{ utsDependencyCount }}</n-text>
            </div>
          </n-gi>
        </n-grid>

        <n-grid :cols="2" :x-gap="14" :y-gap="14" responsive="screen" class="module-grid">
          <n-gi>
            <div class="module-box">
              <n-text strong>Manifest 模块</n-text>
              <n-space v-if="manifestModuleLabels.length" wrap :size="8" class="tag-row">
                <n-tag v-for="label in manifestModuleLabels" :key="label" type="info">{{ label }}</n-tag>
              </n-space>
              <n-text v-else depth="3" class="module-empty">未声明 App 模块</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="module-box">
              <n-text strong>需配置模块</n-text>
              <n-space v-if="androidModuleConfigSummary.length" wrap :size="8" class="tag-row">
                <n-tag v-for="label in androidModuleConfigSummary" :key="label" :type="androidMissingRequired.length ? 'warning' : 'success'">{{ label }}</n-tag>
              </n-space>
              <n-text v-else depth="3" class="module-empty">未检测到 Android 配置项</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="module-box">
              <n-text strong>UTS 插件</n-text>
              <n-space v-if="utsPluginLabels.length" wrap :size="8" class="tag-row">
                <n-tag v-for="label in utsPluginLabels" :key="label" type="success">{{ label }}</n-tag>
              </n-space>
              <n-text v-else depth="3" class="module-empty">未检测到 UTS 插件</n-text>
            </div>
          </n-gi>
        </n-grid>
        <div class="path-summary">
          <n-text depth="3">manifest 路径</n-text>
          <n-text code>{{ insightManifestPath }}</n-text>
          <n-text depth="3">资源包根目录</n-text>
          <n-text code>{{ scanResult.importedPath }}</n-text>
          <n-text depth="3">应用资源目录</n-text>
          <n-text code>{{ scanResult.appResourcePath }}</n-text>
        </div>
      </div>

      <div v-if="scanResult.warnings.length" class="module-section">
        <n-alert v-for="warning in scanResult.warnings" :key="warning" type="warning" style="margin-top: 8px;">
          {{ warning }}
        </n-alert>
      </div>
      <div v-if="manifestReadWarning" class="module-section">
        <n-alert type="warning">{{ manifestReadWarning }}</n-alert>
      </div>
    </n-card>

    <n-card title="构建日志" style="margin-top: 16px;">
      <LogPanel :logs="currentBuild?.logs || []" height="380px" />
      <n-progress
        style="margin-top: 16px;"
        type="line"
        :percentage="currentBuild?.progress || 0"
        :processing="currentBuild?.status === 'building'"
        :status="currentBuild?.status === 'failed' ? 'error' : currentBuild?.status === 'success' ? 'success' : 'default'"
      />
      <n-alert v-for="artifact in artifacts" :key="artifact.path" type="success" style="margin-top: 12px;">
        {{ artifact.platform }}: <n-text code>{{ artifact.path }}</n-text>
      </n-alert>
      <n-alert v-if="generatedProjectPath" type="info" style="margin-top: 12px;">
        <n-space align="center">
          <span>Android 工程已生成:</span>
          <n-text code>{{ generatedProjectPath }}</n-text>
          <n-button size="small" @click="() => { void invoke('tauri', { __tauriModule: 'shell', message: { cmd: 'open', path: generatedProjectPath } }) }">打开目录</n-button>
        </n-space>
      </n-alert>
    </n-card>
  </div>
</template>

<style scoped>
.build-center {
  max-width: 1280px;
}

.page-header {
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.title {
  font-size: 24px;
}

.scan-result {
  margin-top: 16px;
}

.insight-panel {
  padding: 16px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: #fafafa;
}

.insight-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding-bottom: 14px;
  border-bottom: 1px solid #ececec;
}

.insight-title {
  display: block;
  font-size: 18px;
  line-height: 1.35;
}

.insight-subtitle {
  display: block;
  margin-top: 4px;
}

.insight-grid,
.module-grid {
  margin-top: 14px;
}

.summary-tile {
  min-height: 72px;
  padding: 12px;
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background: #fafafa;
}

.summary-value {
  font-size: 24px;
  line-height: 1;
}

.module-section {
  margin-top: 16px;
}

.module-box {
  min-height: 96px;
  padding: 12px;
  border: 1px solid #eeeeee;
  border-radius: 6px;
  background: #ffffff;
}

.tag-row {
  margin-top: 10px;
}

.module-empty {
  display: block;
  margin-top: 10px;
}

.path-summary {
  display: grid;
  grid-template-columns: 96px minmax(0, 1fr);
  gap: 8px 12px;
  margin-top: 14px;
  padding: 12px;
  border: 1px solid #eeeeee;
  border-radius: 6px;
  background: #ffffff;
}

.path-summary :deep(.n-text) {
  word-break: break-all;
}

.android-config-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.android-config-module {
  padding: 12px;
  border: 1px solid #eeeeee;
  border-radius: 6px;
  background: #ffffff;
}

.android-config-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}
</style>
