import { escapeHtml } from '../constants.js'
import { state } from '../state.js'

export function renderConnectionBar() {
  const message = state.connected ? 'Everything is ready. Pair a phone to start using your buttons.' :
    state.hostRunning ? 'Connecting to your PC…' : 'Could not start the PC connection.'
  return `<footer class="connection-bar" role="contentinfo">
    <span class="connection-label" role="status">${message}</span>
    ${state.error ? `<span class="error-msg" role="alert">${escapeHtml(state.error)}</span>` : ''}
    <details class="advanced-options host-controls">
      <summary>Connection settings</summary>
      <div class="host-settings">
        <button class="btn btn-outline btn-sm" id="btn-host"
          ${state.hostBusy || (state.hostRunning && !state.hostManaged) ? 'disabled' : ''}>
          ${state.hostRunning ? 'Stop connection' : 'Retry starting connection'}
        </button>
        <button class="btn btn-outline btn-sm" id="btn-connect">
          ${state.connected ? 'Disconnect editor' : 'Reconnect editor'}
        </button>
      </div>
    </details>
  </footer>`
}
