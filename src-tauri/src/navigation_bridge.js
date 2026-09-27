/**
 * URL 追踪桥：把 WebView 内发生的地址变化上报给 Rust 侧持久化。
 *
 * 由 src-tauri/src/lib.rs 的 build_initialization_script() 通过 include_str!
 * 注入到主窗口的每个页面（含模型站点），不参与 Vite 构建，须保持纯 JS。
 *
 * 上报机制：创建临时 iframe 加载 `chathub://url-change?value=<当前地址>`，
 * Rust 侧 on_navigation 拦截该自定义 scheme、解析 value 参数后取消导航，
 * 因此不会产生真实的页面跳转。
 */
;(function () {
  if (window.__CHATHUB_URL_TRACKER__) {
    return
  }

  window.__CHATHUB_URL_TRACKER__ = true

  /**
   * 通过一次性 iframe 把当前地址通知给 Rust 侧
   */
  const notify = () => {
    const iframe = document.createElement('iframe')
    iframe.style.display = 'none'
    iframe.src = `chathub://url-change?value=${encodeURIComponent(window.location.href)}`
    document.documentElement.appendChild(iframe)
    setTimeout(() => iframe.remove(), 0)
  }

  /**
   * 包装 history.pushState / replaceState，让 SPA 路由变化也能触发上报
   *
   * @param methodName - 要包装的 history 方法名
   */
  const wrapHistoryMethod = (methodName) => {
    const original = history[methodName]
    if (typeof original !== 'function') {
      return
    }

    history[methodName] = function () {
      const result = original.apply(this, arguments)
      notify()
      return result
    }
  }

  wrapHistoryMethod('pushState')
  wrapHistoryMethod('replaceState')

  window.addEventListener('popstate', notify)
  window.addEventListener('hashchange', notify)

  notify()
})()
