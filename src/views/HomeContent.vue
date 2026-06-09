<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NButton, NIcon, NSpace, NText, NEmpty, NTag, useMessage } from 'naive-ui'
import {
  AddOutline,
  FolderOpenOutline,
  LogoAndroid,
  LogoApple,
  PhonePortraitOutline
} from '@vicons/ionicons5'
import { useProjectsStore } from '../stores/projects'
import { useBuildStore } from '../stores/build'

const emit = defineEmits<{
  (e: 'create-project'): void
}>()
const router = useRouter()
const projectsStore = useProjectsStore()
const buildStore = useBuildStore()
const message = useMessage()
const recentProjects = computed(() => projectsStore.projects.slice(0, 5))
const platformStats = computed(() => [
  {
    key: 'android',
    label: 'Android',
    value: projectsStore.projects.filter(project => project.android?.enabled).length,
    icon: LogoAndroid,
    color: '#159766'
  },
  {
    key: 'ios',
    label: 'iOS',
    value: projectsStore.projects.filter(project => project.ios?.enabled).length,
    icon: LogoApple,
    color: '#1f6feb'
  },
  {
    key: 'harmony',
    label: '鸿蒙',
    value: projectsStore.projects.filter(project => project.harmony?.enabled).length,
    icon: PhonePortraitOutline,
    color: '#c77700'
  }
])

function handleGoToBuild(projectId: string) {
  if (buildStore.hasActiveBuilds) {
    message.warning('已有构建任务进行中，请等待完成后再开始构建')
    return
  }
  router.push(`/build/${projectId}`)
}

function handleCreateProject() {
  if (buildStore.hasActiveBuilds) {
    message.warning('已有构建任务进行中，暂不能新建项目')
    return
  }
  emit('create-project')
}

function handleGoToConfig(projectId: string) {
  router.push(`/project/${projectId}`)
}
</script>

<template>
  <div class="home-content">
    <div class="dashboard-header">
      <div>
        <n-text class="page-title">项目列表</n-text>
        <n-text class="page-subtitle">共 {{ projectsStore.projects.length }} 个项目</n-text>
      </div>
      <div class="dashboard-actions">
        <n-button data-guide="create-project" type="primary" :disabled="buildStore.hasActiveBuilds" @click="handleCreateProject">
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          新建项目
        </n-button>
      </div>
    </div>

    <div data-guide="project-overview" class="overview-grid">
      <div class="overview-tile primary">
        <n-text depth="3">项目总数</n-text>
        <n-text strong class="overview-value">{{ projectsStore.projects.length }}</n-text>
      </div>
      <div
        v-for="stat in platformStats"
        :key="stat.key"
        class="overview-tile"
      >
        <div class="overview-icon" :style="{ color: stat.color }">
          <n-icon :size="18"><component :is="stat.icon" /></n-icon>
        </div>
        <n-text depth="3">{{ stat.label }}</n-text>
        <n-text strong class="overview-value">{{ stat.value }}</n-text>
      </div>
    </div>

    <div data-guide="recent-projects" class="projects-section">
      <div class="section-heading">
        <n-text class="section-title">最近项目</n-text>
      </div>

      <div v-if="recentProjects.length > 0" class="project-cards">
        <NCard
          v-for="project in recentProjects"
          :key="project.id"
          class="project-card"
          hoverable
        >
          <div class="project-card-header">
            <n-icon size="20" color="#18a058"><FolderOpenOutline /></n-icon>
            <n-text strong class="project-name">{{ project.name }}</n-text>
          </div>
          <n-text depth="3" class="project-description">
            {{ project.description || '暂无描述' }}
          </n-text>
          <div class="project-card-meta">
            <n-text depth="3" class="project-date">
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
          <div class="project-card-actions">
            <n-button size="small" @click="handleGoToConfig(project.id)">
              配置
            </n-button>
            <n-button size="small" type="primary" :disabled="buildStore.hasActiveBuilds" @click="handleGoToBuild(project.id)">
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
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding-bottom: 18px;
  border-bottom: 1px solid var(--border-soft);
}

.dashboard-actions {
  flex-shrink: 0;
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 14px;
}

.overview-tile {
  min-height: 92px;
  padding: 16px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-color);
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.overview-tile.primary {
  background: var(--primary-soft);
  border-color: rgba(21, 151, 102, 0.22);
}

.overview-icon {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: #fff;
}

.overview-value {
  font-size: 26px;
  line-height: 1;
}

.project-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.project-card {
  min-height: 176px;
}

.project-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.project-name {
  min-width: 0;
  font-size: 16px;
}

.project-description {
  display: block;
  min-height: 40px;
  margin-top: 8px;
  line-height: 1.55;
}

.project-card-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-top: 14px;
}

.project-date {
  flex-shrink: 0;
  font-size: 12px;
}

.project-card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}

@media (max-width: 1180px) {
  .overview-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 720px) {
  .dashboard-header,
  .project-card-meta {
    align-items: flex-start;
    flex-direction: column;
  }

  .overview-grid {
    grid-template-columns: 1fr;
  }
}
</style>
