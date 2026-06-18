<script setup lang="ts">
import {
  NAlert,
  NButton,
  NCard,
  NFormItem,
  NGi,
  NGrid,
  NInput,
  NSelect,
  NSpace,
  NTag,
  NText
} from 'naive-ui'
import type {
  AndroidModuleConfigField,
  AndroidModuleConfigModule,
  UniappManifestInfo
} from './types'
import {
  androidConfigModuleKey,
  formatPlatforms
} from './moduleKeys'
import {
  isFileField,
  isSelectField,
  selectFieldOptions
} from './moduleFields'

defineProps<{
  visible: boolean
  androidModuleConfigLoading: boolean
  latestManifestInfo: UniappManifestInfo | null
  manifestReadWarning: string
  androidConfigurableModules: AndroidModuleConfigModule[]
  selectedManifestModuleCount: number
  androidMissingRequiredCount: number
  activeAndroidConfigModuleKey: string | null
  activeAndroidConfigModule: AndroidModuleConfigModule | null
  isBuildLocked: boolean
  androidConfigModuleStatusType: (mod: AndroidModuleConfigModule) => 'default' | 'success' | 'warning' | 'error'
  configModuleStatusLabel: (mod: AndroidModuleConfigModule) => string
  androidFieldValue: (mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) => string
  fieldStatusType: (mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) => 'default' | 'success' | 'error' | 'info'
  fieldStatusLabel: (mod: AndroidModuleConfigModule, field: AndroidModuleConfigField) => string
  formatFileSize: (base64Value: string) => string
}>()

const emit = defineEmits<{
  (e: 'open-module', mod: AndroidModuleConfigModule): void
  (e: 'update-field', field: AndroidModuleConfigField, value: string): void
  (e: 'pick-file-field', mod: AndroidModuleConfigModule, field: AndroidModuleConfigField): void
  (e: 'clear-file-field', mod: AndroidModuleConfigModule, field: AndroidModuleConfigField): void
}>()
</script>

<template>
  <n-card v-if="visible" title="Android 模块配置" class="build-section-card">
    <n-space vertical :size="14">
      <n-alert v-if="androidModuleConfigLoading" type="info">正在从 manifest 解析 Android 模块配置...</n-alert>
      <n-alert v-else-if="!latestManifestInfo" type="warning">
        {{ manifestReadWarning || '请先在项目配置中设置本地项目路径，以便读取 manifest.json' }}
      </n-alert>
      <n-alert v-else-if="!androidConfigurableModules.length" type="success">
        已选模块暂无需要额外配置项的 Android 模块。
      </n-alert>
      <n-alert v-else :type="androidMissingRequiredCount ? 'warning' : 'success'">
        <n-space vertical :size="6">
          <n-text>
            已选 {{ selectedManifestModuleCount }} 个 Manifest 模块，其中 {{ androidConfigurableModules.length }} 个需要 Android 配置。
          </n-text>
          <n-text v-if="androidMissingRequiredCount">
            还有 {{ androidMissingRequiredCount }} 个必填项未填写，填写完成后才能开始打包。
          </n-text>
          <n-text v-else>模块配置已就绪，可以开始 Android 打包。</n-text>
        </n-space>
      </n-alert>

      <div v-if="androidConfigurableModules.length" class="android-config-list">
        <n-space wrap :size="8" class="android-config-switcher">
          <n-tag
            v-for="mod in androidConfigurableModules"
            :key="androidConfigModuleKey(mod)"
            class="android-config-chip"
            :class="{ 'android-config-chip--active': activeAndroidConfigModuleKey === androidConfigModuleKey(mod) }"
            :type="androidConfigModuleStatusType(mod)"
            :bordered="activeAndroidConfigModuleKey !== androidConfigModuleKey(mod)"
            @click="emit('open-module', mod)"
          >
            {{ mod.name }} · {{ configModuleStatusLabel(mod) }}
          </n-tag>
        </n-space>

        <div v-if="activeAndroidConfigModule" class="android-config-module">
          <div class="android-config-head">
            <n-space align="center" :size="8">
              <n-text strong>{{ activeAndroidConfigModule.name }}</n-text>
              <n-tag size="small" type="info">{{ activeAndroidConfigModule.category }}</n-tag>
              <n-tag v-if="formatPlatforms(activeAndroidConfigModule.platforms)" size="small" :type="activeAndroidConfigModule.platforms.includes('android') ? 'success' : 'default'">{{ formatPlatforms(activeAndroidConfigModule.platforms) }}</n-tag>
            </n-space>
            <n-text depth="3">{{ activeAndroidConfigModule.fields.length }} 项配置</n-text>
          </div>
          <n-grid :cols="2" :x-gap="14" :y-gap="10" responsive="screen">
            <n-gi v-for="field in activeAndroidConfigModule.fields" :key="activeAndroidConfigModule.templateKey + field.key">
              <n-form-item :label="field.label" :feedback="field.required && !androidFieldValue(activeAndroidConfigModule, field).trim() ? '必填项，未填写时不能开始打包' : undefined">
                <template #label>
                  <n-space align="center" :size="6">
                    <n-text>{{ field.label }}</n-text>
                    <n-tag size="tiny" :type="fieldStatusType(activeAndroidConfigModule, field)">{{ fieldStatusLabel(activeAndroidConfigModule, field) }}</n-tag>
                  </n-space>
                </template>
                <template v-if="isFileField(field)">
                  <n-space :size="8" align="center" class="file-field-row">
                    <n-button size="small" :disabled="isBuildLocked" @click="emit('pick-file-field', activeAndroidConfigModule, field)">选择文件</n-button>
                    <n-text v-if="androidFieldValue(activeAndroidConfigModule, field)" depth="3" class="file-field-hint">
                      已选择 ({{ formatFileSize(androidFieldValue(activeAndroidConfigModule, field)) }})
                    </n-text>
                    <n-button v-if="androidFieldValue(activeAndroidConfigModule, field)" size="small" quaternary type="error" :disabled="isBuildLocked" @click="emit('clear-file-field', activeAndroidConfigModule, field)">清除</n-button>
                    <n-text v-else depth="3" class="file-field-hint">{{ field.placeholder }}</n-text>
                  </n-space>
                </template>
                <n-select
                  v-else-if="isSelectField(field)"
                  :value="androidFieldValue(activeAndroidConfigModule, field)"
                  :options="selectFieldOptions(activeAndroidConfigModule, field)"
                  :placeholder="field.placeholder"
                  :disabled="isBuildLocked"
                  @update:value="value => emit('update-field', field, value)"
                />
                <n-input
                  v-else
                  :value="androidFieldValue(activeAndroidConfigModule, field)"
                  :placeholder="field.placeholder"
                  :type="field.secret ? 'password' : 'text'"
                  :show-password-on="field.secret ? 'click' : undefined"
                  :disabled="isBuildLocked"
                  @update:value="value => emit('update-field', field, value)"
                />
              </n-form-item>
            </n-gi>
          </n-grid>
        </div>
      </div>
    </n-space>
  </n-card>
</template>
