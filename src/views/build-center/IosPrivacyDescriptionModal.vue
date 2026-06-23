<script setup lang="ts">
import {
  NButton,
  NModal,
  NSpace
} from 'naive-ui'
import PrivacyDescriptionWorkbench from './PrivacyDescriptionWorkbench.vue'
import type { IosPrivacyDescriptionItem } from './types'

defineProps<{
  show: boolean
  items: IosPrivacyDescriptionItem[]
  missingCount: number
  isBuildLocked: boolean
}>()

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
  (e: 'update-item', item: IosPrivacyDescriptionItem, value: string): void
}>()
</script>

<template>
  <n-modal
    :show="show"
    preset="card"
    title="iOS 权限说明"
    class="ios-privacy-modal"
    :mask-closable="false"
    @update:show="value => emit('update:show', value)"
  >
    <n-space vertical :size="14">
      <PrivacyDescriptionWorkbench
        :items="items"
        :missing-count="missingCount"
        :is-build-locked="isBuildLocked"
        @update-item="(item, value) => emit('update-item', item, value)"
      />

      <n-space justify="end">
        <n-button type="primary" @click="emit('update:show', false)">完成</n-button>
      </n-space>
    </n-space>
  </n-modal>
</template>
