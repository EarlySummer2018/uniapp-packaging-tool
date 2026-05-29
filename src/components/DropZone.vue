<script setup lang="ts">
import { ref } from 'vue'
import { NUpload, NIcon, NText } from 'naive-ui'
import { CloudUploadOutline } from '@vicons/ionicons5'

interface Props {
  accept?: string
  multiple?: boolean
  maxSize?: number
}

withDefaults(defineProps<Props>(), {
  accept: '*',
  multiple: false,
  maxSize: 100
})

const emit = defineEmits<{
  (e: 'files-selected', files: File[]): void
  (e: 'drop'): void
}>()

const isDragging = ref(false)

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  isDragging.value = true
}

function handleDragLeave() {
  isDragging.value = false
}

async function handleDrop(event: DragEvent) {
  event.preventDefault()
  isDragging.value = false
  
  if (event.dataTransfer?.files) {
    const files = Array.from(event.dataTransfer.files)
    emit('files-selected', files)
    emit('drop')
  }
}

function handleFileSelect({ file }: { file: any }) {
  if (file.file) {
    emit('files-selected', [file.file])
  }
}
</script>

<template>
  <div
    class="drop-zone"
    :class="{ 'is-dragging': isDragging }"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <n-upload
      :accept="accept"
      :multiple="multiple"
      :max="maxSize"
      :show-file-list="false"
      @change="handleFileSelect"
    >
      <div class="drop-zone-content">
        <n-icon size="48" color="#18a058">
          <CloudUploadOutline />
        </n-icon>
        <n-text class="drop-zone-text" depth="3">
          拖拽文件到此处，或点击选择文件
        </n-text>
        <n-text class="drop-zone-hint" depth="3" style="font-size: 12px;">
          支持 ZIP、APK、IPA 等格式，最大 {{ maxSize }}MB
        </n-text>
      </div>
    </n-upload>
  </div>
</template>

<style scoped>
.drop-zone {
  border: 2px dashed #d9d9d9;
  border-radius: 8px;
  padding: 40px 20px;
  text-align: center;
  transition: all 0.3s ease;
  cursor: pointer;
  background: #fafafa;
}

.drop-zone:hover {
  border-color: #18a058;
  background: #f0f9eb;
}

.drop-zone.is-dragging {
  border-color: #18a058;
  background: #dcffe4;
  transform: scale(1.02);
}

.drop-zone-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.drop-zone-text {
  font-size: 16px;
  font-weight: 500;
}

.drop-zone-hint {
  margin-top: 8px;
}
</style>
