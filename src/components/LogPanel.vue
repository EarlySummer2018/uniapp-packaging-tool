<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue'
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

// ===== 智能自动滚动 =====

/** 是否处于底部（用户未手动向上滚动） */
const isAtBottom = ref(true)

/** 判定"触底"的阈值（距底部多少 px 内视为触底） */
const BOTTOM_THRESHOLD = 50

/** 获取日志容器的滚动元素 */
function getScrollEl(): HTMLElement | null {
  if (!logRef.value?.$el) return null
  // NLog 的 $el 本身就是滚动容器
  return logRef.value.$el as HTMLElement | null
}

/** 判断当前是否在底部附近 */
function checkIsAtBottom(): boolean {
  const el = getScrollEl()
  if (!el) return true
  return el.scrollHeight - el.scrollTop - el.clientHeight < BOTTOM_THRESHOLD
}

/** 滚动到最底部 */
function scrollToBottom() {
  const el = getScrollEl()
  if (el) {
    el.scrollTop = el.scrollHeight
    isAtBottom.value = true
  }
}

/** 处理容器滚动事件：更新 isAtBottom 状态 */
function handleScroll() {
  isAtBottom.value = checkIsAtBottom()
}

// 监听日志变化，仅在触底时自动滚动
watch(
  () => props.logs.length,
  (_newLen, oldLen) => {
    // 仅当有新日志追加 且 当前处于底部时才自动滚动
    if (_newLen > (oldLen ?? 0) && isAtBottom.value) {
      nextTick(() => scrollToBottom())
    }
  }
)

// ===== 性能优化核心 =====

/** 最大渲染行数，超出部分截断以避免 DOM 爆炸 */
const MAX_RENDER_LINES = 2000

/** 缓存上一次的渲染结果，避免重复计算 */
let cachedLogText = ''
let cachedLogsLength = 0

/** 上一次的日志 ID，用于增量更新检测 */
let lastLogId = ''

/** RAF 节流标记 */
let rafId: number | null = null

/**
 * 增量构建日志文本：
 * - 仅当日志数量变化时才重新计算
 * - 只截取最后 MAX_RENDER_LINES 行，避免大字符串拼接
 * - 使用缓存 + 增量追加策略
 */
const logLines = computed(() => {
  const logs = props.logs
  if (!logs.length) return ''

  // 如果日志没有变化，直接返回缓存
  const currentLastId = logs.length > 0 ? logs[logs.length - 1].id : ''
  if (currentLastId === lastLogId && cachedLogText && logs.length === cachedLogsLength) {
    return cachedLogText
  }

  // 截取最后 MAX_RENDER_LINES 行进行渲染
  const slicedLogs = logs.length > MAX_RENDER_LINES
    ? logs.slice(-MAX_RENDER_LINES)
    : logs

  const levelMap: Record<string, string> = {
    info: '',
    warn: '[WARN]',
    error: '[ERROR]',
    success: '[SUCCESS]'
  }

  // 构建文本
  const text = slicedLogs.map(log => {
    return `${new Date(log.timestamp).toLocaleTimeString()} ${levelMap[log.level] || ''} ${log.message}`
  }).join('\n')

  // 更新缓存
  cachedLogText = text
  cachedLogsLength = logs.length
  lastLogId = currentLastId

  return text
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

onUnmounted(() => {
  if (copyTimer) clearTimeout(copyTimer)
  if (rafId !== null) cancelAnimationFrame(rafId)
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

    <div class="log-container" :style="{ height: computedHeight }" @scroll="handleScroll">
      <n-log
        ref="logRef"
        :log="logLines"
        :loading="false"
        :font-size="13"
        :rows="15"
        language="text"
      />
    </div>
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

.truncated-hint {
  color: var(--n-text-color-3);
  font-size: 11px;
}

/* ===== 性能优化：日志容器 ===== */
.log-container {
  overflow: auto;
  /* 启用 content-visibility 让浏览器跳过不可见内容的渲染 */
  content-visibility: auto;
  contain-intrinsic-size: auto 300px;
}

/* 优化 NLog 内部行的渲染 */
.log-container :deep(.n-log) {
  /* 减少行内元素的重排开销 */
  will-change: transform;
}

/* 单条日志行使用 content-visibility 优化 */
.log-container :deep(.n-log-line) {
  contain: layout style paint;
  content-visibility: auto;
  contain-intrinsic-size: auto 20px;
}
</style>
