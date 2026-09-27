/* eslint-disable */
/// <reference types="vite/client" />

// Vite 客户端类型与 .vue 单文件组件的模块声明

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}
