<script setup lang="ts">
import { NCard, NIcon, NText } from 'naive-ui'
import type { Component } from 'vue'

export type PlatformType = 'android' | 'ios' | 'harmony'

interface PlatformConfig {
  key: PlatformType
  label: string
  icon: Component
  description: string
  color: string
  bgColor: string
}

const props = defineProps<{
  platforms: PlatformConfig[]
  selectedPlatforms: PlatformType[]
  disabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle', platform: PlatformType): void
}>()

function isSelected(platform: PlatformType): boolean {
  return props.selectedPlatforms.includes(platform)
}

function togglePlatform(platform: PlatformType) {
  if (props.disabled) return
  emit('toggle', platform)
}
</script>

<template>
  <div class="platform-grid">
    <n-card
      v-for="platform in platforms"
      :key="platform.key"
      class="platform-card"
      :class="{ 'is-selected': isSelected(platform.key), 'is-disabled': disabled }"
      :hoverable="!disabled"
      :clickable="!disabled"
      @click="togglePlatform(platform.key)"
      content-style="padding: 10px;"
    >
      <div class="platform-content">
        <div 
          class="platform-icon-wrapper"
          :style="{ backgroundColor: platform.bgColor }"
        >
          <n-icon :size="16" :color="platform.color">
            <component :is="platform.icon" />
          </n-icon>
        </div>
        
        <div class="platform-info">
          <n-text strong class="platform-label">
            {{ platform.label }}
          </n-text>
          <n-text depth="3" class="platform-desc">
            ({{ platform.description }})
          </n-text>
        </div>
        
        <!-- <div
          v-if="isSelected(platform.key)"
          class="platform-check"
        >
          <n-icon :size="18"><CheckmarkCircleOutline /></n-icon>
        </div> -->
      </div>
    </n-card>
  </div>
</template>

<style scoped>
.platform-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 12px;
  margin-top: 16px;
}

.platform-card {
  border: 1px solid var(--border-soft);
  transition: border-color 0.18s ease, box-shadow 0.18s ease, transform 0.18s ease;
}

.platform-card:not(.is-disabled):hover {
  border-color: rgba(21, 151, 102, 0.24);
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}

.platform-card.is-selected {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(21, 151, 102, 0.14);
}

.platform-card.is-disabled {
  cursor: not-allowed;
  opacity: 0.62;
}

.platform-content {
  display: flex;
  /* flex-direction: column; */
  align-items: center;
  gap: 10px;
  position: relative;
}

.platform-icon-wrapper {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.platform-info {
  flex: 1;
  display: flex;
  align-items: baseline;
  /* gap: 4px; */
}

.platform-label {
  font-size: 16px;
  line-height: 1.3;
}

.platform-desc {
  font-size: 13px;
  line-height: 1.45;
  white-space: normal;
}

.platform-check {
  position: absolute;
  top: -2px;
  right: -2px;
  color: var(--primary-color);
  line-height: 1;
}
</style>
