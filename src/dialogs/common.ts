export interface HistoryListOptions {
  emptyText: string
  deleteText: string
  onPick: (value: string) => void
  onDelete: (value: string) => void | Promise<void>
}

export interface HistoryListController {
  setItems(items: string[]): void
}

export function readText(name: string, fallback: string): string {
  return new URLSearchParams(window.location.search).get(name) || fallback
}

export function requireElement<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id)
  if (!element) {
    throw new Error(`页面缺少元素 #${id}`)
  }
  return element as T
}

/**
 * 设置弹窗共用的历史记录列表，用 DOM API 渲染用户数据，避免 innerHTML 注入
 */
export function mountHistoryList(
  container: HTMLElement,
  options: HistoryListOptions
): HistoryListController {
  let items: string[] = []

  const render = () => {
    container.textContent = ''

    if (!items.length) {
      const empty = document.createElement('div')
      empty.className = 'history-empty'
      empty.textContent = options.emptyText
      container.appendChild(empty)
      return
    }

    for (const item of items) {
      const row = document.createElement('div')
      row.className = 'history-item'

      const value = document.createElement('div')
      value.className = 'history-value'
      value.textContent = item
      value.addEventListener('click', () => options.onPick(item))

      const remove = document.createElement('button')
      remove.className = 'history-delete'
      remove.type = 'button'
      remove.textContent = options.deleteText
      remove.addEventListener('click', () => {
        void Promise.resolve(options.onDelete(item))
      })

      row.append(value, remove)
      container.appendChild(row)
    }
  }

  return {
    setItems(next: string[]) {
      items = [...next]
      render()
    }
  }
}
