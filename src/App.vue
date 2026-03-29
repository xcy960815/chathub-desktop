<template>
  <div class="loading-container" v-if="isLoading">
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

const isLoading = ref(true)
const loadingText = ref('模型加载中...')
const DEFAULT_MODEL_URL = 'https://chatgpt.com'
const cleanupFns: Array<() => void | Promise<void>> = []

function redirectTo(url: string) {
  window.location.href = url
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

  // Listen for model switch event from Rust
  cleanupFns.push(
    await listen('switch-model', (event: Event<string>) => {
      isLoading.value = true
      loadingText.value = '模型加载中...'
      setTimeout(() => {
        redirectTo(event.payload as string)
      }, 300)
    })
  )

  // 监听登录成功
  cleanupFns.push(
    await listen('login_success', (event: Event<Record<string, unknown>>) => {
      const userInfo = event.payload
      console.log('[OAuth] 登录成功:', userInfo)
      // TODO: 更新 UI 状态（如显示用户头像、用户名等）
    })
  )

  // 监听登录失败
  cleanupFns.push(
    await listen('login_error', (event: Event<string>) => {
      console.error('[OAuth] 登录失败:', event.payload)
      // TODO: 显示错误提示给用户
    })
  )

  try {
    const url = await invoke<string>('get_last_model_url')
    if (url && typeof url === 'string') {
      redirectTo(url)
    } else {
      redirectTo(DEFAULT_MODEL_URL)
    }
  } catch (e) {
    console.error('Failed to get last model url', e)
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

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
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
