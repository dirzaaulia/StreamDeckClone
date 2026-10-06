// [LINE BUDGET AUDIT] 35/250
// StreamDeck Configurator State Container

import { HOST_WS_URL } from './constants.js'

export const state = {
  ws: null,
  connected: false,
  selectedKey: null,
  activeProfile: 'Default',
  keys: [],
  error: '',
  pending: false,
  pendingProfile: null,
  manualDisconnect: false,
  hostUrl: HOST_WS_URL,
  pairingCode: '',
  fingerprint: '',
  pairedDevices: [],
  hostRunning: false,
  hostManaged: false,
  hostBusy: false,
  moveSource: null,
}

export const net = {
  wifi_name: null,
  ip: '127.0.0.1',
  port: 4455,
}

export let pairingQr = ''

export function setPairingQr(qr) {
  pairingQr = qr
}
