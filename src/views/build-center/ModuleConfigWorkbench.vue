<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  NAlert,
  NButton,
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
  WorkbenchField,
  WorkbenchModule,
  WorkbenchStatusFilter
} from './types'
import { formatPlatforms } from './moduleKeys'

const props = defineProps<{
  modules: WorkbenchModule<any, any>[]
  activeModuleKey: string | null
  isBuildLocked: boolean
  emptyText: string
  fieldValue: (mod: WorkbenchModule<any, any>, field: WorkbenchField<any>) => string
  fieldStatusType: (mod: WorkbenchModule<any, any>, field: WorkbenchField<any>) => 'default' | 'success' | 'warning' | 'error' | 'info'
  fieldStatusLabel: (mod: WorkbenchModule<any, any>, field: WorkbenchField<any>) => string
  selectOptions?: (mod: WorkbenchModule<any, any>, field: WorkbenchField<any>) => Array<{ label: string; value: string; disabled?: boolean }>
  formatFileSize?: (base64Value: string) => string
}>()

const emit = defineEmits<{
  (e: 'open-module', mod: WorkbenchModule<any, any>): void
  (e: 'update-field', mod: WorkbenchModule<any, any>, field: WorkbenchField<any>, value: string): void
  (e: 'pick-file-field', mod: WorkbenchModule<any, any>, field: WorkbenchField<any>): void
  (e: 'clear-file-field', mod: WorkbenchModule<any, any>, field: WorkbenchField<any>): void
}>()

const search = ref('')
const statusFilter = ref<WorkbenchStatusFilter>('all')

const filterOptions: Array<{ label: string; value: WorkbenchStatusFilter }> = [
  { label: '全部', value: 'all' },
  { label: '未配置', value: 'missing' },
  { label: '必填', value: 'required' },
  { label: '已配置', value: 'configured' }
]

const totalFields = computed(() => props.modules.reduce((sum, mod) => sum + mod.totalCount, 0))
const filledFields = computed(() => props.modules.reduce((sum, mod) => sum + mod.filledCount, 0))
const missingRequired = computed(() => props.modules.reduce((sum, mod) => sum + mod.missingRequiredCount, 0))
const requiredFields = computed(() => props.modules.reduce((sum, mod) => {
  return sum + mod.fields.filter(field => field.required).length
}, 0))

const filteredModules = computed(() => {
  const keyword = search.value.trim().toLowerCase()
  return props.modules.filter(mod => {
    if (statusFilter.value === 'missing' && mod.missingRequiredCount === 0) return false
    if (statusFilter.value === 'required' && !mod.fields.some(field => field.required)) return false
    if (statusFilter.value === 'configured' && (mod.missingRequiredCount > 0 || mod.totalCount === 0)) return false
    if (!keyword) return true
    const haystack = [
      mod.name,
      mod.category,
      formatPlatforms(mod.platforms),
      ...mod.fields.map(field => field.label),
      ...mod.fields.map(field => field.key)
    ].join(' ').toLowerCase()
    return haystack.includes(keyword)
  })
})

const activeModule = computed(() => {
  return props.modules.find(mod => mod.key === props.activeModuleKey)
    || props.modules.find(mod => mod.missingRequiredCount > 0)
    || props.modules[0]
    || null
})

function openModule(mod: WorkbenchModule<any, any>) {
  emit('open-module', mod)
}

function openNextMissing() {
  const missing = props.modules.filter(mod => mod.missingRequiredCount > 0)
  if (!missing.length) return
  const currentIndex = missing.findIndex(mod => mod.key === activeModule.value?.key)
  const next = missing[currentIndex >= 0 ? (currentIndex + 1) % missing.length : 0]
  emit('open-module', next)
}

function moduleItemClass(mod: WorkbenchModule<any, any>) {
  return [
    'module-workbench-item',
    `module-workbench-item--${mod.status}`,
    { 'module-workbench-item--active': activeModule.value?.key === mod.key }
  ]
}

function fieldFeedback(mod: WorkbenchModule<any, any>, field: WorkbenchField<any>) {
  return field.required && !props.fieldValue(mod, field).trim()
    ? '必填项，未填写时不能开始打包'
    : undefined
}

function fieldSelectOptions(mod: WorkbenchModule<any, any>, field: WorkbenchField<any>) {
  return props.selectOptions?.(mod, field) || []
}

function statusButtonType(value: WorkbenchStatusFilter) {
  return statusFilter.value === value ? 'primary' : 'default'
}

function statusButtonSecondary(value: WorkbenchStatusFilter) {
  return statusFilter.value !== value
}

function fileSize(value: string) {
  return props.formatFileSize?.(value) || ''
}
</script>

<template>
  <div class="module-workbench">
    <div class="module-workbench-summary">
      <div class="module-workbench-stat">
        <n-text depth="3">模块</n-text>
        <n-text strong>{{ modules.length }}</n-text>
      </div>
      <div class="module-workbench-stat">
        <n-text depth="3">字段</n-text>
        <n-text strong>{{ filledFields }} / {{ totalFields }}</n-text>
      </div>
      <div class="module-workbench-stat">
        <n-text depth="3">必填</n-text>
        <n-text strong>{{ requiredFields - missingRequired }} / {{ requiredFields }}</n-text>
      </div>
      <div class="module-workbench-action">
        <n-button
          size="small"
          secondary
          type="warning"
          :disabled="!missingRequired"
          @click="openNextMissing"
        >
          下一个未填
        </n-button>
      </div>
    </div>

    <div class="module-workbench-body">
      <aside class="module-workbench-sidebar">
        <n-input
          v-model:value="search"
          size="small"
          clearable
          placeholder="搜索模块或字段"
        />
        <div class="module-workbench-filter">
          <n-button
            v-for="option in filterOptions"
            :key="option.value"
            size="small"
            :type="statusButtonType(option.value)"
            :secondary="statusButtonSecondary(option.value)"
            @click="statusFilter = option.value"
          >
            {{ option.label }}
          </n-button>
        </div>
        <div class="module-workbench-list">
          <button
            v-for="mod in filteredModules"
            :key="mod.key"
            type="button"
            :class="moduleItemClass(mod)"
            @click="openModule(mod)"
          >
            <span class="module-workbench-item-main">
              <span class="module-workbench-item-title">{{ mod.name }}</span>
              <span class="module-workbench-item-meta">
                {{ formatPlatforms(mod.platforms) || mod.category }}
              </span>
            </span>
            <span class="module-workbench-item-side">
              <n-tag size="tiny" :type="mod.status" :bordered="false">{{ mod.statusLabel }}</n-tag>
              <span v-if="mod.missingRequiredCount" class="module-workbench-missing">
                {{ mod.missingRequiredCount }}
              </span>
            </span>
          </button>
          <n-alert v-if="!filteredModules.length" type="default" class="module-workbench-empty">
            没有匹配的模块。
          </n-alert>
        </div>
      </aside>

      <section class="module-workbench-detail">
        <template v-if="activeModule">
          <div class="module-workbench-detail-head">
            <n-space align="center" :size="8">
              <n-text strong>{{ activeModule.name }}</n-text>
              <n-tag size="small" type="info">{{ activeModule.category }}</n-tag>
              <n-tag
                v-if="formatPlatforms(activeModule.platforms)"
                size="small"
                :type="activeModule.platforms.includes('android') || activeModule.platforms.includes('ios') ? 'success' : 'default'"
              >
                {{ formatPlatforms(activeModule.platforms) }}
              </n-tag>
            </n-space>
            <n-text depth="3">{{ activeModule.filledCount }} / {{ activeModule.totalCount }} 已填写</n-text>
          </div>

          <n-alert v-if="!activeModule.fields.length" type="info">
            {{ emptyText }}
          </n-alert>

          <n-grid v-else :cols="2" :x-gap="14" :y-gap="10" responsive="screen">
            <n-gi v-for="field in activeModule.fields" :key="activeModule.key + field.key">
              <n-form-item :label="field.label" :feedback="fieldFeedback(activeModule, field)">
                <template #label>
                  <n-space align="center" :size="6">
                    <n-text>{{ field.label }}</n-text>
                    <n-tag size="tiny" :type="fieldStatusType(activeModule, field)">
                      {{ fieldStatusLabel(activeModule, field) }}
                    </n-tag>
                  </n-space>
                </template>
                <template v-if="field.fieldType === 'file'">
                  <n-space :size="8" align="center" class="file-field-row">
                    <n-button
                      size="small"
                      :disabled="isBuildLocked"
                      @click="emit('pick-file-field', activeModule, field)"
                    >
                      选择文件
                    </n-button>
                    <n-text v-if="fieldValue(activeModule, field)" depth="3" class="file-field-hint">
                      已选择{{ fileSize(fieldValue(activeModule, field)) ? ` (${fileSize(fieldValue(activeModule, field))})` : '' }}
                    </n-text>
                    <n-button
                      v-if="fieldValue(activeModule, field)"
                      size="small"
                      quaternary
                      type="error"
                      :disabled="isBuildLocked"
                      @click="emit('clear-file-field', activeModule, field)"
                    >
                      清除
                    </n-button>
                    <n-text v-else depth="3" class="file-field-hint">{{ field.placeholder }}</n-text>
                  </n-space>
                </template>
                <n-select
                  v-else-if="field.fieldType === 'select'"
                  :value="fieldValue(activeModule, field)"
                  :options="fieldSelectOptions(activeModule, field)"
                  :placeholder="field.placeholder"
                  :disabled="isBuildLocked"
                  @update:value="value => emit('update-field', activeModule, field, value)"
                />
                <n-input
                  v-else-if="field.fieldType === 'textarea'"
                  type="textarea"
                  :autosize="{ minRows: 2, maxRows: 4 }"
                  :value="fieldValue(activeModule, field)"
                  :placeholder="field.placeholder"
                  :disabled="isBuildLocked"
                  @update:value="value => emit('update-field', activeModule, field, value)"
                />
                <n-input
                  v-else
                  :value="fieldValue(activeModule, field)"
                  :placeholder="field.placeholder"
                  :type="field.secret ? 'password' : 'text'"
                  :show-password-on="field.secret ? 'click' : undefined"
                  :disabled="isBuildLocked"
                  @update:value="value => emit('update-field', activeModule, field, value)"
                />
              </n-form-item>
            </n-gi>
          </n-grid>
        </template>
      </section>
    </div>
  </div>
</template>
