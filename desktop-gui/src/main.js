// [LINE BUDGET AUDIT] target: ≤ 250 lines
// StreamDeck Clone — Desktop Configurator frontend (Vanilla JS + Tauri API)
// Connects to host-desktop via WebSocket; renders a drag-and-drop key grid.

import './style.css'
import { invoke } from '@tauri-apps/api/core'

// ─── State ────────────────────────────────────────────────────────────────────
const HOST_WS_URL = 'ws://127.0.0.1:4455'

const HOTKEY_ACTIONS = [
  { id: 'ctrl+c', label: 'Copy', icon: '📋' }, { id: 'ctrl+v', label: 'Paste', icon: '📌' },
  { id: 'ctrl+z', label: 'Undo', icon: '↩' }, { id: 'ctrl+s', label: 'Save', icon: '💾' },
  { id: 'ctrl+w', label: 'Close Tab', icon: '✕' }, { id: 'f5', label: 'Refresh', icon: '🔄' },
  { id: 'vol_up', label: 'Volume +', icon: '🔊' }, { id: 'vol_down', label: 'Volume -', icon: '🔉' },
  { id: 'mute', label: 'Mute', icon: '🔇' },
]

const PROFILES = ['Default', 'Browser', 'VSCode', 'OBS', 'VisualStudio']

let state = {
  ws: null, connected: false, selectedKey: null, activeProfile: 'Default',
  keys: Array.from({ length: 9 }, (_, i) => ({ id: i, label: '', action: '', icon: '⬜' })),
}

// ─── DOM refs ─────────────────────────────────────────────────────────────────
const app = document.getElementById('app')

// ─── Render ───────────────────────────────────────────────────────────────────
function render() {
  app.innerHTML = `
    ${renderHeader()}
    <div class="layout">
      ${renderSidebar()}
      ${renderGridArea()}
      ${renderInspector()}
    </div>
    ${renderConnectionBar()}
  `
  bindEvents()
}

let net = { wifi_name: null, ip: '127.0.0.1', port: 4455 }
invoke('get_network_info')
  .then(info => { if (info) { net = info; render() } })
  .catch(() => {
    invoke('get_local_ip').then(ip => { if (ip) { net.ip = ip; render() } }).catch(() => {})
  })

let copied = false
function copyIp() {
  navigator.clipboard.writeText(`${net.ip}:${net.port}`).catch(() => {})
  copied = true
  render()
  setTimeout(() => { copied = false; render() }, 1500)
}

function renderHeader() {
  const cls = state.connected ? 'connected' : ''
  const txt = state.connected ? 'Engine Online' : 'Connecting...'
  const target = `${net.ip}:${net.port}`
  const wifiPrefix = net.wifi_name ? `Wi-Fi: "${net.wifi_name}" • ` : 'Network: '
  const displayLabel = copied ? 'Copied!' : `${wifiPrefix}${target}`
  return `
    <header>
      <div style="display:flex;align-items:center;gap:12px">
        <h1>🎛 StreamDeck Configurator</h1>
        <div class="ip-badge" id="btn-copy-ip" title="Click to copy host address for Phone" style="cursor:pointer">
          <span><strong>${displayLabel}</strong></span>
        </div>
      </div>
      <div style="display:flex;align-items:center;gap:8px">
        <span style="font-size:12px;color:var(--text-secondary)">${txt}</span>
        <span class="status-dot ${cls}" title="${txt}"></span>
      </div>
    </header>`
}

function renderSidebar() {
  return `
    <aside class="sidebar">
      <h2>Profiles</h2>
      ${PROFILES.map(p => `
        <button class="profile-btn ${p === state.activeProfile ? 'active' : ''}"
                data-profile="${p}">${p}</button>`).join('')}
    </aside>`
}

function renderGridArea() {
  return `
    <main class="grid-area">
      <h2>Key Layout — ${state.activeProfile}</h2>
      <div class="key-grid">
        ${state.keys.map(k => renderKeyCard(k)).join('')}
      </div>
    </main>`
}

function renderKeyCard(k) {
  const sel = k.id === state.selectedKey ? 'selected' : ''
  const lbl = k.label || `Key ${k.id + 1}`
  const act = k.action || '—'
  return `
    <div class="key-card ${sel}" data-key="${k.id}">
      <span class="key-icon">${k.icon}</span>
      <span class="key-label">${lbl}</span>
      <span class="key-action">${act}</span>
    </div>`
}

function renderInspector() {
  const k = state.keys.find(x => x.id === state.selectedKey)
  if (!k) {
    return `<aside class="inspector"><h2>Inspector</h2>
      <p style="color:var(--text-secondary);font-size:12px">Select a key to configure it.</p>
    </aside>`
  }

  const options = HOTKEY_ACTIONS.map(a =>
    `<option value="${a.id}" ${k.action === a.id ? 'selected' : ''}>${a.icon} ${a.label}</option>`
  ).join('')

  return `
    <aside class="inspector">
      <h2>Key ${k.id + 1}</h2>
      <div class="field">
        <label>Label</label>
        <input id="inp-label" type="text" value="${k.label}" placeholder="Key ${k.id + 1}" />
      </div>
      <div class="field">
        <label>Action</label>
        <select id="inp-action">${options}</select>
      </div>
      <div class="field">
        <label>Icon</label>
        <input id="inp-icon" type="text" value="${k.icon}" placeholder="emoji or URL" />
      </div>
      <button class="btn btn-primary" id="btn-save">Save</button>
      <button class="btn btn-outline" id="btn-clear">Clear</button>
    </aside>`
}

function renderConnectionBar() {
  const addr = state.ws?.url ?? HOST_WS_URL
  const btnLabel = state.connected ? 'Disconnect' : 'Connect'
  const btnCls   = state.connected ? 'btn-danger' : 'btn-primary'
  return `
    <div class="connection-bar">
      <span class="connection-label">Host</span>
      <input id="inp-host" type="text" value="${addr}" placeholder="ws://127.0.0.1:4455" />
      <button class="btn ${btnCls}" id="btn-connect">${btnLabel}</button>
    </div>`
}

// ─── Events ───────────────────────────────────────────────────────────────────
function bindEvents() {
  document.querySelectorAll('.key-card').forEach(el => {
    el.addEventListener('click', () => { state.selectedKey = +el.dataset.key; render() })
  })

  document.querySelectorAll('.profile-btn').forEach(el => {
    el.addEventListener('click', () => { state.activeProfile = el.dataset.profile; render() })
  })

  document.getElementById('btn-save')?.addEventListener('click', saveKey)
  document.getElementById('btn-clear')?.addEventListener('click', clearKey)
  document.getElementById('btn-connect')?.addEventListener('click', toggleConnection)
  document.getElementById('btn-copy-ip')?.addEventListener('click', copyIp)
}

function saveKey() {
  const k = state.keys.find(x => x.id === state.selectedKey)
  if (!k) return
  k.label  = document.getElementById('inp-label').value.trim()
  k.action = document.getElementById('inp-action').value
  k.icon   = document.getElementById('inp-icon').value.trim() || '⬜'
  sendLayout()
  render()
}

function clearKey() {
  const k = state.keys.find(x => x.id === state.selectedKey)
  if (!k) return
  k.label = ''; k.action = ''; k.icon = '⬜'
  sendLayout()
  render()
}

// ─── WebSocket ────────────────────────────────────────────────────────────────
function toggleConnection() {
  if (state.connected) { disconnect() } else { connect() }
}

function connect() {
  const url = document.getElementById('inp-host')?.value ?? HOST_WS_URL
  state.ws = new WebSocket(url)
  state.ws.binaryType = 'arraybuffer'

  state.ws.onopen = () => {
    state.connected = true
    sendLayout()
    render()
  }
  state.ws.onclose = () => {
    state.connected = false; state.ws = null; render()
    setTimeout(() => { if (!state.connected) connect() }, 2000)
  }
  state.ws.onerror = () => { state.connected = false; state.ws = null; render() }
  state.ws.onmessage = (ev) => handleMessage(ev.data)
}

function disconnect() {
  state.ws?.close()
  state.ws = null
  state.connected = false
  render()
}

function sendLayout() {
  if (!state.ws || state.ws.readyState !== WebSocket.OPEN) return
  const layout = {
    type: 'layout_update',
    keys: state.keys.map(k => ({
      id: k.id, label: k.label || `Key ${k.id + 1}`, icon: k.icon, action: k.action,
    })),
  }
  state.ws.send(JSON.stringify(layout))
}

function handleMessage(data) {
  try {
    const msg = typeof data === 'string' ? JSON.parse(data) : null
    if (msg?.type === 'profile_switch') {
      state.activeProfile = msg.profile ?? state.activeProfile
      render()
    }
  } catch (_) { /* binary protobuf frames */ }
}

// ─── Boot ─────────────────────────────────────────────────────────────────────
render()
connect()
