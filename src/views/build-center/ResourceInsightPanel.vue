<script setup lang="ts">
import {
  NAlert,
  NCard,
  NCheckbox,
  NGi,
  NGrid,
  NSpace,
  NTag,
  NText
} from 'naive-ui'
import type { DetectedModule, ModuleStatusTone, ResourceScanResult } from './types'
import { formatPlatforms, manifestModuleKey } from './moduleKeys'

defineProps<{
  scanResult: ResourceScanResult | null
  insightAppName: string | number
  insightAppId: string | number
  insightVersionName: string | number
  insightVersionCode: string | number
  insightManifestPath: string | number
  selectedManifestModules: DetectedModule[]
  manifestModules: DetectedModule[]
  utsDependencyCount: number
  utsPluginLabels: string[]
  manifestReadWarning: string
  isBuildLocked: boolean
  isManifestModuleSelected: (mod: DetectedModule) => boolean
  manifestModuleStatusType: (mod: DetectedModule) => ModuleStatusTone
  manifestModuleStatusClass: (mod: DetectedModule) => string
  manifestModuleStatusLabel: (mod: DetectedModule) => string
  manifestModuleFieldSummaries: (mod: DetectedModule) => string[]
}>()

const emit = defineEmits<{
  (e: 'set-manifest-module-selected', mod: DetectedModule, checked: boolean): void
}>()
</script>

<template>
  <n-card v-if="scanResult" title="识别到的资源与模块" class="build-section-card">
    <div class="insight-panel">
      <div class="insight-head">
        <div>
          <n-text strong class="insight-title">{{ insightAppName }}</n-text>
          <n-text depth="3" class="insight-subtitle">{{ insightAppId }} · {{ insightVersionName }} / {{ insightVersionCode }}</n-text>
        </div>
        <n-tag :type="scanResult.isZip ? 'warning' : 'success'">{{ scanResult.isZip ? 'ZIP 导入' : '目录导入' }}</n-tag>
      </div>
      <n-grid :cols="4" :x-gap="12" :y-gap="12" responsive="screen" class="insight-grid">
        <n-gi>
          <div class="summary-tile">
            <n-text depth="3">已选模块</n-text>
            <n-text strong class="summary-value">{{ selectedManifestModules.length }} / {{ manifestModules.length }}</n-text>
          </div>
        </n-gi>
        <n-gi>
          <div class="summary-tile">
            <n-text depth="3">UTS 内置模块</n-text>
            <n-text strong class="summary-value">{{ scanResult.uts.builtinModules.length }}</n-text>
          </div>
        </n-gi>
        <n-gi>
          <div class="summary-tile">
            <n-text depth="3">UTS 自定义插件</n-text>
            <n-text strong class="summary-value">{{ scanResult.uts.customPlugins.length }}</n-text>
          </div>
        </n-gi>
        <n-gi>
          <div class="summary-tile">
            <n-text depth="3">远程依赖</n-text>
            <n-text strong class="summary-value">{{ utsDependencyCount }}</n-text>
          </div>
        </n-gi>
      </n-grid>

      <div class="module-grid">
        <div class="module-box module-box--manifest">
          <div class="module-box-head">
            <n-text strong>Manifest 模块</n-text>
            <n-text v-if="manifestModules.length" depth="3">{{ selectedManifestModules.length }} / {{ manifestModules.length }} 已选</n-text>
          </div>
          <div v-if="manifestModules.length" class="module-choice-grid">
            <n-checkbox
              v-for="mod in manifestModules"
              :key="manifestModuleKey(mod)"
              class="module-choice"
              :class="manifestModuleStatusClass(mod)"
              :checked="isManifestModuleSelected(mod)"
              :disabled="isBuildLocked"
              @update:checked="checked => emit('set-manifest-module-selected', mod, checked)"
            >
              <span class="module-choice-content">
                <span class="module-choice-row">
                  <span class="module-choice-main">{{ mod.name }}</span>
                  <span v-if="formatPlatforms(mod.platforms)" class="module-choice-platform">{{ formatPlatforms(mod.platforms) }}</span>
                  <n-tag size="tiny" :type="manifestModuleStatusType(mod)" :bordered="false">
                    {{ manifestModuleStatusLabel(mod) }}
                  </n-tag>
                </span>
                <span
                  v-for="summary in manifestModuleFieldSummaries(mod)"
                  :key="summary"
                  class="module-choice-config"
                >
                  {{ summary }}
                </span>
              </span>
            </n-checkbox>
          </div>
          <n-text v-else depth="3" class="module-empty">未声明 App 模块</n-text>
        </div>
        <div class="module-box">
          <n-text strong>UTS 插件</n-text>
          <n-space v-if="utsPluginLabels.length" wrap :size="8" class="tag-row">
            <n-tag v-for="label in utsPluginLabels" :key="label" type="success">{{ label }}</n-tag>
          </n-space>
          <n-text v-else depth="3" class="module-empty">未检测到 UTS 插件</n-text>
        </div>
      </div>
      <div class="path-summary">
        <n-text depth="3">manifest 路径</n-text>
        <n-text code>{{ insightManifestPath }}</n-text>
        <n-text depth="3">资源包根目录</n-text>
        <n-text code>{{ scanResult.importedPath }}</n-text>
        <n-text depth="3">应用资源目录</n-text>
        <n-text code>{{ scanResult.appResourcePath }}</n-text>
      </div>
    </div>

    <div v-if="scanResult.warnings.length" class="module-section">
      <n-alert v-for="warning in scanResult.warnings" :key="warning" type="warning">
        {{ warning }}
      </n-alert>
    </div>
    <div v-if="manifestReadWarning" class="module-section">
      <n-alert type="warning">{{ manifestReadWarning }}</n-alert>
    </div>
  </n-card>
</template>
