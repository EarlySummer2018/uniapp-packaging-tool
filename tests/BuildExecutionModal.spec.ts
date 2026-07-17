import { defineComponent, h, nextTick, type Component, type PropType } from 'vue'
import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import BuildExecutionModal from '../src/views/build-center/BuildExecutionModal.vue'
import type {
  BuildExecutionAvailability,
  BuildExecutionSource,
  BuildStartSelection,
  Platform,
} from '../src/views/build-center/types'

const ModalStub = defineComponent({
  name: 'NModal',
  props: {
    show: Boolean,
    maskClosable: Boolean,
  },
  emits: ['update:show'],
  setup(props, { slots }) {
    return () => props.show
      ? h('div', { 'data-testid': 'modal' }, [slots.default?.(), slots.action?.()])
      : null
  },
})

const PassthroughStub = defineComponent({
  inheritAttrs: false,
  setup(_, { slots }) {
    return () => h('div', slots.default?.())
  },
})

const RadioGroupStub = defineComponent({
  name: 'NRadioGroup',
  inheritAttrs: false,
  props: {
    value: String as PropType<string | undefined>,
  },
  emits: ['update:value'],
  setup(_, { slots }) {
    return () => h('div', { 'data-testid': 'radio-group' }, slots.default?.())
  },
})

const RadioButtonStub = defineComponent({
  name: 'NRadioButton',
  inheritAttrs: false,
  props: {
    value: String,
    disabled: Boolean,
  },
  setup(props, { slots }) {
    return () => h('button', {
      type: 'button',
      disabled: props.disabled,
      'data-radio-value': props.value,
    }, slots.default?.())
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
  setup(props, { emit, slots }) {
    return () => h('button', {
      type: 'button',
      disabled: props.disabled,
      'data-loading': String(props.loading),
      onClick: () => emit('click'),
    }, slots.default?.())
  },
})

const enabled = (reason = '可用') => ({ enabled: true, reason })
const disabled = (reason = '不可用') => ({ enabled: false, reason })

function availability(
  overrides: Partial<Record<Platform, Partial<BuildExecutionAvailability>>> = {},
): Record<Platform, BuildExecutionAvailability> {
  return {
    android: {
      local: enabled('Android 本地可用'),
      github: enabled('Android GitHub 可用'),
      ...overrides.android,
    },
    ios: {
      local: enabled('iOS 本地可用'),
      github: enabled('iOS GitHub 可用'),
      ...overrides.ios,
    },
    harmony: {
      local: enabled('HarmonyOS 本地可用'),
      ...overrides.harmony,
    },
  }
}

function mountModal(
  platforms: Platform[],
  options: {
    availability?: Record<Platform, BuildExecutionAvailability>
    loading?: boolean
  } = {},
) {
  return mount(BuildExecutionModal, {
    props: {
      show: false,
      platforms,
      loading: options.loading ?? false,
      availability: options.availability ?? availability(),
      sdkInspections: {},
    },
    global: {
      stubs: {
        Modal: ModalStub,
        Alert: PassthroughStub,
        Button: ButtonStub,
        Icon: PassthroughStub,
        RadioButton: RadioButtonStub,
        RadioGroup: RadioGroupStub,
        Space: PassthroughStub,
        Spin: PassthroughStub,
        Tag: PassthroughStub,
        Text: PassthroughStub,
        CloudOutline: true,
        HardwareChipOutline: true,
        LogoAndroid: true,
        LogoApple: true,
        PhonePortraitOutline: true,
      } satisfies Record<string, Component | boolean>,
    },
  })
}

async function openModal(wrapper: ReturnType<typeof mountModal>) {
  await wrapper.setProps({ show: true })
  await nextTick()
}

function buttonByText(wrapper: ReturnType<typeof mountModal>, text: string) {
  const button = wrapper.findAll('button').find(item => item.text().includes(text))
  if (!button) throw new Error(`找不到按钮：${text}`)
  return button
}

function radioGroups(wrapper: ReturnType<typeof mountModal>) {
  return wrapper.findAllComponents(RadioGroupStub)
}

async function selectMode(
  wrapper: ReturnType<typeof mountModal>,
  groupIndex: number,
  mode: BuildExecutionSource | 'autoMigration' | 'localPod',
) {
  radioGroups(wrapper)[groupIndex].vm.$emit('update:value', mode)
  await nextTick()
}

describe('BuildExecutionModal', () => {
  it('禁止点击遮罩关闭，避免误取消本次选择', async () => {
    const wrapper = mountModal(['android'])
    await openModal(wrapper)

    expect(wrapper.findComponent(ModalStub).props('maskClosable')).toBe(false)
  })

  it('每次打开时 Android 和 iOS 均不预选执行位置', async () => {
    const wrapper = mountModal(['android', 'ios'])
    await openModal(wrapper)

    const groups = radioGroups(wrapper)
    expect(groups).toHaveLength(3)
    expect(groups[0].props('value')).toBeUndefined()
    expect(groups[1].props('value')).toBeUndefined()
    expect(wrapper.text().match(/请选择/g)).toHaveLength(2)
    expect(buttonByText(wrapper, '确认并开始打包').attributes('disabled')).toBeDefined()

    await selectMode(wrapper, 0, 'local')
    await wrapper.setProps({ show: false })
    await openModal(wrapper)

    expect(radioGroups(wrapper)[0].props('value')).toBeUndefined()
    expect(buttonByText(wrapper, '确认并开始打包').attributes('disabled')).toBeDefined()
  })

  it('HarmonyOS 固定为本地并将 local 写入确认 payload', async () => {
    const wrapper = mountModal(['harmony'])
    await openModal(wrapper)

    expect(wrapper.text()).toContain('仅支持本地')
    expect(wrapper.text()).toContain('本地打包')
    expect(radioGroups(wrapper)).toHaveLength(0)
    expect(buttonByText(wrapper, '确认并开始打包').attributes('disabled')).toBeUndefined()

    await buttonByText(wrapper, '确认并开始打包').trigger('click')
    expect(wrapper.emitted<BuildStartSelection[]>('confirm')).toEqual([[{
      executionModes: { harmony: 'local' },
      iosPackagingMode: undefined,
    }]])
  })

  it('iOS 集成方式默认 autoMigration', async () => {
    const wrapper = mountModal(['ios'])
    await openModal(wrapper)

    expect(radioGroups(wrapper)[1].props('value')).toBe('autoMigration')
    await selectMode(wrapper, 0, 'local')
    await buttonByText(wrapper, '确认并开始打包').trigger('click')

    expect(wrapper.emitted<BuildStartSelection[]>('confirm')?.[0]?.[0]).toEqual({
      executionModes: { ios: 'local' },
      iosPackagingMode: 'autoMigration',
    })
  })

  it('禁用不可用的执行位置，且不可用选择不能通过确认', async () => {
    const wrapper = mountModal(['android'], {
      availability: availability({
        android: { local: disabled('缺少 Android 本地环境') },
      }),
    })
    await openModal(wrapper)

    const localRadio = wrapper.find('[data-radio-value="local"]')
    const githubRadio = wrapper.find('[data-radio-value="github"]')
    expect(localRadio.attributes('disabled')).toBeDefined()
    expect(githubRadio.attributes('disabled')).toBeUndefined()

    await selectMode(wrapper, 0, 'local')
    expect(buttonByText(wrapper, '确认并开始打包').attributes('disabled')).toBeDefined()
    await selectMode(wrapper, 0, 'github')
    expect(buttonByText(wrapper, '确认并开始打包').attributes('disabled')).toBeUndefined()
  })

  it('确认时发送各平台的一次性选择与 iOS localPod 方式', async () => {
    const wrapper = mountModal(['android', 'ios', 'harmony'])
    await openModal(wrapper)

    await selectMode(wrapper, 0, 'github')
    await selectMode(wrapper, 1, 'local')
    await selectMode(wrapper, 2, 'localPod')
    await buttonByText(wrapper, '确认并开始打包').trigger('click')

    expect(wrapper.emitted<BuildStartSelection[]>('confirm')).toEqual([[{
      executionModes: {
        android: 'github',
        ios: 'local',
        harmony: 'local',
      },
      iosPackagingMode: 'localPod',
    }]])
  })

  it('取消只关闭弹窗，不发送 confirm', async () => {
    const wrapper = mountModal(['android'])
    await openModal(wrapper)

    await buttonByText(wrapper, '取消').trigger('click')

    expect(wrapper.emitted('update:show')).toEqual([[false]])
    expect(wrapper.emitted('confirm')).toBeUndefined()
  })
})
