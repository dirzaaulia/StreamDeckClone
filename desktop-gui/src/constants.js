// [LINE BUDGET AUDIT] 42/250
// StreamDeck Clone — Desktop Configurator Constants

export const HOST_WS_URL = 'ws://127.0.0.1:4455'

export const HOTKEY_ACTIONS = [
  { id: 'ctrl_c', label: 'Copy', iconKey: 'copy' },
  { id: 'ctrl_v', label: 'Paste', iconKey: 'paste' },
  { id: 'ctrl_z', label: 'Undo', iconKey: 'undo' },
  { id: 'ctrl_s', label: 'Save', iconKey: 'save' },
  { id: 'ctrl_w', label: 'Close Tab', iconKey: 'close' },
  { id: 'f5', label: 'Refresh', iconKey: 'refresh' },
  { id: 'vol_up', label: 'Volume +', iconKey: 'volUp' },
  { id: 'vol_down', label: 'Volume -', iconKey: 'volDown' },
  { id: 'vol_mute', label: 'Mute', iconKey: 'mute' },
  { id: 'media_play_pause', label: 'Play / Pause', iconKey: 'playPause' },
  { id: 'media_prev', label: 'Previous Track', iconKey: 'prev' },
  { id: 'media_next', label: 'Next Track', iconKey: 'next' },
  { id: 'desktop', label: 'Show Desktop', iconKey: 'desktop' },
  { id: 'taskmgr', label: 'Task Manager', iconKey: 'taskmgr' },
  { id: 'screenshot', label: 'Screenshot', iconKey: 'screenshot' },
]

export const PROFILES = ['Default', 'Browser', 'VSCode', 'OBS', 'VisualStudio']

export const emptyKeys = () => Array.from({ length: 9 }, (_, i) => ({
  id: i,
  label: '',
  action: '',
  icon: '⬜',
}))

export function escapeHtml(value) {
  return String(value ?? '').replace(/[&<>"']/g, c => ({
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  })[c])
}
