<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { NLog, NButton, NIcon, useMessage } from 'naive-ui'
import { CopyOutline, CheckmarkOutline } from '@vicons/ionicons5'
import type { BuildLog } from '../stores/build'

interface Props {
  logs: BuildLog[]
  height?: string
}

const props = withDefaults(defineProps<Props>(), {
  height: '400px'
})

const message = useMessage()
const logRef = ref<InstanceType<typeof NLog> | null>(null)
const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | null = null

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

// 计算高度（减去工具栏高度）
const computedHeight = computed(() => {
  const baseHeight = parseInt(props.height) || 400
  return `${baseHeight - 40}px`
})

// 复制按钮文本
const copyButtonText = computed(() => copied.value ? '已复制' : '复制日志')

// 复制图标
const copyButtonIcon = computed(() => copied.value ? CheckmarkOutline : CopyOutline)

async function copyLogs() {
  if (!props.logs.length) {
    message.warning('暂无日志可复制')
    return
  }

  try {
    await navigator.clipboard.writeText(logLines.value)

    copied.value = true
    message.success(`已复制 ${props.logs.length} 条日志到剪贴板`)

    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => {
      copied.value = false
    }, 2000)

  } catch (err) {
    console.error('复制失败:', err)
    fallbackCopy(logLines.value)
  }
}

function fallbackCopy(text: string) {
  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.position = 'fixed'
  textarea.style.left = '-9999px'
  document.body.appendChild(textarea)
  textarea.select()

  try {
    document.execCommand('copy')
    copied.value = true
    message.success('已复制到剪贴板')

    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => { copied.value = false }, 2000)
  } catch (e) {
    message.error('复制失败，请手动选择文本复制')
  }

  document.body.removeChild(textarea)
}

function scrollToBottom() {
  if (logRef.value) {
    const el = logRef.value.$el
    if (el) {
      el.scrollTop = el.scrollHeight
    }
  }
}

onUnmounted(() => {
  if (copyTimer) clearTimeout(copyTimer)
})
</script>

<template>
  <div class="log-panel-wrapper">
    <div class="log-toolbar">
      <div class="toolbar-left">
        <span class="log-count">共 {{ props.logs.length }} 条日志</span>
      </div>
      <div class="toolbar-right">
        <n-button
          size="small"
          quaternary
          :type="copied ? 'success' : 'primary'"
          :disabled="!props.logs.length"
          @click="copyLogs"
          aria-label="复制构建日志到剪贴板"
        >
          <template #icon>
            <n-icon :component="copyButtonIcon" />
          </template>
          {{ copyButtonText }}
        </n-button>
      </div>
    </div>

    <n-log
      ref="logRef"
      :log="logLines"
      :height="computedHeight"
      :loading="false"
      :font-size="13"
      :rows="15"
      language="text"
      @scroll-to-bottom="scrollToBottom"
    />
  </div>
</template>

<style scoped>
.log-panel-wrapper {
  position: relative;
  border: 1px solid var(--n-border-color);
  border-radius: var(--n-border-radius);
  overflow: hidden;
}

.log-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--n-color);
  border-bottom: 1px solid var(--n-border-color);
  font-size: 12px;
}

.toolbar-left {
  color: var(--n-text-color-3);
}

.toolbar-right {
  display: flex;
  gap: 8px;
}

.log-count {
  font-size: 12px;
}
</style>
