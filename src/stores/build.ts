import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type BuildStatus = 'idle' | 'building' | 'success' | 'failed' | 'cancelled'

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
  status: BuildStatus
  startTime: string | null
  endTime: string | null
  progress: number
  logs: BuildLog[]
  artifactPath: string | null
}

export const useBuildStore = defineStore('build', () => {
  const builds = ref<Record<string, BuildTask>>({})
  const activeBuildIds = ref<string[]>([])

  const MAX_LOG_ENTRIES = 5000

  const currentBuilds = computed(() => {
    return activeBuildIds.value.map(id => builds.value[id]).filter(Boolean)
  })

  const hasActiveBuilds = computed(() => {
    return activeBuildIds.value.some(id => builds.value[id]?.status === 'building')
  })

  function generateBuildId(): string {
    return `build_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  function generateLogId(): string {
    return `log_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  function startBuild(projectId: string, platform: BuildTask['platform']): string {
    const buildId = generateBuildId()
    const buildTask: BuildTask = {
      id: buildId,
      projectId,
      platform,
      status: 'building',
      startTime: new Date().toISOString(),
      endTime: null,
      progress: 0,
      logs: [],
      artifactPath: null
    }
    
    builds.value[buildId] = buildTask
    activeBuildIds.value.push(buildId)
    
    addLog(buildId, 'info', `开始构建 ${platform} 版本...`)
    
    return buildId
  }

  function stopBuild(buildId: string, success: boolean = false) {
    const build = builds.value[buildId]
    if (!build) return
    
    build.status = success ? 'success' : 'cancelled'
    build.endTime = new Date().toISOString()
    build.progress = success ? 100 : build.progress
    
    addLog(buildId, success ? 'success' : 'warn', 
      success ? '构建完成！' : '构建已取消')
  }

  function failBuild(buildId: string, error: string) {
    const build = builds.value[buildId]
    if (!build) return
    
    build.status = 'failed'
    build.endTime = new Date().toISOString()
    addLog(buildId, 'error', `构建失败: ${error}`)
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

    // 防止内存无限增长
    if (build.logs.length > MAX_LOG_ENTRIES) {
      build.logs = build.logs.slice(-MAX_LOG_ENTRIES)
    }
  }

  function updateProgress(buildId: string, progress: number) {
    const build = builds.value[buildId]
    if (!build) return
    
    build.progress = Math.min(100, Math.max(0, progress))
  }

  function getBuild(buildId: string): BuildTask | undefined {
    return builds.value[buildId]
  }

  function getProjectBuilds(projectId: string): BuildTask[] {
    return Object.values(builds.value).filter(b => b.projectId === projectId)
  }

  function clearBuild(buildId: string) {
    delete builds.value[buildId]
    const index = activeBuildIds.value.indexOf(buildId)
    if (index > -1) {
      activeBuildIds.value.splice(index, 1)
    }
  }

  function clearAllBuilds() {
    builds.value = {}
    activeBuildIds.value = []
  }

  return {
    builds,
    activeBuildIds,
    currentBuilds,
    hasActiveBuilds,
    startBuild,
    stopBuild,
    failBuild,
    addLog,
    updateProgress,
    getBuild,
    getProjectBuilds,
    clearBuild,
    clearAllBuilds
  }
})
