<script setup lang="ts">
import {
  NAlert,
  NButton,
  NCard,
  NProgress,
  NSpace,
  NText
} from 'naive-ui'
import LogPanel from '../../components/LogPanel.vue'
import type { BuildArtifact } from './types'
import type { BuildLog, BuildStatus } from '../../stores/build'

defineProps<{
  logs: BuildLog[]
  progress: number
  status: BuildStatus | undefined
  visibleArtifacts: BuildArtifact[]
  currentGeneratedProjectPath: string | null
  currentGeneratedProjectLabel: string
}>()

const emit = defineEmits<{
  (e: 'open-generated-project'): void
}>()
</script>

<template>
  <n-card data-guide="build-log" title="构建日志" class="build-section-card log-section-card">
    <LogPanel :logs="logs" height="380px" />
    <n-progress
      class="build-progress"
      type="line"
      indicator-placement="inside"
      :percentage="progress"
      :processing="status === 'building'"
      :status="status === 'failed' ? 'error' : status === 'success' ? 'success' : 'default'"
    />
    <div class="alert-stack log-result-stack">
      <n-alert v-for="artifact in visibleArtifacts" :key="artifact.path" type="success">
        {{ artifact.platform }}: <n-text code class="path-text">{{ artifact.path }}</n-text>
      </n-alert>
      <n-alert v-if="currentGeneratedProjectPath" type="info">
        <n-space align="center">
          <span>{{ currentGeneratedProjectLabel }}:</span>
          <n-text code class="path-text">{{ currentGeneratedProjectPath }}</n-text>
          <n-button size="small" @click="emit('open-generated-project')">打开目录</n-button>
        </n-space>
      </n-alert>
    </div>
  </n-card>
</template>
