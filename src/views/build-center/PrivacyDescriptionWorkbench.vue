<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  NAlert,
  NButton,
  NInput,
  NSpace,
  NTag,
  NText
} from 'naive-ui'
import type { IosPrivacyDescriptionItem } from './types'

const props = defineProps<{
  items: IosPrivacyDescriptionItem[]
  missingCount: number
  isBuildLocked: boolean
}>()

const emit = defineEmits<{
  (e: 'update-item', item: IosPrivacyDescriptionItem, value: string): void
}>()

const activeKey = ref('')
const search = ref('')
const filter = ref<'all' | 'missing' | 'configured'>('all')

const sortedItems = computed(() => {
  return [...props.items].sort((left, right) => {
    if (left.missing !== right.missing) return left.missing ? -1 : 1
    return left.label.localeCompare(right.label, 'zh-Hans-CN')
  })
})

const filteredItems = computed(() => {
  const keyword = search.value.trim().toLowerCase()
  return sortedItems.value.filter(item => {
    if (filter.value === 'missing' && !item.missing) return false
    if (filter.value === 'configured' && item.missing) return false
    if (!keyword) return true
    const haystack = [
      item.label,
      item.key,
      item.modules.join(' ')
    ].join(' ').toLowerCase()
    return haystack.includes(keyword)
  })
})

const activeItem = computed(() => {
  return props.items.find(item => item.fieldKey === activeKey.value)
    || sortedItems.value.find(item => item.missing)
    || sortedItems.value[0]
    || null
})

const filledCount = computed(() => props.items.length - props.missingCount)

watch(() => props.items, () => {
  if (!props.items.length) {
    activeKey.value = ''
    return
  }
  if (props.items.some(item => item.fieldKey === activeKey.value)) return
  activeKey.value = sortedItems.value.find(item => item.missing)?.fieldKey || sortedItems.value[0]?.fieldKey || ''
}, { immediate: true })

function openItem(item: IosPrivacyDescriptionItem) {
  activeKey.value = item.fieldKey
}

function openNextMissing() {
  const missing = sortedItems.value.filter(item => item.missing)
  if (!missing.length) return
  const currentIndex = missing.findIndex(item => item.fieldKey === activeItem.value?.fieldKey)
  activeKey.value = missing[currentIndex >= 0 ? (currentIndex + 1) % missing.length : 0].fieldKey
}

function itemClass(item: IosPrivacyDescriptionItem) {
  return [
    'privacy-workbench-item',
    item.missing ? 'privacy-workbench-item--missing' : 'privacy-workbench-item--success',
    { 'privacy-workbench-item--active': activeItem.value?.fieldKey === item.fieldKey }
  ]
}
</script>

<template>
  <div class="privacy-workbench">
    <div class="privacy-workbench-summary">
      <div class="privacy-workbench-stat">
        <n-text depth="3">权限说明</n-text>
        <n-text strong>{{ filledCount }} / {{ items.length }}</n-text>
      </div>
      <div class="privacy-workbench-stat">
        <n-text depth="3">未填写</n-text>
        <n-text strong>{{ missingCount }}</n-text>
      </div>
      <div class="privacy-workbench-action">
        <n-button size="small" secondary type="warning" :disabled="!missingCount" @click="openNextMissing">
          下一个未填
        </n-button>
      </div>
    </div>

    <div class="privacy-workbench-body">
      <aside class="privacy-workbench-sidebar">
        <n-input
          v-model:value="search"
          size="small"
          clearable
          placeholder="搜索权限或模块"
        />
        <div class="privacy-workbench-filter">
          <n-button size="small" :type="filter === 'all' ? 'primary' : 'default'" :secondary="filter !== 'all'" @click="filter = 'all'">
            全部
          </n-button>
          <n-button size="small" :type="filter === 'missing' ? 'primary' : 'default'" :secondary="filter !== 'missing'" @click="filter = 'missing'">
            未填写
          </n-button>
          <n-button size="small" :type="filter === 'configured' ? 'primary' : 'default'" :secondary="filter !== 'configured'" @click="filter = 'configured'">
            已填写
          </n-button>
        </div>
        <div class="privacy-workbench-list">
          <button
            v-for="item in filteredItems"
            :key="item.fieldKey"
            type="button"
            :class="itemClass(item)"
            @click="openItem(item)"
          >
            <span class="privacy-workbench-item-main">
              <span class="privacy-workbench-item-title">{{ item.label }}</span>
              <span class="privacy-workbench-item-meta">{{ item.modules.join('、') }}</span>
            </span>
            <n-tag size="tiny" :type="item.missing ? 'error' : 'success'" :bordered="false">
              {{ item.missing ? '必填' : '已填写' }}
            </n-tag>
          </button>
          <n-alert v-if="!filteredItems.length" type="default" class="privacy-workbench-empty">
            没有匹配的权限说明。
          </n-alert>
        </div>
      </aside>

      <section class="privacy-workbench-detail">
        <template v-if="activeItem">
          <div class="privacy-workbench-detail-head">
            <div>
              <n-space align="center" :size="8">
                <n-text strong>{{ activeItem.label }}</n-text>
                <n-tag size="small" :type="activeItem.missing ? 'error' : 'success'" :bordered="false">
                  {{ activeItem.missing ? '必填' : '已填写' }}
                </n-tag>
              </n-space>
              <n-text depth="3" class="privacy-workbench-modules">
                {{ activeItem.modules.join('、') }}
              </n-text>
            </div>
          </div>
          <n-input
            type="textarea"
            :autosize="{ minRows: 5, maxRows: 8 }"
            :value="activeItem.value"
            :placeholder="activeItem.placeholder"
            :status="activeItem.missing ? 'error' : undefined"
            :disabled="isBuildLocked"
            @update:value="value => emit('update-item', activeItem, value)"
          />
        </template>
      </section>
    </div>
  </div>
</template>
