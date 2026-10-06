import { invoke } from '@tauri-apps/api/core'

export async function controlRequest(message) {
  return invoke('control_request', { message })
}

export async function controlDisconnect() {
  return invoke('control_disconnect')
}
