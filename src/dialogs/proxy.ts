import './dialog.css'
import './proxy.css'
import { invoke } from '@tauri-apps/api/core'
import { mountHistoryList, readText, requireElement } from './common'

function validateProxy(value: string): boolean {
  if (!value) {
    return true
  }

  const candidate = value.includes('://') ? value : `http://${value}`

  try {
    const parsed = new URL(candidate)
    return Boolean(parsed.hostname && parsed.port)
  } catch {
    return false
  }
}

async function init(): Promise<void> {
  const input = requireElement<HTMLInputElement>('proxy-input')
  const hintEl = requireElement('hint-text')
  const historyLabel = requireElement('history-label')
  const listEl = requireElement('history-list')
  const clearBtn = requireElement<HTMLButtonElement>('clear-btn')
  const cancelBtn = requireElement<HTMLButtonElement>('cancel-btn')
  const saveBtn = requireElement<HTMLButtonElement>('save-btn')

  hintEl.textContent = readText('hint', '输入代理地址（如 socks5://127.0.0.1:7897）')
  historyLabel.textContent = readText('historyLabel', '最近使用')
  input.placeholder = readText('placeholder', '留空则禁用代理')
  clearBtn.textContent = readText('clearText', '清空')
  cancelBtn.textContent = readText('cancelText', '取消')
  saveBtn.textContent = readText('okText', '保存')

  const state = await invoke<{ current: string; history: string[] }>('get_proxy_dialog_data')
  let history = Array.isArray(state.history) ? [...state.history] : []
  input.value = state.current || readText('current', '')

  const historyList = mountHistoryList(listEl, {
    emptyText: readText('emptyHistory', '暂无历史记录'),
    deleteText: readText('deleteText', '删除'),
    onPick: (value) => {
      input.value = value
      input.focus()
    },
    onDelete: async (value) => {
      await invoke('remove_proxy_history', { proxy: value })
      history = history.filter((item) => item !== value)
      historyList.setItems(history)
    }
  })
  historyList.setItems(history)

  cancelBtn.onclick = () => invoke('close_proxy_window').catch(console.error)
  clearBtn.onclick = () => {
    input.value = ''
    input.focus()
  }

  saveBtn.onclick = async () => {
    const value = input.value.trim()
    if (!validateProxy(value)) {
      alert('代理地址格式无效，请输入类似 socks5://127.0.0.1:7897 的地址。')
      input.focus()
      return
    }

    try {
      await invoke('save_proxy', { proxy: value })
      await invoke('close_proxy_window')
    } catch (error) {
      alert(`保存失败：${error}`)
    }
  }

  input.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
      saveBtn.click()
    }
    if (event.key === 'Escape') {
      cancelBtn.click()
    }
  })

  input.focus()
  input.select()
}

init().catch((error) => {
  alert(`无法连接到程序核心服务：${error}`)
})
