;(function () {
  if (window.__CHATHUB_STEALTH__) {
    return
  }

  window.__CHATHUB_STEALTH__ = true

  const USER_AGENT =
    window.__CHATHUB_USER_AGENT__ ||
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36'

  const CHROME_MAJOR = (() => {
    const match = USER_AGENT.match(/Chrome\/(\d+)/)
    return match ? match[1] : '131'
  })()

  const BRANDS = [
    { brand: 'Not_A Brand', version: '24' },
    { brand: 'Chromium', version: CHROME_MAJOR },
    { brand: 'Google Chrome', version: CHROME_MAJOR }
  ]

  const GOOGLE_HOST_PATTERN = /(^|\.)google\.com$/
  const OPENAI_HOST_PATTERN = /(^|\.)chatgpt\.com$|(^|\.)openai\.com$/
  const CLOUDFLARE_HOST_PATTERN = /(^|\.)cloudflare\.com$/

  const shouldSkipStealth = () => {
    const hostname = String(location.hostname || '')
    return OPENAI_HOST_PATTERN.test(hostname) || CLOUDFLARE_HOST_PATTERN.test(hostname)
  }

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

    const defineGetter = (target, key, getter) => {
      try {
        Object.defineProperty(target, key, {
          configurable: true,
          enumerable: true,
          get: getter
        })
      } catch (_) {}
    }

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

    const patchTarget = (key, getter) => {
      if (navigatorProto) {
        defineGetter(navigatorProto, key, getter)
      }
      defineGetter(navigator, key, getter)
    }

    const createMimeType = (type, suffixes, description, plugin) => ({
      type,
      suffixes,
      description,
      enabledPlugin: plugin
    })

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

    const patchCanvas = () => {
      try {
        const originalToDataURL = HTMLCanvasElement.prototype.toDataURL
        const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData

        HTMLCanvasElement.prototype.toDataURL = markNative(function (...args) {
          try {
            const context = this.getContext('2d')
            if (context) {
              const { width, height } = this
              if (width && height) {
                const imageData = context.getImageData(0, 0, width, height)
                imageData.data[0] = imageData.data[0] ^ 1
                context.putImageData(imageData, 0, 0)
              }
            }
          } catch (_) {}
          return originalToDataURL.apply(this, args)
        }, 'toDataURL')

        CanvasRenderingContext2D.prototype.getImageData = markNative(function (...args) {
          const imageData = originalGetImageData.apply(this, args)
          try {
            if (imageData && imageData.data && imageData.data.length > 0) {
              imageData.data[0] = imageData.data[0] ^ 1
            }
          } catch (_) {}
          return imageData
        }, 'getImageData')
      } catch (_) {}
    }

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

    try {
      for (const key in window) {
        if (key.startsWith('cdc_')) {
          delete window[key]
        }
      }
    } catch (_) {}

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

    try {
      const { pluginArray, mimeTypes } = buildChromePlugins()
      patchTarget('plugins', () => pluginArray)
      patchTarget('mimeTypes', () => mimeTypes)
    } catch (_) {}

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

    try {
      defineGetter(window, 'chrome', () => buildChromeObject())
    } catch (_) {}

    patchPermissions()

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
