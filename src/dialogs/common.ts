/**
 * 设置弹窗（代理 / 快捷键）共享工具。
 *
 * 两个弹窗页面均为 Vite 多入口构建的产物，Rust 侧通过
 * `WebviewUrl::App("proxy.html")` 打开，并用 URL 查询参数传入
 * 本地化文案；本模块承载两者共用的逻辑。
 */

/**
 * 历史列表的交互回调与文案
 */
export interface HistoryListOptions {
  /** 列表为空时展示的占位文案 */
  emptyText: string
  /** 每行删除按钮的文案 */
  deleteText: string
  /** 点击某条历史时的回调，用于把该条记录回填到输入框/展示区 */
  onPick: (value: string) => void
  /** 点击删除按钮时的回调；删除后由调用方调用 setItems 重绘 */
  onDelete: (value: string) => void | Promise<void>
}

/**
 * 历史列表控制器
 */
export interface HistoryListController {
  /** 用新的数据整表重绘 */
  setItems(items: string[]): void
}

/**
 * 读取当前页面 URL 查询参数中的文案，缺失或为空时返回默认值
 *
 * @param name - 查询参数名
 * @param fallback - 参数缺失时使用的默认文案
 * @returns 参数值或默认文案
 */
export function readText(name: string, fallback: string): string {
  return new URLSearchParams(window.location.search).get(name) || fallback
}

/**
 * 按 id 查找元素，找不到时抛出明确错误
 *
 * 避免后续在 null 上取属性时得到含糊的运行时报错。
 *
 * @typeParam T - 期望的元素类型
 * @param id - 元素 id
 * @returns 对应元素
 * @throws 页面缺少该元素时抛出 Error
 */
export function requireElement<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id)
  if (!element) {
    throw new Error(`页面缺少元素 #${id}`)
  }
  return element as T
}

/**
 * 挂载设置弹窗共用的历史记录列表
 *
 * 使用 DOM API（createElement/textContent）渲染用户数据，
 * 不经 innerHTML 拼接，避免注入风险。
 *
 * @param container - 列表容器元素
 * @param options - 文案与交互回调
 * @returns 控制器，调用 setItems 更新列表
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
