<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  NAlert,
  NButton,
  NCard,
  NIcon,
  NSpace,
  NTag,
  NText
} from 'naive-ui'
import { ChevronDownOutline, ChevronUpOutline } from '@vicons/ionicons5'
import ModuleConfigWorkbench from './ModuleConfigWorkbench.vue'
import type {
  IosModuleConfigField,
  IosModuleConfigModule,
  UniappManifestInfo,
  WorkbenchField,
  WorkbenchModule
} from './types'
import {
  iosConfigModuleKey,
  iosModuleFieldValueKey,
  isIosPrivacyField
} from './moduleKeys'
import {
  iosFieldType,
  iosSelectFieldOptions
} from './moduleFields'

const props = defineProps<{
  visible: boolean
  iosMissingRequired: string[]
  bundleId: string
  teamId: string
  iosIconCount: number
  iosPrivacyDescriptionCount: number
  iosPrivacyDescriptionItemCount: number
  iosPrivacyDescriptionMissingCount: number
  insightAppId: string | number
  iosModuleSummaryLabel: string
  iosConfigurableModules: IosModuleConfigModule[]
  selectedManifestModuleCount: number
  iosModuleConfigLoading: boolean
  latestManifestInfo: UniappManifestInfo | null
  manifestReadWarning: string
  iosModuleMissingRequiredCount: number
  activeIosConfigModuleKey: string | null
  activeIosConfigModule: IosModuleConfigModule | null
  isBuildLocked: boolean
  iosConfigModuleStatusType: (mod: IosModuleConfigModule) => 'default' | 'success' | 'warning' | 'error'
  iosConfigModuleStatusLabel: (mod: IosModuleConfigModule) => string
  iosFieldValue: (mod: IosModuleConfigModule, field: IosModuleConfigField) => string
  iosFieldStatusType: (mod: IosModuleConfigModule, field: IosModuleConfigField) => 'default' | 'success' | 'warning' | 'error' | 'info'
  iosFieldStatusLabel: (mod: IosModuleConfigModule, field: IosModuleConfigField) => string
}>()

const emit = defineEmits<{
  (e: 'edit-privacy'): void
  (e: 'open-module', mod: IosModuleConfigModule): void
  (e: 'update-field', field: IosModuleConfigField, value: string): void
}>()

const expanded = ref(true)

const cardStatus = computed(() => {
  if (props.iosModuleConfigLoading) {
    return { type: 'info' as const, text: '正在分析' }
  }
  if (!props.latestManifestInfo) {
    return { type: 'warning' as const, text: '未关联项目' }
  }
  if (props.iosPrivacyDescriptionItemCount === 0 && props.iosConfigurableModules.length === 0) {
    return { type: 'success' as const, text: '无额外配置' }
  }
  if (props.iosModuleMissingRequiredCount > 0) {
    return { type: 'warning' as const, text: `缺少 ${props.iosModuleMissingRequiredCount} 项` }
  }
  if (props.iosPrivacyDescriptionItemCount > 0) {
    const done = props.iosPrivacyDescriptionItemCount - props.iosPrivacyDescriptionMissingCount
    return { type: 'success' as const, text: `配置就绪 · ${done}/${props.iosPrivacyDescriptionItemCount} 权限` }
  }
  return { type: 'success' as const, text: '配置就绪' }
})

function toggle() {
  expanded.value = !expanded.value
}

type IosWorkbenchModule = WorkbenchModule<IosModuleConfigModule, IosModuleConfigField>
type IosWorkbenchField = WorkbenchField<IosModuleConfigField>

function inlineConfigFields(mod: IosModuleConfigModule) {
  return mod.fields.filter(field => field.key !== 'LOCAL_POD' && !isIosPrivacyField(field))
}

const workbenchModules = computed<IosWorkbenchModule[]>(() => {
  return props.iosConfigurableModules.map(mod => {
    const inlineFields = inlineConfigFields(mod)
    const fields: IosWorkbenchField[] = inlineFields.map(field => ({
      key: iosModuleFieldValueKey(mod, field),
      label: field.label,
      required: field.required,
      secret: field.secret,
      placeholder: field.placeholder,
      fieldType: iosFieldType(field),
      raw: field
    }))
    const filledCount = inlineFields.filter(field => props.iosFieldValue(mod, field).trim()).length
    const missingRequiredCount = inlineFields.filter(field => {
      return field.required && !props.iosFieldValue(mod, field).trim()
    }).length
    return {
      key: iosConfigModuleKey(mod),
      name: mod.name,
      category: mod.category,
      platforms: mod.platforms,
      status: props.iosConfigModuleStatusType(mod),
      statusLabel: props.iosConfigModuleStatusLabel(mod),
      missingRequiredCount,
      filledCount,
      totalCount: inlineFields.length,
      fields,
      raw: mod
    }
  })
})

function openWorkbenchModule(mod: IosWorkbenchModule) {
  emit('open-module', mod.raw)
}

function updateWorkbenchField(_mod: IosWorkbenchModule, field: IosWorkbenchField, value: string) {
  emit('update-field', field.raw, value)
}

function workbenchFieldValue(mod: IosWorkbenchModule, field: IosWorkbenchField) {
  return props.iosFieldValue(mod.raw, field.raw)
}

function workbenchFieldStatusType(mod: IosWorkbenchModule, field: IosWorkbenchField) {
  return props.iosFieldStatusType(mod.raw, field.raw)
}

function workbenchFieldStatusLabel(mod: IosWorkbenchModule, field: IosWorkbenchField) {
  return props.iosFieldStatusLabel(mod.raw, field.raw)
}

function workbenchSelectOptions(mod: IosWorkbenchModule, field: IosWorkbenchField) {
  return iosSelectFieldOptions(mod.raw, field.raw)
}
</script>

<template>
  <div v-if="visible" class="ios-panel-stack">
    <!-- <n-card title="iOS 离线 SDK 工程" class="build-section-card">
      <n-space vertical :size="14">
        <n-alert :type="iosMissingRequired.length ? 'warning' : 'success'">
          <n-space vertical :size="6">
            <n-text v-if="iosMissingRequired.length">
              缺少 {{ iosMissingRequired.join('、') }}，补齐后才能生成 iOS 工程或 IPA。
            </n-text>
            <n-text v-else>iOS 基础配置已就绪，将使用 SDK 管理中配置的 HBuilder-Hello* 工程副本。</n-text>
          </n-space>
        </n-alert>

        <n-grid :cols="4" :x-gap="12" :y-gap="12" responsive="screen" class="insight-grid">
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">Bundle ID</n-text>
              <n-text strong class="summary-text">{{ bundleId || '-' }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">Team ID</n-text>
              <n-text strong class="summary-text">{{ teamId || '-' }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">iOS 图标</n-text>
              <n-text strong class="summary-value">{{ iosIconCount }}</n-text>
            </div>
          </n-gi>
          <n-gi>
            <div class="summary-tile">
              <n-text depth="3">隐私描述</n-text>
              <n-text strong class="summary-value">{{ iosPrivacyDescriptionCount }}</n-text>
            </div>
          </n-gi>
        </n-grid>

        <div class="path-summary">
          <n-text depth="3">工程来源</n-text>
          <n-text code>SDK 管理 / DCloud iOS 离线 SDK / HBuilder-Hello*</n-text>
          <n-text depth="3">App 资源</n-text>
          <n-text code>Pandora/apps/{{ insightAppId }}</n-text>
          <n-text depth="3">模块处理</n-text>
          <n-text code>{{ iosModuleSummaryLabel }}</n-text>
        </div>
      </n-space>
    </n-card> -->

    <n-card class="build-section-card" :class="{ 'card-collapsed': !expanded }">
      <template #header>
        <div class="card-collapsible-header" @click="toggle">
          <n-text strong>iOS 模块配置</n-text>
          <NSpace :size="8" align="center" class="card-header-actions">
            <NTag size="small" :type="cardStatus.type" :bordered="false">{{ cardStatus.text }}</NTag>
            <NIcon class="card-chevron" size="16">
              <ChevronUpOutline v-if="expanded" />
              <ChevronDownOutline v-else />
            </NIcon>
          </NSpace>
        </div>
      </template>

      <div v-show="expanded" class="card-body">
        <n-space vertical :size="14">
          <div class="ios-module-panel">
            <n-alert v-if="iosModuleConfigLoading" type="info">正在从 manifest 解析 iOS 模块配置...</n-alert>
            <n-alert v-else-if="!latestManifestInfo" type="warning">
              {{ manifestReadWarning || '请先在项目配置中设置本地项目路径，以便读取 manifest.json' }}
            </n-alert>
            <n-alert v-else-if="!iosConfigurableModules.length" type="success">
              已选模块暂无需要额外配置项的 iOS 模块。
            </n-alert>
            <n-alert v-else :type="iosModuleMissingRequiredCount ? 'warning' : 'success'">
              <n-space vertical :size="6">
                <n-text>
                  已选 {{ selectedManifestModuleCount }} 个 Manifest 模块，其中 {{ iosConfigurableModules.length }} 个需要 iOS 配置。
                </n-text>
                <n-text v-if="iosModuleMissingRequiredCount">
                  还有 {{ iosModuleMissingRequiredCount }} 个必填项未填写，填写完成后才能生成 iOS 工程或 IPA。
                </n-text>
                <n-text v-else>模块配置已就绪，可以开始 iOS 构建。</n-text>
              </n-space>
            </n-alert>

            <div v-if="iosPrivacyDescriptionItemCount" class="ios-privacy-summary">
              <div>
                <n-space align="center" :size="8">
                  <n-text strong>权限说明</n-text>
                  <n-tag
                    size="small"
                    :type="iosPrivacyDescriptionMissingCount ? 'error' : 'success'"
                    :bordered="false"
                  >
                    {{ iosPrivacyDescriptionItemCount - iosPrivacyDescriptionMissingCount }} / {{ iosPrivacyDescriptionItemCount }} 已填写
                  </n-tag>
                </n-space>
                <n-text depth="3" class="ios-privacy-summary-hint">
                  相同权限会自动合并展示，可集中填写 Info.plist 权限说明。
                </n-text>
              </div>
              <n-button
                type="primary"
                secondary
                :disabled="isBuildLocked"
                @click="emit('edit-privacy')"
              >
                编辑权限说明
              </n-button>
            </div>

            <ModuleConfigWorkbench
              v-if="workbenchModules.length"
              :modules="workbenchModules"
              :active-module-key="activeIosConfigModuleKey"
              :is-build-locked="isBuildLocked"
              empty-text="当前模块仅包含权限说明，请使用上方“编辑权限说明”集中填写。"
              :field-value="workbenchFieldValue"
              :field-status-type="workbenchFieldStatusType"
              :field-status-label="workbenchFieldStatusLabel"
              :select-options="workbenchSelectOptions"
              @open-module="openWorkbenchModule"
              @update-field="updateWorkbenchField"
            />
          </div>
        </n-space>
      </div>
    </n-card>
  </div>
</template>

<style scoped>
.card-collapsible-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  cursor: pointer;
  user-select: none;
}

.card-header-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.card-chevron {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  vertical-align: middle;
}

.card-body {
  display: block;
}
</style>
