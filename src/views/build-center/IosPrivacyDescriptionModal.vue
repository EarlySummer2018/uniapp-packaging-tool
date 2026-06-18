<script setup lang="ts">
import {
  NAlert,
  NButton,
  NInput,
  NModal,
  NSpace,
  NTag,
  NText
} from 'naive-ui'
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
      <n-alert :type="missingCount ? 'warning' : 'success'">
        <n-text v-if="missingCount">
          还有 {{ missingCount }} 项权限说明未填写，补齐后才能生成 iOS 工程或 IPA。
        </n-text>
        <n-text v-else>权限说明已填写完整。</n-text>
      </n-alert>

      <div class="ios-privacy-field-list">
        <div
          v-for="item in items"
          :key="item.key"
          class="ios-privacy-field"
          :class="{ 'ios-privacy-field--missing': item.missing }"
        >
          <div class="ios-privacy-field-head">
            <n-text strong>{{ item.label }}</n-text>
            <n-tag v-if="item.missing" size="small" type="error" :bordered="false">必填</n-tag>
            <n-tag v-else size="small" type="success" :bordered="false">已填写</n-tag>
          </div>
          <n-input
            type="textarea"
            :autosize="{ minRows: 2, maxRows: 4 }"
            :value="item.value"
            :placeholder="item.placeholder"
            :status="item.missing ? 'error' : undefined"
            :disabled="isBuildLocked"
            @update:value="value => emit('update-item', item, value)"
          />
        </div>
      </div>

      <n-space justify="end">
        <n-button type="primary" @click="emit('update:show', false)">完成</n-button>
      </n-space>
    </n-space>
  </n-modal>
</template>
