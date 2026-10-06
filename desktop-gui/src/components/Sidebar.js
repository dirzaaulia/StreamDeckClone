import { escapeHtml } from '../constants.js'
import { state, net, pairingQr } from '../state.js'

export function renderSidebar() {
  const devices = state.pairedDevices.map(id => `
    <div class="paired-device">
      <span class="device-name">${escapeHtml(id)}</span>
      <button class="btn btn-outline btn-sm" data-revoke="${escapeHtml(id)}"
              aria-label="Remove device ${escapeHtml(id)}">Remove</button>
    </div>`).join('')
  const pairing = state.pairingCode ? `
    <div class="pairing-details" role="region" aria-label="Pairing code">
      <p>Enter this code on your phone</p>
      <strong class="pairing-code">${escapeHtml(state.pairingCode)}</strong>
      ${pairingQr ? `<img class="pairing-qr" src="${pairingQr}" alt="QR code to pair your phone" />` : ''}
      <small>Code expires in 5 minutes. Compare the certificate fingerprint on your phone before pairing.</small>
    </div>` : ''
  const address = escapeHtml(`${net.ip}:${net.port}`)
  return `
    <aside class="sidebar" aria-label="Phone setup">
      <section class="sidebar-section welcome">
        <span class="eyebrow">YOUR SETUP</span>
        <h2>Make your phone your shortcut pad.</h2>
        <p>Choose a button, pick what it does, then connect your phone.</p>
      </section>
      <section class="sidebar-section pair-card">
        <span class="step-number">1</span>
        <h2>Connect your phone</h2>
        <p>Open StreamDeck on your phone and use the pairing code.</p>
        <button class="btn btn-primary btn-full" id="btn-pair" ${state.connected ? '' : 'disabled'}>
          ${state.pairingCode ? 'Get a new code' : 'Pair my phone'}
        </button>
        ${pairing}
        ${devices ? `<details class="device-details"><summary>Connected phones (${state.pairedDevices.length})</summary>${devices}</details>` : ''}
      </section>
      <details class="advanced-options">
        <summary>Connection details</summary>
        <p>Phone can't find this PC? Use this address:</p>
        <button class="ip-badge" id="btn-copy-ip" title="Copy PC address">${address} · Copy</button>
        <p>Certificate SHA-256 fingerprint:</p>
        <code class="fingerprint">${state.fingerprint ? escapeHtml(state.fingerprint) : 'Start the host to see its fingerprint'}</code>
        ${net.wifi_name ? `<p>Wi-Fi: ${escapeHtml(net.wifi_name)}</p>` : ''}
      </details>
    </aside>`
}
