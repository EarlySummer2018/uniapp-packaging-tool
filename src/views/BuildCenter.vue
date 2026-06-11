<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NAlert,
  NButton,
  NCard,
  NCheckbox,
  NFormItem,
  NGi,
  NGrid,
  NIcon,
  NInput,
  NProgress,
  NSelect,
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
type NonIosPlatform = Exclude<Platform, 'ios'>

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
  permissions?: string[]
  excludePermissions?: string[]
  schemes?: string[]
  abiFilters?: string[]
}

interface PlatformPackages {
  androidPackage?: string | null
  iosBundleId?: string | null
  harmonyBundle?: string | null
}

interface SplashscreenConfig {
  androidStyle?: string | null
  android: Record<string, string>
  iosStyle?: string | null
  iosStoryboard?: string | null
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
  iosPrivacyDescriptions?: Record<string, string>
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
  fieldType?: string
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

type ModuleStatusTone = 'default' | 'success' | 'warning' | 'error'

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
const selectedManifestModuleKeys = ref<Set<string>>(new Set())
const manifestModuleSelectionTouched = ref(false)
const activeAndroidConfigModuleKey = ref<string | null>(null)

let androidModuleConfigSaveTimer: ReturnType<typeof setTimeout> | null = null

const platforms = [
  { key: 'android' as const, label: 'Android', icon: LogoAndroid, description: 'APK 安装包', color: '#2f9e44', bgColor: '#e8f5e9' },
  { key: 'ios' as const, label: 'iOS', icon: LogoApple, description: '离线 SDK / IPA', color: '#1c7ed6', bgColor: '#e7f5ff' },
  { key: 'harmony' as const, label: '鸿蒙', icon: PhonePortraitOutline, description: 'HAP 安装包', color: '#d6336c', bgColor: '#fff0f6' }
]

const selectedNeedsAndroidConfig = computed(() => selectedPlatforms.value.includes('android'))
const selectedNeedsIosConfig = computed(() => selectedPlatforms.value.includes('ios'))
const isBuildLocked = computed(() => buildStore.hasActiveBuilds)
const activeProjectBuild = computed(() => buildStore.getActiveBuildForProject(projectId.value))
const currentBuild = computed(() => {
  if (currentBuildId.value) return buildStore.getBuild(currentBuildId.value) || null
  return activeProjectBuild.value || null
})
const packageBuildLoading = computed(() => currentBuild.value?.status === 'building' && currentBuild.value.kind === 'package')
const androidGenerateLoading = computed(() => currentBuild.value?.status === 'building' && currentBuild.value.kind === 'generateAndroidProject')
const iosGenerateLoading = computed(() => currentBuild.value?.status === 'building' && currentBuild.value.kind === 'generateIosProject')
const harmonyGenerateLoading = computed(() => currentBuild.value?.status === 'building' && currentBuild.value.kind === 'generateHarmonyProject')
const singleSelectedPlatform = computed<Platform | null>(() => selectedPlatforms.value.length === 1 ? selectedPlatforms.value[0] : null)
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
  return report.modules
    .filter(mod => isAndroidConfigModuleSelected(mod))
    .flatMap(mod => mod.fields
      .filter(field => field.required && !androidFieldValue(mod, field).trim())
      .map(field => ({ moduleName: mod.name, key: field.key, label: field.label })))
})
const iosMissingRequired = computed(() => {
  if (!selectedNeedsIosConfig.value) return []
  const ios = currentProject.value?.ios
  if (!ios) return ['iOS 项目配置']
  const missing: string[] = []
  if (!ios.dcloudAppKey?.trim()) missing.push('DCloud AppKey')
  if (!ios.bundleId?.trim()) missing.push('Bundle ID')
  if (!ios.teamId?.trim()) missing.push('Team ID')
  if (!ios.provisioningProfile?.trim()) missing.push('描述文件')
  return missing
})
const iosBuildReady = computed(() => !selectedNeedsIosConfig.value || iosMissingRequired.value.length === 0)
const iosIconCount = computed(() => Object.keys(latestManifestInfo.value?.iosIcons?.ios || {}).length)
const iosPrivacyDescriptionCount = computed(() => Object.keys(latestManifestInfo.value?.iosPrivacyDescriptions || {}).length)
const canBuild = computed(() => {
  if (!scanResult.value || selectedPlatforms.value.length === 0 || isBuildLocked.value) return false
  if (selectedNeedsAndroidConfig.value && !androidModulesReady.value) return false
  if (!iosBuildReady.value) return false
  return true
})
const canGenerateAndroid = computed(() => {
  if (!canGenerateNativeProject('android')) return false
  if (selectedNeedsAndroidConfig.value && !androidModulesReady.value) return false
  return true
})
const canGenerateIos = computed(() => canGenerateIosProject())
const canGenerateHarmony = computed(() => canGenerateNativeProject('harmony'))
const currentGeneratedProjectLabel = computed(() => {
  const platform = currentBuild.value?.platform
  return platform ? `${platformProjectName(platform)}项目已生成` : '项目已生成'
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
  if (selectedNeedsIosConfig.value && iosMissingRequired.value.length) {
    return `还有 ${iosMissingRequired.value.length} 个 iOS 必填配置未填写`
  }
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
const selectedManifestModules = computed(() => manifestModules.value.filter(mod => isManifestModuleSelected(mod)))
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
const androidConfigModulesByKey = computed(() => {
  const modules = androidModuleConfigReport.value?.modules || []
  return new Map(modules.map(mod => [androidConfigModuleKey(mod), mod]))
})
const androidConfigurableModules = computed(() => {
  const modules = androidModuleConfigReport.value?.modules || []
  return modules.filter(mod => mod.fields.length > 0 && isAndroidConfigModuleSelected(mod))
})
const activeAndroidConfigModule = computed(() => {
  if (!activeAndroidConfigModuleKey.value) return null
  return androidConfigurableModules.value.find(mod => androidConfigModuleKey(mod) === activeAndroidConfigModuleKey.value) || null
})

watch(manifestModules, modules => {
  selectedManifestModuleKeys.value = new Set(modules.map(mod => manifestModuleKey(mod)))
  manifestModuleSelectionTouched.value = false
}, { immediate: true })

watch(androidConfigurableModules, modules => {
  if (!modules.length) {
    activeAndroidConfigModuleKey.value = null
    return
  }
  const currentKey = activeAndroidConfigModuleKey.value
  if (currentKey && modules.some(mod => androidConfigModuleKey(mod) === currentKey)) return
  activeAndroidConfigModuleKey.value = androidConfigModuleKey(preferredAndroidConfigModule(modules))
}, { immediate: true })

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

function canGenerateNativeProject(platform: Platform) {
  return !!scanResult.value && singleSelectedPlatform.value === platform && !isBuildLocked.value
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
  const buildManifestInfo = selectedManifestInfoForBuild(manifestInfo)
  const androidModuleConfig = buildAndroidModuleConfigPayload()
  await persistAndroidModuleConfigCache()
  let lastBuildId: string | null = null
  const buildIds: string[] = []
  for (const platform of selectedPlatforms.value) {
    const buildId = platform === 'ios'
      ? await buildIosIpa(runProjectId, runProjectName, importedResourcePath, buildManifestInfo)
      : await buildStandardPackage(platform, runProjectId, runProjectName, importedResourcePath, buildManifestInfo, androidModuleConfig)
    lastBuildId = buildId
    buildIds.push(buildId)
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

async function buildIosIpa(
  runProjectId: string,
  runProjectName: string,
  importedResourcePath: string,
  buildManifestInfo: UniappManifestInfo
) {
  const platform: Platform = 'ios'
  const buildId = buildStore.startBuild(runProjectId, platform, 'package')
  currentBuildId.value = buildId
  buildStore.setActiveEventBuildId(buildId)
  const startedAt = new Date()
  await createBuildRecord(buildId, platform, startedAt, runProjectId, runProjectName, importedResourcePath)
  await appendManifestLog(buildId, buildManifestInfo, platform)
  await buildStore.appendBuildLogLines(buildId, [
    '[info] iOS 离线 SDK 流程: 复制 SDK 自带 HBuilder-Hello* 并配置 workspace 副本',
    `[info] iOS 图标配置: ${iosIconCount.value} 项，隐私描述: ${iosPrivacyDescriptionCount.value} 项`
  ])
  try {
    const artifact = await invoke<BuildArtifact>('build_ios_ipa', {
      projectId: runProjectId,
      resourcePath: importedResourcePath,
      buildId,
      manifestInfo: buildManifestInfo
    })
    await buildStore.flushBuildLogs(buildId)
    await appendFinalLog(buildId, 'success')
    await finalizeBuildRecord(buildId, 'success', startedAt, artifact)
    buildStore.stopBuild(buildId, true, {
      artifactPath: artifact.path,
      artifactSizeBytes: artifact.sizeBytes
    })
    message.success('iOS IPA 构建完成')
  } catch (e: any) {
    await buildStore.flushBuildLogs(buildId)
    await appendFinalLog(buildId, 'failed', String(e))
    await finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
    buildStore.failBuild(buildId, String(e))
    message.error(`iOS IPA 构建失败: ${String(e)}`)
  } finally {
    await cleanupBuildTemporaryFiles(buildId, buildId, null)
    await buildStore.flushBuildLogs(buildId)
  }
  return buildId
}

async function buildStandardPackage(
  platform: NonIosPlatform,
  runProjectId: string,
  runProjectName: string,
  importedResourcePath: string,
  buildManifestInfo: UniappManifestInfo,
  androidModuleConfig: Record<string, string>
) {
  const buildId = buildStore.startBuild(runProjectId, platform, 'package')
  currentBuildId.value = buildId
  buildStore.setActiveEventBuildId(buildId)
  const startedAt = new Date()
  await createBuildRecord(buildId, platform, startedAt, runProjectId, runProjectName, importedResourcePath)
  await appendManifestLog(buildId, buildManifestInfo, platform)
  try {
    const command = platform === 'android' ? 'build_android_apk' : 'build_harmony_hap'
    const artifact = await invoke<BuildArtifact>(command, {
      projectId: runProjectId,
      resourcePath: importedResourcePath,
      buildId,
      manifestInfo: buildManifestInfo,
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
  return buildId
}

async function generateAndroidProject() {
  await generateNativeProject('android')
}

async function generateIosProject() {
  await generateIosOfflineProject()
}

async function generateHarmonyProject() {
  await generateNativeProject('harmony')
}

function canGenerateIosProject() {
  return !!scanResult.value && singleSelectedPlatform.value === 'ios' && !isBuildLocked.value && iosBuildReady.value
}

async function generateIosOfflineProject() {
  if (!canGenerateIosProject()) {
    if (isBuildLocked.value) message.warning('已有构建任务进行中，请等待完成后再生成项目')
    return
  }
  const runProjectId = projectId.value
  const runProjectName = getProjectName()
  const importedResourcePath = scanResult.value!.importedPath
  let manifestInfo: UniappManifestInfo
  try {
    manifestInfo = await ensureManifestInfoLoaded({ persist: true })
  } catch (e: any) {
    message.error(String(e))
    return
  }
  const buildManifestInfo = selectedManifestInfoForBuild(manifestInfo)
  await persistAndroidModuleConfigCache()
  const buildId = buildStore.startBuild(runProjectId, 'ios', 'generateIosProject')
  currentBuildId.value = buildId
  buildStore.setActiveEventBuildId(buildId)
  const startedAt = new Date()
  await createBuildRecord(buildId, 'ios', startedAt, runProjectId, runProjectName, importedResourcePath)
  await appendManifestLog(buildId, buildManifestInfo, 'ios')
  await buildStore.appendBuildLogLines(buildId, [
    '[info] iOS 工程生成: 复制 SDK 自带 HBuilder-Hello* 后配置 workspace 副本',
    `[info] iOS 图标配置: ${iosIconCount.value} 项，隐私描述: ${iosPrivacyDescriptionCount.value} 项`
  ])
  try {
    const projectPath = await invoke<string>('generate_ios_project', {
      projectId: runProjectId,
      resourcePath: importedResourcePath,
      buildId,
      manifestInfo: buildManifestInfo
    })
    await buildStore.flushBuildLogs(buildId)
    await appendFinalLog(buildId, 'success')
    await finalizeBuildRecord(buildId, 'success', startedAt, null)
    buildStore.stopBuild(buildId, true, { generatedProjectPath: projectPath })
    message.success(`iOS 工程已生成: ${projectPath}`)
  } catch (e: any) {
    await buildStore.flushBuildLogs(buildId)
    await appendFinalLog(buildId, 'failed', String(e))
    await finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
    buildStore.failBuild(buildId, String(e))
    message.error(`生成 iOS 工程失败: ${String(e)}`)
  } finally {
    await buildStore.flushBuildLogs(buildId)
    if (buildStore.activeEventBuildId === buildId) buildStore.setActiveEventBuildId(null)
  }
}

async function generateNativeProject(platform: NonIosPlatform) {
  if (!scanResult.value || singleSelectedPlatform.value !== platform || isBuildLocked.value) {
    if (isBuildLocked.value) message.warning('已有构建任务进行中，请等待完成后再生成项目')
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
  if (platform === 'android' && !(await ensureAndroidModuleConfigReadyForBuild())) {
    return
  }
  const buildManifestInfo = selectedManifestInfoForBuild(manifestInfo)
  const moduleConfig = platform === 'android' ? buildAndroidModuleConfigPayload() : undefined
  await persistAndroidModuleConfigCache()
  const buildId = buildStore.startBuild(runProjectId, platform, generateProjectKind(platform))
  currentBuildId.value = buildId
  buildStore.setActiveEventBuildId(buildId)
  const startedAt = new Date()
  await createBuildRecord(buildId, platform, startedAt, runProjectId, runProjectName, importedResourcePath)
  await appendManifestLog(buildId, buildManifestInfo, platform)
  try {
    const payload: Record<string, unknown> = {
      projectId: runProjectId,
      resourcePath: importedResourcePath,
      buildId,
      manifestInfo: buildManifestInfo
    }
    if (moduleConfig) payload.moduleConfig = moduleConfig
    const projectPath = await invoke<string>(generateProjectCommand(platform), payload)
    await buildStore.flushBuildLogs(buildId)
    await appendFinalLog(buildId, 'success')
    await finalizeBuildRecord(buildId, 'success', startedAt, null)
    buildStore.stopBuild(buildId, true, { generatedProjectPath: projectPath })
    message.success(`${platformProjectName(platform)}项目已生成: ${projectPath}`)
  } catch (e: any) {
    await buildStore.flushBuildLogs(buildId)
    await appendFinalLog(buildId, 'failed', String(e))
    await finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
    buildStore.failBuild(buildId, String(e))
    message.error(`生成${platformProjectName(platform)}项目失败: ${String(e)}`)
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

function platformProjectName(platform: Platform) {
  if (platform === 'android') return '安卓'
  if (platform === 'ios') return '苹果'
  return '鸿蒙'
}

function generateProjectKind(platform: NonIosPlatform) {
  if (platform === 'android') return 'generateAndroidProject' as const
  return 'generateHarmonyProject' as const
}

function generateProjectCommand(platform: NonIosPlatform) {
  if (platform === 'android') return 'generate_android_project'
  return 'generate_harmony_project'
}

function formatPlatforms(platforms: string[]) {
  return platforms.filter(platform => platform && platform !== 'all').join(' / ')
}

function formatModuleWithPlatforms(mod: { name: string; platforms: string[] }) {
  const platforms = formatPlatforms(mod.platforms)
  return platforms ? `${mod.name}(${platforms})` : mod.name
}

function moduleKeyParts(name: string, category: string, platforms: string[], source: string) {
  return [name, category, platforms.join('|'), source || 'manifest.json'].join('::')
}

function manifestModuleKey(mod: DetectedModule) {
  return moduleKeyParts(mod.name, mod.category, mod.platforms, mod.source)
}

function androidConfigModuleKey(mod: AndroidModuleConfigModule) {
  return moduleKeyParts(mod.name, mod.category, mod.platforms, mod.source)
}

function androidModuleFieldValueKey(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  return `${mod.templateKey}.${field.key}`
}

function isManifestModuleSelected(mod: DetectedModule) {
  return selectedManifestModuleKeys.value.has(manifestModuleKey(mod))
}

function isAndroidConfigModuleSelected(mod: AndroidModuleConfigModule) {
  return selectedManifestModuleKeys.value.has(androidConfigModuleKey(mod))
}

function setManifestModuleSelected(mod: DetectedModule, checked: boolean) {
  if (isBuildLocked.value) return
  manifestModuleSelectionTouched.value = true
  const key = manifestModuleKey(mod)
  const next = new Set(selectedManifestModuleKeys.value)
  if (checked) next.add(key)
  else next.delete(key)
  selectedManifestModuleKeys.value = next

  const configModule = androidConfigModulesByKey.value.get(key)
  if (checked && configModule?.fields.length) {
    activeAndroidConfigModuleKey.value = androidConfigModuleKey(configModule)
  } else if (!checked && activeAndroidConfigModuleKey.value === key) {
    activeAndroidConfigModuleKey.value = androidConfigurableModules.value.length
      ? androidConfigModuleKey(preferredAndroidConfigModule(androidConfigurableModules.value))
      : null
  }
}

function manifestConfigModule(mod: DetectedModule) {
  return androidConfigModulesByKey.value.get(manifestModuleKey(mod)) || null
}

function configFieldFilled(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  return androidFieldValue(mod, field).trim().length > 0
}

function configModuleMissingRequiredCount(mod: AndroidModuleConfigModule) {
  return mod.fields.filter(field => field.required && !configFieldFilled(mod, field)).length
}

function configModuleFilledCount(mod: AndroidModuleConfigModule) {
  return mod.fields.filter(field => configFieldFilled(mod, field)).length
}

function configModuleStatusTone(mod: AndroidModuleConfigModule): ModuleStatusTone {
  if (!mod.fields.length) return 'success'
  if (configModuleMissingRequiredCount(mod) === 0) return 'success'
  return configModuleFilledCount(mod) > 0 ? 'warning' : 'error'
}

function configModuleStatusLabel(mod: AndroidModuleConfigModule) {
  const missing = configModuleMissingRequiredCount(mod)
  if (!mod.fields.length) return '已选'
  if (missing === 0) return '已配置'
  if (configModuleFilledCount(mod) > 0) return '部分配置'
  return '需配置'
}

function preferredAndroidConfigModule(modules: AndroidModuleConfigModule[]) {
  return modules.find(mod => configModuleStatusTone(mod) === 'error')
    || modules.find(mod => configModuleStatusTone(mod) === 'warning')
    || modules[0]
}

function manifestModuleStatusTone(mod: DetectedModule): ModuleStatusTone {
  if (!isManifestModuleSelected(mod)) return 'default'
  const configModule = manifestConfigModule(mod)
  if (!configModule) return 'success'
  return configModuleStatusTone(configModule)
}

function manifestModuleStatusType(mod: DetectedModule) {
  return manifestModuleStatusTone(mod)
}

function manifestModuleStatusClass(mod: DetectedModule) {
  return `module-choice--${manifestModuleStatusTone(mod)}`
}

function manifestModuleStatusLabel(mod: DetectedModule) {
  if (!isManifestModuleSelected(mod)) return '未勾选'
  const configModule = manifestConfigModule(mod)
  if (!configModule) return '已选'
  return configModuleStatusLabel(configModule)
}

function androidConfigModuleStatusType(mod: AndroidModuleConfigModule) {
  return configModuleStatusTone(mod)
}

function openAndroidConfigModule(mod: AndroidModuleConfigModule) {
  activeAndroidConfigModuleKey.value = androidConfigModuleKey(mod)
}

function selectedManifestInfoForBuild(info: UniappManifestInfo): UniappManifestInfo {
  if (!manifestModuleSelectionTouched.value && selectedManifestModuleKeys.value.size === 0) {
    return info
  }
  return {
    ...info,
    detectedModules: info.detectedModules.filter(mod => isManifestModuleSelected(mod))
  }
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
  const cached = cachedAndroidModuleConfig()
  for (const mod of report.modules) {
    for (const field of mod.fields) {
      const scopedKey = androidModuleFieldValueKey(mod, field)
      next[scopedKey] = cached[scopedKey] ?? field.value ?? cached[field.key] ?? ''
    }
  }
  androidModuleConfigValues.value = next
  syncAndroidModuleConfigCache()
  scheduleAndroidModuleConfigCacheSave()
}

function androidFieldValue(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  return androidModuleConfigValues.value[androidModuleFieldValueKey(mod, field)]
    ?? androidModuleConfigValues.value[field.key]
    ?? field.value
    ?? ''
}

function updateAndroidField(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField, value: string) {
  if (isBuildLocked.value) return
  androidModuleConfigValues.value = {
    ...androidModuleConfigValues.value,
    [androidModuleFieldValueKey(mod, field)]: value
  }
  syncAndroidModuleConfigCache()
  scheduleAndroidModuleConfigCacheSave()
}

function updateActiveAndroidField(field: AndroidModuleConfigField, value: string) {
  const mod = activeAndroidConfigModule.value
  if (!mod) return
  updateAndroidField(mod, field, value)
}

async function pickAndroidFileField(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
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
    updateAndroidField(mod, field, base64)
  } catch (e) {
    message.error('读取文件失败: ' + String(e))
  }
}

function clearAndroidFileField(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  if (isBuildLocked.value) return
  updateAndroidField(mod, field, '')
}

function isFileField(field: AndroidModuleConfigField): boolean {
  return androidFieldType(field) === 'file'
}

function isSelectField(field: AndroidModuleConfigField): boolean {
  return androidFieldType(field) === 'select'
}

function selectFieldOptions(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  if (mod.templateKey === 'map' && field.key === 'MAP_PAGE_TYPE') {
    const provider = mapProviderForModule(mod)
    return [
      { label: 'vue', value: 'vue', disabled: provider === 'google' },
      { label: 'nvue', value: 'nvue', disabled: provider === 'baidu' }
    ]
  }
  return []
}

function mapProviderForModule(mod: AndroidModuleConfigModule) {
  if (mod.fields.some(field => field.key === 'BAIDU_MAP_AK')) return 'baidu'
  if (mod.fields.some(field => field.key === 'AMAP_KEY')) return 'amap'
  if (mod.fields.some(field => field.key === 'GOOGLE_MAPS_API_KEY')) return 'google'
  if (mod.fields.some(field => field.key === 'TENCENT_MAP_KEY')) return 'tencent'
  return 'amap'
}

function formatFileSize(base64Value: string): string {
  if (!base64Value) return ''
  const kb = Math.ceil((base64Value.length * 3 / 4) / 1024)
  return `${kb}KB`
}

function buildAndroidModuleConfigPayload() {
  const payload: Record<string, string> = {}
  const report = androidModuleConfigReport.value
  if (!report) {
    for (const [key, value] of Object.entries(androidModuleConfigValues.value)) {
      if (value.trim()) payload[key] = value.trim()
    }
    return payload
  }
  for (const mod of report.modules) {
    if (!isAndroidConfigModuleSelected(mod)) continue
    for (const field of mod.fields) {
      const value = androidFieldValue(mod, field).trim()
      if (value) payload[androidModuleFieldValueKey(mod, field)] = value
    }
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
      const value = androidFieldValue(mod, field).trim()
      if (value && field.valueSource !== 'manifest') {
        next[androidModuleFieldValueKey(mod, field)] = value
      }
    }
  }
  project.androidModuleConfig = next
  return true
}

function fieldStatusType(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  const value = androidFieldValue(mod, field).trim()
  if (!value && field.required) return 'error'
  if (field.valueSource === 'manifest' && value) return 'success'
  if (value) return 'info'
  return 'default'
}

function fieldStatusLabel(mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) {
  const value = androidFieldValue(mod, field).trim()
  if (!value && field.required) return '必填'
  if (!value) return '可选'
  if (field.valueSource === 'manifest') return 'manifest'
  if (field.valueSource === 'default') return '默认'
  return '已填写'
}

function androidFieldType(field: AndroidModuleConfigField): string {
  return field.fieldType || field.field_type || 'text'
}

function manifestLogLines(info: UniappManifestInfo, platform: Platform) {
  const moduleNames = info.detectedModules.map(mod => formatModuleWithPlatforms(mod))
  const lines = [
    `[info] 已读取 manifest.json: ${info.manifestPath}`,
    `[info] manifest 摘要: ${info.appName || '-'} / ${info.appId || '-'} / v${info.versionName || '-'} (${info.versionCode ?? '-'})`
  ]
  if (platform === 'android') {
    lines.push(`[info] Android SDK: min ${info.android.minSdkVersion ?? '-'}, target ${info.android.targetSdkVersion ?? '-'}, compile ${info.android.compileSdkVersion ?? '-'}`)
  }
  lines.push(`[info] manifest 模块 (${moduleNames.length}): ${moduleNames.length ? moduleNames.join(', ') : '无'}`)
  const report = androidModuleConfigReport.value
  const configurableModules = platform === 'android'
    ? report?.modules.filter(mod => mod.fields.length > 0) || []
    : []
  if (configurableModules.length) {
    let totalFields = 0
    let configuredFields = 0
    const missingRequired: string[] = []
    const missingOptional: string[] = []
    for (const mod of configurableModules) {
      for (const field of mod.fields) {
        totalFields += 1
        const value = androidFieldValue(mod, field).trim()
        if (value) {
          configuredFields += 1
        } else if (field.required) {
          missingRequired.push(`${mod.name} / ${field.label}`)
        } else {
          missingOptional.push(`${mod.name} / ${field.label}`)
        }
      }
    }
    lines.push(`[info] Android 模块配置: ${configurableModules.length} 个模块，${configuredFields}/${totalFields} 项已填写`)
    for (const item of missingRequired) lines.push(`[warn] 缺失 Android 必填配置: ${item}`)
    if (missingOptional.length) lines.push(`[info] 未填写可选配置: ${missingOptional.join('、')}`)
  }
  return lines
}

async function appendManifestLog(buildId: string, info: UniappManifestInfo, platform: Platform) {
  await buildStore.appendBuildLogLines(buildId, manifestLogLines(info, platform))
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
        <n-card data-guide="resource-import" title="1. 导入 UniApp 资源" class="build-step-card import-card">
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
        <n-card data-guide="platform-select" title="2. 选择平台" class="build-step-card">
          <PlatformCard
            :platforms="platforms"
            :selected-platforms="selectedPlatforms"
            :disabled="isBuildLocked"
            @toggle="togglePlatform"
          />
          <n-space justify="end" class="build-action-row">
            <n-text v-if="buildDisabledReason && !canBuild" depth="3">{{ buildDisabledReason }}</n-text>
            <n-button v-if="singleSelectedPlatform === 'android'" type="primary" :disabled="!canGenerateAndroid" :loading="androidGenerateLoading" @click="generateAndroidProject">
              <template #icon><n-icon><LogoAndroid /></n-icon></template>
              生成安卓项目
            </n-button>
            <n-button v-if="singleSelectedPlatform === 'ios'" type="primary" :disabled="!canGenerateIos" :loading="iosGenerateLoading" @click="generateIosProject">
              <template #icon><n-icon><LogoApple /></n-icon></template>
              生成苹果项目
            </n-button>
            <n-button v-if="singleSelectedPlatform === 'harmony'" type="primary" :disabled="!canGenerateHarmony" :loading="harmonyGenerateLoading" @click="generateHarmonyProject">
              <template #icon><n-icon><PhonePortraitOutline /></n-icon></template>
              生成鸿蒙项目
            </n-button>
            <n-button type="success" :disabled="!canBuild" :loading="packageBuildLoading" @click="startBuild">
              <template #icon><n-icon><PlayOutline /></n-icon></template>
              开始打包
            </n-button>
          </n-space>
        </n-card>
      </n-gi>
    </n-grid>
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
              <n-text depth="3">已选模块</n-text>
              <n-text strong class="summary-value">{{ selectedManifestModules.length }} / {{ manifestModules.length }}</n-text>
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

        <div class="module-grid">
          <div class="module-box module-box--manifest">
            <div class="module-box-head">
              <n-text strong>Manifest 模块</n-text>
              <n-text v-if="manifestModules.length" depth="3">{{ selectedManifestModules.length }} / {{ manifestModules.length }} 已选</n-text>
            </div>
            <div v-if="manifestModules.length" class="module-choice-grid">
              <n-checkbox
                v-for="mod in manifestModules"
                :key="manifestModuleKey(mod)"
                class="module-choice"
                :class="manifestModuleStatusClass(mod)"
                :checked="isManifestModuleSelected(mod)"
                :disabled="isBuildLocked"
                @update:checked="(checked: boolean) => setManifestModuleSelected(mod, checked)"
              >
                <span class="module-choice-content">
                  <span class="module-choice-main">{{ mod.name }}</span>
                  <span v-if="formatPlatforms(mod.platforms)" class="module-choice-platform">{{ formatPlatforms(mod.platforms) }}</span>
                  <n-tag size="tiny" :type="manifestModuleStatusType(mod)" :bordered="false">
                    {{ manifestModuleStatusLabel(mod) }}
                  </n-tag>
                </span>
              </n-checkbox>
            </div>
            <n-text v-else depth="3" class="module-empty">未声明 App 模块</n-text>
          </div>
          <div class="module-box">
            <n-text strong>UTS 插件</n-text>
            <n-space v-if="utsPluginLabels.length" wrap :size="8" class="tag-row">
              <n-tag v-for="label in utsPluginLabels" :key="label" type="success">{{ label }}</n-tag>
            </n-space>
            <n-text v-else depth="3" class="module-empty">未检测到 UTS 插件</n-text>
          </div>
        </div>
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

    <n-card v-if="scanResult && selectedPlatforms.includes('ios')" title="iOS 离线 SDK 工程" class="build-section-card">
      <n-space vertical :size="14">
        <n-alert :type="iosMissingRequired.length ? 'warning' : 'success'">
          <n-space vertical :size="6">
            <n-text v-if="iosMissingRequired.length">
              缺少 {{ iosMissingRequired.join('、') }}，补齐后才能生成 iOS 工程或 IPA。
            </n-text>
            <n-text v-else>iOS 基础配置已就绪，将使用 SDK 管理中配置的 HBuilder-Hello* 工程副本。</n-text>
          </n-space>
        </n-alert>
        <n-grid :cols="4" :x-gap="12" :y-gap="12" responsive="screen" class="insight-grid">
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">Bundle ID</n-text>
              <n-text strong class="summary-text">{{ currentProject?.ios.bundleId || '-' }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">Team ID</n-text>
              <n-text strong class="summary-text">{{ currentProject?.ios.teamId || '-' }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">iOS 图标</n-text>
              <n-text strong class="summary-value">{{ iosIconCount }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">隐私描述</n-text>
              <n-text strong class="summary-value">{{ iosPrivacyDescriptionCount }}</n-text>
            </div>
          </n-gi>
        </n-grid>
        <div class="path-summary">
          <n-text depth="3">工程来源</n-text>
          <n-text code>SDK 管理 / DCloud iOS 离线 SDK / HBuilder-Hello*</n-text>
          <n-text depth="3">App 资源</n-text>
          <n-text code>Pandora/apps/{{ insightAppId }}</n-text>
          <n-text depth="3">模块处理</n-text>
          <n-text code>本轮未启用</n-text>
        </div>
      </n-space>
    </n-card>

    <n-card v-if="scanResult && selectedPlatforms.includes('android')" title="Android 模块配置" class="build-section-card">
      <n-space vertical :size="14">
        <n-alert v-if="androidModuleConfigLoading" type="info">正在从 manifest 解析 Android 模块配置...</n-alert>
        <n-alert v-else-if="!latestManifestInfo" type="warning">
          {{ manifestReadWarning || '请先在项目配置中设置本地项目路径，以便读取 manifest.json' }}
        </n-alert>
        <n-alert v-else-if="!androidConfigurableModules.length" type="success">
          已选模块暂无需要额外配置项的 Android 模块。
        </n-alert>
        <n-alert v-else :type="androidMissingRequired.length ? 'warning' : 'success'">
          <n-space vertical :size="6">
            <n-text>
              已选 {{ selectedManifestModules.length }} 个 Manifest 模块，其中 {{ androidConfigurableModules.length }} 个需要 Android 配置。
            </n-text>
            <n-text v-if="androidMissingRequired.length">
              还有 {{ androidMissingRequired.length }} 个必填项未填写，填写完成后才能开始打包。
            </n-text>
            <n-text v-else>模块配置已就绪，可以开始 Android 打包。</n-text>
          </n-space>
        </n-alert>

        <div v-if="androidConfigurableModules.length" class="android-config-list">
          <n-space wrap :size="8" class="android-config-switcher">
            <n-tag
              v-for="mod in androidConfigurableModules"
              :key="androidConfigModuleKey(mod)"
              class="android-config-chip"
              :class="{ 'android-config-chip--active': activeAndroidConfigModuleKey === androidConfigModuleKey(mod) }"
              :type="androidConfigModuleStatusType(mod)"
              :bordered="activeAndroidConfigModuleKey !== androidConfigModuleKey(mod)"
              @click="openAndroidConfigModule(mod)"
            >
              {{ mod.name }} · {{ configModuleStatusLabel(mod) }}
            </n-tag>
          </n-space>

          <div v-if="activeAndroidConfigModule" class="android-config-module">
            <div class="android-config-head">
              <n-space align="center" :size="8">
                <n-text strong>{{ activeAndroidConfigModule.name }}</n-text>
                <n-tag size="small" type="info">{{ activeAndroidConfigModule.category }}</n-tag>
                <n-tag v-if="formatPlatforms(activeAndroidConfigModule.platforms)" size="small" :type="activeAndroidConfigModule.platforms.includes('android') ? 'success' : 'default'">{{ formatPlatforms(activeAndroidConfigModule.platforms) }}</n-tag>
              </n-space>
              <n-text depth="3">{{ activeAndroidConfigModule.fields.length }} 项配置</n-text>
            </div>
            <n-grid :cols="2" :x-gap="14" :y-gap="10" responsive="screen">
              <n-gi v-for="field in activeAndroidConfigModule.fields" :key="activeAndroidConfigModule.templateKey + field.key">
                <n-form-item :label="field.label" :feedback="field.required && !androidFieldValue(activeAndroidConfigModule, field).trim() ? '必填项，未填写时不能开始打包' : undefined">
                  <template #label>
                    <n-space align="center" :size="6">
                      <n-text>{{ field.label }}</n-text>
                      <n-tag size="tiny" :type="fieldStatusType(activeAndroidConfigModule, field)">{{ fieldStatusLabel(activeAndroidConfigModule, field) }}</n-tag>
                    </n-space>
                  </template>
                  <template v-if="isFileField(field)">
                    <n-space :size="8" align="center" class="file-field-row">
                      <n-button size="small" :disabled="isBuildLocked" @click="pickAndroidFileField(activeAndroidConfigModule, field)">选择文件</n-button>
                      <n-text v-if="androidFieldValue(activeAndroidConfigModule, field)" depth="3" class="file-field-hint">
                        已选择 ({{ formatFileSize(androidFieldValue(activeAndroidConfigModule, field)) }})
                      </n-text>
                      <n-button v-if="androidFieldValue(activeAndroidConfigModule, field)" size="small" quaternary type="error" :disabled="isBuildLocked" @click="clearAndroidFileField(activeAndroidConfigModule, field)">清除</n-button>
                      <n-text v-else depth="3" class="file-field-hint">{{ field.placeholder }}</n-text>
                    </n-space>
                  </template>
                  <n-select
                    v-else-if="isSelectField(field)"
                    :value="androidFieldValue(activeAndroidConfigModule, field)"
                    :options="selectFieldOptions(activeAndroidConfigModule, field)"
                    :placeholder="field.placeholder"
                    :disabled="isBuildLocked"
                    @update:value="(value: string) => updateActiveAndroidField(field, value)"
                  />
                  <n-input
                    v-else
                    :value="androidFieldValue(activeAndroidConfigModule, field)"
                    :placeholder="field.placeholder"
                    :type="field.secret ? 'password' : 'text'"
                    :show-password-on="field.secret ? 'click' : undefined"
                    :disabled="isBuildLocked"
                    @update:value="(value: string) => updateActiveAndroidField(field, value)"
                  />
                </n-form-item>
              </n-gi>
            </n-grid>
          </div>
        </div>
      </n-space>
    </n-card>

    <n-card data-guide="build-log" title="构建日志" class="build-section-card log-section-card">
      <LogPanel :logs="currentBuild?.logs || []" height="380px" />
      <n-progress
        class="build-progress"
        type="line"
        indicator-placement="inside"
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
            <span>{{ currentGeneratedProjectLabel }}:</span>
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

.insight-grid {
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

.summary-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.module-section {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.module-grid {
  margin-top: 14px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(260px, 360px);
  gap: 14px;
  align-items: start;
}

.module-box {
  min-height: 96px;
  padding: 12px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-color);
}

.module-box-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.tag-row {
  margin-top: 10px;
}

.module-choice-grid {
  margin-top: 10px;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
  gap: 8px;
}

.module-choice {
  --module-choice-bg: var(--surface-color);
  --module-choice-border: var(--border-soft);
  --module-choice-text: inherit;
  width: 100%;
  min-height: 34px;
  padding: 6px 8px;
  border: 1px solid var(--module-choice-border);
  border-radius: 6px;
  background: var(--module-choice-bg);
  color: var(--module-choice-text);
  transition: border-color 0.16s ease, background-color 0.16s ease;
}

.module-choice--success {
  --module-choice-bg: #f0fdf4;
  --module-choice-border: #86d7a2;
}

.module-choice--warning {
  --module-choice-bg: #fff8e1;
  --module-choice-border: #f3c969;
}

.module-choice--error {
  --module-choice-bg: #fff1f0;
  --module-choice-border: #ef9a9a;
}

.module-choice--default {
  --module-choice-bg: #f6f7f9;
  --module-choice-border: #d8dde6;
  --module-choice-text: #8a929e;
}

.module-choice :deep(.n-checkbox__label) {
  flex: 1;
  min-width: 0;
  padding-left: 8px;
  color: var(--module-choice-text);
}

.module-choice-content {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 6px;
  width: 100%;
  min-width: 0;
}

.module-choice-main,
.module-choice-platform {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.module-choice-main {
  font-weight: 500;
}

.module-choice-platform {
  color: var(--text-muted);
  font-size: 12px;
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

.android-config-switcher {
  margin-bottom: 2px;
}

.android-config-chip {
  cursor: pointer;
}

.android-config-chip--active {
  box-shadow: 0 0 0 2px rgba(24, 160, 88, 0.12);
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

  .module-grid {
    grid-template-columns: 1fr;
  }
}
</style>
