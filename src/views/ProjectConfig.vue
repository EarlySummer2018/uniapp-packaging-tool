<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  NAlert,
  NButton,
  NCard,
  NForm,
  NFormItem,
  NGi,
  NGrid,
  NIcon,
  NInput,
  NInputNumber,
  NSelect,
  NSpace,
  NSwitch,
  NTabPane,
  NTabs,
  NText,
  NTooltip,
  useMessage
} from 'naive-ui'
import { ArrowBackOutline, FolderOpenOutline, HelpCircleOutline, SaveOutline } from '@vicons/ionicons5'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useProjectsStore, type Project } from '../stores/projects'
import { useBuildStore } from '../stores/build'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const projectsStore = useProjectsStore()
const buildStore = useBuildStore()

const projectId = computed(() => route.params.id as string)
const projectForm = ref<Project | null>(null)
const loading = ref(false)
const androidStorePassword = ref('')
const androidKeyPassword = ref('')
const iosCertificatePassword = ref('')
const harmonyStorePassword = ref('')
const harmonyKeyPassword = ref('')
const MIN_16KB_COMPILE_SDK = 36
const isBuildLocked = computed(() => buildStore.hasActiveBuilds)

const compileSdkWarning = computed(() => {
  const value = projectForm.value?.android.compileSdkVersion
  if (typeof value === 'number' && value < MIN_16KB_COMPILE_SDK) {
    return '为兼容 16KB 内存页，compileSdk 建议设置为 36 或以上。'
  }
  return undefined
})

const exportMethodOptions = [
  { label: 'App Store', value: 'app-store' },
  { label: 'Ad Hoc', value: 'ad-hoc' },
  { label: 'Enterprise', value: 'enterprise' },
  { label: 'Development', value: 'development' }
]

let projectLoadSequence = 0

watch(projectId, async (id) => {
  const sequence = ++projectLoadSequence
  loading.value = true
  projectForm.value = null
  androidStorePassword.value = ''
  androidKeyPassword.value = ''
  iosCertificatePassword.value = ''
  harmonyStorePassword.value = ''
  harmonyKeyPassword.value = ''
  try {
    const project = await projectsStore.getProject(id)
    if (sequence === projectLoadSequence) {
      projectForm.value = JSON.parse(JSON.stringify(project))
    }
  } catch (e: any) {
    if (sequence === projectLoadSequence) message.error(String(e))
  } finally {
    if (sequence === projectLoadSequence) loading.value = false
  }
}, { immediate: true })

async function handleSave() {
  if (!projectForm.value) return
  if (isBuildLocked.value) {
    message.warning('已有构建任务进行中，项目配置暂不可保存')
    return
  }
  loading.value = true
  try {
    await persistSecrets(projectForm.value)
    projectForm.value = await projectsStore.saveProject(projectForm.value)
    message.success('配置已保存')
  } catch (e: any) {
    message.error(String(e))
  } finally {
    loading.value = false
  }
}

async function persistSecrets(project: Project) {
  if (androidStorePassword.value) {
    await invoke('save_signing_secret', {
      projectId: project.id,
      key: 'android-store-password',
      value: androidStorePassword.value
    })
    project.android.keystore.hasStorePassword = true
    androidStorePassword.value = ''
  }
  if (androidKeyPassword.value) {
    await invoke('save_signing_secret', {
      projectId: project.id,
      key: 'android-key-password',
      value: androidKeyPassword.value
    })
    project.android.keystore.hasKeyPassword = true
    androidKeyPassword.value = ''
  }
  if (iosCertificatePassword.value) {
    await invoke('save_signing_secret', {
      projectId: project.id,
      key: 'ios-certificate-password',
      value: iosCertificatePassword.value
    })
    project.ios.hasCertificatePassword = true
    iosCertificatePassword.value = ''
  }
  if (harmonyStorePassword.value) {
    await invoke('save_signing_secret', {
      projectId: project.id,
      key: 'harmony-store-password',
      value: harmonyStorePassword.value
    })
    project.harmony.signingConfig.hasStorePassword = true
    harmonyStorePassword.value = ''
  }
  if (harmonyKeyPassword.value) {
    await invoke('save_signing_secret', {
      projectId: project.id,
      key: 'harmony-key-password',
      value: harmonyKeyPassword.value
    })
    project.harmony.signingConfig.hasKeyPassword = true
    harmonyKeyPassword.value = ''
  }
}

async function chooseDirectory(assign: (value: string) => void) {
  if (isBuildLocked.value) {
    message.warning('已有构建任务进行中，暂不能修改项目路径')
    return
  }
  const selected = await open({ directory: true, multiple: false })
  if (typeof selected === 'string') assign(selected)
}

async function chooseLocalProjectDirectory() {
  if (isBuildLocked.value) {
    message.warning('已有构建任务进行中，暂不能修改项目路径')
    return
  }
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择包含 manifest.json 的 UniApp 项目目录'
  })
  if (typeof selected === 'string' && projectForm.value) {
    projectForm.value.localPath = selected
  }
}

async function chooseFile(assign: (value: string) => void, extensions: string[]) {
  if (isBuildLocked.value) {
    message.warning('已有构建任务进行中，暂不能修改项目文件')
    return
  }
  const selected = await open({
    directory: false,
    multiple: false,
    filters: [{ name: 'File', extensions }]
  })
  if (typeof selected === 'string') assign(selected)
}

function goBack() {
  router.push('/')
}

function goBuild() {
  if (isBuildLocked.value) {
    message.warning('已有构建任务进行中，请等待完成后再开始打包')
    return
  }
  router.push(`/build/${projectId.value}`)
}
</script>

<template>
  <div class="project-config">
    <div class="page-header">
      <n-space align="center">
        <n-button quaternary circle @click="goBack">
          <template #icon><n-icon><ArrowBackOutline /></n-icon></template>
        </n-button>
        <div>
          <n-text strong class="page-title">项目配置</n-text>
          <n-text v-if="projectForm" depth="3" class="page-subtitle">{{ projectForm.name }}</n-text>
        </div>
      </n-space>
      <n-space data-guide="config-actions" class="header-actions">
        <n-button type="primary" :loading="loading" :disabled="isBuildLocked" @click="handleSave">
          <template #icon><n-icon><SaveOutline /></n-icon></template>
          保存配置
        </n-button>
        <n-button type="success" :disabled="isBuildLocked" @click="goBuild">开始打包</n-button>
      </n-space>
    </div>

    <n-alert v-if="isBuildLocked" type="warning">
      当前有构建任务进行中，项目配置暂不可编辑。
    </n-alert>

    <n-alert v-if="!projectForm" type="info">正在加载项目配置...</n-alert>

    <n-card v-else class="config-panel">
      <n-tabs data-guide="config-tabs" type="line" animated class="config-tabs">
        <n-tab-pane name="basic" tab="基础信息">
          <n-form label-placement="left" label-width="130" :disabled="isBuildLocked">
            <n-grid :cols="2" :x-gap="18" :y-gap="4" responsive="screen">
              <n-gi><n-form-item label="项目名称"><n-input v-model:value="projectForm.name" /></n-form-item></n-gi>
              <n-gi span="2">
                <n-form-item data-guide="project-path" label="本地项目路径">
                  <n-space class="inline-field-row">
                    <n-input
                      v-model:value="projectForm.localPath"
                      placeholder="选择本地 UniApp 项目目录"
                    />
                    <n-button :disabled="isBuildLocked" @click="chooseLocalProjectDirectory">
                      <template #icon><n-icon><FolderOpenOutline /></n-icon></template>
                      选择
                    </n-button>
                  </n-space>
                </n-form-item>
              </n-gi>
              <n-gi span="2">
                <n-form-item label="输出目录">
                  <n-space class="inline-field-row">
                    <n-input v-model:value="projectForm.outputDir" />
                    <n-button :disabled="isBuildLocked" @click="chooseDirectory(v => projectForm!.outputDir = v)">选择</n-button>
                  </n-space>
                </n-form-item>
              </n-gi>
              <n-gi span="2"><n-form-item label="项目描述"><n-input v-model:value="projectForm.description" type="textarea" /></n-form-item></n-gi>
            </n-grid>
          </n-form>
        </n-tab-pane>

        <n-tab-pane name="android" tab="Android">
          <n-form label-placement="left" label-width="150" :disabled="isBuildLocked">
            <n-form-item label="启用 Android"><n-switch v-model:value="projectForm.android.enabled" /></n-form-item>
            <n-form-item label="DCloud AppKey">
              <n-input v-model:value="projectForm.android.dcloudAppKey" type="password" show-password-on="click" />
            </n-form-item>
            <n-grid :cols="2" :x-gap="18" :y-gap="4" responsive="screen">
              <n-gi><n-form-item label="包名"><n-input v-model:value="projectForm.android.packageName" /></n-form-item></n-gi>
              <n-gi>
                <n-form-item
                  label="compileSdk"
                  :feedback="compileSdkWarning"
                  :validation-status="compileSdkWarning ? 'warning' : undefined"
                >
                  <div class="compile-sdk-control">
                    <n-input-number
                      v-model:value="projectForm.android.compileSdkVersion"
                      :min="1"
                      class="compile-sdk-input"
                    />
                    <n-tooltip trigger="hover">
                      <template #trigger>
                        <n-icon class="compile-sdk-help">
                          <HelpCircleOutline />
                        </n-icon>
                      </template>
                      为适配 16KB 内存页，compileSdk 建议配置为 36 或以上。
                    </n-tooltip>
                  </div>
                </n-form-item>
              </n-gi>
            </n-grid>
            <n-form-item label="Keystore">
              <n-space class="inline-field-row">
                <n-input v-model:value="projectForm.android.keystore.path" />
                <n-button :disabled="isBuildLocked" @click="chooseFile(v => projectForm!.android.keystore.path = v, ['jks', 'keystore'])">选择</n-button>
              </n-space>
            </n-form-item>
            <n-form-item label="Key Alias"><n-input v-model:value="projectForm.android.keystore.alias" /></n-form-item>
            <n-grid :cols="2" :x-gap="18" :y-gap="4" responsive="screen">
              <n-gi><n-form-item label="Store 密码"><n-input v-model:value="androidStorePassword" type="password" show-password-on="click" :placeholder="projectForm.android.keystore.hasStorePassword ? '已保存，留空不变' : '请输入'" /></n-form-item></n-gi>
              <n-gi><n-form-item label="Key 密码"><n-input v-model:value="androidKeyPassword" type="password" show-password-on="click" :placeholder="projectForm.android.keystore.hasKeyPassword ? '已保存，留空不变' : '请输入'" /></n-form-item></n-gi>
            </n-grid>
          </n-form>
        </n-tab-pane>

        <n-tab-pane name="ios" tab="iOS">
          <n-form label-placement="left" label-width="150" :disabled="isBuildLocked">
            <n-form-item label="启用 iOS"><n-switch v-model:value="projectForm.ios.enabled" /></n-form-item>
            <n-form-item label="DCloud AppKey">
              <n-input v-model:value="projectForm.ios.dcloudAppKey" type="password" show-password-on="click" />
            </n-form-item>
            <n-grid :cols="2" :x-gap="18" :y-gap="4" responsive="screen">
              <n-gi><n-form-item label="Bundle ID"><n-input v-model:value="projectForm.ios.bundleId" /></n-form-item></n-gi>
              <n-gi><n-form-item label="Team ID"><n-input v-model:value="projectForm.ios.teamId" /></n-form-item></n-gi>
              <n-gi><n-form-item label="导出方式"><n-select v-model:value="projectForm.ios.exportMethod" :options="exportMethodOptions" /></n-form-item></n-gi>
            </n-grid>
            <n-form-item label="描述文件">
              <n-space class="inline-field-row"><n-input v-model:value="projectForm.ios.provisioningProfile" /><n-button :disabled="isBuildLocked" @click="chooseFile(v => projectForm!.ios.provisioningProfile = v, ['mobileprovision'])">选择</n-button></n-space>
            </n-form-item>
            <n-form-item label="P12 证书">
              <n-space class="inline-field-row"><n-input v-model:value="projectForm.ios.certificate" /><n-button :disabled="isBuildLocked" @click="chooseFile(v => projectForm!.ios.certificate = v, ['p12'])">选择</n-button></n-space>
            </n-form-item>
            <n-form-item label="P12 密码"><n-input v-model:value="iosCertificatePassword" type="password" show-password-on="click" :placeholder="projectForm.ios.hasCertificatePassword ? '已保存，留空不变' : '请输入'" /></n-form-item>
          </n-form>
        </n-tab-pane>

        <n-tab-pane name="harmony" tab="鸿蒙">
          <n-form label-placement="left" label-width="150" :disabled="isBuildLocked">
            <n-form-item label="启用鸿蒙"><n-switch v-model:value="projectForm.harmony.enabled" /></n-form-item>
            <n-grid :cols="2" :x-gap="18" :y-gap="4" responsive="screen">
              <n-gi><n-form-item label="Bundle Name"><n-input v-model:value="projectForm.harmony.bundleName" /></n-form-item></n-gi>
              <n-gi><n-form-item label="运行时版本"><n-input v-model:value="projectForm.harmony.runtimeVersion" /></n-form-item></n-gi>
            </n-grid>
            <n-form-item label="签名文件">
              <n-space class="inline-field-row"><n-input v-model:value="projectForm.harmony.signingConfig.storeFile" /><n-button :disabled="isBuildLocked" @click="chooseFile(v => projectForm!.harmony.signingConfig.storeFile = v, ['p12', 'jks'])">选择</n-button></n-space>
            </n-form-item>
            <n-form-item label="Key Alias"><n-input v-model:value="projectForm.harmony.signingConfig.keyAlias" /></n-form-item>
            <n-grid :cols="2" :x-gap="18" :y-gap="4" responsive="screen">
              <n-gi><n-form-item label="Store 密码"><n-input v-model:value="harmonyStorePassword" type="password" show-password-on="click" /></n-form-item></n-gi>
              <n-gi><n-form-item label="Key 密码"><n-input v-model:value="harmonyKeyPassword" type="password" show-password-on="click" /></n-form-item></n-gi>
            </n-grid>
          </n-form>
        </n-tab-pane>
      </n-tabs>
    </n-card>
  </div>
</template>

<style scoped>
.project-config {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 1280px;
}

.page-header {
  margin-bottom: 2px;
}

.header-actions {
  flex-shrink: 0;
}

.config-panel :deep(.n-tabs-nav) {
  padding: 0 2px;
}

.config-tabs :deep(.n-tab-pane) {
  padding-top: 18px;
}

.inline-field-row {
  width: 100%;
}

.compile-sdk-control {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.compile-sdk-input {
  flex: 1;
  min-width: 0;
}

.compile-sdk-help {
  color: var(--warning-color);
  cursor: help;
  font-size: 18px;
}

</style>
