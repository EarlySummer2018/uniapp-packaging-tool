<script setup lang="ts">
import { computed, h, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NButton,
  NGi,
  NGrid,
  NIcon,
  NRadio,
  NRadioGroup,
  NSpace,
  NText,
  useDialog,
  useMessage
} from 'naive-ui'
import { ArrowBackOutline } from '@vicons/ionicons5'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { invoke } from '@tauri-apps/api/core'
import { useBuildStore } from '../stores/build'
import { useProjectsStore } from '../stores/projects'
import AndroidModuleConfigPanel from './build-center/AndroidModuleConfigPanel.vue'
import BuildLogCard from './build-center/BuildLogCard.vue'
import IosOfflineSdkPanel from './build-center/IosOfflineSdkPanel.vue'
import IosPrivacyDescriptionModal from './build-center/IosPrivacyDescriptionModal.vue'
import PlatformSelectCard from './build-center/PlatformSelectCard.vue'
import ResourceImportCard from './build-center/ResourceImportCard.vue'
import ResourceInsightPanel from './build-center/ResourceInsightPanel.vue'
import { platforms } from './build-center/platforms'
import type {
  AndroidModuleConfigField,
  AndroidModuleConfigModule,
  AndroidModuleConfigReport,
  BuildArtifact,
  BuildRecord,
  DetectedModule,
  IosModuleConfigField,
  IosModuleConfigModule,
  IosModuleConfigReport,
  IosPrivacyDescriptionItem,
  ModuleStatusTone,
  NonIosPlatform,
  Platform,
  ResourceScanResult,
  UniappManifestInfo
} from './build-center/types'
import {
  androidConfigModuleKey,
  androidModuleFieldValueKey,
  formatModuleWithPlatforms,
  generateProjectCommand,
  generateProjectKind,
  iosConfigModuleKey,
  iosModuleFieldValueKey,
  isIosPrivacyField,
  manifestModuleKey,
  platformProjectName
} from './build-center/moduleKeys'
import {
  iosMapProviderForModule,
  normalizeIosFieldValue,
  normalizeIosMapPageTypeValue
} from './build-center/moduleFields'
import {
  cloneJson,
  ensureObjectPath,
  normalizeBooleanFieldValue
} from './build-center/manifestObject'
import {
  cleanupIosPushSdkConfigs,
  clearIosMapPageTypeConfig,
  ensureIosGeolocationSdkConfig,
  ensureIosMapSdkConfig,
  ensureIosPushSdkConfig,
  ensureIosStatisticSdkConfig,
  ensureIosUnipushConfig,
  setIosAllowsArbitraryLoads,
  setIosBluetoothBackgroundModes,
  setIosGeolocationProviderValue,
  setIosMapProviderValue,
  setIosProviderValue
} from './build-center/iosManifestConfig'

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
const selectedManifestModuleKeys = ref<Set<string>>(new Set())
const manifestModuleSelectionTouched = ref(false)
const activeAndroidConfigModuleKey = ref<string | null>(null)
const activeIosConfigModuleKey = ref<string | null>(null)
const iosPrivacyDialogVisible = ref(false)

let androidModuleConfigSaveTimer: ReturnType<typeof setTimeout> | null = null
let iosModuleConfigSaveTimer: ReturnType<typeof setTimeout> | null = null

type IosPackagingMode = 'autoMigration' | 'localPod'

function iosPackagingModeLabel(mode: IosPackagingMode) {
  if (mode === 'autoMigration') return '自动迁移打包'
  return '本地 Pod 打包'
}

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
  if (selectedNeedsIosConfig.value && iosModuleConfigLoading.value) return '正在分析 iOS 模块配置'
  if (selectedNeedsIosConfig.value && iosModuleMissingRequired.value.length) {
    return `还有 ${iosModuleMissingRequired.value.length} 个 iOS 模块必填配置未填写`
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
const iosConfigModulesByKey = computed(() => {
  const modules = iosModuleConfigReport.value?.modules || []
  return new Map(modules.map(mod => [iosConfigModuleKey(mod), mod]))
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

function chooseIosPackagingMode(actionLabel: string): Promise<IosPackagingMode | null> {
  return new Promise(resolve => {
    let settled = false
    const selectedMode = ref<IosPackagingMode>('autoMigration')
    const dialogInstance = dialog.create({
      type: 'info',
      title: '选择 iOS 打包方式',
      content: () => h(NSpace, { vertical: true, size: 12 }, {
        default: () => [
          h(NText, { depth: 3 }, {
            default: () => `${actionLabel}将使用本次选择的 iOS 打包方式。`
          }),
          h(NRadioGroup, {
            value: selectedMode.value,
            'onUpdate:value': (value: IosPackagingMode) => {
              if (value === 'autoMigration') selectedMode.value = value
            }
          }, {
            default: () => h(NSpace, { vertical: true, size: 8 }, {
              default: () => [
                h(NRadio, { value: 'autoMigration' }, { default: () => '自动迁移打包' }),
                h(NRadio, { value: 'localPod', disabled: true }, { default: () => '本地 Pod（暂不可用）' })
              ]
            })
          })
        ]
      }),
      closable: true,
      maskClosable: false,
      onClose: () => {
        if (!settled) resolve(null)
      },
      action: () => h(NSpace, { justify: 'end' }, {
        default: () => [
          h(NButton, {
            onClick: () => {
              settled = true
              dialogInstance.destroy()
              resolve(null)
            }
          }, { default: () => '取消' }),
          h(NButton, {
            type: 'primary',
            onClick: () => {
              settled = true
              dialogInstance.destroy()
              resolve(selectedMode.value)
            }
          }, { default: () => `使用${iosPackagingModeLabel(selectedMode.value)}` })
        ]
      })
    })
  })
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
  if (!(await ensureIosModuleConfigReadyForBuild())) {
    return
  }
  let iosPackagingMode: IosPackagingMode | null = null
  if (selectedNeedsIosConfig.value) {
    iosPackagingMode = await chooseIosPackagingMode('开始打包')
    if (!iosPackagingMode) return
  }
  const buildManifestInfo = selectedManifestInfoForBuild(manifestInfo)
  const androidModuleConfig = buildAndroidModuleConfigPayload()
  await persistAndroidModuleConfigCache()
  await persistIosModuleConfigCache()
  let lastBuildId: string | null = null
  const buildIds: string[] = []
  for (const platform of selectedPlatforms.value) {
    const buildId = platform === 'ios'
      ? await buildIosIpa(runProjectId, runProjectName, importedResourcePath, buildManifestInfo, iosPackagingMode!)
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
  buildManifestInfo: UniappManifestInfo,
  packagingMode: IosPackagingMode
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
    `[info] iOS 打包方式: ${iosPackagingModeLabel(packagingMode)}`,
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
  return !!scanResult.value
    && singleSelectedPlatform.value === 'ios'
    && !isBuildLocked.value
    && !iosModuleConfigLoading.value
    && iosModuleMissingRequired.value.length === 0
    && iosBuildReady.value
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
  if (!(await ensureIosModuleConfigReadyForBuild())) {
    return
  }
  const iosPackagingMode = await chooseIosPackagingMode('生成 iOS 原生项目')
  if (!iosPackagingMode) return
  const buildManifestInfo = selectedManifestInfoForBuild(manifestInfo)
  await persistAndroidModuleConfigCache()
  await persistIosModuleConfigCache()
  const buildId = buildStore.startBuild(runProjectId, 'ios', 'generateIosProject')
  currentBuildId.value = buildId
  buildStore.setActiveEventBuildId(buildId)
  const startedAt = new Date()
  await createBuildRecord(buildId, 'ios', startedAt, runProjectId, runProjectName, importedResourcePath)
  await appendManifestLog(buildId, buildManifestInfo, 'ios')
  await buildStore.appendBuildLogLines(buildId, [
    '[info] iOS 工程生成: 复制 SDK 自带 HBuilder-Hello* 后配置 workspace 副本',
    `[info] iOS 打包方式: ${iosPackagingModeLabel(iosPackagingMode)}`,
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
  await persistIosModuleConfigCache()
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

  const iosConfigModule = iosConfigModulesByKey.value.get(key)
  if (checked && iosConfigModule?.fields.length) {
    activeIosConfigModuleKey.value = iosConfigModuleKey(iosConfigModule)
  } else if (!checked && activeIosConfigModuleKey.value === key) {
    activeIosConfigModuleKey.value = iosConfigurableModules.value.length
      ? iosConfigModuleKey(preferredIosConfigModule(iosConfigurableModules.value))
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

function iosConfigFieldFilled(mod: IosModuleConfigModule, field: IosModuleConfigField) {
  return iosFieldValue(mod, field).trim().length > 0
}

function iosConfigModuleMissingRequiredCount(mod: IosModuleConfigModule) {
  return mod.fields.filter(field => field.required && !iosConfigFieldFilled(mod, field)).length
}

function iosConfigModuleFilledCount(mod: IosModuleConfigModule) {
  return mod.fields.filter(field => iosConfigFieldFilled(mod, field)).length
}

function iosConfigModuleStatusTone(mod: IosModuleConfigModule): ModuleStatusTone {
  if (!mod.fields.length) return 'success'
  if (iosConfigModuleMissingRequiredCount(mod) === 0) return 'success'
  return iosConfigModuleFilledCount(mod) > 0 ? 'warning' : 'error'
}

function iosConfigModuleStatusLabel(mod: IosModuleConfigModule) {
  const missing = iosConfigModuleMissingRequiredCount(mod)
  if (!mod.fields.length) return '已选'
  if (missing === 0) return '已配置'
  if (iosConfigModuleFilledCount(mod) > 0) return '部分配置'
  return '需配置'
}

function preferredIosConfigModule(modules: IosModuleConfigModule[]) {
  return modules.find(mod => iosConfigModuleStatusTone(mod) === 'error')
    || modules.find(mod => iosConfigModuleStatusTone(mod) === 'warning')
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

function iosConfigModuleStatusType(mod: IosModuleConfigModule) {
  return iosConfigModuleStatusTone(mod)
}

function openAndroidConfigModule(mod: AndroidModuleConfigModule) {
  activeAndroidConfigModuleKey.value = androidConfigModuleKey(mod)
}

function openIosConfigModule(mod: IosModuleConfigModule) {
  activeIosConfigModuleKey.value = iosConfigModuleKey(mod)
}

function isIosLocalPodField(field: IosModuleConfigField) {
  return field.key === 'LOCAL_POD'
}

function isIosInlineConfigField(field: IosModuleConfigField) {
  return !isIosLocalPodField(field) && !isIosPrivacyField(field)
}

function stripIosLocalPodFields(report: IosModuleConfigReport): IosModuleConfigReport {
  return {
    modules: report.modules.map(mod => ({
      ...mod,
      fields: mod.fields.filter(field => !isIosLocalPodField(field))
    }))
  }
}

function iosPrivacyModuleLabel(mod: IosModuleConfigModule) {
  const labels: Record<string, string> = {
    barcode: '扫码',
    bluetooth: '蓝牙',
    camera: '相机',
    contacts: '通讯录',
    face_id: 'Face ID',
    face_recognition: '实人认证',
    fingerprint: '指纹/面容识别',
    geolocation: '定位',
    ibeacon: 'iBeacon',
    livepusher: 'livePusher',
    map: '地图',
    record: '录音',
    speech: '语音识别'
  }
  return labels[mod.templateKey] || mod.name
}

function iosPrivacyPermissionLabel(plistKey: string, fallback: string) {
  const labels: Record<string, string> = {
    NSBluetoothAlwaysUsageDescription: '蓝牙权限',
    NSBluetoothPeripheralUsageDescription: '蓝牙权限',
    NSCameraUsageDescription: '相机权限',
    NSContactsUsageDescription: '通讯录权限',
    NSFaceIDUsageDescription: 'Face ID 权限',
    NSLocationAlwaysAndWhenInUseUsageDescription: '始终和使用期间定位权限',
    NSLocationAlwaysUsageDescription: '始终定位权限',
    NSLocationWhenInUseUsageDescription: '使用期间定位权限',
    NSMicrophoneUsageDescription: '麦克风权限',
    NSPhotoLibraryAddUsageDescription: '保存到相册权限',
    NSPhotoLibraryUsageDescription: '相册权限',
    NSSpeechRecognitionUsageDescription: '语音识别权限'
  }
  return labels[plistKey] || fallback.replace(/说明$/, '').replace(/权限$/, '权限')
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
  if (!selectedNeedsIosConfig.value) return info
  const manifestValue = cloneJson(info.manifestValue || null)
  if (manifestValue) {
    applyIosGeolocationConfigToManifestValue(manifestValue)
    applyIosMapConfigToManifestValue(manifestValue)
    applyIosPushConfigToManifestValue(manifestValue)
    applyIosBluetoothConfigToManifestValue(manifestValue)
    applyIosVideoPlayerConfigToManifestValue(manifestValue)
    applyIosStatisticConfigToManifestValue(manifestValue)
  }
  return {
    ...info,
    manifestValue,
    iosPrivacyDescriptions: {
      ...(info.iosPrivacyDescriptions || {}),
      ...buildIosPrivacyDescriptionPayload()
    }
  }
}

function applyIosGeolocationConfigToManifestValue(manifestValue: Record<string, any>) {
  const geolocationModules = (iosModuleConfigReport.value?.modules || [])
    .filter(mod => mod.templateKey === 'geolocation' && isIosConfigModuleSelected(mod))
  if (!geolocationModules.length) return

  const sdkConfigs = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'sdkConfigs'])
  const geolocationConfig = ensureIosGeolocationSdkConfig(sdkConfigs)
  for (const mod of geolocationModules) {
    for (const field of mod.fields) {
      if (field.key.startsWith('privacy.')) continue
      const value = iosFieldValue(mod, field).trim()
      if (!value) continue
      if (field.key === 'baidu.appkey_ios') {
        setIosGeolocationProviderValue(geolocationConfig, 'baidu', ['baidu', 'bd'], 'appkey_ios', value)
      } else if (field.key === 'amap.appkey_ios') {
        setIosGeolocationProviderValue(geolocationConfig, 'amap', ['amap', 'gaode'], 'appkey_ios', value)
      }
    }
  }
}

function applyIosMapConfigToManifestValue(manifestValue: Record<string, any>) {
  const mapModules = (iosModuleConfigReport.value?.modules || [])
    .filter(mod => mod.templateKey === 'map' && isIosConfigModuleSelected(mod))
  if (!mapModules.length) return

  const sdkConfigs = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'sdkConfigs'])
  const mapConfig = ensureIosMapSdkConfig(sdkConfigs)
  delete mapConfig.__platform__
  for (const mod of mapModules) {
    const provider = iosMapProviderForModule(mod)
    if (provider === 'google') clearIosMapPageTypeConfig(mapConfig)
    for (const field of mod.fields) {
      if (field.key.startsWith('privacy.')) continue
      const value = iosFieldValue(mod, field).trim()
      if (field.key === 'baidu.appkey_ios') {
        if (value) setIosMapProviderValue(mapConfig, 'baidu', ['baidu', 'bd'], 'appkey_ios', value)
      } else if (field.key === 'amap.appkey_ios') {
        if (value) setIosMapProviderValue(mapConfig, 'amap', ['amap', 'gaode'], 'appkey_ios', value)
      } else if (field.key === 'google.apikey_ios') {
        if (value) setIosMapProviderValue(mapConfig, 'google', ['google', 'googleMap'], 'apikey_ios', value)
      } else if (field.key === 'MAP_PAGE_TYPE') {
        mapConfig.pageType = normalizeIosMapPageTypeValue(provider, value)
      }
    }
  }
}

function applyIosPushConfigToManifestValue(manifestValue: Record<string, any>) {
  const pushModule = (iosModuleConfigReport.value?.modules || [])
    .find(mod => mod.templateKey === 'push' && isIosConfigModuleSelected(mod))
  if (!pushModule) return
  const sdkConfigs = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'sdkConfigs'])
  const pushConfig = ensureIosPushSdkConfig(sdkConfigs)
  cleanupIosPushSdkConfigs(sdkConfigs, pushConfig)
  const unipushConfig = ensureIosUnipushConfig(pushConfig)
  for (const field of pushModule.fields) {
    const value = iosFieldValue(pushModule, field).trim()
    if (!value) continue
    if (field.key === 'pushProvider') continue
    if (field.key === 'unipush.appid') unipushConfig.appid = value
    else if (field.key === 'unipush.appkey') unipushConfig.appkey = value
    else if (field.key === 'unipush.appsecret') unipushConfig.appsecret = value
  }
  if (!('__platform__' in unipushConfig)) unipushConfig.__platform__ = ['ios']
  if (!('version' in unipushConfig)) unipushConfig.version = '2'
}

function applyIosBluetoothConfigToManifestValue(manifestValue: Record<string, any>) {
  const bluetoothModule = (iosModuleConfigReport.value?.modules || [])
    .find(mod => mod.templateKey === 'bluetooth' && isIosConfigModuleSelected(mod))
  if (!bluetoothModule) return
  const backgroundField = bluetoothModule.fields.find(field => field.key === 'backgroundBluetooth')
  if (!backgroundField) return
  const enabled = normalizeBooleanFieldValue(iosFieldValue(bluetoothModule, backgroundField))
  const iosConfig = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'ios'])
  setIosBluetoothBackgroundModes(iosConfig, enabled)
}

function applyIosVideoPlayerConfigToManifestValue(manifestValue: Record<string, any>) {
  const videoPlayerModule = (iosModuleConfigReport.value?.modules || [])
    .find(mod => mod.templateKey === 'video_player' && isIosConfigModuleSelected(mod))
  if (!videoPlayerModule) return
  const atsField = videoPlayerModule.fields.find(field => field.key === 'allowArbitraryLoads')
  if (!atsField) return
  const enabled = normalizeBooleanFieldValue(iosFieldValue(videoPlayerModule, atsField))
  const iosConfig = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'ios'])
  setIosAllowsArbitraryLoads(iosConfig, enabled)
}

function applyIosStatisticConfigToManifestValue(manifestValue: Record<string, any>) {
  const statisticModules = (iosModuleConfigReport.value?.modules || [])
    .filter(mod => mod.templateKey === 'statistic' && isIosConfigModuleSelected(mod))
  if (!statisticModules.length) return

  const sdkConfigs = ensureObjectPath(manifestValue, ['app-plus', 'distribute', 'sdkConfigs'])
  const statisticConfig = ensureIosStatisticSdkConfig(sdkConfigs)
  for (const mod of statisticModules) {
    for (const field of mod.fields) {
      const value = iosFieldValue(mod, field).trim()
      if (field.key === 'UMENG_APPKEY') {
        if (value) setIosProviderValue(statisticConfig, 'umeng', ['umeng', 'umeng-ios'], 'appkey_ios', value)
      } else if (field.key === 'UMENG_CHANNEL') {
        if (value) setIosProviderValue(statisticConfig, 'umeng', ['umeng', 'umeng-ios'], 'channelid_ios', value)
      }
    }
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

async function ensureManifestInfoLoaded(options: { persist: boolean } = { persist: true }): Promise<UniappManifestInfo> {
  if (latestManifestInfo.value) {
    if (!iosModuleConfigReport.value) await refreshIosModuleConfig()
    return latestManifestInfo.value
  }
  const info = await refreshManifestFromLocalProject({ required: true, persist: options.persist })
  if (!info) {
    const warning = manifestReadWarning.value || '请先导入资源，并确保已从本地项目路径读取 manifest.json'
    throw new Error(warning)
  }
  await refreshIosModuleConfig()
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

function iosFieldStatusType(mod: IosModuleConfigModule, field: IosModuleConfigField) {
  const value = iosFieldValue(mod, field).trim()
  if (!value && field.required) return 'error'
  if (field.valueSource === 'manifest' && value) return 'success'
  if (field.valueSource === 'default' && value) return 'default'
  if (value) return 'info'
  return 'default'
}

function iosFieldStatusLabel(mod: IosModuleConfigModule, field: IosModuleConfigField) {
  const value = iosFieldValue(mod, field).trim()
  if (!value && field.required) return '必填'
  if (!value) return '可选'
  if (field.valueSource === 'manifest') return 'manifest'
  if (field.valueSource === 'default') return '默认'
  return '已填写'
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

async function openGeneratedProject() {
  if (!currentGeneratedProjectPath.value) return
  await invoke('tauri', {
    __tauriModule: 'shell',
    message: { cmd: 'open', path: currentGeneratedProjectPath.value }
  })
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
        <ResourceImportCard
          :importing="importing"
          :is-build-locked="isBuildLocked"
          :scan-result="scanResult"
          :insight-app-id="insightAppId"
          :insight-version-name="insightVersionName"
          :insight-version-code="insightVersionCode"
          :insight-manifest-path="insightManifestPath"
          :manifest-read-warning="manifestReadWarning"
          @choose-resource="chooseResource"
        />
      </n-gi>
      <n-gi>
        <PlatformSelectCard
          :platforms="platforms"
          :selected-platforms="selectedPlatforms"
          :is-build-locked="isBuildLocked"
          :build-disabled-reason="buildDisabledReason"
          :can-build="canBuild"
          :can-generate-android="canGenerateAndroid"
          :can-generate-ios="canGenerateIos"
          :can-generate-harmony="canGenerateHarmony"
          :package-build-loading="packageBuildLoading"
          :android-generate-loading="androidGenerateLoading"
          :ios-generate-loading="iosGenerateLoading"
          :harmony-generate-loading="harmonyGenerateLoading"
          :single-selected-platform="singleSelectedPlatform"
          @toggle-platform="togglePlatform"
          @generate-android="generateAndroidProject"
          @generate-ios="generateIosProject"
          @generate-harmony="generateHarmonyProject"
          @start-build="startBuild"
        />
      </n-gi>
    </n-grid>
    <ResourceInsightPanel
      :scan-result="scanResult"
      :insight-app-name="insightAppName"
      :insight-app-id="insightAppId"
      :insight-version-name="insightVersionName"
      :insight-version-code="insightVersionCode"
      :insight-manifest-path="insightManifestPath"
      :selected-manifest-modules="selectedManifestModules"
      :manifest-modules="manifestModules"
      :uts-dependency-count="utsDependencyCount"
      :uts-plugin-labels="utsPluginLabels"
      :manifest-read-warning="manifestReadWarning"
      :is-build-locked="isBuildLocked"
      :is-manifest-module-selected="isManifestModuleSelected"
      :manifest-module-status-type="manifestModuleStatusType"
      :manifest-module-status-class="manifestModuleStatusClass"
      :manifest-module-status-label="manifestModuleStatusLabel"
      @set-manifest-module-selected="setManifestModuleSelected"
    />

    <IosOfflineSdkPanel
      :visible="!!scanResult && selectedPlatforms.includes('ios')"
      :ios-missing-required="iosMissingRequired"
      :bundle-id="currentProject?.ios.bundleId || '-'"
      :team-id="currentProject?.ios.teamId || '-'"
      :ios-icon-count="iosIconCount"
      :ios-privacy-description-count="iosPrivacyDescriptionCount"
      :ios-privacy-description-item-count="iosPrivacyDescriptionItems.length"
      :ios-privacy-description-missing-count="iosPrivacyDescriptionMissingCount"
      :insight-app-id="insightAppId"
      :ios-module-summary-label="iosModuleSummaryLabel"
      :ios-configurable-modules="iosConfigurableModules"
      :selected-manifest-module-count="selectedManifestModules.length"
      :ios-module-config-loading="iosModuleConfigLoading"
      :latest-manifest-info="latestManifestInfo"
      :manifest-read-warning="manifestReadWarning"
      :ios-module-missing-required-count="iosModuleMissingRequired.length"
      :active-ios-config-module-key="activeIosConfigModuleKey"
      :active-ios-config-module="activeIosConfigModule"
      :is-build-locked="isBuildLocked"
      :ios-config-module-status-type="iosConfigModuleStatusType"
      :ios-config-module-status-label="iosConfigModuleStatusLabel"
      :ios-field-value="iosFieldValue"
      :ios-field-status-type="iosFieldStatusType"
      :ios-field-status-label="iosFieldStatusLabel"
      @edit-privacy="openIosPrivacyDescriptionDialog"
      @open-module="openIosConfigModule"
      @update-field="updateActiveIosField"
    />

    <AndroidModuleConfigPanel
      :visible="!!scanResult && selectedPlatforms.includes('android')"
      :android-module-config-loading="androidModuleConfigLoading"
      :latest-manifest-info="latestManifestInfo"
      :manifest-read-warning="manifestReadWarning"
      :android-configurable-modules="androidConfigurableModules"
      :selected-manifest-module-count="selectedManifestModules.length"
      :android-missing-required-count="androidMissingRequired.length"
      :active-android-config-module-key="activeAndroidConfigModuleKey"
      :active-android-config-module="activeAndroidConfigModule"
      :is-build-locked="isBuildLocked"
      :android-config-module-status-type="androidConfigModuleStatusType"
      :config-module-status-label="configModuleStatusLabel"
      :android-field-value="androidFieldValue"
      :field-status-type="fieldStatusType"
      :field-status-label="fieldStatusLabel"
      :format-file-size="formatFileSize"
      @open-module="openAndroidConfigModule"
      @update-field="updateActiveAndroidField"
      @pick-file-field="pickAndroidFileField"
      @clear-file-field="clearAndroidFileField"
    />

    <BuildLogCard
      :logs="currentBuild?.logs || []"
      :progress="currentBuild?.progress || 0"
      :status="currentBuild?.status"
      :visible-artifacts="visibleArtifacts"
      :current-generated-project-path="currentGeneratedProjectPath"
      :current-generated-project-label="currentGeneratedProjectLabel"
      @open-generated-project="openGeneratedProject"
    />

    <IosPrivacyDescriptionModal
      v-model:show="iosPrivacyDialogVisible"
      :items="iosPrivacyDescriptionItems"
      :missing-count="iosPrivacyDescriptionMissingCount"
      :is-build-locked="isBuildLocked"
      @update-item="updateIosPrivacyDescription"
    />
  </div>
</template>

<style src="./build-center/build-center.css"></style>
