/**
 * 快捷键设置弹窗入口。
 *
 * 读写走 Rust 命令：`get_shortcut_dialog_data` / `save_shortcut` /
 * `remove_shortcut_history` / `close_shortcut_window`。
 * 打开期间 Rust 侧会注销全局快捷键，窗口销毁时自动恢复。
 */
import './dialog.css'
import './shortcut.css'
import { invoke } from '@tauri-apps/api/core'
import { mountHistoryList, readText, requireElement } from './common'

/** 无修饰键按下时的按键名，视为尚未构成快捷键 */
const MODIFIER_KEYS = new Set(['CONTROL', 'META', 'ALT', 'SHIFT'])

/**
 * 把快捷键的内部写法转换成展示文案
 *
 * 例：`CommandOrControl+G` → `Cmd/Ctrl + G`、`Alt+Space` → `Option/Alt + Space`
 *
 * @param value - 内部写法的快捷键字符串
 * @returns 面向展示的文案
 */
function formatShortcut(value: string): string {
  return value
    .replaceAll('CommandOrControl', 'Cmd/Ctrl')
    .replaceAll('Alt', 'Option/Alt')
    .replaceAll('+', ' + ')
}

/**
 * 把键盘事件归一化为快捷键的内部写法
 *
 * Ctrl 与 Meta 合并为 `CommandOrControl`（跨平台语义一致）；
 * 仅按修饰键、或未按任何修饰键时返回 null，表示不构成有效快捷键。
 *
 * @param event - 键盘事件
 * @returns 形如 `CommandOrControl+Shift+K` 的字符串；无效组合返回 null
 */
function normalizeKey(event: KeyboardEvent): string | null {
  const modifiers: string[] = []
  if (event.ctrlKey || event.metaKey) {
    modifiers.push('CommandOrControl')
  }
  if (event.altKey) {
    modifiers.push('Alt')
  }
  if (event.shiftKey) {
    modifiers.push('Shift')
  }

  const upper = event.key.toUpperCase()
  if (MODIFIER_KEYS.has(upper) || !modifiers.length) {
    return null
  }

  const finalKey = event.key === ' ' ? 'Space' : upper
  return `${modifiers.join('+')}+${finalKey}`
}

/**
 * 弹窗初始化：回填本地化文案与当前快捷键、渲染历史列表，
 * 并绑定录入（keydown）、保存/取消/恢复默认事件
 *
 * @throws invoke 不可用或调用失败时 reject，由底部的统一 catch 弹窗提示
 */
async function init(): Promise<void> {
  const display = requireElement('shortcut-display')
  const hintEl = requireElement('hint-text')
  const historyLabel = requireElement('history-label')
  const listEl = requireElement('history-list')
  const resetBtn = requireElement<HTMLButtonElement>('reset-btn')
  const cancelBtn = requireElement<HTMLButtonElement>('cancel-btn')
  const saveBtn = requireElement<HTMLButtonElement>('save-btn')

  hintEl.textContent = readText('hint', '按下按键组合以设置快捷键')
  historyLabel.textContent = readText('historyLabel', '最近使用')
  resetBtn.textContent = readText('resetText', '恢复默认')
  cancelBtn.textContent = readText('cancelText', '取消')
  saveBtn.textContent = readText('okText', '保存')

  const state = await invoke<{
    current: string
    defaultShortcut: string
    history: string[]
  }>('get_shortcut_dialog_data')
  let currentShortcut = state.current || readText('current', '')
  const defaultShortcut = state.defaultShortcut || 'CommandOrControl+G'
  let history = Array.isArray(state.history) ? [...state.history] : []

  /**
   * 刷新展示区：普通态展示当前快捷键，录入态高亮边框
   */
  const updateDisplay = (value: string, recording = false) => {
    display.classList.toggle('recording', recording)
    display.classList.toggle('has-value', Boolean(value))
    display.textContent = value ? formatShortcut(value) : '请录入...'
  }

  const historyList = mountHistoryList(listEl, {
    emptyText: readText('emptyHistory', '暂无历史记录'),
    deleteText: readText('deleteText', '删除'),
    onPick: (value) => {
      currentShortcut = value
      updateDisplay(currentShortcut)
    },
    onDelete: async (value) => {
      await invoke('remove_shortcut_history', { shortcut: value })
      history = history.filter((item) => item !== value)
      historyList.setItems(history)
    }
  })

  cancelBtn.onclick = () => invoke('close_shortcut_window').catch(console.error)

  resetBtn.onclick = () => {
    currentShortcut = defaultShortcut
    updateDisplay(currentShortcut)
  }

  saveBtn.onclick = async () => {
    try {
      await invoke('save_shortcut', { shortcut: currentShortcut })
      await invoke('close_shortcut_window')
    } catch (error) {
      alert(`保存失败：${error}`)
    }
  }

  window.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      cancelBtn.click()
      return
    }

    const nextShortcut = normalizeKey(event)
    if (!nextShortcut) {
      if (['Control', 'Meta', 'Alt', 'Shift'].includes(event.key)) {
        event.preventDefault()
        updateDisplay(currentShortcut, true)
      }
      return
    }

    event.preventDefault()
    currentShortcut = nextShortcut
    updateDisplay(currentShortcut)
  })

  updateDisplay(currentShortcut)
  historyList.setItems(history)
}

init().catch((error) => {
  alert(`无法连接到程序核心服务：${error}`)
})
