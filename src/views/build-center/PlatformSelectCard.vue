<script setup lang="ts">
import {
  NButton,
  NCard,
  NIcon,
  NSpace,
  NText
} from 'naive-ui'
import { LogoAndroid, LogoApple, PhonePortraitOutline, PlayOutline } from '@vicons/ionicons5'
import PlatformCard from '../../components/PlatformCard.vue'
import type { Platform } from './types'
import type { PlatformOption } from './platforms'

defineProps<{
  platforms: PlatformOption[]
  selectedPlatforms: Platform[]
  isBuildLocked: boolean
  buildDisabledReason: string
  canBuild: boolean
  canGenerateAndroid: boolean
  canGenerateIos: boolean
  canGenerateHarmony: boolean
  packageBuildLoading: boolean
  androidGenerateLoading: boolean
  iosGenerateLoading: boolean
  harmonyGenerateLoading: boolean
  singleSelectedPlatform: Platform | null
}>()

const emit = defineEmits<{
  (e: 'toggle-platform', platform: Platform): void
  (e: 'generate-android'): void
  (e: 'generate-ios'): void
  (e: 'generate-harmony'): void
  (e: 'start-build'): void
}>()
</script>

<template>
  <n-card data-guide="platform-select" title="2. 选择平台" class="build-step-card">
    <PlatformCard
      :platforms="platforms"
      :selected-platforms="selectedPlatforms"
      :disabled="isBuildLocked"
      @toggle="platform => emit('toggle-platform', platform)"
    />
    <n-space justify="end" class="build-action-row">
      <n-text v-if="buildDisabledReason && !canBuild" depth="3">{{ buildDisabledReason }}</n-text>
      <n-button v-if="singleSelectedPlatform === 'android'" type="primary" :disabled="!canGenerateAndroid" :loading="androidGenerateLoading" @click="emit('generate-android')">
        <template #icon><n-icon><LogoAndroid /></n-icon></template>
        生成安卓项目
      </n-button>
      <n-button v-if="singleSelectedPlatform === 'ios'" type="primary" :disabled="!canGenerateIos" :loading="iosGenerateLoading" @click="emit('generate-ios')">
        <template #icon><n-icon><LogoApple /></n-icon></template>
        生成苹果项目
      </n-button>
      <n-button v-if="singleSelectedPlatform === 'harmony'" type="primary" :disabled="!canGenerateHarmony" :loading="harmonyGenerateLoading" @click="emit('generate-harmony')">
        <template #icon><n-icon><PhonePortraitOutline /></n-icon></template>
        生成鸿蒙项目
      </n-button>
      <n-button type="success" :disabled="!canBuild" :loading="packageBuildLoading" @click="emit('start-build')">
        <template #icon><n-icon><PlayOutline /></n-icon></template>
        开始打包
      </n-button>
    </n-space>
  </n-card>
</template>
