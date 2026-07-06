import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useDialog, useMessage } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener'
import { createBuildCenterActions } from './buildCenterActions'
import { createBuildCenterRecords } from './buildCenterRecords'
import { applyIosModuleConfigToManifestInfo as applyIosModuleConfigToManifest } from './buildCenterIosManifestApply'
import { createModuleStatusHelpers, iosPrivacyModuleLabel, iosPrivacyPermissionLabel, isIosInlineConfigField, isIosLocalPodField, stripIosLocalPodFields } from './buildCenterModuleHelpers'
import { invoke } from '@tauri-apps/api/core'
import { useBuildStore } from '../../stores/build'
import { useProjectsStore } from '../../stores/projects'
import type {
  AndroidModuleConfigField,
  AndroidModuleConfigModule,
  AndroidModuleConfigReport,
  BuildArtifact,
  DetectedModule,
  HarmonyModuleConfigField,
  HarmonyModuleConfigModule,
  HarmonyModuleConfigReport,
  IosModuleConfigField,
  IosModuleConfigModule,
  IosModuleConfigReport,
  IosPrivacyDescriptionItem,
  Platform,
  ResourceScanResult,
  UniappManifestInfo
} from './types'
import {
  androidConfigModuleKey,
  androidModuleFieldValueKey,
  harmonyConfigModuleKey,
  harmonyModuleFieldValueKey,
  iosConfigModuleKey,
  iosModuleFieldValueKey,
  isIosPrivacyField,
  manifestModuleKey,
  platformProjectName
} from './moduleKeys'
import { normalizeIosFieldValue } from './moduleFields'

export function useBuildCenterController() {
const route = useRoute()
const router = useRouter()
const message = useMessage()
const dialog = useDialog()
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
const iosModuleConfigReport = ref<IosModuleConfigReport | null>(null)
const iosModuleConfigLoading = ref(false)
const iosModuleConfigValues = ref<Record<string, string>>({})
const harmonyModuleConfigReport = ref<HarmonyModuleConfigReport | null>(null)
const harmonyModuleConfigLoading = ref(false)
const selectedManifestModuleKeys = ref<Set<string>>(new Set())
const manifestModuleSelectionTouched = ref(false)
const activeAndroidConfigModuleKey = ref<string | null>(null)
const activeIosConfigModuleKey = ref<string | null>(null)
const iosPrivacyDialogVisible = ref(false)

let androidModuleConfigSaveTimer: ReturnType<typeof setTimeout> | null = null
let iosModuleConfigSaveTimer: ReturnType<typeof setTimeout> | null = null

const selectedNeedsAndroidConfig = computed(() => selectedPlatforms.value.includes('android'))
const selectedNeedsIosConfig = computed(() => selectedPlatforms.value.includes('ios'))
const selectedNeedsHarmonyConfig = computed(() => selectedPlatforms.value.includes('harmony'))
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
const iosModuleMissingRequired = computed(() => {
  const report = iosModuleConfigReport.value
  if (!report) return []
  const missing = new Map<string, { moduleName: string; key: string; label: string }>()
  for (const mod of report.modules) {
    if (!isIosConfigModuleSelected(mod)) continue
    for (const field of mod.fields) {
      if (isIosPrivacyField(field)) continue
      if (!field.required || iosFieldValue(mod, field).trim()) continue
      const key = iosModuleFieldValueKey(mod, field)
      if (!missing.has(key)) {
        missing.set(key, { moduleName: mod.name, key: field.key, label: field.label })
      }
    }
  }
  return Array.from(missing.values())
})
const harmonyModuleMissingRequired = computed(() => {
  const report = harmonyModuleConfigReport.value
  if (!report) return []
  const missing = new Map<string, { moduleName: string; key: string; label: string }>()
  for (const mod of report.modules) {
    if (!isHarmonyConfigModuleSelected(mod)) continue
    for (const field of mod.fields) {
      if (!field.required || harmonyFieldValue(mod, field).trim()) continue
      const key = harmonyModuleFieldValueKey(mod, field)
      if (!missing.has(key)) {
        missing.set(key, { moduleName: mod.name, key: field.key, label: field.label })
      }
    }
  }
  return Array.from(missing.values())
})
const iosPrivacyDescriptionItems = computed<IosPrivacyDescriptionItem[]>(() => {
  const report = iosModuleConfigReport.value
  if (!report) return []
  const groups = new Map<string, {
    key: string
    fieldKey: string
    baseLabel: string
    modules: string[]
    required: boolean
    placeholder: string
    value: string
  }>()
  for (const mod of report.modules) {
    if (!isIosConfigModuleSelected(mod)) continue
    for (const field of mod.fields) {
      if (!isIosPrivacyField(field)) continue
      const fieldKey = iosModuleFieldValueKey(mod, field)
      const plistKey = field.key.slice('privacy.'.length)
      const moduleLabel = iosPrivacyModuleLabel(mod)
      const currentValue = iosFieldValue(mod, field)
      const existing = groups.get(fieldKey)
      if (existing) {
        if (!existing.modules.includes(moduleLabel)) existing.modules.push(moduleLabel)
        existing.required = existing.required || field.required
        if (!existing.value.trim() && currentValue.trim()) existing.value = currentValue
        if (!existing.placeholder && field.placeholder) existing.placeholder = field.placeholder
        continue
      }
      groups.set(fieldKey, {
        key: plistKey,
        fieldKey,
        baseLabel: iosPrivacyPermissionLabel(plistKey, field.label),
        modules: [moduleLabel],
        required: field.required,
        placeholder: field.placeholder,
        value: currentValue
      })
    }
  }
  return Array.from(groups.values()).map(item => {
    const value = item.value.trim()
    return {
      key: item.key,
      fieldKey: item.fieldKey,
      label: item.modules.length
        ? `${item.baseLabel}（${item.modules.join('、')}）`
        : item.baseLabel,
      modules: item.modules,
      required: item.required,
      placeholder: item.placeholder,
      value: item.value,
      missing: item.required && !value
    }
  })
})
const iosPrivacyDescriptionMissingCount = computed(() => {
  return iosPrivacyDescriptionItems.value.filter(item => item.missing).length
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
const iosPrivacyDescriptionCount = computed(() => {
  const values = {
    ...(latestManifestInfo.value?.iosPrivacyDescriptions || {}),
    ...buildIosPrivacyDescriptionPayload()
  }
  return Object.keys(values).length
})
const iosModuleSummaryLabel = computed(() => {
  if (iosModuleConfigLoading.value) return '正在分析'
  if (!latestManifestInfo.value) return '未读取 manifest'
  if (!iosConfigurableModules.value.length) return '无 iOS 模块配置'
  return `${iosConfigurableModules.value.length} 个模块需配置`
})
const canBuild = computed(() => {
  if (!scanResult.value || selectedPlatforms.value.length === 0 || isBuildLocked.value) return false
  if (selectedNeedsAndroidConfig.value && !androidModulesReady.value) return false
  if (selectedNeedsIosConfig.value && iosModuleConfigLoading.value) return false
  if (selectedNeedsIosConfig.value && iosModuleMissingRequired.value.length) return false
  if (selectedNeedsHarmonyConfig.value && harmonyModuleConfigLoading.value) return false
  if (selectedNeedsHarmonyConfig.value && harmonyModuleMissingRequired.value.length) return false
  if (!iosBuildReady.value) return false
  return true
})
const canGenerateAndroid = computed(() => {
  if (!canGenerateNativeProject('android')) return false
  if (selectedNeedsAndroidConfig.value && !androidModulesReady.value) return false
  return true
})
const canGenerateIos = computed(() => canGenerateIosProject())
const canGenerateHarmony = computed(() => {
  if (!canGenerateNativeProject('harmony')) return false
  if (selectedNeedsHarmonyConfig.value && harmonyModuleConfigLoading.value) return false
  if (selectedNeedsHarmonyConfig.value && harmonyModuleMissingRequired.value.length) return false
  return true
})
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
  if (selectedNeedsIosConfig.value && iosModuleConfigLoading.value) return '正在分析 iOS 模块配置'
  if (selectedNeedsIosConfig.value && iosModuleMissingRequired.value.length) {
    return `还有 ${iosModuleMissingRequired.value.length} 个 iOS 模块必填配置未填写`
  }
  if (selectedNeedsHarmonyConfig.value && harmonyModuleConfigLoading.value) return '正在分析 Harmony 模块配置'
  if (selectedNeedsHarmonyConfig.value && harmonyModuleMissingRequired.value.length) {
    return `还有 ${harmonyModuleMissingRequired.value.length} 个 Harmony 模块必填配置未填写`
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
const manifestModules = computed(() => latestManifestInfo.value?.detectedModules || scanResult.value?.detectedModules || [])
const selectedManifestModules = computed(() => manifestModules.value.filter(mod => isManifestModuleSelected(mod)))
const insightAppId = computed(() => latestManifestInfo.value?.appId || scanResult.value?.appId || currentProject.value?.app.appId || '-')
const insightVersionName = computed(() => latestManifestInfo.value?.versionName || scanResult.value?.versionName || currentProject.value?.app.version || '-')
const insightVersionCode = computed(() => latestManifestInfo.value?.versionCode ?? scanResult.value?.versionCode ?? currentProject.value?.app.versionCode ?? '-')
const insightManifestPath = computed(() => latestManifestInfo.value?.manifestPath || scanResult.value?.manifestPath || '-')
const androidConfigurableModules = computed(() => {
  const modules = androidModuleConfigReport.value?.modules || []
  return modules.filter(mod => mod.fields.length > 0 && isAndroidConfigModuleSelected(mod))
})
const activeAndroidConfigModule = computed(() => {
  if (!activeAndroidConfigModuleKey.value) return null
  return androidConfigurableModules.value.find(mod => androidConfigModuleKey(mod) === activeAndroidConfigModuleKey.value) || null
})
const iosConfigurableModules = computed(() => {
  const modules = iosModuleConfigReport.value?.modules || []
  return modules.filter(mod => mod.fields.some(isIosInlineConfigField) && isIosConfigModuleSelected(mod))
})
const activeIosConfigModule = computed(() => {
  if (!activeIosConfigModuleKey.value) return null
  return iosConfigurableModules.value.find(mod => iosConfigModuleKey(mod) === activeIosConfigModuleKey.value) || null
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

watch(iosConfigurableModules, modules => {
  if (!modules.length) {
    activeIosConfigModuleKey.value = null
    return
  }
  const currentKey = activeIosConfigModuleKey.value
  if (currentKey && modules.some(mod => iosConfigModuleKey(mod) === currentKey)) return
  activeIosConfigModuleKey.value = iosConfigModuleKey(preferredIosConfigModule(modules))
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
  if (iosModuleConfigSaveTimer) {
    clearTimeout(iosModuleConfigSaveTimer)
    iosModuleConfigSaveTimer = null
  }
  void persistAndroidModuleConfigCache()
  void persistIosModuleConfigCache()
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
  iosModuleConfigReport.value = null
  harmonyModuleConfigReport.value = null
  androidModuleConfigValues.value = {}
  iosModuleConfigValues.value = {}
  try {
    scanResult.value = await invoke<ResourceScanResult>('import_uniapp_resource', {
      projectId: projectId.value,
      resourcePath: path
    })
    await refreshManifestFromLocalProject({ required: false, persist: true })
    await refreshAndroidModuleConfig()
    await refreshIosModuleConfig()
    await refreshHarmonyModuleConfig()
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

const buildCenterRecords = createBuildCenterRecords({
  buildStore, projectId, currentProject, scanResult, latestManifestInfo, androidModuleConfigReport, getProjectName, androidFieldValue
})
const { appendCleanupLines, appendFinalLog, appendManifestLog, cleanupBuildTemporaryFiles, createBuildRecord, finalizeBuildRecord } = buildCenterRecords
const buildCenterActions = createBuildCenterActions({
  dialog, message, buildStore, projectId, currentBuildId, scanResult, selectedPlatforms, singleSelectedPlatform, isBuildLocked,
  selectedNeedsIosConfig, iosModuleConfigLoading, iosModuleMissingRequired, iosBuildReady, getProjectName, ensureManifestInfoLoaded,
  ensureAndroidModuleConfigReadyForBuild, ensureIosModuleConfigReadyForBuild, ensureHarmonyModuleConfigReadyForBuild,
  selectedManifestInfoForBuild, buildAndroidModuleConfigPayload, persistAndroidModuleConfigCache, persistIosModuleConfigCache,
  cleanupBuildTemporaryFiles, appendCleanupLines, appendManifestLog, appendFinalLog, createBuildRecord, finalizeBuildRecord,
  iosIconCount: () => iosIconCount.value, iosPrivacyDescriptionCount: () => iosPrivacyDescriptionCount.value
})
const { canGenerateNativeProject, canGenerateIosProject, generateAndroidProject, generateIosProject, generateHarmonyProject, startBuild } = buildCenterActions
const moduleStatusHelpers = createModuleStatusHelpers({ androidFieldValue, iosFieldValue })
const { androidConfigModuleStatusType, configModuleStatusLabel, fieldStatusLabel, fieldStatusType, iosConfigModuleStatusLabel, iosConfigModuleStatusType, iosFieldStatusLabel, iosFieldStatusType, preferredAndroidConfigModule, preferredIosConfigModule } = moduleStatusHelpers

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

async function persistIosModuleConfigCache() {
  if (iosModuleConfigSaveTimer) {
    clearTimeout(iosModuleConfigSaveTimer)
    iosModuleConfigSaveTimer = null
  }
  const project = currentProject.value
  if (!project) return
  if (!syncIosModuleConfigCache()) return
  await projectsStore.saveProject(project)
}

function scheduleIosModuleConfigCacheSave() {
  if (iosModuleConfigSaveTimer) {
    clearTimeout(iosModuleConfigSaveTimer)
  }
  iosModuleConfigSaveTimer = setTimeout(() => {
    iosModuleConfigSaveTimer = null
    void persistIosModuleConfigCache()
  }, 300)
}

function getProjectName() {
  return currentProject.value?.name || currentProject.value?.app.name || projectId.value
}

function isManifestModuleSelected(mod: DetectedModule) {
  return selectedManifestModuleKeys.value.has(manifestModuleKey(mod))
}

function isAndroidConfigModuleSelected(mod: AndroidModuleConfigModule) {
  return selectedManifestModuleKeys.value.has(androidConfigModuleKey(mod))
}

function isIosConfigModuleSelected(mod: IosModuleConfigModule) {
  return selectedManifestModuleKeys.value.has(iosConfigModuleKey(mod))
}

function isHarmonyConfigModuleSelected(mod: HarmonyModuleConfigModule) {
  return selectedManifestModuleKeys.value.has(harmonyConfigModuleKey(mod))
}

function openAndroidConfigModule(mod: AndroidModuleConfigModule) {
  activeAndroidConfigModuleKey.value = androidConfigModuleKey(mod)
}

function openIosConfigModule(mod: IosModuleConfigModule) {
  activeIosConfigModuleKey.value = iosConfigModuleKey(mod)
}

function openIosPrivacyDescriptionDialog() {
  if (!iosPrivacyDescriptionItems.value.length) {
    message.info('当前已选 iOS 模块暂无权限说明需要填写')
    return
  }
  iosPrivacyDialogVisible.value = true
}

function updateIosPrivacyDescription(item: IosPrivacyDescriptionItem, value: string) {
  if (isBuildLocked.value) return
  iosModuleConfigValues.value = {
    ...iosModuleConfigValues.value,
    [item.fieldKey]: value
  }
  syncIosModuleConfigCache()
  scheduleIosModuleConfigCacheSave()
}

function selectedManifestInfoForBuild(info: UniappManifestInfo): UniappManifestInfo {
  const detectedModules = !manifestModuleSelectionTouched.value && selectedManifestModuleKeys.value.size === 0
    ? info.detectedModules
    : info.detectedModules.filter(mod => isManifestModuleSelected(mod))
  return applyIosModuleConfigToManifestInfo({
    ...info,
    detectedModules
  })
}

function applyIosModuleConfigToManifestInfo(info: UniappManifestInfo): UniappManifestInfo {
  return applyIosModuleConfigToManifest(info, {
    selectedNeedsIosConfig,
    iosModuleConfigReport,
    isIosConfigModuleSelected,
    iosFieldValue,
    buildIosPrivacyDescriptionPayload
  })
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
  if (!project.iosModuleConfig) project.iosModuleConfig = {}
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

async function refreshIosModuleConfig() {
  if (!latestManifestInfo.value) {
    iosModuleConfigReport.value = null
    return
  }
  iosModuleConfigLoading.value = true
  try {
    const report = await invoke<IosModuleConfigReport>('analyze_ios_module_config', {
      manifestInfo: latestManifestInfo.value,
      userConfig: cachedIosModuleConfig()
    })
    iosModuleConfigReport.value = stripIosLocalPodFields(report)
    mergeIosModuleConfigDefaults(iosModuleConfigReport.value)
  } catch (e: any) {
    iosModuleConfigReport.value = null
    manifestReadWarning.value = String(e)
  } finally {
    iosModuleConfigLoading.value = false
  }
}

async function refreshHarmonyModuleConfig() {
  if (!latestManifestInfo.value) {
    harmonyModuleConfigReport.value = null
    return
  }
  harmonyModuleConfigLoading.value = true
  try {
    harmonyModuleConfigReport.value = await invoke<HarmonyModuleConfigReport>('analyze_harmony_module_config', {
      manifestInfo: latestManifestInfo.value
    })
  } catch (e: any) {
    harmonyModuleConfigReport.value = null
    manifestReadWarning.value = String(e)
  } finally {
    harmonyModuleConfigLoading.value = false
  }
}

async function ensureManifestInfoLoaded(options: { persist: boolean } = { persist: true }): Promise<UniappManifestInfo> {
  if (latestManifestInfo.value) {
    if (!iosModuleConfigReport.value) await refreshIosModuleConfig()
    if (!harmonyModuleConfigReport.value) await refreshHarmonyModuleConfig()
    return latestManifestInfo.value
  }
  const info = await refreshManifestFromLocalProject({ required: true, persist: options.persist })
  if (!info) {
    const warning = manifestReadWarning.value || '请先导入资源，并确保已从本地项目路径读取 manifest.json'
    throw new Error(warning)
  }
  await refreshIosModuleConfig()
  await refreshHarmonyModuleConfig()
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

async function ensureIosModuleConfigReadyForBuild() {
  if (!selectedNeedsIosConfig.value) return true
  if (!iosModuleConfigReport.value) {
    await refreshIosModuleConfig()
  }
  if (!iosModuleConfigReport.value) {
    message.error(manifestReadWarning.value || 'iOS 模块配置分析失败')
    return false
  }
  if (iosPrivacyDescriptionMissingCount.value) {
    openIosPrivacyDescriptionDialog()
    return false
  }
  if (iosModuleMissingRequired.value.length) {
    message.error(`请先填写 iOS 模块配置: ${iosModuleMissingRequired.value.map(item => `${item.moduleName}-${item.label}`).join('、')}`)
    return false
  }
  return true
}

async function ensureHarmonyModuleConfigReadyForBuild() {
  if (!selectedNeedsHarmonyConfig.value) return true
  if (!harmonyModuleConfigReport.value) {
    await refreshHarmonyModuleConfig()
  }
  if (!harmonyModuleConfigReport.value) {
    message.error(manifestReadWarning.value || 'Harmony 模块配置分析失败')
    return false
  }
  if (harmonyModuleMissingRequired.value.length) {
    message.error(`请先填写 Harmony 模块配置: ${harmonyModuleMissingRequired.value.map(item => `${item.moduleName}-${item.label}`).join('、')}`)
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

function mergeIosModuleConfigDefaults(report: IosModuleConfigReport) {
  const next: Record<string, string> = {}
  const cached = cachedIosModuleConfig()
  for (const mod of report.modules) {
    for (const field of mod.fields) {
      if (isIosLocalPodField(field)) continue
      const scopedKey = iosModuleFieldValueKey(mod, field)
      if (next[scopedKey] === undefined) {
        const value = cached[scopedKey] ?? field.value ?? ''
        next[scopedKey] = normalizeIosFieldValue(mod, field, value)
      }
    }
  }
  iosModuleConfigValues.value = next
  syncIosModuleConfigCache()
  scheduleIosModuleConfigCacheSave()
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

function iosFieldValue(mod: IosModuleConfigModule, field: IosModuleConfigField) {
  const value = iosModuleConfigValues.value[iosModuleFieldValueKey(mod, field)]
    ?? iosModuleConfigValues.value[field.key]
    ?? field.value
    ?? ''
  return normalizeIosFieldValue(mod, field, value)
}

function harmonyFieldValue(_mod: HarmonyModuleConfigModule, field: HarmonyModuleConfigField) {
  return field.value ?? ''
}

function updateIosField(mod: IosModuleConfigModule, field: IosModuleConfigField, value: string) {
  if (isBuildLocked.value) return
  if (isIosLocalPodField(field)) return
  iosModuleConfigValues.value = {
    ...iosModuleConfigValues.value,
    [iosModuleFieldValueKey(mod, field)]: normalizeIosFieldValue(mod, field, value)
  }
  syncIosModuleConfigCache()
  scheduleIosModuleConfigCacheSave()
}

function updateActiveIosField(field: IosModuleConfigField, value: string) {
  const mod = activeIosConfigModule.value
  if (!mod) return
  updateIosField(mod, field, value)
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

function buildIosPrivacyDescriptionPayload() {
  const payload: Record<string, string> = {}
  const report = iosModuleConfigReport.value
  if (!report) return payload
  for (const mod of report.modules) {
    if (!isIosConfigModuleSelected(mod)) continue
    for (const field of mod.fields) {
      if (isIosLocalPodField(field)) continue
      if (!isIosPrivacyField(field)) continue
      const value = iosFieldValue(mod, field).trim()
      if (!value) continue
      const plistKey = field.key.slice('privacy.'.length)
      if (payload[plistKey] === undefined) payload[plistKey] = value
    }
  }
  return payload
}

function cachedAndroidModuleConfig() {
  return currentProject.value?.androidModuleConfig || {}
}

function cachedIosModuleConfig() {
  return currentProject.value?.iosModuleConfig || {}
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

function syncIosModuleConfigCache() {
  const project = currentProject.value
  const report = iosModuleConfigReport.value
  if (!project || !report) return false
  const next: Record<string, string> = {}
  for (const mod of report.modules) {
    for (const field of mod.fields) {
      if (isIosLocalPodField(field)) continue
      const rawValue = iosFieldValue(mod, field).trim()
      const value = normalizeIosFieldValue(mod, field, rawValue).trim()
      if (value) next[iosModuleFieldValueKey(mod, field)] = value
    }
  }
  project.iosModuleConfig = next
  return true
}

async function openGeneratedProject() {
  const path = currentGeneratedProjectPath.value
  if (!path) return
  try {
    await revealItemInDir(path)
  } catch (e) {
    try {
      await openPath(path)
    } catch {
      message.error(`打开目录失败：${String(e)}`)
    }
  }
}

function goBack() {
  router.push(`/project/${projectId.value}`)
}

return {
  currentProject, getProjectName, goBack, selectedPlatforms, importing, isBuildLocked, scanResult, insightAppId, insightVersionName, insightVersionCode, insightManifestPath, manifestReadWarning, chooseResource,
  buildDisabledReason, canBuild, canGenerateAndroid, canGenerateIos, canGenerateHarmony, packageBuildLoading, androidGenerateLoading, iosGenerateLoading, harmonyGenerateLoading, singleSelectedPlatform, togglePlatform, generateAndroidProject, generateIosProject, generateHarmonyProject, startBuild,
  iosMissingRequired, iosIconCount, iosPrivacyDescriptionCount, iosPrivacyDescriptionItems, iosPrivacyDescriptionMissingCount, iosModuleSummaryLabel, iosConfigurableModules, selectedManifestModules, iosModuleConfigLoading, latestManifestInfo, iosModuleMissingRequired, activeIosConfigModuleKey, activeIosConfigModule, iosConfigModuleStatusType, iosConfigModuleStatusLabel, iosFieldValue, iosFieldStatusType, iosFieldStatusLabel, openIosPrivacyDescriptionDialog, openIosConfigModule, updateActiveIosField,
  androidModuleConfigLoading, androidConfigurableModules, androidMissingRequired, activeAndroidConfigModuleKey, activeAndroidConfigModule, androidConfigModuleStatusType, configModuleStatusLabel, androidFieldValue, fieldStatusType, fieldStatusLabel, formatFileSize, openAndroidConfigModule, updateActiveAndroidField, pickAndroidFileField, clearAndroidFileField,
  currentBuild, visibleArtifacts, currentGeneratedProjectPath, currentGeneratedProjectLabel, openGeneratedProject, iosPrivacyDialogVisible, updateIosPrivacyDescription
}
}
