<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NCard, NList, NListItem, NTag, NSpin, NIcon } from 'naive-ui'
import { CheckmarkCircle, CloseCircle } from '@vicons/ionicons5'

interface EnvItem {
  name: string
  status: 'installed' | 'not_installed' | 'checking'
  version?: string
  path?: string
}

const envItems = ref<EnvItem[]>([
  { name: 'Node.js', status: 'checking' },
  { name: 'Java JDK', status: 'checking' },
  { name: 'Android SDK', status: 'checking' },
  { name: 'Gradle', status: 'checking' },
  { name: 'Xcode', status: 'checking' },
  { name: 'HarmonyOS SDK', status: 'checking' }
])

const isChecking = ref(true)

onMounted(async () => {
  await checkEnvironments()
})

async function checkEnvironments() {
  for (let i = 0; i < envItems.value.length; i++) {
    await new Promise(resolve => setTimeout(resolve, 300))
    
    const item = envItems.value[i]
    switch (item.name) {
      case 'Node.js':
        item.status = 'installed'
        item.version = 'v20.10.0'
        break
      case 'Java JDK':
        item.status = 'installed'
        item.version = 'JDK 17'
        break
      case 'Android SDK':
        item.status = Math.random() > 0.3 ? 'installed' : 'not_installed'
        item.version = item.status === 'installed' ? '34.0.0' : undefined
        break
      case 'Gradle':
        item.status = 'installed'
        item.version = '8.4'
        break
      case 'Xcode':
        item.status = Math.random() > 0.7 ? 'installed' : 'not_installed'
        item.version = item.status === 'installed' ? '15.2' : undefined
        break
      case 'HarmonyOS SDK':
        item.status = Math.random() > 0.6 ? 'installed' : 'not_installed'
        item.version = item.status === 'installed' ? 'API 12' : undefined
        break
    }
  }
  
  isChecking.value = false
}

function getStatusType(status: EnvItem['status']) {
  switch (status) {
    case 'installed': return 'success'
    case 'not_installed': return 'error'
    default: return 'default'
  }
}

function getStatusText(status: EnvItem['status']) {
  switch (status) {
    case 'installed': return '已安装'
    case 'not_installed': return '未安装'
    default: return '检测中...'
  }
}
</script>

<template>
  <n-card title="环境检测" size="small">
    <template #header-extra>
      <n-spin v-if="isChecking" size="small" />
    </template>
    
    <n-list bordered>
      <n-list-item
        v-for="item in envItems"
        :key="item.name"
      >
        <div class="env-item">
          <span class="env-name">{{ item.name }}</span>
          <n-tag :type="getStatusType(item.status)" size="small" round>
            {{ getStatusText(item.status) }}
          </n-tag>
          <span v-if="item.version" class="env-version">
            {{ item.version }}
          </span>
          <n-icon 
            v-if="item.status === 'installed'" 
            color="#18a058" 
            size="16"
          >
            <CheckmarkCircle />
          </n-icon>
          <n-icon 
            v-else-if="item.status === 'not_installed'" 
            color="#d03050" 
            size="16"
          >
            <CloseCircle />
          </n-icon>
        </div>
      </n-list-item>
    </n-list>
  </n-card>
</template>

<style scoped>
.env-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
}

.env-name {
  flex: 1;
  font-weight: 500;
}

.env-version {
  color: #666;
  font-size: 12px;
}
</style>
