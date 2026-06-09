<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { NButton, NIcon, useMessage } from 'naive-ui'
import { CopyOutline, CheckmarkOutline } from '@vicons/ionicons5'
import LogDisplay from './LogDisplay.vue'
import type { BuildLog } from '../stores/build'

interface Props {
  logs: BuildLog[]
  height?: string
}

const props = withDefaults(defineProps<Props>(), {
  height: '400px'
})

const message = useMessage()
const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | null = null

// ===== 性能优化 =====

/** 最大渲染行数，超出部分截断以避免 DOM 爆炸 */
const MAX_RENDER_LINES = 2000

// 转换 BuildLog[] 为 LogDisplay 需要的 LogEntry 格式
const logEntries = computed(() => {
  return props.logs.map(log => ({
    level: log.level,
    message: log.message,
    timestamp: log.timestamp
  }))
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
    // 复制全部日志（不受 MAX_RENDER_LINES 限制）
    const allText = props.logs.map(log => {
      const levelMap: Record<string, string> = {
        info: '',
        warn: '[WARN]',
        error: '[ERROR]',
        success: '[SUCCESS]'
      }
      return `${new Date(log.timestamp).toLocaleTimeString()} ${levelMap[log.level] || ''} ${log.message}`
    }).join('\n')

    await navigator.clipboard.writeText(allText)

    copied.value = true
    message.success(`已复制 ${props.logs.length} 条日志到剪贴板`)

    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => {
      copied.value = false
    }, 2000)

  } catch (err) {
    console.error('复制失败:', err)
    // fallback: 复制原始文本
    let textarea: HTMLTextAreaElement | null = null
    try {
      const allText = props.logs.map(log => log.message).join('\n')
      textarea = document.createElement('textarea')
      textarea.value = allText
      textarea.style.position = 'fixed'
      textarea.style.left = '-9999px'
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      copied.value = true
      message.success('已复制到剪贴板')
    } catch (e) {
      message.error('复制失败，请手动选择文本复制')
    }
    if (textarea?.parentNode) document.body.removeChild(textarea)
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
        <span class="log-count">
          共 {{ props.logs.length }} 条日志
          <span v-if="props.logs.length > MAX_RENDER_LINES" class="truncated-hint">
            （仅显示最近 {{ MAX_RENDER_LINES }} 条）
          </span>
        </span>
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

    <LogDisplay
      :logs="logEntries"
      :height="props.height"
      :show-toolbar="false"
    />
  </div>
</template>

<style scoped>
.log-panel-wrapper {
  position: relative;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  overflow: hidden;
  background: var(--surface-color);
}

.log-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  min-height: 40px;
  padding: 8px 12px;
  background: var(--surface-muted);
  border-bottom: 1px solid var(--border-soft);
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

.truncated-hint {
  color: var(--n-text-color-3);
  font-size: 11px;
}
</style>
