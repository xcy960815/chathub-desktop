/**
 * 应用外壳入口：挂载 Vue 根组件。
 *
 * 主窗口加载后由 App.vue 向 Rust 索取启动计划，
 * 决定跳转到对应模型页面还是展示外开提示。
 */
import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

createApp(App).mount('#app')
