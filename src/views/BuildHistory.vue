<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, h } from 'vue'
import { useRouter } from 'vue-router'
import {
  NCard, NDataTable, NTag, NButton, NSpace, NText, NIcon, NSelect,
  NInput, NPagination, NEmpty, NSpin, NModal, useMessage,
  NGrid, NGi, NAlert, NStatistic
} from 'naive-ui'
import { RefreshOutline, TrashOutline,
  LogoAndroid, LogoApple, PhonePortraitOutline,
  CopyOutline, CheckmarkOutline, SearchOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { openUrl, revealItemInDir } from '@tauri-apps/plugin-opener'
import LogDisplay from '../components/LogDisplay.vue'
import type { LogLevel, LogEntry } from '../components/LogDisplay.vue'
import { useProjectsStore } from '../stores/projects'
import { useBuildStore } from '../stores/build'

const message = useMessage()
const projectsStore = useProjectsStore()
const buildStore = useBuildStore()
const router = useRouter()

interface BuildRecord {
  id: string
  project_id: string
  project_name: string
  platform: string
  status: string
  artifact_path?: string | null
  artifact_size_mb?: number | null
  version_name: string
  version_code: number
  build_mode: string
  build_source?: string | null
  cloud_run_url?: string | null
  duration_secs: number
  started_at: string
  finished_at?: string | null
  error_message?: string | null
  log_path?: string | null
}

const loading = ref(false)
const allRecords = ref<BuildRecord[]>([])

const platformFilter = ref<string | null>(null)
const statusFilter = ref<string | null>(null)
const dateRange = ref<number | null>(null)
const projectFilter = ref<string | null>(null)
const keywordSearch = ref('')

const pageSize = ref(15)
const currentPage = ref(1)

const showLogModal = ref(false)
const currentLogContent = ref('')
const currentLogEntries = ref<LogEntry[]>([])
const currentLogRecord = ref<BuildRecord | null>(null)
const historyCopied = ref(false)
let historyCopyTimer: ReturnType<typeof setTimeout> | null = null

const currentLogBuild = computed(() => {
  const record = currentLogRecord.value
  if (!record) return null
  return buildStore.builds[record.id] || null
})

const isStoreBackedLog = computed(() => !!currentLogBuild.value)
const isRealtimeLog = computed(() => currentLogBuild.value?.status === 'building')
const currentLogNotice = computed(() => {
  if (isRealtimeLog.value) return '构建中 · 实时日志'
  if (currentLogRecord.value?.status === 'building' && !currentLogBuild.value) {
    return '该记录仍显示构建中，但当前应用没有对应运行时任务，可能来自上次异常退出'
  }
  return ''
})

const displayedLogEntries = computed<LogEntry[]>(() => {
  const build = currentLogBuild.value
  if (!build) return currentLogEntries.value
  return build.logs.map(log => ({
    level: log.level,
    message: log.message,
    timestamp: log.timestamp
  }))
})

const displayedLogContent = computed(() => {
  const build = currentLogBuild.value
  if (!build) return currentLogContent.value
  return build.logs.map(log => `[${log.level}] ${log.message}`).join('\n')
})

onMounted(async () => {
  await projectsStore.initStore()
  loadHistory()
})

watch(
  () => buildStore.hasActiveBuilds,
  (active, previous) => {
    if (!active && previous) void loadHistory()
  }
)

watch(
  () => currentLogBuild.value?.status,
  (status, previous) => {
    if (previous === 'building' && status && status !== 'building') void loadHistory()
  }
)

async function loadHistory() {
  loading.value = true
  try {
    allRecords.value = await invoke<BuildRecord[]>('get_build_history', { projectId: null })
  } catch (e) {
    console.error('Failed to load build history:', e)
    message.error('加载打包历史失败')
  } finally {
    loading.value = false
  }
}

function refresh() {
  loadHistory()
  message.success('已刷新')
}

function formatDateTime(isoStr: string): string {
  if (!isoStr) return '-'
  try {
    const d = new Date(isoStr)
    const pad = (n: number) => n.toString().padStart(2, '0')
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  } catch {
    return isoStr
  }
}

function formatDuration(secs: number): string {
  if (!secs || secs === 0) return '-'
  if (secs < 60) return `${secs}秒`
  const m = Math.floor(secs / 60)
  const s = secs % 60
  return `${m}分${s}秒`
}

function isWithinDays(dateStr: string, days: number): boolean {
  try {
    const d = new Date(dateStr).getTime()
    const now = Date.now()
    const diff = now - d
    return diff <= days * 24 * 60 * 60 * 1000
  } catch {
    return true
  }
}

const filteredRecords = computed(() => {
  let records = [...allRecords.value]

  if (platformFilter.value) {
    records = records.filter(r => r.platform === platformFilter.value)
  }

  if (statusFilter.value) {
    records = records.filter(r => r.status === statusFilter.value)
  }

  if (dateRange.value) {
    records = records.filter(r => isWithinDays(r.started_at, dateRange.value ?? 99999))
  }

  if (projectFilter.value) {
    records = records.filter(r => r.project_id === projectFilter.value)
  }

  if (keywordSearch.value.trim()) {
    const kw = keywordSearch.value.trim().toLowerCase()
    records = records.filter(r =>
      r.version_name.toLowerCase().includes(kw) ||
      (r.error_message && r.error_message.toLowerCase().includes(kw)) ||
      r.project_name.toLowerCase().includes(kw)
    )
  }

  return records
})

const pagedRecords = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredRecords.value.slice(start, start + pageSize.value)
})

const totalFiltered = computed(() => filteredRecords.value.length)

const stats = computed(() => {
  const total = filteredRecords.value.length
  const successCount = filteredRecords.value.filter(r => r.status === 'success').length
  const successRate = total > 0 ? ((successCount / total) * 100).toFixed(1) : '0'
  const withDuration = filteredRecords.value.filter(r => r.duration_secs > 0)
  const avgDuration = withDuration.length > 0
    ? Math.round(withDuration.reduce((a, b) => a + b.duration_secs, 0) / withDuration.length)
    : 0
  return { total, successRate, avgDuration }
})

const platformOptions = [
  { label: 'Android', value: 'android' },
  { label: 'iOS', value: 'ios' },
  { label: '鸿蒙', value: 'harmony' }
]

const platformFilterOptions = computed<any>(() => [
  { label: '全部平台', value: null },
  ...platformOptions
])

const statusOptions = [
  { label: '成功', value: 'success' },
  { label: '失败', value: 'failed' },
  { label: '构建中', value: 'building' },
  { label: '已取消', value: 'cancelled' }
]

const statusFilterOptions = computed<any>(() => [
  { label: '全部状态', value: null },
  ...statusOptions
])

const dateRangeOptions = [
  { label: '最近 7 天', value: 7 },
  { label: '最近 30 天', value: 30 },
  { label: '最近 90 天', value: 90 },
  { label: '全部', value: 99999 }
]

const projectOptions = computed<any>(() => [
  { label: '全部项目', value: null },
  ...projectsStore.projects.map(p => ({ label: p.name, value: p.id }))
])

function getPlatformTag(record: BuildRecord) {
  switch (record.platform) {
    case 'android':
      return { type: 'success' as const, icon: LogoAndroid, label: 'Android' }
    case 'ios':
      return { type: 'info' as const, icon: LogoApple, label: 'iOS' }
    case 'harmony':
      return { type: 'error' as const, icon: PhonePortraitOutline, label: '鸿蒙' }
    default:
      return { type: 'default' as const, icon: undefined, label: record.platform }
  }
}

function getStatusTagType(status: string) {
  switch (status) {
    case 'success': return 'success'
    case 'failed': return 'error'
    case 'building': return 'warning'
    case 'cancelled': return 'info'
    default: return 'default'
  }
}

function getStatusLabel(status: string) {
  const map: Record<string, string> = {
    success: '成功',
    failed: '失败',
    building: '构建中',
    cancelled: '已取消',
    idle: '待机'
  }
  return map[status] || status
}

const columns = [
  {
    title: '时间',
    key: 'started_at',
    width: 170,
    render: (row: BuildRecord) => formatDateTime(row.started_at)
  },
  {
    title: '项目名',
    key: 'project_name',
    width: 140,
    ellipsis: { tooltip: true }
  },
  {
    title: '平台',
    key: 'platform',
    width: 100,
    render: (row: BuildRecord) => {
      const tag = getPlatformTag(row)
      return h(NTag, { type: tag.type, size: 'small', round: true, bordered: false }, {
        default: () => [
          tag.icon ? h(NIcon, { size: 13 }, { default: () => h(tag.icon) }) : h(NText, {}, { default: () => row.platform }),
          ' ' + tag.label
        ]
      })
    }
  },
  {
    title: '状态',
    key: 'status',
    width: 90,
    render: (row: BuildRecord) => h(NTag, {
      type: getStatusTagType(row.status),
      size: 'small',
      round: true,
      bordered: false
    }, { default: () => getStatusLabel(row.status) })
  },
  {
    title: '版本号',
    key: 'version_name',
    width: 120,
    render: (row: BuildRecord) => h(NText, { code: true, style: 'font-size:12px' }, { default: () => row.version_name })
  },
  {
    title: '模式',
    key: 'build_mode',
    width: 80,
    render: (row: BuildRecord) => h(NTag, {
      type: row.build_mode === 'release' ? 'success' : 'default',
      size: 'tiny',
      round: true,
      bordered: false
    }, { default: () => row.build_mode })
  },
  {
    title: '来源',
    key: 'build_source',
    width: 90,
    render: (row: BuildRecord) => h(NTag, {
      type: row.build_source === 'github' ? 'info' : 'default',
      size: 'tiny',
      round: true,
      bordered: false
    }, { default: () => row.build_source === 'github' ? 'GitHub' : '本地' })
  },
  {
    title: '耗时',
    key: 'duration_secs',
    width: 90,
    render: (row: BuildRecord) => formatDuration(row.duration_secs)
  },
  {
    title: '产物大小',
    key: 'artifact_size_mb',
    width: 100,
    render: (row: BuildRecord) => row.artifact_size_mb ? `${row.artifact_size_mb.toFixed(2)} MB` : '-'
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    fixed: 'right' as const,
    render: (row: BuildRecord) => h(NSpace, { size: 4 }, {
      default: () => [
        row.artifact_path ? h(NButton, {
          size: 'tiny',
          quaternary: true,
          type: 'info',
          onClick: () => row.artifact_path && revealItemInDir(row.artifact_path)
        }, {
          default: () => ['打开目录']
        }) : null,
        row.cloud_run_url ? h(NButton, {
          size: 'tiny',
          quaternary: true,
          type: 'primary',
          onClick: () => row.cloud_run_url && openUrl(row.cloud_run_url)
        }, {
          default: () => ['远端 Run']
        }) : null,
        h(NButton, {
          size: 'tiny',
          quaternary: true,
          onClick: () => viewLog(row)
        }, {
          default: () => '查看日志'
        }),
        h(NButton, {
          size: 'tiny',
          quaternary: true,
          type: 'primary',
          disabled: buildStore.hasActiveBuilds,
          onClick: () => rebuild(row)
        }, {
          default: () => '重新构建'
        })
      ].filter(Boolean)
    })
  }
]

function parseLogFile(raw: string): LogEntry[] {
  const entries: LogEntry[] = []
  const lines = raw.split('\n')
  // Pattern: [info] msg, [warn] msg, [error] msg, [success] msg
  // Also plain lines: timestamp + message
  for (const line of lines) {
    if (!line.trim()) continue
    const match = line.match(/^\[(info|warn|error|success)\]\s*(.+)$/i)
    if (match) {
      entries.push({ level: match[1].toLowerCase() as LogLevel, message: match[2] })
    } else {
      // Try to extract timestamp from beginning
      const tsMatch = line.match(/^(\d{2}:\d{2}:\d{2}(?:\.\d+)?)\s+(.+)$/i)
      if (tsMatch) {
        entries.push({ level: 'info' as LogLevel, message: tsMatch[2], timestamp: tsMatch[1] })
      } else {
        entries.push({ level: 'info' as LogLevel, message: line })
      }
    }
  }
  return entries
}

async function viewLog(record: BuildRecord) {
  currentLogRecord.value = record
  currentLogContent.value = ''
  currentLogEntries.value = []
  showLogModal.value = true

  if (record.status === 'building' && buildStore.builds[record.id]) {
    return
  }

  if (record.log_path) {
    try {
      const content = await invoke<string>('read_text_file', { path: record.log_path })
      currentLogContent.value = content
      currentLogEntries.value = parseLogFile(content)
    } catch {
      currentLogContent.value = '无法读取日志文件'
    }
  } else {
    currentLogContent.value = record.status === 'building'
      ? '该构建记录仍显示构建中，但当前应用没有实时日志，且没有关联日志文件'
      : '该构建记录没有关联的日志文件'
  }
}

function rebuild(record: BuildRecord) {
  if (buildStore.hasActiveBuilds) {
    message.warning('已有构建任务进行中，请等待完成后再重新构建')
    return
  }
  router.push(`/build/${record.project_id}`)
}

async function clearAllHistory() {
  if (buildStore.hasActiveBuilds) {
    message.warning('已有构建任务进行中，请等待完成后再清空历史')
    return
  }
  try {
    await invoke('clear_build_history', { projectId: null })
    allRecords.value = []
    message.success('已清空所有打包历史')
  } catch (e) {
    message.error(String(e))
  }
}

async function copyHistoryLog() {
  const content = displayedLogContent.value
  if (!content) {
    message.warning('暂无日志可复制')
    return
  }

  if (content.includes('无法读取') || content.includes('没有关联')) {
    message.warning('当前无可复制的有效日志')
    return
  }

  try {
    await navigator.clipboard.writeText(content)

    historyCopied.value = true
    const lineCount = content.split('\n').length
    message.success(`已复制 ${lineCount} 行日志到剪贴板`)

    if (historyCopyTimer) clearTimeout(historyCopyTimer)
    historyCopyTimer = setTimeout(() => {
      historyCopied.value = false
    }, 2000)
  } catch (err) {
    console.error('复制历史日志失败:', err)
    fallbackCopyHistoryLog(content)
  }
}

function fallbackCopyHistoryLog(text: string) {
  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.position = 'fixed'
  textarea.style.left = '-9999px'
  document.body.appendChild(textarea)
  textarea.select()

  try {
    document.execCommand('copy')
    historyCopied.value = true
    message.success('已复制到剪贴板')

    if (historyCopyTimer) clearTimeout(historyCopyTimer)
    historyCopyTimer = setTimeout(() => { historyCopied.value = false }, 2000)
  } catch (e) {
    message.error('复制失败，请手动选择文本复制（Ctrl/Cmd + A, Ctrl/Cmd + C）')
  }

  document.body.removeChild(textarea)
}

function formatLogSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

onUnmounted(() => {
  if (historyCopyTimer) clearTimeout(historyCopyTimer)
})
</script>

<template>
  <div class="build-history">
    <div class="page-header">
      <n-space align="center" justify="space-between" class="history-header-row">
        <n-space align="center">
          <n-text strong class="page-title">打包历史</n-text>
        </n-space>
        <n-space>
          <n-button @click="refresh" :loading="loading">
            <template #icon><n-icon><RefreshOutline /></n-icon></template>
            刷新
          </n-button>
          <n-button type="error" secondary @click="clearAllHistory" :disabled="!allRecords.length || buildStore.hasActiveBuilds">
            <template #icon><n-icon><TrashOutline /></n-icon></template>
            清空历史
          </n-button>
        </n-space>
      </n-space>
    </div>

    <n-card class="history-card">
      <n-spin :show="loading">

        <n-empty v-if="!loading && !filteredRecords.length && !keywordSearch && !platformFilter && !statusFilter"
          description="暂无打包记录">
          <template #extra>
            <n-text depth="3">前往「构建中心」开始第一次打包</n-text>
          </template>
        </n-empty>

        <n-empty v-else-if="!loading && !filteredRecords.length && (keywordSearch || platformFilter || statusFilter)"
          description="没有匹配的记录">
          <template #extra>
            <n-button size="small" @click="platformFilter = null; statusFilter = null; dateRange = null; projectFilter = null; keywordSearch = ''">清除筛选</n-button>
          </template>
        </n-empty>

        <template v-else-if="filteredRecords.length > 0">
          <!-- 筛选栏 -->
          <div class="filter-bar">
            <n-grid :cols="5" :x-gap="12" :y-gap="12">
              <n-gi>
                <n-text depth="3" class="filter-label">平台</n-text>
                <n-select
                  v-model:value="platformFilter"
                  :options="platformFilterOptions"
                  size="small"
                  clearable
                />
              </n-gi>
              <n-gi>
                <n-text depth="3" class="filter-label">状态</n-text>
                <n-select
                  v-model:value="statusFilter"
                  :options="statusFilterOptions"
                  size="small"
                  clearable
                />
              </n-gi>
              <n-gi>
                <n-text depth="3" class="filter-label">时间范围</n-text>
                <n-select
                  v-model:value="dateRange"
                  :options="dateRangeOptions"
                  size="small"
                />
              </n-gi>
              <n-gi>
                <n-text depth="3" class="filter-label">项目</n-text>
                <n-select
                  v-model:value="projectFilter"
                  :options="projectOptions"
                  size="small"
                  clearable
                />
              </n-gi>
              <n-gi>
                <n-text depth="3" class="filter-label">搜索</n-text>
                <n-input
                  v-model:value="keywordSearch"
                  placeholder="版本号 / 错误信息..."
                  size="small"
                  clearable
                >
                  <template #prefix><n-icon :component="SearchOutline" /></template>
                </n-input>
              </n-gi>
            </n-grid>
          </div>

          <n-data-table
            :columns="(columns as any)"
            :data="pagedRecords"
            :pagination="false"
            :bordered="true"
            size="small"
            striped
            :row-key="(row: BuildRecord) => row.id"
            class="history-table"
            :scroll-x="1000"
          />

          <div class="footer-bar">
            <n-pagination
              v-model:page="currentPage"
              :item-count="totalFiltered"
              :page-size="pageSize"
              :page-sizes="[15, 30, 50]"
              show-size-picker
              show-quick-jumper
              class="history-pagination"
            />

            <n-space :size="28" align="center" class="history-stats">
              <n-statistic label="总次数" :value="stats.total" class="history-stat" />
              <n-statistic label="成功率" :value="stats.successRate" suffix="%" class="history-stat" />
              <n-statistic label="平均耗时" :value="stats.avgDuration" suffix="秒" class="history-stat" />
            </n-space>
          </div>
        </template>
      </n-spin>
    </n-card>

    <!-- 日志查看弹窗 -->
    <n-modal
      v-model:show="showLogModal"
      preset="card"
      :title="`构建日志 — ${currentLogRecord?.project_name || ''} (${currentLogRecord?.version_name || ''})`"
      style="width: 720px;"
      :segmented="{ content: true }"
    >
      <template #action>
        <n-space align="center" justify="space-between" class="modal-action-row">
          <n-text depth="3" class="log-meta-text">
            <template v-if="displayedLogEntries.length">
              共 {{ displayedLogEntries.length }} 行 · {{ formatLogSize(displayedLogContent.length) }}
            </template>
          </n-text>
          <n-button
            size="small"
            :type="historyCopied ? 'success' : 'primary'"
            :disabled="!displayedLogContent || displayedLogContent.includes('无法读取') || displayedLogContent.includes('没有关联')"
            @click="copyHistoryLog"
            aria-label="复制完整构建日志到剪贴板"
          >
            <template #icon>
              <n-icon :component="historyCopied ? CheckmarkOutline : CopyOutline" />
            </template>
            {{ historyCopied ? '已复制' : '复制完整日志' }}
          </n-button>
        </n-space>
      </template>

      <n-alert v-if="currentLogNotice" :type="isRealtimeLog ? 'warning' : 'info'" class="log-notice">
        {{ currentLogNotice }}
      </n-alert>
      <n-alert v-if="!displayedLogContent && !isStoreBackedLog" type="info">日志加载中...</n-alert>
      <LogDisplay
        v-else
        :logs="displayedLogEntries"
        :height="'450px'"
        :show-toolbar="false"
      />
    </n-modal>
  </div>
</template>

<style scoped>
.build-history {
  max-width: 1400px;
}

.page-header {
  margin-bottom: 18px;
}

.history-header-row,
.modal-action-row {
  width: 100%;
}

.history-card {
  overflow: hidden;
}

.filter-bar {
  margin-bottom: 14px;
  padding: 14px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-muted);
}

.filter-label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 600;
}

.history-table {
  margin-top: 16px;
}

.footer-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--border-soft);
  gap: 16px;
}

.history-pagination {
  justify-content: flex-start;
}

.history-stat {
  --n-value-font-size: 18px;
}

.log-meta-text {
  font-size: 12px;
}

.log-notice {
  margin-bottom: 12px;
}

@media (max-width: 1180px) {
  .footer-bar {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
