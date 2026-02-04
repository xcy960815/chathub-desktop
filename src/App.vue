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
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type Event } from '@tauri-apps/api/event'

const isLoading = ref(true)
const loadingText = ref('模型加载中...')

onMounted(async () => {
  // Listen for model switch event from Rust
  await listen('switch-model', (event: Event<string>) => {
    isLoading.value = true
    loadingText.value = '模型加载中...'
    setTimeout(() => {
      window.location.href = event.payload as string
    }, 300)
  })

  try {
    const url = await invoke('get_last_model_url')
    if (url && typeof url === 'string') {
      window.location.href = url
    } else {
      window.location.href = 'https://chatgpt.com'
    }
  } catch (e) {
    console.error('Failed to get last model url', e)
    window.location.href = 'https://chatgpt.com'
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
  gap: 2rem;
}

.dots {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  height: 50px;
}

.dot {
  border-radius: 50%;
  animation: bounce 0.6s ease-in-out infinite;
}

.dot-1 {
  width: 24px;
  height: 24px;
  background-color: #f87171;
  animation-delay: 0s;
}

.dot-2 {
  width: 22px;
  height: 22px;
  background-color: #2dd4bf;
  animation-delay: 0.1s;
}

.dot-3 {
  width: 18px;
  height: 18px;
  background-color: #7dd3fc;
  animation-delay: 0.2s;
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
  font-size: 1.5rem;
  font-weight: 500;
  color: #374151;
}

@media (prefers-color-scheme: dark) {
  .loading-text {
    color: #d1d5db;
  }
}
</style>
