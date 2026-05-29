<script setup lang="ts">
import { NCard, NList, NListItem, NButton, NIcon, NText, NSpace, NTag } from 'naive-ui'
import { Download, TrashBinOutline, DocumentTextOutline } from '@vicons/ionicons5'
import type { BuildTask } from '../stores/build'

interface Artifact {
  id: string
  buildId: string
  fileName: string
  fileSize: number
  filePath: string
  createdAt: string
  platform: BuildTask['platform']
}

defineProps<{
  artifacts: Artifact[]
  loading?: boolean
}>()

defineEmits<{
  (e: 'download', artifact: Artifact): void
  (e: 'delete', artifact: Artifact): void
  (e: 'open-folder', path: string): void
}>()

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

function getPlatformColor(platform: Artifact['platform']): string {
  switch (platform) {
    case 'android': return '#3DDC84'
    case 'ios': return '#007AFF'
    case 'harmony': return '#E33B64'
    default: return '#666'
  }
}

function getPlatformLabel(platform: Artifact['platform']): string {
  switch (platform) {
    case 'android': return 'Android'
    case 'ios': return 'iOS'
    case 'harmony': return '鸿蒙'
    default: return '未知'
  }
}
</script>

<template>
  <n-card title="构建产物" size="small">
    <n-list v-if="artifacts.length > 0" bordered>
      <n-list-item
        v-for="artifact in artifacts"
        :key="artifact.id"
      >
        <div class="artifact-item">
          <div class="artifact-info">
            <div class="artifact-header">
              <n-icon :size="24" color="#666">
                <DocumentTextOutline />
              </n-icon>
              <n-text strong>{{ artifact.fileName }}</n-text>
              <n-tag 
                :color="{ color: getPlatformColor(artifact.platform), textColor: '#fff' }"
                size="small"
                round
              >
                {{ getPlatformLabel(artifact.platform) }}
              </n-tag>
            </div>
            
            <n-text depth="3" class="artifact-meta">
              {{ formatFileSize(artifact.fileSize) }} · 
              {{ new Date(artifact.createdAt).toLocaleString() }}
            </n-text>
          </div>
          
          <n-space>
            <n-button 
              size="small" 
              quaternary 
              type="primary"
              @click="$emit('download', artifact)"
            >
              <template #icon>
                <n-icon><Download /></n-icon>
              </template>
              下载
            </n-button>
            
            <n-button 
              size="small" 
              quaternary
              @click="$emit('open-folder', artifact.filePath)"
            >
              打开位置
            </n-button>
            
            <n-button 
              size="small" 
              quaternary 
              type="error"
              @click="$emit('delete', artifact)"
            >
              <template #icon>
                <n-icon><TrashBinOutline /></n-icon>
              </template>
            </n-button>
          </n-space>
        </div>
      </n-list-item>
    </n-list>
    
    <n-text v-else depth="3" class="empty-hint">
      暂无构建产物
    </n-text>
  </n-card>
</template>

<style scoped>
.artifact-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  width: 100%;
}

.artifact-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.artifact-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.artifact-meta {
  font-size: 12px;
  margin-left: 32px;
}

.empty-hint {
  display: block;
  text-align: center;
  padding: 40px 0;
}
</style>
