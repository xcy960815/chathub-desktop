import './dialog.css'
import './shortcut.css'
import { invoke } from '@tauri-apps/api/core'
import { mountHistoryList, readText, requireElement } from './common'

const MODIFIER_KEYS = new Set(['CONTROL', 'META', 'ALT', 'SHIFT'])

function formatShortcut(value: string): string {
  return value
    .replaceAll('CommandOrControl', 'Cmd/Ctrl')
    .replaceAll('Alt', 'Option/Alt')
    .replaceAll('+', ' + ')
}

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
