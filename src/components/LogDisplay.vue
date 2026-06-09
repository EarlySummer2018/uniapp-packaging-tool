<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue'
import { NButton, NIcon, useMessage } from 'naive-ui'
import { CopyOutline, CheckmarkOutline } from '@vicons/ionicons5'

export type LogLevel = 'info' | 'warn' | 'error' | 'success'

export interface LogEntry {
  level: LogLevel
  message: string
  timestamp?: string
}

interface Props {
  /** 日志条目列表 */
  logs: LogEntry[]
  /** 容器高度 */
  height?: string
  /** 是否显示工具栏 */
  showToolbar?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  height: '400px',
  showToolbar: true
})

const message = useMessage()
const logContainerRef = ref<HTMLDivElement | null>(null)
const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | null = null

// ===== 智能自动滚动 =====

const isAtBottom = ref(true)
const BOTTOM_THRESHOLD = 50

function checkIsAtBottom(): boolean {
  const el = logContainerRef.value
  if (!el) return true
  return el.scrollHeight - el.scrollTop - el.clientHeight < BOTTOM_THRESHOLD
}

function scrollToBottom() {
  const el = logContainerRef.value
  if (el) {
    el.scrollTop = el.scrollHeight
    isAtBottom.value = true
  }
}

function handleScroll() {
  isAtBottom.value = checkIsAtBottom()
}

// 监听日志变化，仅在触底时自动滚动
watch(
  () => props.logs.length,
  (newLen, oldLen) => {
    if (newLen > (oldLen ?? 0) && isAtBottom.value) {
      nextTick(() => scrollToBottom())
    }
  }
)

// ===== 性能优化 =====

const MAX_RENDER_LINES = 2000
let cachedLines: ReturnType<typeof renderLogLines> | null = null
let cachedLogsLength = 0
let lastLogId = ''

/** 渲染日志行 HTML */
function renderLogLines(logs: LogEntry[]) {
  if (!logs.length) return { html: '', lineCount: 0 }

  const sliced = logs.length > MAX_RENDER_LINES ? logs.slice(-MAX_RENDER_LINES) : logs

  let html = ''
  for (const log of sliced) {
    const levelTag = getLevelTag(log.level)
    const time = log.timestamp ? new Date(log.timestamp).toLocaleTimeString() : ''
    html += `<div class="log-line ${log.level}" data-level="${log.level}"><span class="log-time">${time}</span><span class="log-level">${levelTag}</span><span class="log-msg">${escapeHtml(log.message)}</span></div>`
  }

  return { html, lineCount: sliced.length }
}

function getLevelTag(level: LogLevel): string {
  switch (level) {
    case 'info': return '<span class="level-info">INFO</span>'
    case 'warn': return '<span class="level-warn">WARN</span>'
    case 'error': return '<span class="level-error">ERROR</span>'
    case 'success': return '<span class="level-success">SUCCESS</span>'
    default: return ''
  }
}

function escapeHtml(str: string): string {
  const div = document.createElement('div')
  div.textContent = str
  return div.innerHTML
}

const renderedLog = computed(() => {
  const logs = props.logs
  if (!logs.length) return { html: '', lineCount: 0 }

  const currentLastId = logs.length > 0 ? `${logs[logs.length - 1].level}:${logs[logs.length - 1].message}` : ''
  if (currentLastId === lastLogId && cachedLines && logs.length === cachedLogsLength) {
    return cachedLines
  }

  const result = renderLogLines(logs)
  cachedLines = result
  cachedLogsLength = logs.length
  lastLogId = currentLastId
  return result
})

const displayHtml = computed(() => renderedLog.value.html)
const isTruncated = computed(() => props.logs.length > MAX_RENDER_LINES)

const computedHeight = computed(() => {
  const baseHeight = parseInt(props.height) || 400
  return props.showToolbar ? `${baseHeight - 40}px` : props.height
})

const copyButtonText = computed(() => copied.value ? '已复制' : '复制日志')
const copyButtonIcon = computed(() => copied.value ? CheckmarkOutline : CopyOutline)

async function copyLogs() {
  if (!props.logs.length) {
    message.warning('暂无日志可复制')
    return
  }

  try {
    const allText = props.logs.map(log => {
      const time = log.timestamp ? new Date(log.timestamp).toLocaleTimeString() : ''
      const levelPrefix: Record<string, string> = { info: '', warn: '[WARN]', error: '[ERROR]', success: '[SUCCESS]' }
      return `${time}${levelPrefix[log.level] || ''} ${log.message}`
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
    // fallback
    const textarea = document.createElement('textarea')
    textarea.value = props.logs.map(log => `${log.message}`).join('\n')
    textarea.style.position = 'fixed'
    textarea.style.left = '-9999px'
    document.body.appendChild(textarea)
    textarea.select()
    try {
      document.execCommand('copy')
      copied.value = true
      message.success('已复制到剪贴板')
    } catch (e) {
      message.error('复制失败，请手动选择文本复制')
    }
    document.body.removeChild(textarea)
  }
}

onUnmounted(() => {
  if (copyTimer) clearTimeout(copyTimer)
})
</script>

<template>
  <div class="log-display">
    <!-- 工具栏 -->
    <div v-if="showToolbar" class="log-toolbar">
      <div class="toolbar-left">
        <span class="log-count">
          共 {{ logs.length }} 条日志
          <span v-if="isTruncated" class="truncated-hint">
            （仅显示最近 {{ MAX_RENDER_LINES }} 条）
          </span>
        </span>
      </div>
      <div class="toolbar-right">
        <n-button
          size="small"
          quaternary
          :type="copied ? 'success' : 'primary'"
          :disabled="!logs.length"
          @click="copyLogs"
          aria-label="复制日志到剪贴板"
        >
          <template #icon>
            <n-icon :component="copyButtonIcon" />
          </template>
          {{ copyButtonText }}
        </n-button>
      </div>
    </div>

    <!-- 日志容器 -->
    <div
      ref="logContainerRef"
      class="log-container"
      :style="{ height: computedHeight }"
      @scroll="handleScroll"
      v-html="displayHtml"
    />
  </div>
</template>

<style scoped>
.log-display {
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

/* ===== 日志容器 ===== */
.log-container {
  overflow-y: auto;
  overflow-x: hidden;
  background: #15181d;
  color: #d7dde7;
  font-family: 'Menlo', 'Monaco', 'Consolas', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.7;
  padding: 12px 16px;
  contain: layout style paint;
}

/* 单行日志 */
.log-container :deep(.log-line) {
  padding: 1px 0;
  white-space: pre-wrap;
  word-break: break-all;
  contain: layout style paint;
  content-visibility: auto;
  contain-intrinsic-size: auto 22px;
}

.log-container :deep(.log-time) {
  color: #7fb069;
  margin-right: 8px;
  user-select: none;
}

.log-container :deep(.log-level) {
  display: inline-block;
  min-width: 50px;
  margin-right: 8px;
  font-weight: 600;
  text-align: center;
  padding: 0 4px;
  border-radius: 2px;
  font-size: 10px;
  letter-spacing: 0.5px;
  user-select: none;
}

.log-container :deep(.log-msg) {
  color: #d4d4d4;
}

/* ===== 日志级别颜色 ===== */

/* INFO - 蓝色 */
.log-container :deep(.info .log-level) {
  color: #79b8ff;
  background: rgba(121, 184, 255, 0.14);
}
.log-container :deep(.info .log-msg) {
  color: #b9d8ff;
}

/* WARN - 黄色 */
.log-container :deep(.warn .log-level) {
  color: #f2cc60;
  background: rgba(242, 204, 96, 0.12);
}
.log-container :deep(.warn .log-msg) {
  color: #f2cc60;
}

/* ERROR - 红色 */
.log-container :deep(.error .log-level) {
  color: #ff7b72;
  background: rgba(255, 123, 114, 0.14);
}
.log-container :deep(.error .log-msg) {
  color: #ff9b92;
  font-weight: 500;
}

/* SUCCESS - 绿色 */
.log-container :deep(.success .log-level) {
  color: #7ee787;
  background: rgba(126, 231, 135, 0.12);
}
.log-container :deep(.success .log-msg) {
  color: #97d88b;
  font-weight: 500;
}

/* 滚动条样式 */
.log-container::-webkit-scrollbar {
  width: 6px;
}
.log-container::-webkit-scrollbar-track {
  background: transparent;
}
.log-container::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 3px;
}
.log-container::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.25);
}
</style>
