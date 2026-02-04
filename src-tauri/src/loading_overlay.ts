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
        <div class="loading-text">模型加载中...</div>
    `
  document.documentElement.appendChild(overlay)

  // Redirect to the target model URL after a short delay
  setTimeout(function () {
    window.location.href = '__TARGET_URL__'
  }, 800)

  // Auto-remove after long timeout (failsafe)
  setTimeout(() => {
    if (document.getElementById(overlayId)) {
      overlay.style.opacity = '0'
      setTimeout(() => overlay.remove(), 500)
    }
  }, 10000)
})()
