import { h, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { NButton, NRadio, NRadioGroup, NSpace, NText } from 'naive-ui'
import type { BuildArtifact, NonIosPlatform, Platform, UniappManifestInfo } from './types'
import { generateProjectCommand, generateProjectKind, platformProjectName } from './moduleKeys'

type IosPackagingMode = 'autoMigration' | 'localPod'

function iosPackagingModeLabel(mode: IosPackagingMode) {
  return mode === 'autoMigration' ? '自动迁移打包' : '本地 Pod 打包'
}

export function createBuildCenterActions(ctx: any) {
  function chooseIosPackagingMode(actionLabel: string): Promise<IosPackagingMode | null> {
    return new Promise(resolve => {
      let settled = false
      const selectedMode = ref<IosPackagingMode>('autoMigration')
      const dialogInstance = ctx.dialog.create({
        type: 'info',
        title: '选择 iOS 打包方式',
        content: () => h(NSpace, { vertical: true, size: 12 }, {
          default: () => [
            h(NText, { depth: 3 }, { default: () => `${actionLabel}将使用本次选择的 iOS 打包方式。` }),
            h(NRadioGroup, {
              value: selectedMode.value,
              'onUpdate:value': (value: IosPackagingMode) => {
                selectedMode.value = value
              }
            }, {
              default: () => h(NSpace, { vertical: true, size: 8 }, {
                default: () => [
                  h(NRadio, { value: 'autoMigration' }, { default: () => '自动迁移打包' }),
                  h(NRadio, { value: 'localPod' }, { default: () => '本地 Pod 打包' })
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
    return !!ctx.scanResult.value && ctx.singleSelectedPlatform.value === platform && !ctx.isBuildLocked.value
  }

  async function startBuild() {
    if (!ctx.scanResult.value || ctx.selectedPlatforms.value.length === 0 || ctx.isBuildLocked.value) {
      if (ctx.isBuildLocked.value) ctx.message.warning('已有构建任务进行中，请等待完成后再开始新的构建')
      return
    }
    const runProjectId = ctx.projectId.value
    const runProjectName = ctx.getProjectName()
    const importedResourcePath = ctx.scanResult.value.importedPath
    let manifestInfo: UniappManifestInfo
    try {
      manifestInfo = await ctx.ensureManifestInfoLoaded({ persist: true })
    } catch (e: any) {
      ctx.message.error(String(e))
      return
    }
    if (!(await ctx.ensureAndroidModuleConfigReadyForBuild())) return
    if (!(await ctx.ensureIosModuleConfigReadyForBuild())) return
    if (!(await ctx.ensureHarmonyModuleConfigReadyForBuild())) return

    let iosPackagingMode: IosPackagingMode | null = null
    if (ctx.selectedNeedsIosConfig.value) {
      iosPackagingMode = await chooseIosPackagingMode('开始打包')
      if (!iosPackagingMode) return
    }
    const buildManifestInfo = ctx.selectedManifestInfoForBuild(manifestInfo)
    const androidModuleConfig = ctx.buildAndroidModuleConfigPayload()
    await ctx.persistAndroidModuleConfigCache()
    await ctx.persistIosModuleConfigCache()

    let lastBuildId: string | null = null
    const buildIds: string[] = []
    for (const platform of ctx.selectedPlatforms.value as Platform[]) {
      const buildId = platform === 'ios'
        ? await buildIosIpa(runProjectId, runProjectName, importedResourcePath, buildManifestInfo, iosPackagingMode!)
        : await buildStandardPackage(platform, runProjectId, runProjectName, importedResourcePath, buildManifestInfo, androidModuleConfig)
      lastBuildId = buildId
      buildIds.push(buildId)
    }
    if (lastBuildId) {
      const resourceCleanupLines = await ctx.cleanupBuildTemporaryFiles(lastBuildId, null, importedResourcePath)
      for (const buildId of buildIds) {
        if (buildId !== lastBuildId) await ctx.appendCleanupLines(buildId, resourceCleanupLines)
      }
    }
    ctx.buildStore.setActiveEventBuildId(null)
    ctx.scanResult.value = null
  }

  async function buildIosIpa(
    runProjectId: string,
    runProjectName: string,
    importedResourcePath: string,
    buildManifestInfo: UniappManifestInfo,
    packagingMode: IosPackagingMode
  ) {
    const platform: Platform = 'ios'
    const buildId = ctx.buildStore.startBuild(runProjectId, platform, 'package')
    ctx.currentBuildId.value = buildId
    ctx.buildStore.setActiveEventBuildId(buildId)
    const startedAt = new Date()
    await ctx.createBuildRecord(buildId, platform, startedAt, runProjectId, runProjectName, importedResourcePath)
    await ctx.appendManifestLog(buildId, buildManifestInfo, platform)
    await ctx.buildStore.appendBuildLogLines(buildId, [
      '[info] iOS 离线 SDK 流程: 复制 SDK 自带 HBuilder-Hello* 并配置 workspace 副本',
      `[info] iOS 打包方式: ${iosPackagingModeLabel(packagingMode)}`,
      `[info] iOS 图标配置: ${ctx.iosIconCount()} 项，隐私描述: ${ctx.iosPrivacyDescriptionCount()} 项`
    ])
    try {
      const artifact = await invoke<BuildArtifact>('build_ios_ipa', {
        projectId: runProjectId,
        resourcePath: importedResourcePath,
        buildId,
        manifestInfo: buildManifestInfo,
        iosPackagingMode: packagingMode
      })
      await ctx.buildStore.flushBuildLogs(buildId)
      await ctx.appendFinalLog(buildId, 'success')
      await ctx.finalizeBuildRecord(buildId, 'success', startedAt, artifact)
      ctx.buildStore.stopBuild(buildId, true, { artifactPath: artifact.path, artifactSizeBytes: artifact.sizeBytes })
      ctx.message.success('iOS IPA 构建完成')
    } catch (e: any) {
      await ctx.buildStore.flushBuildLogs(buildId)
      await ctx.appendFinalLog(buildId, 'failed', String(e))
      await ctx.finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
      ctx.buildStore.failBuild(buildId, String(e))
      ctx.message.error(`iOS IPA 构建失败: ${String(e)}`)
    } finally {
      await ctx.cleanupBuildTemporaryFiles(buildId, buildId, null)
      await ctx.buildStore.flushBuildLogs(buildId)
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
    const buildId = ctx.buildStore.startBuild(runProjectId, platform, 'package')
    ctx.currentBuildId.value = buildId
    ctx.buildStore.setActiveEventBuildId(buildId)
    const startedAt = new Date()
    await ctx.createBuildRecord(buildId, platform, startedAt, runProjectId, runProjectName, importedResourcePath)
    await ctx.appendManifestLog(buildId, buildManifestInfo, platform)
    try {
      const command = platform === 'android' ? 'build_android_apk' : 'build_harmony_hap'
      const artifact = await invoke<BuildArtifact>(command, {
        projectId: runProjectId,
        resourcePath: importedResourcePath,
        buildId,
        manifestInfo: buildManifestInfo,
        moduleConfig: platform === 'android' ? androidModuleConfig : undefined
      })
      await ctx.buildStore.flushBuildLogs(buildId)
      await ctx.appendFinalLog(buildId, 'success')
      await ctx.finalizeBuildRecord(buildId, 'success', startedAt, artifact)
      ctx.buildStore.stopBuild(buildId, true, { artifactPath: artifact.path, artifactSizeBytes: artifact.sizeBytes })
      ctx.message.success(`${platform} 构建完成`)
    } catch (e: any) {
      await ctx.buildStore.flushBuildLogs(buildId)
      await ctx.appendFinalLog(buildId, 'failed', String(e))
      await ctx.finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
      ctx.buildStore.failBuild(buildId, String(e))
      ctx.message.error(`${platform} 构建失败: ${String(e)}`)
    } finally {
      await ctx.cleanupBuildTemporaryFiles(buildId, buildId, null)
      await ctx.buildStore.flushBuildLogs(buildId)
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
    return !!ctx.scanResult.value
      && ctx.singleSelectedPlatform.value === 'ios'
      && !ctx.isBuildLocked.value
      && !ctx.iosModuleConfigLoading.value
      && ctx.iosModuleMissingRequired.value.length === 0
      && ctx.iosBuildReady.value
  }

  async function generateIosOfflineProject() {
    if (!canGenerateIosProject()) {
      if (ctx.isBuildLocked.value) ctx.message.warning('已有构建任务进行中，请等待完成后再生成项目')
      return
    }
    const runProjectId = ctx.projectId.value
    const runProjectName = ctx.getProjectName()
    const importedResourcePath = ctx.scanResult.value!.importedPath
    let manifestInfo: UniappManifestInfo
    try {
      manifestInfo = await ctx.ensureManifestInfoLoaded({ persist: true })
    } catch (e: any) {
      ctx.message.error(String(e))
      return
    }
    if (!(await ctx.ensureIosModuleConfigReadyForBuild())) return
    const iosPackagingMode = await chooseIosPackagingMode('生成 iOS 原生项目')
    if (!iosPackagingMode) return
    const buildManifestInfo = ctx.selectedManifestInfoForBuild(manifestInfo)
    await ctx.persistAndroidModuleConfigCache()
    await ctx.persistIosModuleConfigCache()

    const buildId = ctx.buildStore.startBuild(runProjectId, 'ios', 'generateIosProject')
    ctx.currentBuildId.value = buildId
    ctx.buildStore.setActiveEventBuildId(buildId)
    const startedAt = new Date()
    await ctx.createBuildRecord(buildId, 'ios', startedAt, runProjectId, runProjectName, importedResourcePath)
    await ctx.appendManifestLog(buildId, buildManifestInfo, 'ios')
    await ctx.buildStore.appendBuildLogLines(buildId, [
      '[info] iOS 工程生成: 复制 SDK 自带 HBuilder-Hello* 后配置 workspace 副本',
      `[info] iOS 打包方式: ${iosPackagingModeLabel(iosPackagingMode)}`,
      `[info] iOS 图标配置: ${ctx.iosIconCount()} 项，隐私描述: ${ctx.iosPrivacyDescriptionCount()} 项`
    ])
    try {
      const projectPath = await invoke<string>('generate_ios_project', {
        projectId: runProjectId,
        resourcePath: importedResourcePath,
        buildId,
        manifestInfo: buildManifestInfo,
        iosPackagingMode
      })
      await ctx.buildStore.flushBuildLogs(buildId)
      await ctx.appendFinalLog(buildId, 'success')
      await ctx.finalizeBuildRecord(buildId, 'success', startedAt, null)
      ctx.buildStore.stopBuild(buildId, true, { generatedProjectPath: projectPath })
      ctx.message.success(`iOS 工程已生成: ${projectPath}`)
    } catch (e: any) {
      await ctx.buildStore.flushBuildLogs(buildId)
      await ctx.appendFinalLog(buildId, 'failed', String(e))
      await ctx.finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
      ctx.buildStore.failBuild(buildId, String(e))
      ctx.message.error(`生成 iOS 工程失败: ${String(e)}`)
    } finally {
      await ctx.buildStore.flushBuildLogs(buildId)
      if (ctx.buildStore.activeEventBuildId === buildId) ctx.buildStore.setActiveEventBuildId(null)
    }
  }

  async function generateNativeProject(platform: NonIosPlatform) {
    if (!ctx.scanResult.value || ctx.singleSelectedPlatform.value !== platform || ctx.isBuildLocked.value) {
      if (ctx.isBuildLocked.value) ctx.message.warning('已有构建任务进行中，请等待完成后再生成项目')
      return
    }
    const runProjectId = ctx.projectId.value
    const runProjectName = ctx.getProjectName()
    const importedResourcePath = ctx.scanResult.value.importedPath
    let manifestInfo: UniappManifestInfo
    try {
      manifestInfo = await ctx.ensureManifestInfoLoaded({ persist: true })
    } catch (e: any) {
      ctx.message.error(String(e))
      return
    }
    if (platform === 'android' && !(await ctx.ensureAndroidModuleConfigReadyForBuild())) return
    if (platform === 'harmony' && !(await ctx.ensureHarmonyModuleConfigReadyForBuild())) return
    const buildManifestInfo = ctx.selectedManifestInfoForBuild(manifestInfo)
    const moduleConfig = platform === 'android' ? ctx.buildAndroidModuleConfigPayload() : undefined
    await ctx.persistAndroidModuleConfigCache()
    await ctx.persistIosModuleConfigCache()

    const buildId = ctx.buildStore.startBuild(runProjectId, platform, generateProjectKind(platform))
    ctx.currentBuildId.value = buildId
    ctx.buildStore.setActiveEventBuildId(buildId)
    const startedAt = new Date()
    await ctx.createBuildRecord(buildId, platform, startedAt, runProjectId, runProjectName, importedResourcePath)
    await ctx.appendManifestLog(buildId, buildManifestInfo, platform)
    try {
      const payload: Record<string, unknown> = { projectId: runProjectId, resourcePath: importedResourcePath, buildId, manifestInfo: buildManifestInfo }
      if (moduleConfig) payload.moduleConfig = moduleConfig
      const projectPath = await invoke<string>(generateProjectCommand(platform), payload)
      await ctx.buildStore.flushBuildLogs(buildId)
      await ctx.appendFinalLog(buildId, 'success')
      await ctx.finalizeBuildRecord(buildId, 'success', startedAt, null)
      ctx.buildStore.stopBuild(buildId, true, { generatedProjectPath: projectPath })
      ctx.message.success(`${platformProjectName(platform)}项目已生成: ${projectPath}`)
    } catch (e: any) {
      await ctx.buildStore.flushBuildLogs(buildId)
      await ctx.appendFinalLog(buildId, 'failed', String(e))
      await ctx.finalizeBuildRecord(buildId, 'failed', startedAt, null, String(e))
      ctx.buildStore.failBuild(buildId, String(e))
      ctx.message.error(`生成${platformProjectName(platform)}项目失败: ${String(e)}`)
    } finally {
      await ctx.buildStore.flushBuildLogs(buildId)
      if (ctx.buildStore.activeEventBuildId === buildId) ctx.buildStore.setActiveEventBuildId(null)
    }
  }

  return {
    canGenerateNativeProject,
    canGenerateIosProject,
    generateAndroidProject,
    generateIosProject,
    generateHarmonyProject,
    startBuild
  }
}
