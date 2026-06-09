<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { NButton, NIcon } from 'naive-ui'
import {
  ArrowBackOutline,
  ArrowForwardOutline,
  CheckmarkOutline
} from '@vicons/ionicons5'

interface GuideStep {
  target: string
  title: string
  description: string
}

const props = defineProps<{
  steps: GuideStep[]
  guideKey: string
}>()

const visible = ref(false)
const activeIndex = ref(0)
const targetRect = ref<DOMRect | null>(null)
const guideCardRef = ref<HTMLElement | null>(null)

let autoStartTimer: ReturnType<typeof setTimeout> | null = null
let targetRefreshTimer: ReturnType<typeof setTimeout> | null = null

const currentStep = computed(() => props.steps[activeIndex.value] || null)
const isLastStep = computed(() => activeIndex.value === props.steps.length - 1)
const storageKey = computed(() => `unipack:guide:${props.guideKey}:v1`)

const highlightStyle = computed(() => {
  const rect = targetRect.value
  if (!rect) return {}
  const padding = 7
  const top = Math.min(Math.max(8, rect.top - padding), window.innerHeight - 8)
  const right = Math.min(Math.max(8, rect.right + padding), window.innerWidth - 8)
  const bottom = Math.min(Math.max(8, rect.bottom + padding), window.innerHeight - 8)
  const left = Math.min(Math.max(8, rect.left - padding), window.innerWidth - 8)
  return {
    top: `${top}px`,
    left: `${left}px`,
    width: `${Math.max(0, right - left)}px`,
    height: `${Math.max(0, bottom - top)}px`
  }
})

const cardStyle = computed(() => {
  const rect = targetRect.value
  if (!rect) {
    return {
      top: '50%',
      left: '50%',
      transform: 'translate(-50%, -50%)'
    }
  }

  const cardWidth = Math.min(360, window.innerWidth - 32)
  const cardHeight = guideCardRef.value?.offsetHeight || 230
  const gap = 16
  const left = Math.min(
    Math.max(16, rect.left + rect.width / 2 - cardWidth / 2),
    window.innerWidth - cardWidth - 16
  )
  const fitsBelow = rect.bottom + gap + cardHeight < window.innerHeight - 16
  const top = fitsBelow
    ? rect.bottom + gap
    : Math.max(16, rect.top - cardHeight - gap)

  return {
    top: `${top}px`,
    left: `${left}px`,
    width: `${cardWidth}px`
  }
})

function hasCompletedGuide() {
  try {
    return localStorage.getItem(storageKey.value) === 'completed'
  } catch {
    return false
  }
}

function markGuideCompleted() {
  try {
    localStorage.setItem(storageKey.value, 'completed')
  } catch {
    // The guide still works when storage is unavailable.
  }
}

function clearTimers() {
  if (autoStartTimer) clearTimeout(autoStartTimer)
  if (targetRefreshTimer) clearTimeout(targetRefreshTimer)
  autoStartTimer = null
  targetRefreshTimer = null
}

function scheduleAutoStart() {
  clearTimers()
  if (!props.steps.length || hasCompletedGuide()) return
  autoStartTimer = setTimeout(() => {
    void start()
  }, 650)
}

async function refreshTarget() {
  if (targetRefreshTimer) clearTimeout(targetRefreshTimer)
  await nextTick()
  const selector = currentStep.value?.target
  const target = selector ? document.querySelector<HTMLElement>(selector) : null
  if (!target) {
    targetRect.value = null
    return
  }

  target.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'nearest' })
  targetRect.value = target.getBoundingClientRect()
  targetRefreshTimer = setTimeout(() => {
    targetRect.value = target.getBoundingClientRect()
  }, 280)
}

function updateTargetRect() {
  if (!visible.value) return
  const selector = currentStep.value?.target
  const target = selector ? document.querySelector<HTMLElement>(selector) : null
  targetRect.value = target?.getBoundingClientRect() || null
}

async function start() {
  if (!props.steps.length) return
  clearTimers()
  activeIndex.value = 0
  visible.value = true
  await refreshTarget()
}

async function previous() {
  if (activeIndex.value === 0) return
  activeIndex.value -= 1
  await refreshTarget()
}

async function next() {
  if (isLastStep.value) {
    finish()
    return
  }
  activeIndex.value += 1
  await refreshTarget()
}

function finish() {
  markGuideCompleted()
  visible.value = false
  targetRect.value = null
  clearTimers()
}

function handleKeydown(event: KeyboardEvent) {
  if (!visible.value) return
  if (event.key === 'Escape') finish()
  if (event.key === 'ArrowLeft') void previous()
  if (event.key === 'ArrowRight') void next()
}

watch(() => props.guideKey, () => {
  visible.value = false
  targetRect.value = null
  scheduleAutoStart()
})

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  window.addEventListener('resize', updateTargetRect)
  window.addEventListener('scroll', updateTargetRect, true)
  scheduleAutoStart()
})

onBeforeUnmount(() => {
  clearTimers()
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('resize', updateTargetRect)
  window.removeEventListener('scroll', updateTargetRect, true)
})

defineExpose({ start })
</script>

<template>
  <teleport to="body">
    <div v-if="visible && currentStep" class="app-guide" aria-live="polite">
      <div class="guide-click-shield" />
      <div v-if="targetRect" class="guide-highlight" :style="highlightStyle" />
      <div v-else class="guide-dimmer" />

      <section
        ref="guideCardRef"
        class="guide-card"
        :style="cardStyle"
        role="dialog"
        aria-modal="true"
        :aria-label="currentStep.title"
      >
        <div class="guide-card-head">
          <span class="guide-kicker">使用引导</span>
          <span class="guide-count">{{ activeIndex + 1 }} / {{ steps.length }}</span>
        </div>

        <h2>{{ currentStep.title }}</h2>
        <p>{{ currentStep.description }}</p>

        <div class="guide-progress" aria-hidden="true">
          <span
            v-for="(_, index) in steps"
            :key="index"
            :class="{ active: index === activeIndex, completed: index < activeIndex }"
          />
        </div>

        <div class="guide-actions">
          <n-button quaternary @click="finish">跳过</n-button>
          <div class="guide-nav-actions">
            <n-button v-if="activeIndex > 0" @click="previous">
              <template #icon><n-icon><ArrowBackOutline /></n-icon></template>
              上一步
            </n-button>
            <n-button type="primary" @click="next">
              {{ isLastStep ? '完成' : '下一步' }}
              <template #icon>
                <n-icon>
                  <CheckmarkOutline v-if="isLastStep" />
                  <ArrowForwardOutline v-else />
                </n-icon>
              </template>
            </n-button>
          </div>
        </div>
      </section>
    </div>
  </teleport>
</template>

<style scoped>
.guide-click-shield,
.guide-dimmer {
  position: fixed;
  inset: 0;
}

.guide-click-shield {
  z-index: 3998;
}

.guide-dimmer {
  z-index: 3999;
  background: rgba(15, 23, 42, 0.58);
}

.guide-highlight {
  position: fixed;
  z-index: 3999;
  pointer-events: none;
  border: 2px solid var(--primary-color);
  border-radius: 11px;
  box-shadow:
    0 0 0 9999px rgba(15, 23, 42, 0.58),
    0 0 0 5px rgba(21, 151, 102, 0.2);
  transition: top 0.22s ease, left 0.22s ease, width 0.22s ease, height 0.22s ease;
}

.guide-card {
  position: fixed;
  z-index: 4000;
  width: 360px;
  max-width: calc(100vw - 32px);
  padding: 20px;
  color: var(--text-color);
  background: var(--surface-color);
  border: 1px solid var(--border-soft);
  border-radius: 12px;
  box-shadow: 0 24px 64px rgba(15, 23, 42, 0.26);
}

.guide-card-head,
.guide-actions,
.guide-nav-actions {
  display: flex;
  align-items: center;
}

.guide-card-head,
.guide-actions {
  justify-content: space-between;
}

.guide-kicker,
.guide-count {
  font-size: 12px;
}

.guide-kicker {
  color: var(--primary-strong);
  font-weight: 700;
}

.guide-count {
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

.guide-card h2 {
  margin-top: 12px;
  font-size: 18px;
  line-height: 1.4;
}

.guide-card p {
  margin-top: 8px;
  color: var(--text-muted);
  font-size: 14px;
  line-height: 1.65;
}

.guide-progress {
  display: flex;
  gap: 6px;
  margin-top: 18px;
}

.guide-progress span {
  width: 22px;
  height: 3px;
  border-radius: 999px;
  background: var(--border-color);
}

.guide-progress span.active,
.guide-progress span.completed {
  background: var(--primary-color);
}

.guide-actions {
  gap: 12px;
  margin-top: 18px;
}

.guide-nav-actions {
  gap: 8px;
}
</style>
