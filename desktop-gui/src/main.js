// [LINE BUDGET AUDIT] 178/250
// StreamDeck Clone — Desktop Configurator Frontend Main Entry

import './style.css'
import { invoke } from '@tauri-apps/api/core'
import QRCode from 'qrcode'

import { state, net, pairingQr, setPairingQr } from './state.js'
import { emptyKeys, HOTKEY_ACTIONS } from './constants.js'
import { swapKeyContents } from './keyMove.js'
import { formatPairingQrPayload } from './qrPayload.js'
import { controlRequest, controlDisconnect } from './controlClient.js'

import { renderHeader } from './components/Header.js'
import { renderSidebar } from './components/Sidebar.js'
import { renderGridArea } from './components/GridArea.js'
import { renderInspector } from './components/Inspector.js'
import { renderConnectionBar } from './components/ConnectionBar.js'

const app = document.getElementById('app')
state.keys = emptyKeys()

// ─── Rendering ────────────────────────────────────────────────────────────────
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

async function updatePairingQr() {
  const payload = formatPairingQrPayload(net.ip, net.port, state.pairingCode, state.fingerprint)
  if (!payload) { setPairingQr(''); render(); return }
  try {
    const qrUrl = await QRCode.toDataURL(payload, { margin: 1, width: 180 })
    setPairingQr(qrUrl)
  } catch (_) { setPairingQr('') }
  render()
}

invoke('host_fingerprint').then(fingerprint => { state.fingerprint = fingerprint; if (state.pairingCode) updatePairingQr(); else render() }).catch(() => {})
invoke('get_network_info')
  .then(info => { if (info) { Object.assign(net, info); if (state.pairingCode) updatePairingQr(); else render() } })
  .catch(() => {
    invoke('get_local_ip').then(ip => { if (ip) { net.ip = ip; if (state.pairingCode) updatePairingQr(); else render() } }).catch(() => {})
  })

function copyIp() {
  navigator.clipboard.writeText(`${net.ip}:${net.port}`).then(() => {
    const button = document.getElementById('btn-copy-ip')
    if (button) button.textContent = 'Copied!'
  }).catch(() => { state.error = 'Could not copy the address'; render() })
}

// ─── DOM Events ───────────────────────────────────────────────────────────────
function bindEvents() {
  bindKeyCardEvents()
  document.getElementById('profile-select')?.addEventListener('change', el => {
    state.moveSource = null; state.activeProfile = el.target.value; state.keys = emptyKeys()
    requestProfile(); render()
  })
  document.getElementById('inp-action')?.addEventListener('change', ev => {
    const choice = HOTKEY_ACTIONS.find(action => action.id === ev.target.value)
    const label = document.getElementById('inp-label')
    if (choice && label && !label.value.trim()) label.value = choice.label
  })
  document.getElementById('btn-save')?.addEventListener('click', saveKey)
  document.getElementById('btn-clear')?.addEventListener('click', clearKey)
  document.getElementById('btn-connect')?.addEventListener('click', toggleConnection)
  document.getElementById('btn-host')?.addEventListener('click', toggleHost)
  document.getElementById('btn-copy-ip')?.addEventListener('click', copyIp)
  document.getElementById('btn-pair')?.addEventListener('click', () => sendControl({ type: 'start_pairing' }))
  document.querySelectorAll('[data-revoke]').forEach(el => el.addEventListener('click', () => {
    sendControl({ type: 'revoke_device', device_id: el.dataset.revoke })
  }))
}

function bindKeyCardEvents() {
  document.querySelectorAll('.key-card').forEach(el => {
    el.addEventListener('click', () => {
      const id = +el.dataset.key
      if (state.moveSource !== null) { moveKey(state.moveSource, id); return }
      state.selectedKey = id; render()
    })
    el.addEventListener('keydown', ev => {
      if (ev.key.toLowerCase() === 'm' && state.connected && !state.pending) {
        ev.preventDefault(); state.moveSource = +el.dataset.key; state.selectedKey = state.moveSource; render()
      } else if (ev.key === 'Escape') {
        state.moveSource = null; render()
      } else if (ev.key === 'Enter' || ev.key === ' ') {
        ev.preventDefault(); el.click()
      }
    })
    bindDragEvents(el)
  })
}

function bindDragEvents(el) {
  el.addEventListener('dragstart', ev => {
    if (!state.connected || state.pending) { ev.preventDefault(); return }
    state.moveSource = +el.dataset.key
    ev.dataTransfer.effectAllowed = 'move'
    ev.dataTransfer.setData('text/plain', String(state.moveSource))
    el.classList.add('dragging')
  })
  el.addEventListener('dragover', ev => {
    if (state.moveSource === null || state.pending) return
    ev.preventDefault(); ev.dataTransfer.dropEffect = 'move'
    el.classList.add('drop-target')
  })
  el.addEventListener('dragleave', () => el.classList.remove('drop-target'))
  el.addEventListener('drop', ev => {
    ev.preventDefault(); el.classList.remove('drop-target')
    if (state.moveSource !== null) moveKey(state.moveSource, +el.dataset.key)
  })
  el.addEventListener('dragend', () => {
    state.moveSource = null
    document.querySelectorAll('.key-card').forEach(card => card.classList.remove('dragging', 'drop-target'))
  })
}

// ─── Actions ──────────────────────────────────────────────────────────────────
function moveKey(sourceId, targetId) {
  state.moveSource = null
  if (!state.connected || state.pending) return
  const keys = swapKeyContents(state.keys, sourceId, targetId)
  if (!keys) return
  state.selectedKey = targetId
  sendLayout(keys); render()
}

function saveKey() {
  const k = state.keys.find(x => x.id === state.selectedKey)
  if (!k || !state.connected || state.pending) return
  const keys = state.keys.map(key => key.id === k.id ? {
    ...key,
    label: document.getElementById('inp-label').value.trim(),
    action: document.getElementById('inp-action').value,
    icon: document.getElementById('inp-icon').value.trim() || '⬜',
  } : key)
  sendLayout(keys); render()
}

function clearKey() {
  const k = state.keys.find(x => x.id === state.selectedKey)
  if (!k || !state.connected || state.pending) return
  const keys = state.keys.map(key => key.id === k.id ? { ...key, label: '', action: '', icon: '⬜' } : key)
  sendLayout(keys); render()
}

// ─── WebSocket & Host Management ─────────────────────────────────────────────
async function refreshHostStatus() {
  try {
    const status = await invoke('host_status')
    if (state.hostRunning !== status.running || state.hostManaged !== status.managed) {
      state.hostRunning = status.running; state.hostManaged = status.managed
      render()
    }
  } catch (error) { state.error = String(error); render() }
}

async function toggleHost() {
  if (state.hostBusy || (state.hostRunning && !state.hostManaged)) return
  state.hostBusy = true; state.error = ''; render()
  try {
    if (state.hostRunning) {
      state.manualDisconnect = true; state.ws = null; state.connected = false
      await controlDisconnect()
      await invoke('stop_host')
    } else {
      await invoke('launch_host'); state.manualDisconnect = false; connect()
    }
  } catch (error) { state.error = String(error) }
  finally { state.hostBusy = false; await refreshHostStatus() }
}

function toggleConnection() {
  if (state.connected) { disconnect() } else { state.manualDisconnect = false; connect() }
}

async function sendControl(message) {
  if (!state.connected) return
  try { handleMessage(await controlRequest(message)) }
  catch (error) {
    state.error = String(error); state.connected = false; state.pending = false; render()
    if (!state.manualDisconnect) setTimeout(connect, 2000)
  }
}

async function connect() {
  if (state.ws || state.manualDisconnect) return
  const attempt = {}
  state.ws = attempt
  try {
    const [profile, devices, fingerprint] = await Promise.all([
      controlRequest({ type: 'get_profile', profile: state.activeProfile }),
      controlRequest({ type: 'list_devices' }),
      invoke('host_fingerprint'),
    ])
    if (state.manualDisconnect || state.ws !== attempt) return
    state.fingerprint = fingerprint
    state.connected = true; state.error = ''
    handleMessage(profile); handleMessage(devices)
  } catch (error) {
    if (state.ws !== attempt) return
    state.connected = false; state.error = String(error)
    if (!state.manualDisconnect) setTimeout(() => { if (!state.ws && !state.manualDisconnect && !state.connected) connect() }, 2000)
  } finally { if (state.ws === attempt) state.ws = null; render() }
}

function disconnect() {
  state.manualDisconnect = true; state.ws = null; state.connected = false
  state.pending = false; state.pendingProfile = null
  controlDisconnect().catch(() => {}); render()
}

function requestProfile() {
  sendControl({ type: 'get_profile', profile: state.activeProfile })
}

function sendLayout(keys) {
  if (!state.connected || state.pending || keys.length !== 9) return
  state.pending = true; state.pendingProfile = state.activeProfile; state.error = ''
  sendControl({
    type: 'save_profile', profile: state.activeProfile,
    keys: keys.map(k => ({ id: k.id, label: k.label, icon: k.icon, action: k.action })),
  })
}

function handleMessage(msg) {
  if (!msg || typeof msg !== 'object') return
  if (msg.type === 'pairing_code') {
    state.pairingCode = msg.code; updatePairingQr()
  } else if (msg.type === 'devices') {
    state.pairedDevices = msg.device_ids; render()
  } else if (msg.type === 'profile' && msg.profile === state.activeProfile) {
    state.keys = Array.isArray(msg.keys) && msg.keys.length === 9 ? msg.keys : emptyKeys()
    render()
  } else if (msg.type === 'saved' && msg.profile === state.pendingProfile) {
    state.pending = false; state.pendingProfile = null
    if (msg.profile === state.activeProfile) requestProfile()
    render()
  } else if (msg.type === 'error') {
    state.error = msg.message || 'Could not save profile'
    state.pending = false; state.pendingProfile = null
    if (state.connected) requestProfile()
    render()
  }
}

// ─── Boot ─────────────────────────────────────────────────────────────────────
render()
refreshHostStatus()
connect()
setInterval(refreshHostStatus, 3000)
