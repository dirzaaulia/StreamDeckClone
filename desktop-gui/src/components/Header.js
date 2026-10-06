import { state } from '../state.js'
import { getIconSvg } from '../icons.js'

export function renderHeader() {
  const status = state.connected ? 'Ready for your phone' :
    state.hostRunning ? 'Getting things ready…' : 'Not ready yet'
  return `
    <header class="fluent-header" role="banner">
      <div class="header-left">
        <span class="header-logo" aria-hidden="true">${getIconSvg('grid')}</span>
        <h1>StreamDeck</h1>
      </div>
      <div class="header-right" role="status" aria-live="polite">
        <span class="status-dot ${state.connected ? 'connected' : ''}" aria-hidden="true"></span>
        <span class="status-text">${status}</span>
      </div>
    </header>`
}
