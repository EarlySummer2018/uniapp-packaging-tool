<script setup lang="ts">
import { h, ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NLayout, NLayoutSider, NLayoutContent, NMenu, NButton, NIcon, NModal, NInput, NSpace, NText, useMessage, useDialog } from 'naive-ui'
import { AddOutline, FolderOutline, SettingsOutline, TimeOutline, TrashOutline, OptionsOutline } from '@vicons/ionicons5'
import { useProjectsStore } from '../stores/projects'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const dialog = useDialog()
const projectsStore = useProjectsStore()

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

async function handleCreateProject() {
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
        <n-text strong class="app-title">UniPack Tool</n-text>
        <n-button
          type="primary"
          size="small"
          round
          @click="showModal = true"
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
        <n-text depth="3" class="section-title">我的项目</n-text>

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
              @click.stop="handleDeleteProject(project.id)"
            >
              <template #icon><n-icon><TrashOutline /></n-icon></template>
            </n-button>
          </div>
        </div>

        <n-text v-else depth="3" style="padding: 12px; display: block; font-size: 13px;">
          暂无项目，点击上方按钮创建
        </n-text>
      </div>
    </n-layout-sider>

    <n-layout-content class="main-content">
      <div class="page-container">
        <router-view @create-project="showModal = true" />
      </div>
    </n-layout-content>

    <n-modal
      v-model:show="showModal"
      preset="card"
      title="新建项目"
      style="width: 500px;"
    >
      <n-space vertical :size="16">
        <div>
          <n-text strong>项目名称</n-text>
          <n-input
            v-model:value="newProjectName"
            placeholder="请输入项目名称"
            style="margin-top: 8px;"
          />
        </div>

        <div>
          <n-text strong>项目描述</n-text>
          <n-input
            v-model:value="newProjectDesc"
            type="textarea"
            placeholder="请输入项目描述（可选）"
            :rows="3"
            style="margin-top: 8px;"
          />
        </div>
      </n-space>

      <template #footer>
        <n-space justify="end">
          <n-button @click="showModal = false">取消</n-button>
          <n-button type="primary" @click="handleCreateProject">创建</n-button>
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
  background: #fff;
  overflow-x: hidden;
  overflow-y: auto;
}

.sidebar-header {
  padding: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #f0f0f0;
}

.app-title {
  font-size: 18px;
}

.project-list {
  padding: 16px;
  flex: 1;
  overflow-y: auto;
}

.section-title {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 12px;
  display: block;
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
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.project-item:hover {
  background: #f5f7fa;
}

.project-item.active {
  background: #e6f7ff;
  border-left: 3px solid #18a058;
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
  background: #f5f7fa;
}

.page-container {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}
</style>
