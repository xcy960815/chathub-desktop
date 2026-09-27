/**
 * 模型加载过渡层：切换模型时先展示全屏加载动效，再跳转目标地址，
 * 避免页面切换出现白屏闪烁。
 *
 * 由 src-tauri/src/models.rs 的 switch_model() 通过 include_str! 注入当前页面，
 * 不参与 Vite 构建，须保持纯 JS。注入前 Rust 侧会替换以下占位符：
 * - `__TARGET_URL__`：跳转目标地址
 * - `__LOADING_TEXT__`：本地化的加载文案（跟随托盘语言设置）
 */
;(function () {
  const overlayId = 'chathub-loading-overlay'
  if (document.getElementById(overlayId)) return

  const overlay = document.createElement('div')
  overlay.id = overlayId
  overlay.innerHTML = `
        <style>
            #${overlayId} {
                position: fixed;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                background-color: #f6f6f6;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                z-index: 999999;
                transition: opacity 0.5s ease-out;
            }
            @media (prefers-color-scheme: dark) {
                #${overlayId} {
                    background-color: #2f2f2f;
                }
            }
            #${overlayId} .dots {
                display: flex;
                align-items: flex-end;
                gap: 8px;
                height: 30px;
                margin-bottom: 2rem;
            }
            #${overlayId} .dot {
                border-radius: 50%;
                animation: chathub-bounce 1.5s ease-in-out infinite;
            }
            #${overlayId} .dot-1 {
                width: 14px;
                height: 14px;
                background-color: #f87171;
                animation-delay: 0s;
            }
            #${overlayId} .dot-2 {
                width: 12px;
                height: 12px;
                background-color: #2dd4bf;
                animation-delay: 0.3s;
            }
            #${overlayId} .dot-3 {
                width: 10px;
                height: 10px;
                background-color: #7dd3fc;
                animation-delay: 0.6s;
            }
            @keyframes chathub-bounce {
                0%, 100% { transform: translateY(0); }
                50% { transform: translateY(-20px); }
            }
            #${overlayId} .loading-text {
                font-size: 1rem;
                font-weight: 500;
                color: #374151;
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, "PingFang SC", "Microsoft YaHei", sans-serif;
            }
            @media (prefers-color-scheme: dark) {
                #${overlayId} .loading-text {
                    color: #d1d5db;
                }
            }
        </style>
        <div class="dots">
            <div class="dot dot-1"></div>
            <div class="dot dot-2"></div>
            <div class="dot dot-3"></div>
        </div>
        <div class="loading-text">__LOADING_TEXT__</div>
    `
  document.documentElement.appendChild(overlay)

  // 短暂展示动效后跳转目标地址（给遮罩渲染留出时间，避免闪烁）
  setTimeout(function () {
    window.location.href = '__TARGET_URL__'
  }, 800)

  // 兜底：10 秒后无论跳转是否成功都淡出并移除遮罩
  setTimeout(() => {
    if (document.getElementById(overlayId)) {
      overlay.style.opacity = '0'
      setTimeout(() => overlay.remove(), 500)
    }
  }, 10000)
})()
