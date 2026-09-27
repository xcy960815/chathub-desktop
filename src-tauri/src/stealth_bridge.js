/**
 * 指纹伪装桥（stealth bridge）：把 WebView 环境伪装成标准 Chrome 浏览器。
 *
 * 由 src-tauri/src/lib.rs 的 build_initialization_script() 通过 include_str!
 * 原样注入，仅在非 macOS 平台启用（macOS 上注入会破坏 Cloudflare
 * Turnstile，见 docs/oauth-issues.md）。不参与 Vite 构建，须保持纯 JS。
 *
 * 结构：runStealthPatches() 承载全部补丁；injectIntoPageContext() 把同一
 * 函数注入页面主世界再执行一次（初始化脚本运行在隔离世界，页面脚本
 * 看不到其中的修改）。OpenAI / Cloudflare 域名整体跳过，避免干扰人机验证。
 * 调试：在页面控制台执行 __CHATHUB_STEALTH_SNAPSHOT__() 查看伪装状态。
 */
;(function () {
  if (window.__CHATHUB_STEALTH__) {
    return
  }

  window.__CHATHUB_STEALTH__ = true

  /** 伪装目标 UA；优先使用 Rust 侧注入的同源 UA，保证 HTTP 头与 JS 指纹一致 */
  const USER_AGENT =
    window.__CHATHUB_USER_AGENT__ ||
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36'

  const CHROME_MAJOR = (() => {
    const match = USER_AGENT.match(/Chrome\/(\d+)/)
    return match ? match[1] : '131'
  })()

  /** Chrome 实际上报的 brands 列表（含 GREASE 占位品牌），版本号跟随 UA */
  const BRANDS = [
    { brand: 'Not_A Brand', version: '24' },
    { brand: 'Chromium', version: CHROME_MAJOR },
    { brand: 'Google Chrome', version: CHROME_MAJOR }
  ]

  const GOOGLE_HOST_PATTERN = /(^|\.)google\.com$/
  const OPENAI_HOST_PATTERN = /(^|\.)chatgpt\.com$|(^|\.)openai\.com$/
  const CLOUDFLARE_HOST_PATTERN = /(^|\.)cloudflare\.com$/

  /**
   * OpenAI / Cloudflare 域名整体跳过伪装：
   * Turnstile 会检测脚本环境被篡改的痕迹，注入反而导致人机验证失败
   */
  const shouldSkipStealth = () => {
    const hostname = String(location.hostname || '')
    return OPENAI_HOST_PATTERN.test(hostname) || CLOUDFLARE_HOST_PATTERN.test(hostname)
  }

  /**
   * 应用全部指纹补丁（幂等，同一世界内重复调用直接返回）
   */
  const runStealthPatches = () => {
    if (window.__CHATHUB_STEALTH_PATCHED__) {
      return
    }

    if (shouldSkipStealth()) {
      window.__CHATHUB_STEALTH_PATCHED__ = true
      window.__CHATHUB_STEALTH_SKIPPED__ = 'turnstile-safe'
      return
    }

    window.__CHATHUB_STEALTH_PATCHED__ = true

    const hostname = String(location.hostname || '')
    const isGoogleSurface = GOOGLE_HOST_PATTERN.test(hostname)
    const isOpenAISurface = OPENAI_HOST_PATTERN.test(hostname)

    /**
     * 以 defineProperty 写入只读值；属性不可配置时静默失败
     */
    const defineValue = (target, key, value) => {
      try {
        Object.defineProperty(target, key, {
          configurable: true,
          enumerable: true,
          writable: false,
          value
        })
      } catch (_) {}
    }

    /**
     * 以 defineProperty 写入 getter；属性不可配置时静默失败
     */
    const defineGetter = (target, key, getter) => {
      try {
        Object.defineProperty(target, key, {
          configurable: true,
          enumerable: true,
          get: getter
        })
      } catch (_) {}
    }

    /**
     * 伪装原生函数：改写 name 并让 toString() 返回
     * `function x() { [native code] }`，避免补丁函数被 fn.toString() 识破
     *
     * @returns 传入的函数本身，便于链式定义
     */
    const markNative = (fn, name) => {
      try {
        Object.defineProperty(fn, 'name', {
          configurable: true,
          value: name
        })
      } catch (_) {}

      try {
        Object.defineProperty(fn, 'toString', {
          configurable: true,
          value: () => `function ${name}() { [native code] }`
        })
      } catch (_) {}

      return fn
    }

    const navigatorProto = (() => {
      try {
        return Object.getPrototypeOf(navigator)
      } catch (_) {
        return null
      }
    })()

    /**
     * 同时修补 navigator 原型与实例上的同名属性
     *
     * 只改实例会被 Object.getPrototypeOf(navigator) 上的原始 getter 穿透。
     */
    const patchTarget = (key, getter) => {
      if (navigatorProto) {
        defineGetter(navigatorProto, key, getter)
      }
      defineGetter(navigator, key, getter)
    }

    /** 构造一条与 Chrome 一致的 mimeType 记录 */
    const createMimeType = (type, suffixes, description, plugin) => ({
      type,
      suffixes,
      description,
      enabledPlugin: plugin
    })

    /**
     * 构造单个插件对象：含 mimeTypes 反向引用与 item / namedItem 方法
     */
    const createPlugin = (name, filename, description, mimeTypes) => {
      const plugin = {
        name,
        filename,
        description,
        length: mimeTypes.length,
        item: markNative((index) => mimeTypes[index] || null, 'item'),
        namedItem: markNative((mimeType) => {
          for (let index = 0; index < mimeTypes.length; index += 1) {
            if (mimeTypes[index] && mimeTypes[index].type === mimeType) {
              return mimeTypes[index]
            }
          }
          return null
        }, 'namedItem')
      }

      mimeTypes.forEach((mimeType, index) => {
        plugin[index] = mimeType
        mimeType.enabledPlugin = plugin
      })

      return plugin
    }

    /**
     * 构造 Chrome 桌面版标准的三个插件（PDF Plugin / PDF Viewer /
     * Native Client）及对应的 PluginArray / MimeTypeArray 形状
     */
    const buildChromePlugins = () => {
      const pdfPlugin = createPlugin(
        'Chrome PDF Plugin',
        'internal-pdf-viewer',
        'Portable Document Format',
        [createMimeType('application/x-google-chrome-pdf', 'pdf', 'Portable Document Format', null)]
      )

      const pdfViewer = createPlugin('Chrome PDF Viewer', 'mhjfbmdgcfjbbpaeojofohoefgiehjai', '', [
        createMimeType('application/pdf', 'pdf', '', null)
      ])

      const nativeClient = createPlugin('Native Client', 'internal-nacl-plugin', '', [
        createMimeType('application/x-nacl', '', 'Native Client Executable', null),
        createMimeType('application/x-pnacl', '', 'Portable Native Client Executable', null)
      ])

      const plugins = [pdfPlugin, pdfViewer, nativeClient]
      const pluginArray = {
        length: plugins.length,
        item: markNative((index) => plugins[index] || null, 'item'),
        namedItem: markNative((name) => {
          for (let index = 0; index < plugins.length; index += 1) {
            if (plugins[index] && plugins[index].name === name) {
              return plugins[index]
            }
          }
          return null
        }, 'namedItem'),
        refresh: markNative(() => {}, 'refresh'),
        [Symbol.iterator]: markNative(function* () {
          for (let index = 0; index < plugins.length; index += 1) {
            yield plugins[index]
          }
        }, 'values')
      }

      plugins.forEach((plugin, index) => {
        pluginArray[index] = plugin
      })

      const mimeTypeList = []
      plugins.forEach((plugin) => {
        for (let index = 0; index < plugin.length; index += 1) {
          mimeTypeList.push(plugin[index])
        }
      })

      const mimeTypes = {
        length: mimeTypeList.length,
        item: markNative((index) => mimeTypeList[index] || null, 'item'),
        namedItem: markNative((type) => {
          for (let index = 0; index < mimeTypeList.length; index += 1) {
            if (mimeTypeList[index] && mimeTypeList[index].type === type) {
              return mimeTypeList[index]
            }
          }
          return null
        }, 'namedItem'),
        [Symbol.iterator]: markNative(function* () {
          for (let index = 0; index < mimeTypeList.length; index += 1) {
            yield mimeTypeList[index]
          }
        }, 'values')
      }

      mimeTypeList.forEach((mimeType, index) => {
        mimeTypes[index] = mimeType
      })

      return { pluginArray, mimeTypes }
    }

    /**
     * 构造 window.chrome 对象（app / csi / loadTimes / runtime / webstore），
     * 形状参照真实 Chrome，缺省属性用空实现占位
     */
    const buildChromeObject = () => ({
      app: {
        isInstalled: false,
        InstallState: {
          DISABLED: 'disabled',
          INSTALLED: 'installed',
          NOT_INSTALLED: 'not_installed'
        },
        RunningState: {
          CANNOT_RUN: 'cannot_run',
          READY_TO_RUN: 'ready_to_run',
          RUNNING: 'running'
        }
      },
      csi: markNative(() => ({}), 'csi'),
      loadTimes: markNative(
        () => ({
          commitLoadTime: Date.now() / 1000,
          connectionInfo: 'http/1.1',
          finishDocumentLoadTime: Date.now() / 1000,
          finishLoadTime: Date.now() / 1000,
          firstPaintAfterLoadTime: 0,
          firstPaintTime: Date.now() / 1000,
          navigationType: 'Other',
          npnNegotiatedProtocol: 'unknown',
          requestTime: Date.now() / 1000 - 0.16,
          startLoadTime: Date.now() / 1000 - 0.2,
          wasAlternateProtocolAvailable: false,
          wasFetchedViaSpdy: false,
          wasNpnNegotiated: false
        }),
        'loadTimes'
      ),
      runtime: {
        connect: markNative(
          () => ({ onDisconnect: { addListener: markNative(() => {}, 'addListener') } }),
          'connect'
        ),
        id: undefined,
        sendMessage: markNative(() => {}, 'sendMessage'),
        getURL: markNative((path = '') => `chrome-extension://${path}`, 'getURL'),
        onMessage: {
          addListener: markNative(() => {}, 'addListener'),
          removeListener: markNative(() => {}, 'removeListener')
        },
        onInstalled: {
          addListener: markNative(() => {}, 'addListener')
        }
      },
      webstore: {
        onInstallStageChanged: {},
        onDownloadProgress: {}
      }
    })

    /**
     * 伪装 WebGL 渲染信息：UNMASKED_VENDOR_WEBGL(37445) 与
     * UNMASKED_RENDERER_WEBGL(37446) 报告为 Intel Iris，
     * 掩盖 WebView/虚拟机的真实 GPU 信息
     *
     * @param Ctor - WebGLRenderingContext 或 WebGL2RenderingContext
     */
    const patchWebGL = (Ctor) => {
      try {
        if (!Ctor || !Ctor.prototype || !Ctor.prototype.getParameter) {
          return
        }

        const originalGetParameter = Ctor.prototype.getParameter
        Ctor.prototype.getParameter = markNative(function (parameter) {
          if (parameter === 37445) {
            return 'Intel Inc.'
          }
          if (parameter === 37446) {
            return 'Intel(R) Iris(R) Plus Graphics 640'
          }
          return originalGetParameter.call(this, parameter)
        }, 'getParameter')
      } catch (_) {}
    }

    /**
     * 为 canvas 指纹加入稳定噪声：翻转首像素最低有效位，
     * 视觉不可察觉但足以破坏精确指纹匹配
     */
    const patchCanvas = () => {
      try {
        const originalToDataURL = HTMLCanvasElement.prototype.toDataURL
        const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData

        CanvasRenderingContext2D.prototype.getImageData = markNative(function (...args) {
          const imageData = originalGetImageData.apply(this, args)
          try {
            if (imageData && imageData.data && imageData.data.length > 0) {
              imageData.data[0] = imageData.data[0] ^ 1
            }
          } catch (_) {}
          return imageData
        }, 'getImageData')

        HTMLCanvasElement.prototype.toDataURL = markNative(function (...args) {
          try {
            const context = this.getContext('2d')
            if (context) {
              const { width, height } = this
              if (width && height) {
                // 翻转首像素最低有效位改变指纹，视觉上不可察觉；
                // 使用未包装的 getImageData 避免噪声被二次翻转抵消
                const imageData = originalGetImageData.call(context, 0, 0, width, height)
                imageData.data[0] = imageData.data[0] ^ 1
                context.putImageData(imageData, 0, 0)
              }
            }
          } catch (_) {}
          return originalToDataURL.apply(this, args)
        }, 'toDataURL')
      } catch (_) {}
    }

    /**
     * 过滤 WebRTC 本地 IP 泄漏：从 SDP 中剔除 `typ host` 候选，
     * 防止通过本地地址反推真实网络环境
     */
    const patchWebRTC = () => {
      try {
        const OriginalRTCPeerConnection = window.RTCPeerConnection
        if (!OriginalRTCPeerConnection) {
          return
        }

        const sanitizeSdp = (sdp) => {
          if (typeof sdp !== 'string') {
            return sdp
          }

          return sdp
            .split('\n')
            .filter((line) => {
              if (line.startsWith('a=candidate:') && line.includes(' typ host ')) {
                return false
              }
              return true
            })
            .join('\n')
        }

        const wrapPeerConnection = markNative(function (...args) {
          const pc = new OriginalRTCPeerConnection(...args)
          const originalCreateOffer = pc.createOffer.bind(pc)
          const originalCreateAnswer = pc.createAnswer.bind(pc)

          pc.createOffer = markNative(async (...offerArgs) => {
            const offer = await originalCreateOffer(...offerArgs)
            return {
              ...offer,
              sdp: sanitizeSdp(offer.sdp)
            }
          }, 'createOffer')

          pc.createAnswer = markNative(async (...answerArgs) => {
            const answer = await originalCreateAnswer(...answerArgs)
            return {
              ...answer,
              sdp: sanitizeSdp(answer.sdp)
            }
          }, 'createAnswer')

          return pc
        }, 'RTCPeerConnection')

        wrapPeerConnection.prototype = OriginalRTCPeerConnection.prototype
        defineValue(window, 'RTCPeerConnection', wrapPeerConnection)
      } catch (_) {}
    }

    /**
     * 修补 mediaDevices.enumerateDevices：无权限时 WebView 返回空数组，
     * 这本身是可检测特征，回退为合理的默认设备列表
     */
    const patchMediaDevices = () => {
      try {
        if (!navigator.mediaDevices) {
          return
        }

        if (typeof navigator.mediaDevices.enumerateDevices === 'function') {
          const originalEnumerateDevices = navigator.mediaDevices.enumerateDevices.bind(
            navigator.mediaDevices
          )

          defineValue(
            navigator.mediaDevices,
            'enumerateDevices',
            markNative(async () => {
              const devices = await originalEnumerateDevices()
              if (Array.isArray(devices) && devices.length > 0) {
                return devices
              }

              return [
                {
                  deviceId: 'default',
                  groupId: 'default',
                  kind: 'audioinput',
                  label: 'Default Audio Input',
                  toJSON() {
                    return this
                  }
                },
                {
                  deviceId: 'default',
                  groupId: 'default',
                  kind: 'videoinput',
                  label: 'Default Camera',
                  toJSON() {
                    return this
                  }
                }
              ]
            }, 'enumerateDevices')
          )
        }
      } catch (_) {}
    }

    /**
     * 修补 navigator.permissions.query：对 Chrome 支持同步查询的常见权限
     * 返回一致的 PermissionStatus 形状，其余走原始实现
     */
    const patchPermissions = () => {
      try {
        if (!navigator.permissions || typeof navigator.permissions.query !== 'function') {
          return
        }

        const originalQuery = navigator.permissions.query.bind(navigator.permissions)
        const allowedNames = new Set([
          'notifications',
          'camera',
          'microphone',
          'geolocation',
          'clipboard-read',
          'clipboard-write'
        ])

        defineValue(
          navigator.permissions,
          'query',
          markNative((parameters) => {
            if (parameters && allowedNames.has(parameters.name)) {
              return Promise.resolve({
                state: parameters.name === 'notifications' ? Notification.permission : 'prompt',
                onchange: null
              })
            }

            return originalQuery(parameters)
          }, 'query')
        )
      } catch (_) {}
    }

    /**
     * 隐藏 WebKit 特有 API（window.webkit / ApplePaySession），避免暴露 WKWebView
     */
    const hideWebKitIndicators = () => {
      try {
        if ('webkit' in window) {
          defineGetter(window, 'webkit', () => undefined)
        }
      } catch (_) {}

      try {
        if ('ApplePaySession' in window) {
          defineGetter(window, 'ApplePaySession', () => undefined)
        }
      } catch (_) {}
    }

    // 清理 Selenium / ChromeDriver 注入的 cdc_ 变量痕迹
    try {
      for (const key in window) {
        if (key.startsWith('cdc_')) {
          delete window[key]
        }
      }
    } catch (_) {}

    // —— 基础 navigator 指纹：UA、平台、硬件、语言等 ——
    patchTarget('webdriver', () => false)
    patchTarget('userAgent', () => USER_AGENT)
    patchTarget('appVersion', () => USER_AGENT.replace(/^Mozilla\//, ''))
    patchTarget('platform', () => 'MacIntel')
    patchTarget('vendor', () => 'Google Inc.')
    patchTarget('language', () => 'zh-CN')
    patchTarget('languages', () => ['zh-CN', 'zh', 'en'])
    patchTarget('maxTouchPoints', () => 0)
    patchTarget('hardwareConcurrency', () => 8)
    patchTarget('deviceMemory', () => 8)
    patchTarget('pdfViewerEnabled', () => true)
    patchTarget('productSub', () => '20030107')
    patchTarget('vendorSub', () => '')

    // —— Chrome 插件与 Client Hints 模拟 ——
    try {
      const { pluginArray, mimeTypes } = buildChromePlugins()
      patchTarget('plugins', () => pluginArray)
      patchTarget('mimeTypes', () => mimeTypes)
    } catch (_) {}

    // navigator.userAgentData（Client Hints）：Chrome 独有，缺失即暴露 WebView
    try {
      const uaData = {
        brands: BRANDS,
        mobile: false,
        platform: 'macOS',
        getHighEntropyValues: markNative(
          async () => ({
            brands: BRANDS,
            mobile: false,
            platform: 'macOS',
            architecture: 'x86',
            bitness: '64',
            model: '',
            platformVersion: '14.0.0',
            uaFullVersion: `${CHROME_MAJOR}.0.0.0`,
            fullVersionList: BRANDS
          }),
          'getHighEntropyValues'
        ),
        toJSON: markNative(
          () => ({
            brands: BRANDS,
            mobile: false,
            platform: 'macOS'
          }),
          'toJSON'
        )
      }

      patchTarget('userAgentData', () => uaData)
    } catch (_) {}

    // NetworkInformation：Chrome 存在此 API，WebView 未必有
    try {
      const connection = {
        downlink: 10,
        effectiveType: '4g',
        onchange: null,
        rtt: 50,
        saveData: false,
        type: 'wifi'
      }
      patchTarget('connection', () => connection)
    } catch (_) {}

    // 固定同一实例：window.chrome 每次访问返回新对象会暴露 chrome === chrome 为 false
    const chromeObject = buildChromeObject()
    try {
      defineGetter(window, 'chrome', () => chromeObject)
    } catch (_) {}

    patchPermissions()

    // 窗口外观尺寸与视口对齐；色深与 Chrome 桌面端保持一致
    try {
      defineGetter(window, 'outerWidth', () => window.innerWidth)
      defineGetter(window, 'outerHeight', () => window.innerHeight + 28)
    } catch (_) {}

    try {
      defineGetter(screen, 'colorDepth', () => 24)
      defineGetter(screen, 'pixelDepth', () => 24)
    } catch (_) {}

    if (isGoogleSurface) {
      patchTarget('cookieEnabled', () => true)
      patchTarget('onLine', () => true)
      patchTarget('doNotTrack', () => null)
      patchMediaDevices()
      hideWebKitIndicators()
    }

    patchWebGL(window.WebGLRenderingContext)
    patchWebGL(window.WebGL2RenderingContext)
    patchCanvas()
    patchWebRTC()

    /**
     * 调试快照：输出当前伪装状态，便于在控制台核对补丁是否生效
     */
    try {
      defineValue(
        window,
        '__CHATHUB_STEALTH_SNAPSHOT__',
        markNative(
          () => ({
            host: hostname,
            googleSurface: isGoogleSurface,
            openaiSurface: isOpenAISurface,
            webdriver: navigator.webdriver,
            userAgent: navigator.userAgent,
            languages: navigator.languages,
            platform: navigator.platform,
            vendor: navigator.vendor,
            hardwareConcurrency: navigator.hardwareConcurrency,
            deviceMemory: navigator.deviceMemory,
            maxTouchPoints: navigator.maxTouchPoints,
            pdfViewerEnabled: navigator.pdfViewerEnabled,
            pluginsLength: navigator.plugins ? navigator.plugins.length : 0,
            mimeTypesLength: navigator.mimeTypes ? navigator.mimeTypes.length : 0,
            hasChrome: !!window.chrome,
            hasWebkit: 'webkit' in window,
            hasUserAgentData: !!navigator.userAgentData,
            userAgentData: navigator.userAgentData
              ? {
                  brands: navigator.userAgentData.brands,
                  mobile: navigator.userAgentData.mobile,
                  platform: navigator.userAgentData.platform
                }
              : null
          }),
          '__CHATHUB_STEALTH_SNAPSHOT__'
        )
      )
    } catch (_) {}
  }

  /**
   * 把 runStealthPatches 注入页面主世界执行
   *
   * 初始化脚本运行在隔离世界，页面脚本看不到其中的修改；
   * 通过内联 <script> 在主世界再执行一次。__CHATHUB_STEALTH_PATCHED__
   * 在两个世界各自独立，因此不会互相短路。
   */
  const injectIntoPageContext = () => {
    const mountPoint = document.head || document.documentElement
    if (!mountPoint) {
      setTimeout(injectIntoPageContext, 0)
      return
    }

    try {
      const script = document.createElement('script')
      script.textContent = `;(${runStealthPatches.toString()})()`
      mountPoint.appendChild(script)
      script.remove()
    } catch (_) {}
  }

  runStealthPatches()
  injectIntoPageContext()
})()
