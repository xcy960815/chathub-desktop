<template>
  <div v-if="externalNotice" class="external-container">
    <div class="external-card">
      <h1 class="external-title">ChatGPT 已在系统浏览器中打开</h1>
      <p class="external-text">{{ externalNotice }}</p>
      <button class="external-button" type="button" @click="reopenInBrowser">
        再次在浏览器中打开
      </button>
    </div>
  </div>
  <div v-else-if="isLoading" class="loading-container">
    <div class="dots">
      <div class="dot dot-1"></div>
      <div class="dot dot-2"></div>
      <div class="dot dot-3"></div>
    </div>
    <p class="loading-text">{{ loadingText }}</p>
  </div>
</template>
<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type Event } from '@tauri-apps/api/event'

interface ModelLaunchPlan {
  url: string
  openInSystemBrowser: boolean
  notice?: string | null
}

const isLoading = ref(true)
const loadingText = ref('模型加载中...')
const externalNotice = ref<string | null>(null)
const externalUrl = ref('')
const DEFAULT_MODEL_URL = 'https://chatgpt.com'
const cleanupFns: Array<() => void | Promise<void>> = []

function redirectTo(url: string) {
  window.location.href = url
}

function showExternalNotice(plan: ModelLaunchPlan) {
  externalNotice.value = plan.notice || '已在系统浏览器中打开。'
  externalUrl.value = plan.url
  isLoading.value = false
}

async function reopenInBrowser() {
  if (!externalUrl.value) {
    return
  }

  await invoke('open_model_in_browser', { url: externalUrl.value })
}

async function launchModel() {
  const plan = await invoke<ModelLaunchPlan>('get_model_launch_plan')
  if (plan.openInSystemBrowser) {
    showExternalNotice(plan)
    return
  }

  redirectTo(plan.url || DEFAULT_MODEL_URL)
}

onBeforeUnmount(() => {
  for (const cleanup of cleanupFns) {
    void Promise.resolve(cleanup())
  }
})

onMounted(async () => {
  if (!isTauri()) {
    console.info('[App] Running in browser mode, fallback to default model URL')
    redirectTo(DEFAULT_MODEL_URL)
    return
  }

  cleanupFns.push(
    await listen('switch-model', (event: Event<string>) => {
      isLoading.value = true
      loadingText.value = '模型加载中...'
      externalNotice.value = null
      setTimeout(() => {
        redirectTo(event.payload as string)
      }, 300)
    })
  )

  cleanupFns.push(
    await listen('login_success', (event: Event<Record<string, unknown>>) => {
      const userInfo = event.payload
      console.log('[OAuth] 登录成功:', userInfo)
    })
  )

  cleanupFns.push(
    await listen('login_error', (event: Event<string>) => {
      console.error('[OAuth] 登录失败:', event.payload)
    })
  )

  try {
    await launchModel()
  } catch (e) {
    console.error('Failed to launch model', e)
    redirectTo(DEFAULT_MODEL_URL)
  }
})
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body {
  height: 100%;
  width: 100%;
}

:root {
  font-family: 'PingFang SC', 'Microsoft YaHei', Inter, Avenir, Helvetica, Arial, sans-serif;
  color: #0f0f0f;
  background-color: #f6f6f6;
  height: 100vh;
  width: 100vw;
  display: flex;
  align-items: center;
  justify-content: center;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }
}

.loading-container,
.external-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  width: 100vw;
  padding: 24px;
}

.external-card {
  max-width: 520px;
  padding: 28px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(15, 23, 42, 0.08);
  box-shadow: 0 12px 40px rgba(15, 23, 42, 0.08);
  text-align: center;
}

@media (prefers-color-scheme: dark) {
  .external-card {
    background: rgba(31, 31, 31, 0.96);
    border-color: rgba(255, 255, 255, 0.08);
  }
}

.external-title {
  font-size: 1.25rem;
  margin-bottom: 12px;
}

.external-text {
  line-height: 1.6;
  color: #4b5563;
  margin-bottom: 20px;
}

@media (prefers-color-scheme: dark) {
  .external-text {
    color: #d1d5db;
  }
}

.external-button {
  border: none;
  border-radius: 999px;
  padding: 10px 18px;
  background: #111827;
  color: #fff;
  font-size: 0.95rem;
  cursor: pointer;
}

@media (prefers-color-scheme: dark) {
  .external-button {
    background: #f3f4f6;
    color: #111827;
  }
}

.dots {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  height: 30px;
}

.loading-container {
  gap: 2rem;
}

.dot {
  border-radius: 50%;
  animation: bounce 1.5s ease-in-out infinite;
}

.dot-1 {
  width: 14px;
  height: 14px;
  background-color: #f87171;
  animation-delay: 0s;
}

.dot-2 {
  width: 12px;
  height: 12px;
  background-color: #2dd4bf;
  animation-delay: 0.2s;
}

.dot-3 {
  width: 10px;
  height: 10px;
  background-color: #7dd3fc;
  animation-delay: 0.4s;
}

@keyframes bounce {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-20px);
  }
}

.loading-text {
  font-size: 1rem;
  font-weight: 500;
  color: #374151;
}

@media (prefers-color-scheme: dark) {
  .loading-text {
    color: #d1d5db;
  }
}
</style>
