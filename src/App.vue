<template>
  <div v-if="externalNotice" class="external-container">
    <div class="external-card">
      <h1 class="external-title">{{ externalTitle }}</h1>
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
    <div class="loading-text">{{ loadingText }}</div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type Event } from '@tauri-apps/api/event'

/** Rust 命令 `get_model_launch_plan` 返回的模型启动计划 */
interface ModelLaunchPlan {
  /** 模型目标地址 */
  url: string
  /** 为 true 时模型已在系统浏览器打开，外壳展示提示卡片而非跳转 */
  openInSystemBrowser: boolean
  /** 提示卡片标题，随托盘语言设置返回 */
  title?: string | null
  /** 提示卡片正文，随托盘语言设置返回 */
  notice?: string | null
}

const isLoading = ref(true)
const loadingText = ref('模型加载中...')
const externalTitle = ref('已在系统浏览器中打开')
const externalNotice = ref<string | null>(null)
const externalUrl = ref('')
const DEFAULT_MODEL_URL = 'https://chatgpt.com'
/** onMounted 中注册的事件监听清理函数，卸载时统一执行 */
const cleanupFns: Array<() => void | Promise<void>> = []

/**
 * 让主窗口导航到目标地址
 *
 * 外壳页随之被替换，Vue 实例销毁；回到外壳时会重新走 launchModel。
 */
function redirectTo(url: string) {
  window.location.href = url
}

/**
 * 展示"已在系统浏览器中打开"提示卡片
 */
function showExternalNotice(plan: ModelLaunchPlan) {
  externalTitle.value = plan.title || '已在系统浏览器中打开'
  externalNotice.value = plan.notice || '已在系统浏览器中打开。'
  externalUrl.value = plan.url
  isLoading.value = false
}

/**
 * 提示卡片上的"再次在浏览器中打开"：把当前模型地址重新交给系统浏览器
 */
async function reopenInBrowser() {
  if (!externalUrl.value) {
    return
  }

  await invoke('open_model_in_browser', { url: externalUrl.value })
}

/**
 * 外壳挂载后的统一启动入口：向 Rust 索取启动计划并执行
 *
 * - ChatGPT（macOS）：Rust 已在系统浏览器打开，这里展示提示卡片
 * - 其他模型：跳转到计划中的 URL
 */
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
  cleanupFns.length = 0
})

onMounted(async () => {
  if (!isTauri()) {
    console.info('[App] Running in browser mode, fallback to default model URL')
    redirectTo(DEFAULT_MODEL_URL)
    return
  }

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

<style scoped>
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

.external-title {
  font-size: 1.25rem;
  margin-bottom: 12px;
}

.external-text {
  line-height: 1.6;
  color: #4b5563;
  margin-bottom: 20px;
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

.loading-container {
  gap: 2rem;
}

.dots {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  height: 30px;
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

.loading-text {
  font-size: 1rem;
  font-weight: 500;
  color: #374151;
}

@media (prefers-color-scheme: dark) {
  .external-card {
    background: rgba(31, 31, 31, 0.96);
    border-color: rgba(255, 255, 255, 0.08);
  }

  .external-text {
    color: #d1d5db;
  }

  .external-button {
    background: #f3f4f6;
    color: #111827;
  }

  .loading-text {
    color: #d1d5db;
  }
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
</style>
