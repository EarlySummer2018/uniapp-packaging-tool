<script setup lang="ts">
import { NConfigProvider, NDialogProvider, NMessageProvider, darkTheme, type GlobalThemeOverrides } from 'naive-ui'
import { onMounted, onUnmounted, ref } from 'vue'

const isDark = ref(false)
const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#159766',
    primaryColorHover: '#18a871',
    primaryColorPressed: '#0f7f56',
    infoColor: '#1f6feb',
    successColor: '#159766',
    warningColor: '#c77700',
    errorColor: '#d92d20',
    borderRadius: '8px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif'
  }
}

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
  <n-config-provider :theme="isDark ? darkTheme : null" :theme-overrides="themeOverrides">
    <n-message-provider>
      <n-dialog-provider>
        <router-view />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
