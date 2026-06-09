<script setup lang="ts">
import { h, ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NAlert, NLayout, NLayoutSider, NLayoutContent, NMenu, NButton, NIcon, NModal, NInput, NSpace, NText, useMessage, useDialog } from 'naive-ui'
import { AddOutline, FolderOutline, SettingsOutline, TimeOutline, TrashOutline, OptionsOutline } from '@vicons/ionicons5'
import { useProjectsStore } from '../stores/projects'
import { useBuildStore } from '../stores/build'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const dialog = useDialog()
const projectsStore = useProjectsStore()
const buildStore = useBuildStore()

const showModal = ref(false)
const newProjectName = ref('')
const newProjectDesc = ref('')
const deletingProjectId = ref<string | null>(null)

const menuOptions = [
  {
    label: '项目列表',
    key: 'Home' as string,
    icon: () => h(NIcon, null, { default: () => h(FolderOutline) })
  },
  {
    label: 'SDK & 环境管理',
    key: 'SdkManager' as string,
    icon: () => h(NIcon, null, { default: () => h(OptionsOutline) })
  },
  {
    label: '打包历史',
    key: 'BuildHistory' as string,
    icon: () => h(NIcon, null, { default: () => h(TimeOutline) })
  },
  {
    label: '设置',
    key: 'Settings' as string,
    icon: () => h(NIcon, null, { default: () => h(SettingsOutline) })
  }
]

const currentMenuKey = computed(() => (route.name as string) || 'Home')

onMounted(async () => {
  await projectsStore.initStore()
})

function handleMenuSelect(key: string) {
  const routeMap: Record<string, string> = {
    Home: '/',
    SdkManager: '/sdk-manager',
    BuildHistory: '/history',
    Settings: '/settings'
  }
  router.push(routeMap[key] || '/')
}

function handleOpenCreateProject() {
  if (buildStore.hasActiveBuilds) {
    message.warning('已有构建任务进行中，暂不能新建项目')
    return
  }
  showModal.value = true
}

async function handleCreateProject() {
  if (buildStore.hasActiveBuilds) {
    message.warning('已有构建任务进行中，暂不能新建项目')
    return
  }
  if (!newProjectName.value.trim()) return

  const project = await projectsStore.createProject({
    name: newProjectName.value,
    description: newProjectDesc.value
  })

  if (project) {
    showModal.value = false
    newProjectName.value = ''
    newProjectDesc.value = ''
    router.push(`/project/${project.id}`)
  }
}

async function handleDeleteProject(projectId: string) {
  if (buildStore.hasActiveBuilds) {
    message.warning('已有构建任务进行中，暂不能删除项目')
    return
  }
  const project = projectsStore.projects.find(p => p.id === projectId)
  if (!project) return

  dialog.warning({
    title: '确认删除',
    content: `确定要删除项目「${project.name || projectId}」吗？\n此操作会删除该项目配置、导入资源、日志和构建工作区，且不可恢复。`,
    positiveText: '确认删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      deletingProjectId.value = projectId
      try {
        await projectsStore.deleteProject(projectId)
        message.success('项目已删除')
        if (route.name !== 'Home' && route.params.id === projectId) {
          router.push('/')
        }
      } catch (e: any) {
        message.error(String(e))
      } finally {
        deletingProjectId.value = null
      }
    }
  })
}
</script>

<template>
  <n-layout class="app-layout" has-sider>
    <n-layout-sider
      bordered
      :width="280"
      :native-scrollbar="false"
      class="sidebar"
    >
      <div class="sidebar-header">
        <div class="brand-row">
          <div class="brand-mark">UP</div>
          <div class="brand-copy">
            <n-text strong class="app-title">UniPack</n-text>
            <n-text depth="3" class="app-caption">Tool</n-text>
          </div>
        </div>
        <n-button
          type="primary"
          size="small"
          :disabled="buildStore.hasActiveBuilds"
          @click="handleOpenCreateProject"
        >
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          新建项目
        </n-button>
      </div>

      <n-menu
        :options="menuOptions"
        :value="currentMenuKey"
        @update:value="handleMenuSelect"
      />

      <div class="project-list">
        <n-text depth="3" class="sidebar-section-title">我的项目</n-text>

        <div v-if="projectsStore.projects.length > 0" class="projects">
          <div
            v-for="project in projectsStore.projects"
            :key="project.id"
            class="project-item"
            :class="{ active: project.id === projectsStore.currentProjectId }"
            @click="projectsStore.setCurrentProject(project.id); router.push(`/project/${project.id}`)"
          >
            <n-icon size="20"><FolderOutline /></n-icon>
            <div class="project-info">
              <n-text strong ellipsis>{{ project.name }}</n-text>
              <n-text depth="3" class="project-desc">{{ project.description }}</n-text>
            </div>
            <n-button
              quaternary
              circle
              size="tiny"
              type="error"
              :loading="deletingProjectId === project.id"
              :disabled="buildStore.hasActiveBuilds"
              @click.stop="handleDeleteProject(project.id)"
            >
              <template #icon><n-icon><TrashOutline /></n-icon></template>
            </n-button>
          </div>
        </div>

        <n-text v-else depth="3" class="project-empty">
          暂无项目，点击上方按钮创建
        </n-text>
      </div>
    </n-layout-sider>

    <n-layout-content class="main-content">
      <div class="page-container">
        <n-alert v-if="buildStore.hasActiveBuilds" type="warning" class="build-lock-alert">
          当前有构建任务进行中，新建、删除、配置保存和再次构建暂不可用。
        </n-alert>
        <router-view @create-project="handleOpenCreateProject" />
      </div>
    </n-layout-content>

    <n-modal
      v-model:show="showModal"
      preset="card"
      title="新建项目"
      style="width: 500px;"
    >
      <n-space vertical :size="16">
        <div class="modal-field">
          <n-text strong>项目名称</n-text>
          <n-input
            v-model:value="newProjectName"
            placeholder="请输入项目名称"
          />
        </div>

        <div class="modal-field">
          <n-text strong>项目描述</n-text>
          <n-input
            v-model:value="newProjectDesc"
            type="textarea"
            placeholder="请输入项目描述（可选）"
            :rows="3"
          />
        </div>
      </n-space>

      <template #footer>
        <n-space justify="end">
          <n-button @click="showModal = false">取消</n-button>
          <n-button type="primary" :disabled="buildStore.hasActiveBuilds" @click="handleCreateProject">创建</n-button>
        </n-space>
      </template>
    </n-modal>
  </n-layout>
</template>

<style scoped>
.app-layout {
  height: 100vh;
}

.sidebar {
  background: var(--surface-color);
  overflow-x: hidden;
  overflow-y: auto;
}

.sidebar-header {
  padding: 18px 18px 16px;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 14px;
  border-bottom: 1px solid var(--border-soft);
}

.brand-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-mark {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  background: var(--primary-color);
  font-size: 13px;
  font-weight: 750;
}

.brand-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.app-title {
  font-size: 17px;
}

.app-caption {
  font-size: 12px;
}

.project-list {
  padding: 18px 14px;
  flex: 1;
  overflow-y: auto;
}

.sidebar-section-title {
  font-size: 12px;
  letter-spacing: 0;
  margin-bottom: 12px;
  display: block;
  font-weight: 650;
}

.projects {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.project-item {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 54px;
  padding: 10px 10px;
  border-radius: 8px;
  cursor: pointer;
  border: 1px solid transparent;
  transition: background 0.18s ease, border-color 0.18s ease, box-shadow 0.18s ease;
}

.project-item:hover {
  background: var(--surface-muted);
  border-color: var(--border-soft);
}

.project-item.active {
  background: var(--primary-soft);
  border-color: rgba(21, 151, 102, 0.28);
  box-shadow: inset 3px 0 0 var(--primary-color);
}

.project-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.project-desc {
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.main-content {
  background: var(--bg-color);
}

.page-container {
  max-width: 1420px;
  margin: 0 auto;
}

.build-lock-alert {
  margin-bottom: 16px;
}

.project-empty {
  display: block;
  padding: 12px 10px;
  font-size: 13px;
}

.modal-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

@media (max-width: 760px) {
  .app-layout {
    display: block !important;
    height: auto;
    min-height: 100vh;
    overflow: visible;
  }

  .app-layout :deep(> .n-layout-scroll-container) {
    display: block !important;
    flex-flow: column !important;
    width: 100% !important;
  }

  .sidebar {
    width: 100% !important;
    max-width: 100% !important;
    min-width: 0 !important;
    flex-basis: auto !important;
    height: auto !important;
    min-height: 0 !important;
    border-right: 0;
    border-bottom: 1px solid var(--border-soft);
  }

  .sidebar :deep(.n-scrollbar) {
    min-width: 0 !important;
  }

  .sidebar-header {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }

  .sidebar-header :deep(.n-button) {
    flex-shrink: 0;
  }

  .project-list {
    display: none;
  }

  .main-content {
    width: 100%;
    flex: none;
    overflow: visible;
  }

  .page-container {
    padding: 16px;
  }
}
</style>
