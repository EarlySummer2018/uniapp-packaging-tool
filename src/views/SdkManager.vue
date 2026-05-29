<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { 
  NTabs, NTabPane, NCard, NButton, NIcon, NSpace, NTag,
  NText, useMessage, NSpin,
  NAlert, NTable,
  NModal, NFormItem, NInput, NCheckbox, NSelect
} from 'naive-ui'
import {
  RefreshOutline, FolderOpenOutline, CheckmarkCircleOutline,
  CloseCircleOutline, AlertCircleOutline, LogoAndroid, LogoApple,
  SettingsOutline, CubeOutline
} from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { openPath, openUrl } from '@tauri-apps/plugin-opener'

const message = useMessage()

interface EnvItem {
  name: string
  status: 'ok' | 'warning' | 'error' | 'unknown'
  version?: string
  path?: string
  download_url: string
  tool_key: string
  configurable: boolean
  validating?: boolean
}

interface EnvGroup {
  label: string
  icon: any
  items: EnvItem[]
  ready_count: number
  total_count: number
}

interface BuildRecordSummary {
  id: string
  project_name: string
  platform: string
  status: string
  version_name: string
  started_at: string
}

interface GlobalSdkConfig {
  dcloudAndroidSdkPath: string
  dcloudIosSdkPath: string
  harmonyTemplatePath: string
}

interface GlobalSdkItem {
  name: string
  status: EnvItem['status']
  path: string
  platform: 'android' | 'ios' | 'harmony'
  download_url: string
}

const activeTab = ref('dcloud-sdk')
const loading = ref(false)
const globalSdkLoading = ref(false)

const globalSdkConfig = ref<GlobalSdkConfig>({
  dcloudAndroidSdkPath: '',
  dcloudIosSdkPath: '',
  harmonyTemplatePath: '',
})
const envGroups = ref<EnvGroup[]>([])
const recentBuilds = ref<BuildRecordSummary[]>([])

const globalSdkItems = computed<GlobalSdkItem[]>(() => [
  {
    name: 'Android 离线SDK',
    status: globalSdkConfig.value.dcloudAndroidSdkPath ? 'ok' : 'error',
    path: globalSdkConfig.value.dcloudAndroidSdkPath,
    platform: 'android',
    download_url: 'https://nativesupport.dcloud.net.cn/AppDocs/download/android.html',
  },
  {
    name: 'iOS 离线SDK',
    status: globalSdkConfig.value.dcloudIosSdkPath ? 'ok' : 'error',
    path: globalSdkConfig.value.dcloudIosSdkPath,
    platform: 'ios',
    download_url: 'https://nativesupport.dcloud.net.cn/AppDocs/download/ios.html',
  },
  {
    name: 'Harmony 工程模板',
    status: globalSdkConfig.value.harmonyTemplatePath ? 'ok' : 'error',
    path: globalSdkConfig.value.harmonyTemplatePath,
    platform: 'harmony',
    download_url: 'https://nativesupport.dcloud.net.cn/AppDocs/usesdk/harmony-v1.html',
  },
])

async function openExternal(pathOrUrl: string) {
  if (/^https?:\/\//i.test(pathOrUrl)) {
    await openUrl(pathOrUrl)
  } else {
    await openPath(pathOrUrl)
  }
}

onMounted(() => {
  loadGlobalSdkConfig()
  loadEnvReport()
  loadRecentBuilds()
})

async function loadGlobalSdkConfig() {
  globalSdkLoading.value = true
  try {
    globalSdkConfig.value = await invoke<GlobalSdkConfig>('get_global_sdk_config')
  } catch (e) {
    console.error('Failed to load global SDK config:', e)
    message.error('无法读取全局 SDK 配置')
  } finally {
    globalSdkLoading.value = false
  }
}

async function loadEnvReport() {
  loading.value = true
  try {
    const report = await invoke<any>('get_full_env_report')
    
    const androidItems: EnvItem[] = []
    const iosItems: EnvItem[] = []
    const harmonyItems: EnvItem[] = []
    const commonItems: EnvItem[] = []

    function mk(name: string, status: EnvItem['status'], opts: { version?: string | null; path?: string | null; tool_key?: string; download_url?: string } = {}): EnvItem {
      return {
        name, status,
        version: opts.version || undefined,
        path: opts.path || undefined,
        download_url: opts.download_url || '',
        tool_key: opts.tool_key || name.toLowerCase().replace(/[^a-z0-9]/g, '_').replace(/\s+/, '_'),
        configurable: true,
      }
    }

    if (report.android?.available) {
      androidItems.push(mk('Android SDK', 'ok', { version: report.android.sdk_version, path: report.android.sdk_path, tool_key: 'android_sdk', download_url: 'https://developer.android.com/studio#command-line-tools-only' }))
    } else {
      androidItems.push(mk('Android SDK', 'error', { tool_key: 'android_sdk', download_url: 'https://developer.android.com/studio#command-line-tools-only' }))
    }

    if (report.android_studio?.installed) {
      androidItems.push(mk('Android Studio', 'ok', { version: report.android_studio.version, path: report.android_studio.path, tool_key: 'android_studio', download_url: 'https://developer.android.com/studio' }))
    } else {
      androidItems.push(mk('Android Studio', 'error', { tool_key: 'android_studio', download_url: 'https://developer.android.com/studio' }))
    }

    if (report.java?.installed) {
      androidItems.push(mk('JDK (Java)', 'ok', { version: report.java.version?.replace(/^v/i,'').split('(')[0].trim(), path: report.java.path, tool_key: 'java', download_url: 'https://adoptium.net/' }))
    } else {
      androidItems.push(mk('JDK (Java)', 'error', { tool_key: 'java', download_url: 'https://adoptium.net/' }))
    }

    if (report.gradle?.installed) {
      androidItems.push(mk('Gradle', 'ok', { version: report.gradle.version?.replace(/^v/i,'').split('(')[0].trim(), path: report.gradle.path, tool_key: 'gradle', download_url: 'https://grad.org/install' }))
    } else {
      androidItems.push(mk('Gradle', 'error', { tool_key: 'gradle', download_url: 'https://grad.org/install' }))
    }

    if (report.ndk?.installed) {
      androidItems.push(mk('NDK', 'ok', { version: report.ndk.version, path: report.ndk.path, tool_key: 'ndk', download_url: 'https://developer.android.com/ndk/downloads' }))
    } else {
      androidItems.push(mk('NDK', 'warning', { version: '未安装（可选）', tool_key: 'ndk', download_url: 'https://developer.android.com/ndk/downloads' }))
    }

    if (report.sdk_build_tools?.length > 0) {
      androidItems.push(mk('SDK Build Tools', 'ok', { version: report.sdk_build_tools.join(', '), tool_key: 'sdk_build_tools', download_url: '' }))
    } else if (report.android?.available) {
      androidItems.push(mk('SDK Build Tools', 'warning', { version: '未检测到', tool_key: 'sdk_build_tools', download_url: '' }))
    }

    if (report.ios?.available) {
      iosItems.push(mk('Xcode', 'ok', { version: report.ios.sdk_version || '', path: report.ios.sdk_path, tool_key: 'xcode', download_url: 'https://developer.apple.com/xcode/' }))
    } else {
      iosItems.push(mk('Xcode', 'error', { tool_key: 'xcode', download_url: 'https://developer.apple.com/xcode/' }))
    }

    if (report.command_line_tools?.installed) {
      iosItems.push(mk('Command Line Tools', 'ok', { version: report.command_line_tools.version, path: report.command_line_tools.path, tool_key: 'clt', download_url: 'https://developer.apple.com/download/more/' }))
    } else {
      iosItems.push(mk('Command Line Tools', 'warning', { tool_key: 'clt', download_url: 'https://developer.apple.com/download/more/' }))
    }

    if (report.cocoapods?.installed) {
      iosItems.push(mk('CocoaPods', 'ok', { version: report.cocoapods.version?.replace(/^v/i,'').split('(')[0].trim(), path: report.cocoapods.path, tool_key: 'cocoapods', download_url: 'https://cocoapods.org/' }))
    } else {
      iosItems.push(mk('CocoaPods', 'warning', { tool_key: 'cocoapods', download_url: 'https://cocoapods.org/' }))
    }

    if (report.harmony?.available) {
      harmonyItems.push(mk('DevEco Studio', 'ok', { version: report.harmony.sdk_version || '', path: report.harmony.sdk_path, tool_key: 'harmony', download_url: 'https://developer.huawei.com/consumer/cn/deveco-studio/' }))
    } else {
      harmonyItems.push(mk('DevEco Studio', 'error', { tool_key: 'harmony', download_url: 'https://developer.huawei.com/consumer/cn/deveco-studio/' }))
    }

    if (report.hbuilderx?.installed) {
      commonItems.push(mk('HBuilderX', 'ok', { version: `v${report.hbuilderx.version}`, path: report.hbuilderx.path, tool_key: 'hbuilderx', download_url: 'https://www.dcloud.io/hbuilderx.html' }))
    } else {
      commonItems.push(mk('HBuilderX', 'error', { tool_key: 'hbuilderx', download_url: 'https://www.dcloud.io/hbuilderx.html' }))
    }

    commonItems.push({ name: '磁盘空间', status: report.disk_space?.usage_percent > 80 ? 'warning' : 'ok', version: `可用 ${Math.round(report.disk_space?.free_gb || 0)} GB`, download_url: '', tool_key: '', configurable: false })

    function countOk(items: EnvItem[]) { return items.filter(i => i.status === 'ok').length }
    
    envGroups.value = [
      { label: 'Android 环境', icon: LogoAndroid, items: androidItems, ready_count: countOk(androidItems), total_count: androidItems.length },
      { label: 'iOS 环境', icon: LogoApple, items: iosItems, ready_count: countOk(iosItems), total_count: iosItems.length },
      { label: 'HarmonyOS 环境', icon: CubeOutline, items: harmonyItems, ready_count: countOk(harmonyItems), total_count: harmonyItems.length },
      { label: '通用', icon: SettingsOutline, items: commonItems, ready_count: countOk(commonItems), total_count: commonItems.length },
    ]

    try {
      const overrides = await invoke<Array<{ tool_name: string; version?: string | null; actual_path: string }>>('get_env_overrides')
      for (const ov of overrides) {
        const item = envGroups.value.flatMap(g => g.items).find(i => i.tool_key === ov.tool_name)
        if (item) {
          item.status = 'ok'
          item.version = ov.version || undefined
          item.path = ov.actual_path
          const group = envGroups.value.find(g => g.items.includes(item))
          if (group) group.ready_count = group.items.filter(i => i.status === 'ok').length
        }
      }
    } catch { /* ignore */ }
  } catch (e) {
    console.error('Failed to load env report:', e)
    message.error('无法获取环境信息')
  } finally {
    loading.value = false
  }
}

async function loadRecentBuilds() {
  try {
    const all = await invoke<any[]>('get_build_history', { projectId: null })
    recentBuilds.value = all.slice(0, 5).map((r: any) => ({
      id: r.id,
      project_name: r.project_name,
      platform: r.platform,
      status: r.status,
      version_name: r.version_name,
      started_at: r.started_at,
    }))
  } catch (e) { /* ignore */ }
}

function refreshAll() {
  loadGlobalSdkConfig()
  loadEnvReport()
  loadRecentBuilds()
  message.success('已刷新')
}

function getEnvStatusType(status: EnvItem['status']) {
  switch (status) {
    case 'ok': return 'success'
    case 'warning': return 'warning'
    case 'error': return 'error'
    default: return 'default'
  }
}

function getEnvStatusIcon(status: EnvItem['status']) {
  switch (status) {
    case 'ok': return CheckmarkCircleOutline
    case 'warning': return AlertCircleOutline
    case 'error': return CloseCircleOutline
    default: return AlertCircleOutline
  }
}

function hasDisplayVersion(version?: string) {
  const value = version?.trim()
  if (!value) return false
  const lower = value.toLowerCase()
  return !['detected', 'unknown', '未检测到', '未安装（可选）'].includes(lower)
}

function getEnvInfoText(item?: EnvItem | null) {
  if (!item) return '-'
  if (hasDisplayVersion(item.version)) return item.version!.trim()
  if (item.path?.trim()) return item.path.trim()
  return item.version?.trim() || '-'
}

function isEnvInfoPath(item?: EnvItem | null) {
  return !hasDisplayVersion(item?.version) && !!item?.path?.trim()
}

interface ToolValidationResult {
  valid: boolean
  tool_name: string
  version?: string
  actual_path: string
  details: string[]
  errors: string[]
  set_env_var?: string
}

const showPathDialog = ref(false)
const configuringItem = ref<EnvItem | null>(null)
const inputPath = ref('')
const setEnvVar = ref(false)
const validating = ref(false)
const validationResult = ref<ToolValidationResult | null>(null)

const showAddSdkDialog = ref(false)
const newSdkPlatform = ref<'android' | 'ios' | 'harmony'>('android')
const newSdkPath = ref('')
const addingSdk = ref(false)

function openPathConfig(item: EnvItem) {
  configuringItem.value = item
  inputPath.value = item.path || ''
  setEnvVar.value = false
  validationResult.value = null
  showPathDialog.value = true
}

async function selectEnvPath() {
  if (!configuringItem.value) return
  const selectsAppBundle = ['hbuilderx', 'xcode', 'android_studio'].includes(configuringItem.value.tool_key)
  const selected = await open({
    multiple: false,
    directory: !selectsAppBundle,
    filters: selectsAppBundle ? [{ name: 'macOS 应用', extensions: ['app'] }] : undefined,
    title: selectsAppBundle
      ? `选择 ${configuringItem.value.name} 应用`
      : `选择 ${configuringItem.value.name} 安装目录`,
  })
  if (typeof selected === 'string') {
    inputPath.value = selected
    validationResult.value = null
  }
}

async function validateAndSave() {
  if (!configuringItem.value) return
  validating.value = true
  validationResult.value = null
  try {
    const result = await invoke<ToolValidationResult>('validate_tool_path', {
      toolName: configuringItem.value.tool_key,
      path: inputPath.value,
    })
    validationResult.value = result
    
    if (result.valid) {
      const item = envGroups.value
        .flatMap(g => g.items)
        .find(i => i.name === configuringItem.value?.name)
      if (item) {
        item.status = 'ok'
        item.version = result.version || undefined
        item.path = result.actual_path
        item.validating = false
        
        const group = envGroups.value.find(g => g.items.includes(item))
        if (group) group.ready_count = group.items.filter(i => i.status === 'ok').length
      }
      
      try {
        await invoke('save_env_override', {
          toolName: configuringItem.value.tool_key,
          path: inputPath.value,
          actualPath: result.actual_path,
          version: result.version || null,
        })
      } catch { /* ignore persistence error */ }
      
      message.success(`${configuringItem.value.name} 校验通过${result.version ? ' — ' + result.version : ''}`)
      showPathDialog.value = false
    } else {
      message.error(result.errors.join('; '))
    }
  } catch (e: any) {
    message.error(String(e))
  } finally {
    validating.value = false
  }
}

async function selectSdkDirectory() {
  const selected = await open({
    multiple: false,
    directory: true,
    title: '选择 DCloud 离线 SDK 目录',
  })
  if (typeof selected === 'string') newSdkPath.value = selected
}

async function confirmAddSdk() {
  if (!newSdkPath.value) { message.warning('请先选择 SDK 目录'); return }
  addingSdk.value = true
  try {
    await invoke('add_sdk_path', { platform: newSdkPlatform.value, path: newSdkPath.value })
    showAddSdkDialog.value = false
    newSdkPath.value = ''
    loadGlobalSdkConfig()
    message.success('全局路径已保存')
  } catch (e: any) {
    message.error(String(e))
  } finally {
    addingSdk.value = false
  }
}

function openGlobalSdkConfig(platform: 'android' | 'ios' | 'harmony', currentPath: string) {
  newSdkPlatform.value = platform
  newSdkPath.value = currentPath
  showAddSdkDialog.value = true
}
</script>

<template>
  <div class="sdk-manager">
    <div class="page-header">
      <n-space align="center" justify="space-between" style="width: 100%;">
        <n-text strong style="font-size: 24px;">SDK & 环境管理</n-text>
        <n-button type="primary" @click="refreshAll" :loading="loading">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          刷新检测
        </n-button>
      </n-space>
    </div>

    <n-tabs v-model:value="activeTab" type="line" size="large">
      <!-- Tab 1: DCloud 离线SDK -->
      <n-tab-pane name="dcloud-sdk" tab="DCloud 离线SDK">
        <div class="tab-content">
          <n-spin v-if="globalSdkLoading" />

          <n-card v-else>
            <template #header>
              <n-space align="center">
                <n-icon :size="18"><FolderOpenOutline /></n-icon>
                <n-text strong>全局离线 SDK 配置</n-text>
              </n-space>
            </template>

            <n-table :single-line="true" size="small" :bordered="false">
              <thead>
                <tr>
                  <th>配置项</th>
                  <th style="width: 70px;">状态</th>
                  <th>路径</th>
                  <th style="width: 180px;">操作</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in globalSdkItems" :key="item.platform">
                  <td><n-text>{{ item.name }}</n-text></td>
                  <td>
                    <n-tag :type="getEnvStatusType(item.status)" size="small" round :bordered="false">
                      <template #icon><n-icon :size="12"><component :is="getEnvStatusIcon(item.status)" /></n-icon></template>
                      {{ item.status === 'ok' ? '已配置' : '未配置' }}
                    </n-tag>
                  </td>
                  <td>
                    <n-text v-if="item.path" code style="font-size: 11px;">{{ item.path }}</n-text>
                    <n-text v-else depth="3">-</n-text>
                  </td>
                  <td>
                    <n-space :size="4">
                      <n-button size="tiny" quaternary type="primary" @click="openExternal(item.download_url)">
                        下载
                      </n-button>
                      <n-button size="tiny" quaternary type="info" @click="openGlobalSdkConfig(item.platform, item.path)">
                        {{ item.path ? '修改路径' : '配置路径' }}
                      </n-button>
                    </n-space>
                  </td>
                </tr>
              </tbody>
            </n-table>
          </n-card>
        </div>
      </n-tab-pane>

      <!-- Tab 2: 环境检测 -->
      <n-tab-pane name="env-check" tab="环境检测">
        <div class="tab-content">
          <n-space vertical :size="20" style="width: 100%;">
            <n-card v-for="group in envGroups" :key="group.label">
              <template #header>
                <n-space align="center">
                  <n-icon :size="18"><component :is="group.icon" /></n-icon>
                  <n-text strong>{{ group.label }}</n-text>
                  <n-tag :type="group.ready_count === group.total_count ? 'success' : group.ready_count > 0 ? 'warning' : 'error'" size="small" round>
                    {{ group.ready_count }}/{{ group.total_count }}
                  </n-tag>
                </n-space>
              </template>

              <n-table :single-line="true" size="small" :bordered="false">
                <thead>
                  <tr>
                    <th>环境项</th>
                    <th style="width: 70px;">状态</th>
                    <th>版本/信息</th>
                    <th style="width: 120px;">操作</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="item in group.items" :key="item.name">
                    <td><n-text>{{ item.name }}</n-text></td>
                    <td>
                      <n-tag :type="getEnvStatusType(item.status)" size="small" round :bordered="false">
                        <template #icon><n-icon :size="12"><component :is="getEnvStatusIcon(item.status)" /></n-icon></template>
                        {{ item.status === 'ok' ? '就绪' : item.status === 'warning' ? '警告' : item.status === 'error' ? '缺失' : '未知' }}
                      </n-tag>
                    </td>
                    <td>
                      <n-text v-if="isEnvInfoPath(item)" code style="font-size: 11px;">{{ getEnvInfoText(item) }}</n-text>
                      <n-text v-else depth="3">{{ getEnvInfoText(item) }}</n-text>
                    </td>
                    <td>
                      <n-space :size="4" v-if="item.configurable !== false">
                        <n-button v-if="item.download_url" size="tiny" quaternary type="primary"
                          @click="openExternal(item.download_url)">
                          下载
                        </n-button>
                        <n-button size="tiny" quaternary type="info"
                          @click="openPathConfig(item)"
                          :loading="item.validating">
                          {{ item.status === 'ok' ? '修改路径' : '配置路径' }}
                        </n-button>
                      </n-space>
                    </td>
                  </tr>
                </tbody>
              </n-table>
            </n-card>
          </n-space>

          <!-- 底部汇总 -->
          <n-card size="small" :bordered="true" style="margin-top: 16px;">
            <n-space justify="center" :size="32">
              <n-space v-for="g in envGroups.filter(g => g.label.includes('Android') || g.label.includes('iOS'))" :key="g.label" align="center" :size="8">
                <n-text>{{ g.label.replace(' 环境','') }}:</n-text>
                <n-tag :type="g.ready_count === g.total_count ? 'success' : g.ready_count > 0 ? 'warning' : 'error'" round>
                  {{ g.ready_count === g.total_count ? '✅ 就绪' : `⚠️ ${g.ready_count}/${g.total_count}` }}
                </n-tag>
              </n-space>
            </n-space>
          </n-card>
        </div>
      </n-tab-pane>
    </n-tabs>

    <!-- 路径配置弹窗 -->
    <n-modal v-model:show="showPathDialog" preset="card"
      :title="`配置 ${configuringItem?.name || ''} 路径`"
      style="width: 540px;"
      :mask-closable="!validating">
      <n-space vertical :size="16">
        <n-alert v-if="configuringItem?.status === 'ok'" type="info">
          当前配置：{{ getEnvInfoText(configuringItem) }}。
          如需切换到其他安装位置，请在下方输入或浏览选择新路径。
        </n-alert>

        <n-form-item label="安装路径">
          <n-space :size="8" style="width: 100%;">
            <n-input v-model:value="inputPath" placeholder="/usr/local/... 或 /Applications/..." style="flex: 1;" />
            <n-button @click="selectEnvPath" :disabled="validating">浏览...</n-button>
          </n-space>
        </n-form-item>

        <n-checkbox v-if="configuringItem?.tool_key === 'android_sdk'"
          v-model:checked="setEnvVar">
          同时将此路径设置为 ANDROID_HOME 环境变量
        </n-checkbox>

        <div v-if="validating" style="padding: 8px 0;">
          <n-text depth="3">正在校验...</n-text>
        </div>

        <n-alert v-if="validationResult && !validating"
          :type="validationResult.valid ? 'success' : 'error'">
          <template v-if="validationResult.valid">
            ✓ 校验通过！{{ validationResult.version || '' }}
            <br/>
            <n-text depth="3">{{ validationResult.details.join(' / ') }}</n-text>
            <template v-if="validationResult.set_env_var && setEnvVar">
              <br/>建议设置环境变量: export {{ validationResult.set_env_var }}="{{ inputPath }}"
            </template>
          </template>
          <template v-else>
            ✗ 校验失败
            <br/>{{ validationResult.errors.join('; ') }}
          </template>
        </n-alert>
      </n-space>

      <template #action>
        <n-space justify="end">
          <n-button @click="showPathDialog = false" :disabled="validating">取消</n-button>
          <n-button type="primary" @click="validateAndSave" :loading="validating">
            验证并保存
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 添加 SDK 路径弹窗 -->
    <n-modal v-model:show="showAddSdkDialog" preset="card" title="添加 DCloud 离线 SDK 路径"
      style="width: 560px;">
      <n-space vertical :size="16">
        <n-alert type="info">
          请选择已解压的 DCloud 离线 SDK 或 Harmony 工程模板目录。可以选择下载包解压后的外层目录、SDK 子目录或其上级目录，工具会在所选目录内部自动定位可用内容。
        </n-alert>
        <n-form-item label="平台">
          <n-select v-model:value="newSdkPlatform" disabled :options="[
            { label: 'Android 离线SDK', value: 'android' },
            { label: 'iOS 离线SDK', value: 'ios' },
            { label: 'Harmony 工程模板', value: 'harmony' }
          ]" />
        </n-form-item>
        <n-form-item label="SDK 目录">
          <n-space :size="8" style="width: 100%;">
            <n-input v-model:value="newSdkPath" placeholder="选择或输入 SDK/模板解压路径..." style="flex: 1;" />
            <n-button @click="selectSdkDirectory">浏览...</n-button>
          </n-space>
        </n-form-item>
      </n-space>
      <template #action>
        <n-space justify="end">
          <n-button @click="showAddSdkDialog = false">取消</n-button>
          <n-button type="primary" @click="confirmAddSdk" :loading="addingSdk">确认添加</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
.sdk-manager {
  max-width: 1400px;
}

.page-header {
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.tab-content {
  padding-top: 16px;
}
</style>
