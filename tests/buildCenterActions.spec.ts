import { ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createBuildCenterActions } from '../src/views/build-center/buildCenterActions'
import type { BuildArtifact, Platform } from '../src/views/build-center/types'

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

function createContext(selectedPlatforms: Platform[]) {
  let activeEventBuildId: string | null = null
  const ctx = {
    dialog: { create: vi.fn() },
    message: {
      error: vi.fn(),
      success: vi.fn(),
      warning: vi.fn(),
    },
    buildStore: {
      startBuild: vi.fn((_projectId: string, platform: Platform) => `build-${platform}`),
      setActiveEventBuildId: vi.fn((buildId: string | null) => {
        activeEventBuildId = buildId
      }),
      appendBuildLogLines: vi.fn().mockResolvedValue(undefined),
      flushBuildLogs: vi.fn().mockResolvedValue(undefined),
      stopBuild: vi.fn(),
      failBuild: vi.fn(),
      get activeEventBuildId() {
        return activeEventBuildId
      },
    },
    projectId: ref('project-1'),
    currentBuildId: ref<string | null>(null),
    scanResult: ref({ importedPath: '/tmp/imported-resources' }),
    selectedPlatforms: ref([...selectedPlatforms]),
    singleSelectedPlatform: ref<Platform | null>(null),
    isBuildLocked: ref(false),
    selectedNeedsIosConfig: ref(selectedPlatforms.includes('ios')),
    iosModuleConfigLoading: ref(false),
    iosModuleMissingRequired: ref<string[]>([]),
    iosBuildReady: ref(true),
    getProjectName: vi.fn(() => '测试项目'),
    ensureManifestInfoLoaded: vi.fn().mockResolvedValue({ appid: '__UNI__TEST' }),
    ensureAndroidModuleConfigReadyForBuild: vi.fn().mockResolvedValue(true),
    ensureIosModuleConfigReadyForBuild: vi.fn().mockResolvedValue(true),
    ensureHarmonyModuleConfigReadyForBuild: vi.fn().mockResolvedValue(true),
    selectedManifestInfoForBuild: vi.fn((manifest: object) => manifest),
    buildAndroidModuleConfigPayload: vi.fn(() => ({ mapsKey: 'test-key' })),
    persistAndroidModuleConfigCache: vi.fn().mockResolvedValue(undefined),
    persistIosModuleConfigCache: vi.fn().mockResolvedValue(undefined),
    cleanupBuildTemporaryFiles: vi.fn().mockResolvedValue(['[info] cleanup']),
    appendCleanupLines: vi.fn().mockResolvedValue(undefined),
    appendManifestLog: vi.fn().mockResolvedValue(undefined),
    appendFinalLog: vi.fn().mockResolvedValue(undefined),
    createBuildRecord: vi.fn().mockResolvedValue(undefined),
    finalizeBuildRecord: vi.fn().mockResolvedValue(undefined),
    iosIconCount: vi.fn(() => 0),
    iosPrivacyDescriptionCount: vi.fn(() => 0),
  }
  return ctx
}

describe('buildCenterActions.startBuild', () => {
  beforeEach(() => {
    invokeMock.mockImplementation(async (_command: string, payload: { buildId?: string; request?: { buildId: string } }) => {
      const buildId = payload.buildId || payload.request?.buildId || 'unknown'
      return {
        platform: 'android',
        path: `/tmp/${buildId}.artifact`,
        fileName: `${buildId}.artifact`,
        sizeBytes: 1,
        buildId,
      } satisfies BuildArtifact
    })
  })

  it('按本次选择分流 GitHub、iOS 本地与固定 HarmonyOS 本地命令', async () => {
    const platforms: Platform[] = ['android', 'ios', 'harmony']
    const ctx = createContext(platforms)
    const { startBuild } = createBuildCenterActions(ctx)

    await startBuild({
      executionModes: {
        android: 'github',
        ios: 'local',
        harmony: 'github',
      },
      iosPackagingMode: 'localPod',
    }, platforms)

    expect(invokeMock).toHaveBeenCalledTimes(3)
    expect(invokeMock).toHaveBeenNthCalledWith(1, 'run_github_cloud_build', {
      request: expect.objectContaining({
        projectId: 'project-1',
        platform: 'android',
        buildId: 'build-android',
        moduleConfig: { mapsKey: 'test-key' },
      }),
    })
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'build_ios_ipa', expect.objectContaining({
      projectId: 'project-1',
      buildId: 'build-ios',
      iosPackagingMode: 'localPod',
    }))
    expect(invokeMock).toHaveBeenNthCalledWith(3, 'build_harmony_hap', expect.objectContaining({
      projectId: 'project-1',
      buildId: 'build-harmony',
    }))
    expect(ctx.createBuildRecord.mock.calls[0]?.[6]).toBe('github')
    expect(ctx.createBuildRecord.mock.calls[2]?.[6]).toBeUndefined()
    expect(ctx.scanResult.value).toBeNull()
  })

  it('平台快照变化后中止，不创建构建或调用后端命令', async () => {
    const ctx = createContext(['android'])
    const { startBuild } = createBuildCenterActions(ctx)

    await startBuild({ executionModes: { android: 'local' } }, ['ios'])

    expect(ctx.message.warning).toHaveBeenCalledWith('平台选择已变化，请重新选择本次打包方式')
    expect(ctx.buildStore.startBuild).not.toHaveBeenCalled()
    expect(invokeMock).not.toHaveBeenCalled()
  })

  it('iOS 缺少集成方式时中止且给出错误', async () => {
    const ctx = createContext(['ios'])
    const { startBuild } = createBuildCenterActions(ctx)

    await startBuild({ executionModes: { ios: 'github' } }, ['ios'])

    expect(ctx.message.error).toHaveBeenCalledWith('请选择 iOS 集成方式')
    expect(ctx.buildStore.startBuild).not.toHaveBeenCalled()
    expect(invokeMock).not.toHaveBeenCalled()
  })
})
