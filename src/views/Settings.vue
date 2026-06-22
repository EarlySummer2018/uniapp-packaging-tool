<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import {
  NAlert,
  NButton,
  NCard,
  NFormItem,
  NIcon,
  NInput,
  NModal,
  NSpace,
  NText,
  useMessage
} from 'naive-ui'
import { FolderOpenOutline, SaveOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import LogPanel from '../components/LogPanel.vue'
import type { BuildLog } from '../stores/build'
import { useProjectsStore } from '../stores/projects'

interface AppSettings {
  cacheDir: string
  defaultCacheDir: string
}

interface MigrationResult {
  success: boolean
  cacheDir: string
  logs: string[]
  error?: string | null
}

const message = useMessage()
const projectsStore = useProjectsStore()
const loading = ref(false)
const saving = ref(false)
const settings = ref<AppSettings>({
  cacheDir: '',
  defaultCacheDir: '',
})
const cacheDirInput = ref('')
const showMigrationModal = ref(false)
const migrationRunning = ref(false)
const migrationDone = ref(false)
const migrationSuccess = ref(false)
const migrationError = ref('')
const migrationLogs = ref<BuildLog[]>([])
let unlistenMigrationLog: UnlistenFn | null = null

onMounted(() => {
  loadSettings()
  listen<{ level: BuildLog['level']; message: string }>('settings-migration-log', (event) => {
    addMigrationLog(event.payload.level || 'info', event.payload.message)
  }).then((unlisten) => {
    unlistenMigrationLog = unlisten
  })
})

onUnmounted(() => {
  unlistenMigrationLog?.()
})

async function loadSettings() {
  loading.value = true
  try {
    const loadedSettings = await invoke<AppSettings>('get_app_settings')
    settings.value = loadedSettings
    cacheDirInput.value = settings.value.cacheDir
  } catch (e: any) {
    message.error(String(e))
  } finally {
    loading.value = false
  }
}

async function chooseCacheDir() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择缓存目录',
  })
  if (typeof selected === 'string') {
    cacheDirInput.value = selected
  }
}

function addMigrationLog(level: BuildLog['level'], message: string) {
  migrationLogs.value.push({
    id: `migration_${Date.now()}_${Math.random().toString(36).slice(2)}`,
    timestamp: new Date().toISOString(),
    level,
    message,
  })
}

async function saveCacheDir() {
  const target = cacheDirInput.value.trim()
  if (!target) {
    message.warning('请输入缓存目录')
    return
  }

  showMigrationModal.value = true
  migrationRunning.value = true
  migrationDone.value = false
  migrationSuccess.value = false
  migrationError.value = ''
  migrationLogs.value = []
  addMigrationLog('info', '开始迁移缓存目录...')
  saving.value = true

  try {
    const result = await invoke<MigrationResult>('migrate_cache_dir', {
      newCacheDir: target,
    })
    if (!migrationLogs.value.length) {
      for (const line of result.logs) {
        addMigrationLog(result.success ? 'info' : 'warn', line)
      }
    }
    migrationSuccess.value = result.success
    migrationError.value = result.error || ''
    migrationDone.value = true
    if (result.success) {
      addMigrationLog('success', '迁移完成')
      settings.value.cacheDir = result.cacheDir
      cacheDirInput.value = result.cacheDir
      await projectsStore.loadProjects()
      message.success('缓存目录已更新')
    } else {
      addMigrationLog('error', result.error || '迁移失败')
    }
  } catch (e: any) {
    migrationSuccess.value = false
    migrationError.value = String(e)
    migrationDone.value = true
    addMigrationLog('error', String(e))
  } finally {
    migrationRunning.value = false
    saving.value = false
  }
}
</script>

<template>
  <div class="settings-page">
    <div class="page-header">
      <n-space align="center">
        <n-text strong class="page-title">设置</n-text>
      </n-space>
    </div>

    <n-card title="路径设置" class="settings-card">
      <n-space vertical :size="16">
        <n-alert type="info">
          缓存目录用于保存项目配置、导入资源、构建日志和构建工作区。应用启动时会从这里读取项目列表。
        </n-alert>

        <n-form-item label="缓存目录">
          <n-space :size="8" class="inline-field-row">
            <n-input
              v-model:value="cacheDirInput"
              placeholder="输入或选择缓存目录"
              :disabled="loading || saving"
            />
            <n-button :disabled="loading || saving" @click="chooseCacheDir">
              <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
              选择
            </n-button>
          </n-space>
        </n-form-item>

        <n-text depth="3">默认目录: <n-text code>{{ settings.defaultCacheDir || '-' }}</n-text></n-text>

        <n-space justify="end">
          <n-button type="primary" :loading="saving" @click="saveCacheDir">
            <template #icon><n-icon><SaveOutline /></n-icon></template>
            保存并迁移
          </n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-modal
      v-model:show="showMigrationModal"
      preset="card"
      title="迁移缓存目录"
      style="width: 720px;"
      :mask-closable="false"
      :closable="false"
      :close-on-esc="false"
    >
      <n-space vertical :size="12">
        <n-alert v-if="migrationRunning" type="info">正在迁移，请勿关闭应用。</n-alert>
        <n-alert v-else-if="migrationSuccess" type="success">迁移完成。</n-alert>
        <n-alert v-else-if="migrationDone" type="error">
          迁移失败: {{ migrationError || '未知错误' }}。请根据日志手动将旧缓存目录内容复制到目标目录，然后重新设置缓存目录。
        </n-alert>

        <LogPanel :logs="migrationLogs" height="320px" />
      </n-space>

      <template #action>
        <n-space justify="end">
          <n-button v-if="migrationDone" type="primary" @click="showMigrationModal = false">
            我知道了
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 1080px;
}

.page-header {
  margin-bottom: 2px;
}

.settings-card {
  overflow: hidden;
}

.inline-field-row {
  width: 100%;
}
</style>
