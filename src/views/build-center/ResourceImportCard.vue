<script setup lang="ts">
import {
  NAlert,
  NButton,
  NCard,
  NIcon,
  NSpace,
  NText
} from 'naive-ui'
import { FolderOpenOutline } from '@vicons/ionicons5'
import type { ResourceScanResult } from './types'

defineProps<{
  importing: boolean
  isBuildLocked: boolean
  scanResult: ResourceScanResult | null
  insightAppId: string | number
  insightVersionName: string | number
  insightVersionCode: string | number
  insightManifestPath: string | number
  manifestReadWarning: string
}>()

const emit = defineEmits<{
  (e: 'choose-resource'): void
}>()
</script>

<template>
  <n-card data-guide="resource-import" title="1. 导入 UniApp 资源" class="build-step-card import-card">
    <n-space>
      <n-button type="primary" :loading="importing" :disabled="isBuildLocked" @click="emit('choose-resource')">
        <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
        选择 resources 目录
      </n-button>
    </n-space>
    <div v-if="scanResult" class="scan-result">
      <div class="alert-stack">
        <n-alert type="success" title="资源扫描完成">
          <n-space vertical :size="8">
            <n-text>AppId: <n-text code class="path-text">{{ insightAppId }}</n-text></n-text>
            <n-text>版本: {{ insightVersionName }} / {{ insightVersionCode }}</n-text>
            <n-text>资源包根目录: <n-text code class="path-text">{{ scanResult.importedPath }}</n-text></n-text>
            <n-text>应用资源目录: <n-text code class="path-text">{{ scanResult.appResourcePath }}</n-text></n-text>
            <n-text>manifest 路径: <n-text code class="path-text">{{ insightManifestPath }}</n-text></n-text>
          </n-space>
        </n-alert>
        <n-alert v-for="warning in scanResult.warnings" :key="warning" type="warning">
          {{ warning }}
        </n-alert>
        <n-alert v-if="manifestReadWarning" type="warning">
          {{ manifestReadWarning }}
        </n-alert>
      </div>
    </div>
  </n-card>
</template>
