<script setup lang="ts">
import { NButton, NGi, NGrid, NIcon, NSpace, NText } from 'naive-ui'
import { ArrowBackOutline } from '@vicons/ionicons5'
import AndroidModuleConfigPanel from './build-center/AndroidModuleConfigPanel.vue'
import BuildLogCard from './build-center/BuildLogCard.vue'
import IosOfflineSdkPanel from './build-center/IosOfflineSdkPanel.vue'
import IosPrivacyDescriptionModal from './build-center/IosPrivacyDescriptionModal.vue'
import PlatformSelectCard from './build-center/PlatformSelectCard.vue'
import ResourceImportCard from './build-center/ResourceImportCard.vue'
import { platforms } from './build-center/platforms'
import { useBuildCenterController } from './build-center/useBuildCenterController'

const {
  currentProject, getProjectName, goBack, selectedPlatforms, importing, isBuildLocked, scanResult,
  insightAppId, insightVersionName, insightVersionCode, insightManifestPath, manifestReadWarning, chooseResource,
  buildDisabledReason, canBuild, canGenerateAndroid, canGenerateIos, canGenerateHarmony, packageBuildLoading,
  androidGenerateLoading, iosGenerateLoading, harmonyGenerateLoading, singleSelectedPlatform, togglePlatform,
  buildExecutionModes, buildExecutionModeOptions, buildExecutionModeHints, updateBuildExecutionMode,
  generateAndroidProject, generateIosProject, generateHarmonyProject, startBuild, iosMissingRequired, iosIconCount,
  iosPrivacyDescriptionCount, iosPrivacyDescriptionItems, iosPrivacyDescriptionMissingCount, iosModuleSummaryLabel,
  iosConfigurableModules, selectedManifestModules, iosModuleConfigLoading, latestManifestInfo, iosModuleMissingRequired,
  activeIosConfigModuleKey, activeIosConfigModule, iosConfigModuleStatusType, iosConfigModuleStatusLabel, iosFieldValue,
  iosFieldStatusType, iosFieldStatusLabel, openIosPrivacyDescriptionDialog, openIosConfigModule, updateActiveIosField,
  androidModuleConfigLoading, androidConfigurableModules, androidMissingRequired, activeAndroidConfigModuleKey,
  activeAndroidConfigModule, androidConfigModuleStatusType, configModuleStatusLabel, androidFieldValue, fieldStatusType,
  fieldStatusLabel, formatFileSize, openAndroidConfigModule, updateActiveAndroidField, pickAndroidFileField,
  clearAndroidFileField, currentBuild, visibleArtifacts, currentGeneratedProjectPath, currentGeneratedProjectLabel,
  openGeneratedProject, iosPrivacyDialogVisible, updateIosPrivacyDescription
} = useBuildCenterController()
</script>

<template>
  <div class="build-center">
    <div class="page-header">
      <n-space align="center">
        <n-button quaternary circle @click="goBack">
          <template #icon><n-icon><ArrowBackOutline /></n-icon></template>
        </n-button>
        <div>
          <n-text strong class="page-title">构建中心</n-text>
          <n-text v-if="currentProject" depth="3" class="page-subtitle">{{ getProjectName() }}</n-text>
        </div>
      </n-space>
    </div>
    <n-grid cols="1 s:1 m:3" :x-gap="18" :y-gap="18" responsive="screen" class="build-grid">
      <n-gi span="1 m:1">
        <ResourceImportCard
          :importing="importing"
          :is-build-locked="isBuildLocked"
          :scan-result="scanResult"
          :insight-app-id="insightAppId"
          :insight-version-name="insightVersionName"
          :insight-version-code="insightVersionCode"
          :insight-manifest-path="insightManifestPath"
          :manifest-read-warning="manifestReadWarning"
          @choose-resource="chooseResource"
        />
      </n-gi>
      <n-gi span="2 m:2">
        <PlatformSelectCard
          :platforms="platforms"
          :selected-platforms="selectedPlatforms"
          :is-build-locked="isBuildLocked"
          :build-disabled-reason="buildDisabledReason"
          :can-build="canBuild"
          :can-generate-android="canGenerateAndroid"
          :can-generate-ios="canGenerateIos"
          :can-generate-harmony="canGenerateHarmony"
          :package-build-loading="packageBuildLoading"
          :android-generate-loading="androidGenerateLoading"
          :ios-generate-loading="iosGenerateLoading"
          :harmony-generate-loading="harmonyGenerateLoading"
          :single-selected-platform="singleSelectedPlatform"
          :build-execution-modes="buildExecutionModes"
          :build-execution-mode-options="buildExecutionModeOptions"
          :build-execution-mode-hints="buildExecutionModeHints"
          @toggle-platform="togglePlatform"
          @update-build-mode="updateBuildExecutionMode"
          @generate-android="generateAndroidProject"
          @generate-ios="generateIosProject"
          @generate-harmony="generateHarmonyProject"
          @start-build="startBuild"
        />
      </n-gi>
    </n-grid>
    <IosOfflineSdkPanel
      :visible="!!scanResult && selectedPlatforms.includes('ios')"
      :ios-missing-required="iosMissingRequired"
      :bundle-id="currentProject?.ios.bundleId || '-'"
      :team-id="currentProject?.ios.teamId || '-'"
      :ios-icon-count="iosIconCount"
      :ios-privacy-description-count="iosPrivacyDescriptionCount"
      :ios-privacy-description-item-count="iosPrivacyDescriptionItems.length"
      :ios-privacy-description-missing-count="iosPrivacyDescriptionMissingCount"
      :insight-app-id="insightAppId"
      :ios-module-summary-label="iosModuleSummaryLabel"
      :ios-configurable-modules="iosConfigurableModules"
      :selected-manifest-module-count="selectedManifestModules.length"
      :ios-module-config-loading="iosModuleConfigLoading"
      :latest-manifest-info="latestManifestInfo"
      :manifest-read-warning="manifestReadWarning"
      :ios-module-missing-required-count="iosModuleMissingRequired.length"
      :active-ios-config-module-key="activeIosConfigModuleKey"
      :active-ios-config-module="activeIosConfigModule"
      :is-build-locked="isBuildLocked"
      :ios-config-module-status-type="iosConfigModuleStatusType"
      :ios-config-module-status-label="iosConfigModuleStatusLabel"
      :ios-field-value="iosFieldValue"
      :ios-field-status-type="iosFieldStatusType"
      :ios-field-status-label="iosFieldStatusLabel"
      @edit-privacy="openIosPrivacyDescriptionDialog"
      @open-module="openIosConfigModule"
      @update-field="updateActiveIosField"
    />

    <AndroidModuleConfigPanel
      :visible="!!scanResult && selectedPlatforms.includes('android')"
      :android-module-config-loading="androidModuleConfigLoading"
      :latest-manifest-info="latestManifestInfo"
      :manifest-read-warning="manifestReadWarning"
      :android-configurable-modules="androidConfigurableModules"
      :selected-manifest-module-count="selectedManifestModules.length"
      :android-missing-required-count="androidMissingRequired.length"
      :active-android-config-module-key="activeAndroidConfigModuleKey"
      :active-android-config-module="activeAndroidConfigModule"
      :is-build-locked="isBuildLocked"
      :android-config-module-status-type="androidConfigModuleStatusType"
      :config-module-status-label="configModuleStatusLabel"
      :android-field-value="androidFieldValue"
      :field-status-type="fieldStatusType"
      :field-status-label="fieldStatusLabel"
      :format-file-size="formatFileSize"
      @open-module="openAndroidConfigModule"
      @update-field="updateActiveAndroidField"
      @pick-file-field="pickAndroidFileField"
      @clear-file-field="clearAndroidFileField"
    />

    <BuildLogCard
      :logs="currentBuild?.logs || []"
      :progress="currentBuild?.progress || 0"
      :status="currentBuild?.status"
      :visible-artifacts="visibleArtifacts"
      :current-generated-project-path="currentGeneratedProjectPath"
      :current-generated-project-label="currentGeneratedProjectLabel"
      @open-generated-project="openGeneratedProject"
    />

    <IosPrivacyDescriptionModal
      v-model:show="iosPrivacyDialogVisible"
      :items="iosPrivacyDescriptionItems"
      :missing-count="iosPrivacyDescriptionMissingCount"
      :is-build-locked="isBuildLocked"
      @update-item="updateIosPrivacyDescription"
    />
  </div>
</template>

<style src="./build-center/build-center.css"></style>
