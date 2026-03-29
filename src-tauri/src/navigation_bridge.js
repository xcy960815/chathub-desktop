;(function () {
  if (window.__CHATHUB_URL_TRACKER__) {
    return
  }

  window.__CHATHUB_URL_TRACKER__ = true

  const notify = () => {
    const iframe = document.createElement('iframe')
    iframe.style.display = 'none'
    iframe.src = `chathub://url-change?value=${encodeURIComponent(window.location.href)}`
    document.documentElement.appendChild(iframe)
    setTimeout(() => iframe.remove(), 0)
  }

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
