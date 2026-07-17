<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  NAlert,
  NButton,
  NIcon,
  NModal,
  NRadioButton,
  NRadioGroup,
  NSpace,
  NSpin,
  NTag,
  NText,
} from 'naive-ui'
import {
  CloudOutline,
  HardwareChipOutline,
  LogoAndroid,
  LogoApple,
  PhonePortraitOutline,
} from '@vicons/ionicons5'
import type {
  BuildExecutionAvailability,
  BuildExecutionSource,
  BuildStartSelection,
  IosPackagingMode,
  LocalSdkCacheInspection,
  Platform,
} from './types'

const props = defineProps<{
  show: boolean
  platforms: Platform[]
  loading: boolean
  availability: Partial<Record<Platform, BuildExecutionAvailability>>
  sdkInspections: Partial<Record<Platform, LocalSdkCacheInspection>>
}>()

const emit = defineEmits<{
  (event: 'update:show', value: boolean): void
  (event: 'confirm', selection: BuildStartSelection): void
}>()

const executionModes = ref<Partial<Record<Platform, BuildExecutionSource>>>({})
const iosPackagingMode = ref<IosPackagingMode>('autoMigration')

watch(() => props.show, show => {
  if (!show) return
  executionModes.value = props.platforms.includes('harmony') ? { harmony: 'local' } : {}
  iosPackagingMode.value = 'autoMigration'
})

const platformMeta: Record<Platform, { label: string; icon: typeof LogoAndroid }> = {
  android: { label: 'Android', icon: LogoAndroid },
  ios: { label: 'iOS', icon: LogoApple },
  harmony: { label: 'HarmonyOS', icon: PhonePortraitOutline },
}

const selectionComplete = computed(() => props.platforms.every(platform => {
  if (platform === 'harmony') return props.availability[platform]?.local.enabled === true
  const mode = executionModes.value[platform]
  if (!mode) return false
  return props.availability[platform]?.[mode]?.enabled === true
}))

function updateExecutionMode(platform: Platform, value: BuildExecutionSource) {
  if (platform === 'harmony') return
  executionModes.value = { ...executionModes.value, [platform]: value }
}

function availabilityFor(platform: Platform, mode: BuildExecutionSource) {
  return props.availability[platform]?.[mode] || { enabled: false, reason: '正在检测可用性' }
}

function formatBytes(size: number) {
  if (!Number.isFinite(size) || size <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const unitIndex = Math.min(Math.floor(Math.log(size) / Math.log(1024)), units.length - 1)
  const value = size / 1024 ** unitIndex
  return `${value >= 10 || unitIndex === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unitIndex]}`
}

function shortFingerprint(fingerprint: string) {
  return fingerprint.length > 20 ? `${fingerprint.slice(0, 16)}…${fingerprint.slice(-4)}` : fingerprint
}

function close() {
  emit('update:show', false)
}

function confirm() {
  if (props.loading || !selectionComplete.value) return
  emit('confirm', {
    executionModes: { ...executionModes.value },
    iosPackagingMode: props.platforms.includes('ios') ? iosPackagingMode.value : undefined,
  })
}
</script>

<template>
  <n-modal
    :show="show"
    preset="card"
    title="选择本次打包方式"
    class="build-execution-modal"
    style="width: min(720px, calc(100vw - 32px)); max-height: calc(100vh - 48px);"
    content-scrollable
    :mask-closable="false"
    @update:show="value => emit('update:show', value)"
  >
    <n-space vertical :size="16">
      <n-alert type="info">
        以下选择仅用于本次打包，不会保存为默认设置。GitHub 云端打包使用“DCloud 离线SDK”页配置的本地 SDK。
      </n-alert>

      <n-spin :show="loading" description="正在重新校验环境、GitHub 配置与 SDK 缓存…">
        <div class="execution-platform-list">
          <section v-for="platform in platforms" :key="platform" class="execution-platform-card">
            <div class="execution-platform-header">
              <n-space align="center" :size="8">
                <n-icon :size="20"><component :is="platformMeta[platform].icon" /></n-icon>
                <n-text strong>{{ platformMeta[platform].label }}</n-text>
              </n-space>
              <n-tag v-if="platform === 'harmony'" size="small" type="default" round>仅支持本地</n-tag>
              <n-tag v-else-if="!executionModes[platform]" size="small" type="warning" round>请选择</n-tag>
            </div>

            <template v-if="platform === 'harmony'">
              <div class="fixed-execution-mode">
                <n-space align="center" :size="8">
                  <n-icon><HardwareChipOutline /></n-icon>
                  <n-text>本地打包</n-text>
                </n-space>
                <n-text
                  depth="3"
                  :type="availabilityFor(platform, 'local').enabled ? 'default' : 'error'"
                  class="execution-mode-reason"
                >
                  {{ availabilityFor(platform, 'local').reason }}
                </n-text>
              </div>
            </template>

            <template v-else>
              <n-radio-group
                :value="executionModes[platform]"
                class="execution-mode-options"
                @update:value="value => updateExecutionMode(platform, value as BuildExecutionSource)"
              >
                <n-radio-button
                  value="local"
                  :disabled="!availabilityFor(platform, 'local').enabled"
                >
                  <n-space align="center" :size="6">
                    <n-icon><HardwareChipOutline /></n-icon>
                    本地打包
                  </n-space>
                </n-radio-button>
                <n-radio-button
                  value="github"
                  :disabled="!availabilityFor(platform, 'github').enabled"
                >
                  <n-space align="center" :size="6">
                    <n-icon><CloudOutline /></n-icon>
                    GitHub 云端打包
                  </n-space>
                </n-radio-button>
              </n-radio-group>

              <div class="execution-availability-notes">
                <n-text
                  depth="3"
                  :type="availabilityFor(platform, 'local').enabled ? 'default' : 'error'"
                >
                  本地：{{ availabilityFor(platform, 'local').reason }}
                </n-text>
                <n-text
                  depth="3"
                  :type="availabilityFor(platform, 'github').enabled ? 'default' : 'error'"
                >
                  GitHub：{{ availabilityFor(platform, 'github').reason }}
                </n-text>
              </div>

              <div v-if="sdkInspections[platform]" class="sdk-upload-summary">
                <n-space align="center" justify="space-between" :wrap="true">
                  <n-space align="center" :size="8">
                    <n-tag
                      size="small"
                      :type="sdkInspections[platform]!.cacheHit ? 'success' : 'warning'"
                      :bordered="false"
                    >
                      {{ sdkInspections[platform]!.cacheHit ? 'SDK 缓存已命中' : '首次需要上传 SDK' }}
                    </n-tag>
                    <n-text depth="3">{{ formatBytes(sdkInspections[platform]!.sizeBytes) }}</n-text>
                  </n-space>
                  <n-text code depth="3">{{ shortFingerprint(sdkInspections[platform]!.fingerprint) }}</n-text>
                </n-space>
                <n-text depth="3" class="sdk-upload-hint">
                  {{ sdkInspections[platform]!.cacheHit
                    ? '云端已有与当前本地 SDK 匹配的完整缓存，本次无需重复上传。'
                    : '云端尚无匹配缓存；确认 GitHub 打包后会分片上传，显示大小为本地 SDK 压缩前体积。' }}
                </n-text>
              </div>
            </template>
          </section>
        </div>

        <section v-if="platforms.includes('ios')" class="ios-packaging-section">
          <div>
            <n-text strong>iOS 集成方式</n-text>
            <n-text depth="3" class="ios-packaging-hint">同样仅对本次 iOS 打包生效</n-text>
          </div>
          <n-radio-group v-model:value="iosPackagingMode">
            <n-space vertical :size="8">
              <n-radio-button value="autoMigration">自动迁移打包</n-radio-button>
              <n-radio-button value="localPod">本地 Pod 打包</n-radio-button>
            </n-space>
          </n-radio-group>
        </section>
      </n-spin>
    </n-space>

    <template #action>
      <n-space justify="end">
        <n-button @click="close">取消</n-button>
        <n-button type="primary" :disabled="loading || !selectionComplete" :loading="loading" @click="confirm">
          确认并开始打包
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<style scoped>
.execution-platform-list {
  display: grid;
  gap: 12px;
}

.execution-platform-card,
.ios-packaging-section {
  padding: 14px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-muted);
}

.execution-platform-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.execution-mode-options {
  width: 100%;
}

.execution-mode-options :deep(.n-radio-button) {
  width: 50%;
}

.execution-mode-options :deep(.n-radio-button__content) {
  display: flex;
  justify-content: center;
}

.execution-availability-notes {
  display: grid;
  gap: 4px;
  margin-top: 10px;
}

.fixed-execution-mode {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--border-soft);
  border-radius: 6px;
  background: var(--surface-color);
}

.execution-mode-reason {
  text-align: right;
}

.sdk-upload-summary {
  margin-top: 12px;
  padding: 10px;
  border: 1px solid var(--border-soft);
  border-radius: 6px;
  background: var(--surface-color);
}

.sdk-upload-hint,
.ios-packaging-hint {
  display: block;
  margin-top: 6px;
}

.ios-packaging-section {
  display: grid;
  grid-template-columns: minmax(180px, 1fr) minmax(220px, 1fr);
  gap: 16px;
  margin-top: 12px;
}

@media (max-width: 640px) {
  .ios-packaging-section {
    grid-template-columns: 1fr;
  }

  .fixed-execution-mode {
    align-items: flex-start;
    flex-direction: column;
  }

  .execution-mode-reason {
    text-align: left;
  }
}
</style>
