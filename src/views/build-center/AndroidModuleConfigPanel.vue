<script setup lang="ts">
import { computed } from 'vue'
import {
  NAlert,
  NCard,
  NSpace,
  NText
} from 'naive-ui'
import ModuleConfigWorkbench from './ModuleConfigWorkbench.vue'
import type {
  AndroidModuleConfigField,
  AndroidModuleConfigModule,
  UniappManifestInfo,
  WorkbenchField,
  WorkbenchModule
} from './types'
import {
  androidConfigModuleKey,
  androidModuleFieldValueKey
} from './moduleKeys'
import {
  androidFieldType,
  selectFieldOptions
} from './moduleFields'

const props = defineProps<{
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

type AndroidWorkbenchModule = WorkbenchModule<AndroidModuleConfigModule, AndroidModuleConfigField>
type AndroidWorkbenchField = WorkbenchField<AndroidModuleConfigField>

const workbenchModules = computed<AndroidWorkbenchModule[]>(() => {
  return props.androidConfigurableModules.map(mod => {
    const fields: AndroidWorkbenchField[] = mod.fields.map(field => ({
      key: androidModuleFieldValueKey(mod, field),
      label: field.label,
      required: field.required,
      secret: field.secret,
      placeholder: field.placeholder,
      fieldType: androidFieldType(field),
      raw: field
    }))
    const filledCount = mod.fields.filter(field => props.androidFieldValue(mod, field).trim()).length
    const missingRequiredCount = mod.fields.filter(field => {
      return field.required && !props.androidFieldValue(mod, field).trim()
    }).length
    return {
      key: androidConfigModuleKey(mod),
      name: mod.name,
      category: mod.category,
      platforms: mod.platforms,
      status: props.androidConfigModuleStatusType(mod),
      statusLabel: props.configModuleStatusLabel(mod),
      missingRequiredCount,
      filledCount,
      totalCount: mod.fields.length,
      fields,
      raw: mod
    }
  })
})

function openWorkbenchModule(mod: AndroidWorkbenchModule) {
  emit('open-module', mod.raw)
}

function updateWorkbenchField(_mod: AndroidWorkbenchModule, field: AndroidWorkbenchField, value: string) {
  emit('update-field', field.raw, value)
}

function pickWorkbenchFile(mod: AndroidWorkbenchModule, field: AndroidWorkbenchField) {
  emit('pick-file-field', mod.raw, field.raw)
}

function clearWorkbenchFile(mod: AndroidWorkbenchModule, field: AndroidWorkbenchField) {
  emit('clear-file-field', mod.raw, field.raw)
}

function workbenchFieldValue(mod: AndroidWorkbenchModule, field: AndroidWorkbenchField) {
  return props.androidFieldValue(mod.raw, field.raw)
}

function workbenchFieldStatusType(mod: AndroidWorkbenchModule, field: AndroidWorkbenchField) {
  return props.fieldStatusType(mod.raw, field.raw)
}

function workbenchFieldStatusLabel(mod: AndroidWorkbenchModule, field: AndroidWorkbenchField) {
  return props.fieldStatusLabel(mod.raw, field.raw)
}

function workbenchSelectOptions(mod: AndroidWorkbenchModule, field: AndroidWorkbenchField) {
  return selectFieldOptions(mod.raw, field.raw)
}
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

      <ModuleConfigWorkbench
        v-if="workbenchModules.length"
        :modules="workbenchModules"
        :active-module-key="activeAndroidConfigModuleKey"
        :is-build-locked="isBuildLocked"
        empty-text="当前模块暂无需要填写的 Android 配置。"
        :field-value="workbenchFieldValue"
        :field-status-type="workbenchFieldStatusType"
        :field-status-label="workbenchFieldStatusLabel"
        :select-options="workbenchSelectOptions"
        :format-file-size="formatFileSize"
        @open-module="openWorkbenchModule"
        @update-field="updateWorkbenchField"
        @pick-file-field="pickWorkbenchFile"
        @clear-file-field="clearWorkbenchFile"
      />
    </n-space>
  </n-card>
</template>
