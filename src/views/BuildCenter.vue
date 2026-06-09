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
import { LogoAndroid, LogoApple, PhonePortraitOutline } from '@vicons/ionicons5'
import PlatformCard from '../components/PlatformCard.vue'
import LogPanel from '../components/LogPanel.vue'
import { useBuildStore } from '../stores/build'
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
const currentBuildId = ref<string | null>(null)
const scanResult = ref<ResourceScanResult | null>(null)
const latestManifestInfo = ref<UniappManifestInfo | null>(null)
const manifestReadWarning = ref('')
const androidModuleConfigReport = ref<AndroidModuleConfigReport | null>(null)
const androidModuleConfigValues = ref<Record<string, string>>({})
const androidModuleConfigLoading = ref(false)

let androidModuleConfigSaveTimer: ReturnType<typeof setTimeout> | null = null

const platforms = [
  { key: 'android' as const, label: 'Android', icon: LogoAndroid, description: 'APK 安装包', color: '#2f9e44', bgColor: '#e8f5e9' },
  { key: 'ios' as const, label: 'iOS', icon: LogoApple, description: 'IPA 安装包', color: '#1c7ed6', bgColor: '#e7f5ff' },
  { key: 'harmony' as const, label: '鸿蒙', icon: PhonePortraitOutline, description: 'HAP 安装包', color: '#d6336c', bgColor: '#fff0f6' }
]

const selectedNeedsAndroidConfig = computed(() => selectedPlatforms.value.includes('android'))
const isBuildLocked = computed(() => buildStore.hasActiveBuilds)
const activeProjectBuild = computed(() => buildStore.getActiveBuildForProject(projectId.value))
const currentBuild = computed(() => {
  if (currentBuildId.value) return buildStore.getBuild(currentBuildId.value) || null
  return activeProjectBuild.value || null
})
const packageBuildLoading = computed(() => currentBuild.value?.status === 'building' && currentBuild.value.kind === 'package')
const androidGenerateLoading = computed(() => currentBuild.value?.status === 'building' && currentBuild.value.kind === 'generateAndroidProject')
const visibleArtifacts = computed<BuildArtifact[]>(() => {
  const build = currentBuild.value
  if (!build?.artifactPath) return []
  return [{
    platform: build.platform,
    path: build.artifactPath,
    fileName: build.artifactPath.split('/').pop() || build.artifactPath,
    sizeBytes: build.artifactSizeBytes || 0,
    buildId: build.id
  }]
})
const currentGeneratedProjectPath = computed(() => currentBuild.value?.generatedProjectPath || null)
const androidMissingRequired = computed(() => {
  const report = androidModuleConfigReport.value
  if (!report) return []
  return report.modules.flatMap(mod => mod.fields
    .filter(field => field.required && !androidFieldValue(field).trim())
    .map(field => ({ moduleName: mod.name, key: field.key, label: field.label })))
})
const canBuild = computed(() => {
  if (!scanResult.value || selectedPlatforms.value.length === 0 || isBuildLocked.value) return false
  if (selectedNeedsAndroidConfig.value && !androidModulesReady.value) return false
  return true
})
const canGenerateAndroid = computed(() => {
  if (!scanResult.value || !selectedPlatforms.value.includes('android') || isBuildLocked.value) return false
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
  if (isBuildLocked.value) return '正在构建中'
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
  const activeBuild = buildStore.getActiveBuildForProject(projectId.value)
  if (activeBuild) currentBuildId.value = activeBuild.id
})

onUnmounted(() => {
  if (androidModuleConfigSaveTimer) {
    clearTimeout(androidModuleConfigSaveTimer)
    androidModuleConfigSaveTimer = null
  }
  void persistAndroidModuleConfigCache()
})

async function chooseResource() {
  if (isBuildLocked.value) {
    message.warning('已有构建任务进行中，请等待完成后再导入资源')
    return
  }
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
  if (isBuildLocked.value) {
    message.warning('已有构建任务进行中，请等待完成后再导入资源')
    return
  }
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
  if (isBuildLocked.value) return
  const index = selectedPlatforms.value.indexOf(platform)
  if (index >= 0) selectedPlatforms.value.splice(index, 1)
  else selectedPlatforms.value.push(platform)
}

async function startBuild() {
  if (!scanResult.value || selectedPlatforms.value.length === 0 || isBuildLocked.value) {
    if (isBuildLocked.value) message.warning('已有构建任务进行中，请等待完成后再开始新的构建')
    return
  }
  const runProjectId = projectId.value
  const runProjectName = getProjectName()
  const importedResourcePath = scanResult.value.importedPath
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
  let lastBuildId: string | null = null
  const buildIds: string[] = []
  for (const platform of selectedPlatforms.value) {
    const buildId = buildStore.startBuild(runProjectId, platform, 'package')
    lastBuildId = buildId
    buildIds.push(buildId)
    currentBuildId.value = buildId
    buildStore.setActiveEventBuildId(buildId)
    const startedAt = new Date()
    await createBuildRecord(buildId, platform, startedAt, runProjectId, runProjectName, importedResourcePath)
    await appendManifestLog(buildId, manifestInfo)
    try {
      const command = platform === 'android'
        ? 'build_android_apk'
        : platform === 'ios'
          ? 'build_ios_ipa'
          : 'build_harmony_hap'
      const artifact = await invoke<BuildArtifact>(command, {
        projectId: runProjectId,
        resourcePath: importedResourcePath,
        buildId,
        manifestInfo,
        moduleConfig: platform === 'android' ? androidModuleConfig : undefined
      })
      await buildStore.flushBuildLogs(buildId)
      await appendFinalLog(buildId, 'success')
      await finalizeBuildRecord(buildId, 'success', startedAt, artifact)
      buildStore.stopBuild(buildId, true, {
        artifactPath: artifact.path,
        artifactSizeBytes: artifact.sizeBytes
      })
      message.success(`${platform} 构建完成`)
    } catch (e: any) {
      await buildStore.flushBuildLogs(buildId)
      await appendFinalLog(buildId, 'failed', String(e))
      await finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
      buildStore.failBuild(buildId, String(e))
      message.error(`${platform} 构建失败: ${String(e)}`)
    } finally {
      await cleanupBuildTemporaryFiles(buildId, buildId, null)
      await buildStore.flushBuildLogs(buildId)
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
  buildStore.setActiveEventBuildId(null)
  scanResult.value = null
}

async function generateAndroidProject() {
  if (!scanResult.value || !selectedPlatforms.value.includes('android') || isBuildLocked.value) {
    if (isBuildLocked.value) message.warning('已有构建任务进行中，请等待完成后再生成安卓工程')
    return
  }
  const runProjectId = projectId.value
  const runProjectName = getProjectName()
  const importedResourcePath = scanResult.value.importedPath
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
  const buildId = buildStore.startBuild(runProjectId, 'android', 'generateAndroidProject')
  currentBuildId.value = buildId
  buildStore.setActiveEventBuildId(buildId)
  const startedAt = new Date()
  await createBuildRecord(buildId, 'android', startedAt, runProjectId, runProjectName, importedResourcePath)
  await appendManifestLog(buildId, manifestInfo)
  try {
    const projectPath = await invoke<string>('generate_android_project', {
      projectId: runProjectId,
      resourcePath: importedResourcePath,
      buildId,
      manifestInfo,
      moduleConfig: androidModuleConfig,
    })
    await buildStore.flushBuildLogs(buildId)
    await appendFinalLog(buildId, 'success')
    await finalizeBuildRecord(buildId, 'success', startedAt, null)
    buildStore.stopBuild(buildId, true, { generatedProjectPath: projectPath })
    message.success(`安卓工程已生成: ${projectPath}`)
  } catch (e: any) {
    await buildStore.flushBuildLogs(buildId)
    await appendFinalLog(buildId, 'failed', String(e))
    await finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
    buildStore.failBuild(buildId, String(e))
    message.error(`生成安卓工程失败: ${String(e)}`)
  } finally {
    await buildStore.flushBuildLogs(buildId)
    if (buildStore.activeEventBuildId === buildId) buildStore.setActiveEventBuildId(null)
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
  if (isBuildLocked.value) return
  androidModuleConfigValues.value = {
    ...androidModuleConfigValues.value,
    [field.key]: value
  }
  syncAndroidModuleConfigCache()
  scheduleAndroidModuleConfigCacheSave()
}

async function pickAndroidFileField(field: AndroidModuleConfigField) {
  if (isBuildLocked.value) {
    message.warning('已有构建任务进行中，暂不能修改模块配置')
    return
  }
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
  if (isBuildLocked.value) return
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
  await buildStore.appendBuildLogLines(buildId, manifestLogLines(info))
}

async function appendFinalLog(buildId: string, status: 'success' | 'failed', errorMessage?: string) {
  await buildStore.appendBuildLogLines(buildId, [
    status === 'success' ? '[success] 构建完成！' : `[error] 构建失败: ${errorMessage || '未知错误'}`
  ])
}

async function createBuildRecord(
  buildId: string,
  platform: Platform,
  startedAt: Date,
  recordProjectId: string = projectId.value,
  recordProjectName: string = getProjectName(),
  recordResourcePath: string | null = scanResult.value?.importedPath || null
) {
  const record: BuildRecord = {
    id: buildId,
    project_id: recordProjectId,
    project_name: recordProjectName,
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
    resource_path: recordResourcePath
  }
  try {
    await invoke('add_build_record', { record })
    await buildStore.appendBuildLogLines(buildId, [`[info] 开始构建 ${platform} 版本...`])
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
    const build = buildStore.getBuild(buildId)
    await invoke('update_build_record', {
      id: buildId,
      update: {
        status,
        artifact_path: artifact?.path || null,
        artifact_size_mb: artifact ? artifact.sizeBytes / 1024 / 1024 : null,
        finished_at: finishedAt.toISOString(),
        error_message: errorMessage || null,
        log_path: build?.logPath || null,
        duration_secs: Math.max(1, Math.round((finishedAt.getTime() - startedAt.getTime()) / 1000))
      }
    })
  } catch (e) {
    console.warn('Failed to update build history:', e)
  }
}

async function cleanupBuildTemporaryFiles(
  logBuildId: string,
  cleanupBuildId: string | null,
  resourcePath: string | null
): Promise<string[]> {
  try {
    const cleanupProjectId = buildStore.getBuild(logBuildId)?.projectId || projectId.value
    if (!cleanupProjectId) throw new Error('缺少项目 ID，无法清理临时文件')
    const result = await invoke<{ items: Array<{ label: string; path: string; status: string; message: string }> }>(
      'cleanup_build_temporary_files',
      {
        projectId: cleanupProjectId,
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
  await buildStore.appendBuildLogLines(buildId, lines).catch(() => undefined)
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
        <div>
          <n-text strong class="page-title">构建中心</n-text>
          <n-text v-if="currentProject" depth="3" class="page-subtitle">{{ getProjectName() }}</n-text>
        </div>
      </n-space>
    </div>
    <n-grid cols="1 s:1 m:2" :x-gap="18" :y-gap="18" responsive="screen" class="build-grid">
      <n-gi>
        <n-card title="1. 导入 UniApp 资源" class="build-step-card import-card">
          <n-space>
            <n-button type="primary" :loading="importing" :disabled="isBuildLocked" @click="chooseResource">
              <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
              选择 resources 目录
            </n-button>
          </n-space>
          <div v-if="scanResult" class="scan-result">
            <div class="alert-stack">
              <n-alert type="success" title="资源扫描完成">
                <n-space vertical :size="8">
                  <n-text>AppId: <n-text code class="path-text">{{ insightAppId }}</n-text></n-text>
                  <n-text>版本: {{ insightVersionName }} / {{ insightVersionCode }}</n-text>
                  <n-text>资源包根目录: <n-text code class="path-text">{{ scanResult.importedPath }}</n-text></n-text>
                  <n-text>应用资源目录: <n-text code class="path-text">{{ scanResult.appResourcePath }}</n-text></n-text>
                  <n-text>manifest 路径: <n-text code class="path-text">{{ insightManifestPath }}</n-text></n-text>
                </n-space>
              </n-alert>
              <n-alert v-for="warning in scanResult.warnings" :key="warning" type="warning">
                {{ warning }}
              </n-alert>
              <n-alert v-if="manifestReadWarning" type="warning">
                {{ manifestReadWarning }}
              </n-alert>
            </div>
          </div>
        </n-card>
      </n-gi>
      <n-gi>
        <n-card title="2. 选择平台" class="build-step-card">
          <PlatformCard
            :platforms="platforms"
            :selected-platforms="selectedPlatforms"
            :disabled="isBuildLocked"
            @toggle="togglePlatform"
          />
          <n-space justify="end" class="build-action-row">
            <n-text v-if="buildDisabledReason && !canBuild" depth="3">{{ buildDisabledReason }}</n-text>
            <n-button type="primary" :disabled="!canGenerateAndroid" :loading="androidGenerateLoading" @click="generateAndroidProject">
              <template #icon><n-icon><LogoAndroid /></n-icon></template>
              生成安卓项目
            </n-button>
            <n-button type="success" :disabled="!canBuild" :loading="packageBuildLoading" @click="startBuild">
              <template #icon><n-icon><PlayOutline /></n-icon></template>
              开始打包
            </n-button>
          </n-space>
        </n-card>
      </n-gi>
    </n-grid>
    <n-card v-if="scanResult && selectedPlatforms.includes('android')" title="Android 模块配置" class="build-section-card">
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
                  <template v-if="isFileField(field)">
                    <n-space :size="8" align="center" class="file-field-row">
                      <n-button size="small" :disabled="isBuildLocked" @click="pickAndroidFileField(field)">选择文件</n-button>
                      <n-text v-if="androidFieldValue(field)" depth="3" class="file-field-hint">
                        已选择 ({{ formatFileSize(androidFieldValue(field)) }})
                      </n-text>
                      <n-button v-if="androidFieldValue(field)" size="small" quaternary type="error" :disabled="isBuildLocked" @click="clearAndroidFileField(field)">清除</n-button>
                      <n-text v-else depth="3" class="file-field-hint">{{ field.placeholder }}</n-text>
                    </n-space>
                  </template>
                  <n-input
                    v-else
                    :value="androidFieldValue(field)"
                    :placeholder="field.placeholder"
                    :type="field.secret ? 'password' : 'text'"
                    :show-password-on="field.secret ? 'click' : undefined"
                    :disabled="isBuildLocked"
                    @update:value="(value: string) => updateAndroidField(field, value)"
                  />
                </n-form-item>
              </n-gi>
            </n-grid>
            <n-divider class="module-divider" />
          </div>
        </div>
      </n-space>
    </n-card>

    <n-card v-if="scanResult" title="识别到的资源与模块" class="build-section-card">
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
        <n-alert v-for="warning in scanResult.warnings" :key="warning" type="warning">
          {{ warning }}
        </n-alert>
      </div>
      <div v-if="manifestReadWarning" class="module-section">
        <n-alert type="warning">{{ manifestReadWarning }}</n-alert>
      </div>
    </n-card>

    <n-card title="构建日志" class="build-section-card log-section-card">
      <LogPanel :logs="currentBuild?.logs || []" height="380px" />
      <n-progress
        class="build-progress"
        type="line"
        :percentage="currentBuild?.progress || 0"
        :processing="currentBuild?.status === 'building'"
        :status="currentBuild?.status === 'failed' ? 'error' : currentBuild?.status === 'success' ? 'success' : 'default'"
      />
      <div class="alert-stack log-result-stack">
        <n-alert v-for="artifact in visibleArtifacts" :key="artifact.path" type="success">
          {{ artifact.platform }}: <n-text code class="path-text">{{ artifact.path }}</n-text>
        </n-alert>
        <n-alert v-if="currentGeneratedProjectPath" type="info">
          <n-space align="center">
            <span>Android 工程已生成:</span>
            <n-text code class="path-text">{{ currentGeneratedProjectPath }}</n-text>
            <n-button size="small" @click="() => { void invoke('tauri', { __tauriModule: 'shell', message: { cmd: 'open', path: currentGeneratedProjectPath } }) }">打开目录</n-button>
          </n-space>
        </n-alert>
      </div>
    </n-card>
  </div>
</template>

<style scoped>
.build-center {
  max-width: 1280px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.page-header {
  margin-bottom: 2px;
}

.build-grid {
  align-items: start;
}

.build-step-card {
  height: auto;
}

.import-card {
  min-height: 220px;
}

.build-action-row {
  margin-top: 18px;
}

.build-section-card {
  margin-top: 0;
}

.scan-result {
  margin-top: 16px;
}

.insight-panel {
  padding: 16px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-muted);
}

.insight-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border-soft);
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
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background: var(--surface-color);
}

.summary-value {
  font-size: 24px;
  line-height: 1;
}

.module-section {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.module-box {
  min-height: 96px;
  padding: 12px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-color);
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
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-color);
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
  padding: 14px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-muted);
}

.android-config-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.file-field-row {
  width: 100%;
}

.file-field-hint {
  font-size: 12px;
}

.module-divider {
  margin: 4px 0 0;
}

.build-progress {
  margin-top: 16px;
}

.log-result-stack {
  margin-top: 12px;
}

@media (max-width: 1180px) {
  .android-config-head,
  .insight-head {
    align-items: flex-start;
    flex-direction: column;
  }

  .path-summary {
    grid-template-columns: 1fr;
  }
}
</style>
