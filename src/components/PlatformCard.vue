<script setup lang="ts">
import { NCard, NIcon, NText, NBadge } from 'naive-ui'
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
}>()

const emit = defineEmits<{
  (e: 'toggle', platform: PlatformType): void
}>()

function isSelected(platform: PlatformType): boolean {
  return props.selectedPlatforms.includes(platform)
}

function togglePlatform(platform: PlatformType) {
  emit('toggle', platform)
}
</script>

<template>
  <div class="platform-grid">
    <n-card
      v-for="platform in platforms"
      :key="platform.key"
      class="platform-card"
      :class="{ 'is-selected': isSelected(platform.key) }"
      hoverable
      clickable
      @click="togglePlatform(platform.key)"
    >
      <div class="platform-content">
        <div 
          class="platform-icon-wrapper"
          :style="{ backgroundColor: platform.bgColor }"
        >
          <n-icon :size="32" :color="platform.color">
            <component :is="platform.icon" />
          </n-icon>
        </div>
        
        <div class="platform-info">
          <n-text strong class="platform-label">
            {{ platform.label }}
          </n-text>
          <n-text depth="3" class="platform-desc">
            {{ platform.description }}
          </n-text>
        </div>
        
        <n-badge 
          v-if="isSelected(platform.key)"
          type="success"
          value="✓"
          class="platform-badge"
        />
      </div>
    </n-card>
  </div>
</template>

<style scoped>
.platform-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
  margin-top: 16px;
}

.platform-card {
  transition: all 0.3s ease;
}

.platform-card.is-selected {
  border-color: #18a058;
  box-shadow: 0 0 0 2px rgba(24, 160, 88, 0.2);
}

.platform-content {
  display: flex;
  align-items: center;
  gap: 16px;
  position: relative;
}

.platform-icon-wrapper {
  width: 64px;
  height: 64px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.platform-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.platform-label {
  font-size: 16px;
}

.platform-desc {
  font-size: 13px;
}

.platform-badge {
  position: absolute;
  top: -8px;
  right: -8px;
}
</style>
