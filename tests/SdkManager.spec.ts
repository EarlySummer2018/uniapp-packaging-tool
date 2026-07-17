import { flushPromises, shallowMount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SdkManager from '../src/views/SdkManager.vue'

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  message: {
    error: vi.fn(),
    success: vi.fn(),
    warning: vi.fn(),
  },
  dialog: {
    warning: vi.fn(),
  },
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mocks.invoke,
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-opener', () => ({
  openPath: vi.fn(),
  openUrl: vi.fn(),
}))

vi.mock('naive-ui', async () => {
  const actual = await vi.importActual<typeof import('naive-ui')>('naive-ui')
  return {
    ...actual,
    useMessage: () => mocks.message,
    useDialog: () => mocks.dialog,
  }
})

const firstAndroidFingerprint = `${'a'.repeat(60)}1111`
const secondAndroidFingerprint = `${'b'.repeat(60)}2222`
const omittedAndroidFingerprint = `${'c'.repeat(60)}3333`
const iosFingerprint = `${'d'.repeat(60)}4444`

const cacheEntries = [
  {
    platform: 'android',
    fingerprint: firstAndroidFingerprint,
    compressedSizeBytes: 1024 * 1024,
    uploadedAt: '2026-07-13T08:00:00Z',
    matchesCurrentLocalSdk: true,
  },
  {
    platform: 'android',
    fingerprint: secondAndroidFingerprint,
    compressedSizeBytes: 2 * 1024 * 1024,
    uploadedAt: '2026-07-12T08:00:00Z',
    matchesCurrentLocalSdk: false,
  },
  {
    platform: 'android',
    fingerprint: omittedAndroidFingerprint,
    compressedSizeBytes: 3 * 1024 * 1024,
    uploadedAt: '2026-07-11T08:00:00Z',
    matchesCurrentLocalSdk: false,
  },
  {
    platform: 'ios',
    fingerprint: iosFingerprint,
    compressedSizeBytes: 4 * 1024 * 1024,
    uploadedAt: '2026-07-10T08:00:00Z',
    matchesCurrentLocalSdk: false,
  },
] as const

function shortFingerprint(fingerprint: string) {
  return `${fingerprint.slice(0, 12)}…${fingerprint.slice(-4)}`
}

const CardStub = defineComponent({
  name: 'NCard',
  setup(_, { slots }) {
    return () => h('section', [
      slots.header?.(),
      slots.default?.(),
      slots.action?.(),
    ])
  },
})

const ButtonStub = defineComponent({
  name: 'NButton',
  inheritAttrs: false,
  props: {
    disabled: Boolean,
    loading: Boolean,
  },
  emits: ['click'],
  setup(props, { attrs, emit, slots }) {
    return () => h('button', {
      ...attrs,
      disabled: props.disabled,
      onClick: () => emit('click'),
    }, [slots.icon?.(), slots.default?.()])
  },
})

const FormItemStub = defineComponent({
  name: 'NFormItem',
  props: {
    label: String,
  },
  setup(props, { slots }) {
    return () => h('label', [
      h('span', { class: 'form-item-label' }, props.label),
      slots.default?.(),
    ])
  },
})

async function mountManager() {
  const wrapper = shallowMount(SdkManager, {
    global: {
      renderStubDefaultSlot: true,
      stubs: {
        Card: CardStub,
        Button: ButtonStub,
        FormItem: FormItemStub,
      },
    },
  })
  await flushPromises()
  await flushPromises()
  return wrapper
}

describe('SdkManager GitHub 云端打包配置', () => {
  beforeEach(() => {
    mocks.invoke.mockImplementation(async (command: string) => {
      switch (command) {
        case 'get_global_sdk_config':
          return {
            dcloudAndroidSdkPath: '/sdk/android',
            dcloudIosSdkPath: '/sdk/ios',
            harmonyTemplatePath: '/sdk/harmony',
          }
        case 'get_full_env_report':
          return {}
        case 'get_env_overrides':
        case 'get_build_history':
          return []
        case 'get_github_cloud_build_config':
          return {
            owner: 'example',
            repo: 'private-builds',
            ref: 'main',
            workflowFile: 'cloud-build.yml',
            hasToken: true,
          }
        case 'get_github_sdk_cache_status':
          return cacheEntries.map(entry => ({ ...entry }))
        case 'delete_github_sdk_cache':
          return undefined
        default:
          throw new Error(`unexpected command: ${command}`)
      }
    })
  })

  it('GitHub 页只展示仓库配置，不再展示默认方式或 SDK 下载 URL', async () => {
    const wrapper = await mountManager()
    const text = wrapper.text()

    expect(text).toContain('GitHub Actions 云端打包')
    expect(text).toContain('SDK 缓存状态')
    expect(text).not.toContain('默认方式')
    expect(text).not.toContain('Android SDK 下载 URL')
    expect(text).not.toContain('iOS SDK 下载 URL')

    const labels = wrapper.findAll('.form-item-label').map(item => item.text())
    expect(labels).toEqual(expect.arrayContaining([
      'Owner',
      'Repo',
      'Ref',
      'Workflow 文件',
      'GitHub Token',
    ]))
  })

  it('每个平台最多展示最近两个缓存版本，并标识匹配当前 SDK 的版本', async () => {
    const wrapper = await mountManager()
    const text = wrapper.text()

    expect(text).toContain('Android2 个版本')
    expect(text).toContain('iOS1 个版本')
    expect(text).toContain(shortFingerprint(firstAndroidFingerprint))
    expect(text).toContain(shortFingerprint(secondAndroidFingerprint))
    expect(text).not.toContain(shortFingerprint(omittedAndroidFingerprint))
    expect(text).toContain(shortFingerprint(iosFingerprint))
    expect(text.match(/当前 SDK/g)).toHaveLength(1)
  })

  it('删除缓存先弹出确认，确认后再调用删除命令并刷新状态', async () => {
    const wrapper = await mountManager()
    const deleteButton = wrapper.findAll('[aria-label="删除 Android SDK 缓存"]')[0]

    expect(deleteButton).toBeDefined()
    await deleteButton.trigger('click')

    expect(mocks.invoke).not.toHaveBeenCalledWith('delete_github_sdk_cache', expect.anything())
    expect(mocks.dialog.warning).toHaveBeenCalledTimes(1)
    const confirmation = mocks.dialog.warning.mock.calls[0][0]
    expect(confirmation).toMatchObject({
      title: '删除 Android SDK 缓存',
      positiveText: '确认删除',
      negativeText: '取消',
    })

    await confirmation.onPositiveClick()

    expect(mocks.invoke).toHaveBeenCalledWith('delete_github_sdk_cache', {
      platform: 'android',
      fingerprint: firstAndroidFingerprint,
    })
    expect(mocks.invoke.mock.calls.filter(([command]) => command === 'get_github_sdk_cache_status')).toHaveLength(2)
    expect(mocks.message.success).toHaveBeenCalledWith('Android SDK 缓存已删除')
  })
})
