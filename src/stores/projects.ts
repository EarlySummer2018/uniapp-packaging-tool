import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AppConfig {
  name: string
  appId: string
  version: string
  versionCode: number
  icon1024: string
}

export interface AndroidKeystoreConfig {
  path: string
  alias: string
  hasStorePassword: boolean
  hasKeyPassword: boolean
}

export interface AndroidConfig {
  enabled: boolean
  dcloudAppKey: string
  packageName: string
  minSdkVersion: number
  targetSdkVersion: number
  compileSdkVersion: number
  keystore: AndroidKeystoreConfig
}

export interface IosConfig {
  enabled: boolean
  dcloudAppKey: string
  bundleId: string
  teamId: string
  provisioningProfile: string
  certificate: string
  exportMethod: string
  hasCertificatePassword: boolean
}

export interface HarmonySigningConfig {
  storeFile: string
  keyAlias: string
  hasStorePassword: boolean
  hasKeyPassword: boolean
}

export interface HarmonyConfig {
  enabled: boolean
  bundleName: string
  runtimeVersion: string
  signingConfig: HarmonySigningConfig
}

export interface Project {
  id: string
  name: string
  description: string
  localPath: string
  app: AppConfig
  android: AndroidConfig
  ios: IosConfig
  harmony: HarmonyConfig
  androidModuleConfig: Record<string, string>
  outputDir: string
  createdAt: string
  updatedAt: string
}

export const useProjectsStore = defineStore('projects', () => {
  const projects = ref<Project[]>([])
  const currentProjectId = ref<string | null>(null)
  const loading = ref(false)

  const currentProject = computed(() => {
    if (!currentProjectId.value) return null
    return projects.value.find(p => p.id === currentProjectId.value) || null
  })

  async function initStore() {
    await loadProjects()
  }

  async function loadProjects() {
    loading.value = true
    try {
      projects.value = await invoke<Project[]>('list_projects')
      if (!currentProjectId.value && projects.value.length > 0) {
        currentProjectId.value = projects.value[0].id
      }
    } finally {
      loading.value = false
    }
  }

  async function getProject(id: string) {
    currentProjectId.value = id
    const project = await invoke<Project>('get_project', { projectId: id })
    const index = projects.value.findIndex(p => p.id === id)
    if (index >= 0) projects.value[index] = project
    else projects.value.push(project)
    return project
  }

  async function createProject(data: { name: string; description?: string }) {
    const project = await invoke<Project>('create_project', {
      name: data.name,
      description: data.description || '',
      config: null
    })
    projects.value.unshift(project)
    currentProjectId.value = project.id
    return project
  }

  async function saveProject(project: Project) {
    const saved = await invoke<Project>('save_project_config', {
      projectId: project.id,
      config: project
    })
    const index = projects.value.findIndex(p => p.id === saved.id)
    if (index >= 0) projects.value[index] = saved
    else projects.value.unshift(saved)
    return saved
  }

  async function updateProject(id: string, updates: Partial<Project>) {
    const updated = await invoke<Project>('update_project', {
      projectId: id,
      updates
    })
    const index = projects.value.findIndex(p => p.id === id)
    if (index >= 0) projects.value[index] = updated
    return updated
  }

  async function deleteProject(id: string) {
    await invoke('delete_project', { projectId: id })
    projects.value = projects.value.filter(p => p.id !== id)
    if (currentProjectId.value === id) {
      currentProjectId.value = projects.value[0]?.id || null
    }
  }

  function setCurrentProject(id: string) {
    currentProjectId.value = id
  }

  function getCurrentProject() {
    return currentProject.value
  }

  return {
    projects,
    currentProjectId,
    currentProject,
    loading,
    initStore,
    loadProjects,
    getProject,
    createProject,
    saveProject,
    updateProject,
    deleteProject,
    setCurrentProject,
    getCurrentProject
  }
})
