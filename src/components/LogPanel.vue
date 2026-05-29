<script setup lang="ts">
import { ref, computed } from 'vue'
import { NLog } from 'naive-ui'
import type { BuildLog } from '../stores/build'

interface Props {
  logs: BuildLog[]
  height?: string
}

const props = withDefaults(defineProps<Props>(), {
  height: '400px'
})

const logRef = ref<InstanceType<typeof NLog> | null>(null)

const logLines = computed(() => {
  return props.logs.map(log => {
    const levelMap = {
      info: '',
      warn: '[WARN]',
      error: '[ERROR]',
      success: '[SUCCESS]'
    }
    return `${new Date(log.timestamp).toLocaleTimeString()} ${levelMap[log.level]} ${log.message}`
  }).join('\n')
})

function scrollToBottom() {
  if (logRef.value) {
    const el = logRef.value.$el
    if (el) {
      el.scrollTop = el.scrollHeight
    }
  }
}
</script>

<template>
  <n-log
    ref="logRef"
    :log="logLines"
    :height="height"
    :loading="false"
    :font-size="13"
    :rows="15"
    language="text"
    @scroll-to-bottom="scrollToBottom"
  />
</template>
