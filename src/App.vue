<script setup lang="ts">
import { NConfigProvider, NDialogProvider, NMessageProvider, darkTheme } from 'naive-ui'
import { onMounted, onUnmounted, ref } from 'vue'

const isDark = ref(false)

function isEditableTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false
  if (target.isContentEditable) return true
  const tagName = target.tagName.toLowerCase()
  return tagName === 'input' || tagName === 'textarea' || tagName === 'select'
}

function handleGlobalSelectAll(event: KeyboardEvent) {
  if (event.key.toLowerCase() !== 'a') return
  if (!event.metaKey && !event.ctrlKey) return
  if (event.altKey) return
  if (isEditableTarget(event.target)) return

  event.preventDefault()
  window.getSelection()?.removeAllRanges()
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalSelectAll)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalSelectAll)
})
</script>

<template>
  <n-config-provider :theme="isDark ? darkTheme : null">
    <n-message-provider>
      <n-dialog-provider>
        <router-view />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
