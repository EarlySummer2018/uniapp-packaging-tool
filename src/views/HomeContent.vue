<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NCard, NButton, NIcon, NSpace, NText, NEmpty, NTag } from 'naive-ui'
import {
  AddOutline,
  FolderOpenOutline
} from '@vicons/ionicons5'
import { useProjectsStore } from '../stores/projects'

const router = useRouter()
const projectsStore = useProjectsStore()

function handleGoToBuild(projectId: string) {
  router.push(`/build/${projectId}`)
}

function handleGoToConfig(projectId: string) {
  router.push(`/project/${projectId}`)
}
</script>

<template>
  <div class="home-content">
    <div class="welcome-banner">
      <div class="welcome-text">
        <n-text class="welcome-title" style="font-size: 28px; font-weight: 700;">
          UniPack Tool
        </n-text>
        <n-text depth="3" style="font-size: 15px; margin-top: 8px;color: #fff;">
          UniApp 离线打包自动化工具 — Android / iOS / 鸿蒙 一键构建
        </n-text>
      </div>
      <div class="welcome-actions">
        <n-button type="primary" size="large" @click="$emit('create-project')">
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          新建项目
        </n-button>
      </div>
    </div>

    <div class="projects-section" style="margin-top: 32px;">
      <NSpace align="center" justify="space-between" style="margin-bottom: 16px;">
        <n-text strong style="font-size: 18px;">最近项目</n-text>
      </NSpace>

      <div v-if="projectsStore.projects.length > 0" class="project-cards">
        <NCard
          v-for="project in projectsStore.projects.slice(0, 5)"
          :key="project.id"
          class="project-card"
          hoverable
        >
          <div class="project-card-header">
            <n-icon size="20" color="#18a058"><FolderOpenOutline /></n-icon>
            <n-text strong style="font-size: 16px;">{{ project.name }}</n-text>
          </div>
          <n-text depth="3" style="margin-top: 4px; display: block;">
            {{ project.description || '暂无描述' }}
          </n-text>
          <div class="project-card-meta">
            <n-text depth="3" style="font-size: 12px;">
              创建于 {{ new Date(project.createdAt).toLocaleDateString() }}
            </n-text>
            <NSpace :size="8">
              <n-tag v-if="project.android?.enabled" size="small" :bordered="false" type="success">
                Android
              </n-tag>
              <n-tag v-if="project.ios?.enabled" size="small" :bordered="false" type="info">
                iOS
              </n-tag>
              <n-tag v-if="project.harmony?.enabled" size="small" :bordered="false" type="warning">
                鸿蒙
              </n-tag>
            </NSpace>
          </div>
          <div class="project-card-actions" style="margin-top: 12px;">
            <n-button size="small" @click="handleGoToConfig(project.id)">
              配置
            </n-button>
            <n-button size="small" type="primary" @click="handleGoToBuild(project.id)">
              构建
            </n-button>
          </div>
        </NCard>
      </div>

      <NEmpty v-else description="暂无项目，点击上方新建按钮创建第一个项目" size="large" />
    </div>
  </div>
</template>

<style scoped>
.home-content {
  max-width: 1200px;
}

.welcome-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 32px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  color: #fff;
}

.welcome-title {
  color: #fff !important;
}

.welcome-text {
  display: flex;
  flex-direction: column;
}

.quick-action-card {
  cursor: pointer;
}

.card-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.card-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.project-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 16px;
}

.project-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.project-card-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
}

.project-card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
