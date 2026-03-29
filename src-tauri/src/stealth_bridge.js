;(function () {
  if (window.__CHATHUB_STEALTH__) {
    return
  }

  window.__CHATHUB_STEALTH__ = true

  const USER_AGENT =
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36'

  const BRANDS = [
    { brand: 'Not_A Brand', version: '24' },
    { brand: 'Chromium', version: '131' },
    { brand: 'Google Chrome', version: '131' }
  ]

  const GOOGLE_HOST_PATTERN = /(^|\.)google\.com$/
  const OPENAI_HOST_PATTERN = /(^|\.)chatgpt\.com$|(^|\.)openai\.com$/

  const runStealthPatches = () => {
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

    const patchGoogleSurface = () => {
      try {
        defineGetter(navigatorProto || navigator, 'languages', () => ['en-US', 'en', 'zh-CN', 'zh'])
        defineGetter(navigator, 'languages', () => ['en-US', 'en', 'zh-CN', 'zh'])
      } catch (_) {}

      try {
        defineGetter(navigatorProto || navigator, 'cookieEnabled', () => true)
        defineGetter(navigator, 'cookieEnabled', () => true)
      } catch (_) {}

      try {
        defineGetter(navigatorProto || navigator, 'onLine', () => true)
        defineGetter(navigator, 'onLine', () => true)
      } catch (_) {}

      try {
        defineGetter(navigatorProto || navigator, 'doNotTrack', () => null)
        defineGetter(navigator, 'doNotTrack', () => null)
      } catch (_) {}

      try {
        const chromeObject = window.chrome || {}
        if (!chromeObject.webstore) {
          chromeObject.webstore = {
            onInstallStageChanged: {},
            onDownloadProgress: {}
          }
        }
        if (!chromeObject.runtime) {
          chromeObject.runtime = {}
        }
        if (!('id' in chromeObject.runtime)) {
          chromeObject.runtime.id = undefined
        }
        if (!chromeObject.runtime.getURL) {
          chromeObject.runtime.getURL = markNative(
            (path = '') => `chrome-extension://${path}`,
            'getURL'
          )
        }
        defineGetter(window, 'chrome', () => chromeObject)
      } catch (_) {}

      try {
        if (navigator.permissions && typeof navigator.permissions.query === 'function') {
          const originalQuery = navigator.permissions.query.bind(navigator.permissions)
          const patchedQuery = markNative((parameters) => {
            const allowedNames = new Set([
              'notifications',
              'camera',
              'microphone',
              'geolocation',
              'clipboard-read',
              'clipboard-write'
            ])

            if (parameters && allowedNames.has(parameters.name)) {
              return Promise.resolve({
                state: parameters.name === 'notifications' ? Notification.permission : 'prompt',
                onchange: null
              })
            }

            return originalQuery(parameters)
          }, 'query')

          defineValue(navigator.permissions, 'query', patchedQuery)
        }
      } catch (_) {}

      patchMediaDevices()
    }

    const patchOpenAISurface = () => {
      try {
        defineGetter(navigatorProto || navigator, 'languages', () => ['en-US', 'en', 'zh-CN', 'zh'])
        defineGetter(navigator, 'languages', () => ['en-US', 'en', 'zh-CN', 'zh'])
      } catch (_) {}

      try {
        defineGetter(navigatorProto || navigator, 'cookieEnabled', () => true)
        defineGetter(navigator, 'cookieEnabled', () => true)
      } catch (_) {}
    }

    try {
      for (const key in window) {
        if (key.startsWith('cdc_')) {
          delete window[key]
        }
      }
    } catch (_) {}

    const navigatorProto = (() => {
      try {
        return Object.getPrototypeOf(navigator)
      } catch (_) {
        return null
      }
    })()

    if (navigatorProto) {
      defineGetter(navigatorProto, 'webdriver', () => undefined)
      defineGetter(navigatorProto, 'userAgent', () => USER_AGENT)
      defineGetter(navigatorProto, 'appVersion', () => USER_AGENT.replace(/^Mozilla\//, ''))
      defineGetter(navigatorProto, 'platform', () => 'MacIntel')
      defineGetter(navigatorProto, 'vendor', () => 'Google Inc.')
      defineGetter(navigatorProto, 'languages', () => ['zh-CN', 'zh', 'en'])
      defineGetter(navigatorProto, 'maxTouchPoints', () => 0)
      defineGetter(navigatorProto, 'hardwareConcurrency', () => 8)
      defineGetter(navigatorProto, 'deviceMemory', () => 8)
      defineGetter(navigatorProto, 'pdfViewerEnabled', () => true)
      defineGetter(navigatorProto, 'productSub', () => '20030107')
      defineGetter(navigatorProto, 'vendorSub', () => '')
    }

    defineGetter(navigator, 'webdriver', () => undefined)
    defineGetter(navigator, 'userAgent', () => USER_AGENT)
    defineGetter(navigator, 'appVersion', () => USER_AGENT.replace(/^Mozilla\//, ''))
    defineGetter(navigator, 'platform', () => 'MacIntel')
    defineGetter(navigator, 'vendor', () => 'Google Inc.')
    defineGetter(navigator, 'languages', () => ['zh-CN', 'zh', 'en'])
    defineGetter(navigator, 'maxTouchPoints', () => 0)
    defineGetter(navigator, 'hardwareConcurrency', () => 8)
    defineGetter(navigator, 'deviceMemory', () => 8)
    defineGetter(navigator, 'pdfViewerEnabled', () => true)

    try {
      const pluginTemplate = {
        0: {
          type: 'application/x-google-chrome-pdf',
          suffixes: 'pdf',
          description: 'Portable Document Format',
          enabledPlugin: null
        },
        description: 'Portable Document Format',
        filename: 'internal-pdf-viewer',
        length: 1,
        name: 'Chrome PDF Plugin'
      }
      pluginTemplate[0].enabledPlugin = pluginTemplate

      const pluginArray = {
        0: pluginTemplate,
        1: pluginTemplate,
        2: pluginTemplate,
        length: 3,
        item: markNative((index) => pluginArray[index] || null, 'item'),
        namedItem: markNative((name) => {
          for (let index = 0; index < pluginArray.length; index += 1) {
            if (pluginArray[index] && pluginArray[index].name === name) {
              return pluginArray[index]
            }
          }
          return null
        }, 'namedItem'),
        refresh: markNative(() => {}, 'refresh'),
        [Symbol.iterator]: markNative(function* () {
          for (let index = 0; index < pluginArray.length; index += 1) {
            yield pluginArray[index]
          }
        }, 'values')
      }

      const mimeTypes = {
        0: pluginTemplate[0],
        length: 1,
        item: markNative(() => pluginTemplate[0], 'item'),
        namedItem: markNative(() => pluginTemplate[0], 'namedItem'),
        [Symbol.iterator]: markNative(function* () {
          yield pluginTemplate[0]
        }, 'values')
      }

      defineGetter(navigatorProto || navigator, 'plugins', () => pluginArray)
      defineGetter(navigator, 'plugins', () => pluginArray)
      defineGetter(navigatorProto || navigator, 'mimeTypes', () => mimeTypes)
      defineGetter(navigator, 'mimeTypes', () => mimeTypes)
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
            uaFullVersion: '131.0.0.0',
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

      defineGetter(navigatorProto || navigator, 'userAgentData', () => uaData)
      defineGetter(navigator, 'userAgentData', () => uaData)
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
      defineGetter(navigatorProto || navigator, 'connection', () => connection)
      defineGetter(navigator, 'connection', () => connection)
    } catch (_) {}

    try {
      const chromeMock = {
        runtime: {
          connect: markNative(() => {}, 'connect'),
          sendMessage: markNative(() => {}, 'sendMessage'),
          onMessage: {
            addListener: markNative(() => {}, 'addListener'),
            removeListener: markNative(() => {}, 'removeListener')
          },
          onInstalled: {
            addListener: markNative(() => {}, 'addListener')
          }
        },
        loadTimes: markNative(() => ({}), 'loadTimes'),
        csi: markNative(() => ({}), 'csi'),
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
        }
      }

      defineGetter(window, 'chrome', () => chromeMock)
    } catch (_) {}

    try {
      if (navigator.permissions && typeof navigator.permissions.query === 'function') {
        const originalQuery = navigator.permissions.query.bind(navigator.permissions)
        const patchedQuery = markNative((parameters) => {
          if (parameters && parameters.name === 'notifications') {
            return Promise.resolve({
              state: Notification.permission,
              onchange: null
            })
          }

          return originalQuery(parameters)
        }, 'query')

        defineValue(navigator.permissions, 'query', patchedQuery)
      }
    } catch (_) {}

    try {
      defineGetter(window, 'outerWidth', () =>
        Math.max(window.innerWidth, screen.availWidth || 1440)
      )
      defineGetter(window, 'outerHeight', () =>
        Math.max(window.innerHeight, (screen.availHeight || 900) - 24)
      )
    } catch (_) {}

    try {
      defineGetter(screen, 'colorDepth', () => 24)
      defineGetter(screen, 'pixelDepth', () => 24)
    } catch (_) {}

    if (isGoogleSurface) {
      patchGoogleSurface()
    }

    if (isOpenAISurface) {
      patchOpenAISurface()
    }

    patchWebGL(window.WebGLRenderingContext)
    patchWebGL(window.WebGL2RenderingContext)

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

  const inject = () => {
    const mountPoint = document.head || document.documentElement
    if (!mountPoint) {
      setTimeout(inject, 10)
      return
    }

    try {
      const script = document.createElement('script')
      script.textContent = `;(${runStealthPatches.toString()})()`
      mountPoint.appendChild(script)
      script.remove()
    } catch (_) {}
  }

  inject()
})()
