import { PROFILES, escapeHtml } from '../constants.js'
import { state } from '../state.js'
import { renderKeyIcon } from '../icons.js'

export function renderGridArea() {
  const hint = state.moveSource !== null
    ? 'Choose where to move this button. Press Esc to cancel.'
    : 'Click a button to change it.'
  return `
    <main class="grid-area" role="main">
      <div class="grid-header">
        <span class="eyebrow">STEP 2 · MAKE IT YOURS</span>
        <h2>Your buttons</h2>
        <p class="move-hint" role="status" aria-live="polite">${hint}</p>
        <label class="profile-picker" for="profile-select">Set of buttons
          <select id="profile-select" aria-label="Set of buttons">
            ${PROFILES.map(p => `<option value="${p}" ${p === state.activeProfile ? 'selected' : ''}>${p}</option>`).join('')}
          </select>
        </label>
      </div>
      <div class="key-grid" role="grid" aria-label="Your nine buttons">
        ${state.keys.map(renderKeyCard).join('')}
      </div>
      <p class="grid-footnote">Tip: drag buttons to swap their places.</p>
    </main>`
}

function renderKeyCard(k) {
  const sel = k.id === state.selectedKey ? 'selected' : ''
  const mov = k.id === state.moveSource ? 'moving-source' : ''
  const label = escapeHtml(k.label || 'Add shortcut')
  const action = escapeHtml(k.action || 'Empty')
  const draggable = state.connected && !state.pending
  return `
    <div class="key-card ${sel} ${mov}" data-key="${k.id}"
         draggable="${draggable}" tabindex="0" role="gridcell"
         aria-selected="${k.id === state.selectedKey}"
         aria-label="Button ${k.id + 1}: ${label}, action: ${action}. Press Enter to edit or M to move">
      <div class="key-icon">${renderKeyIcon(k.icon, k.action)}</div>
      <span class="key-label">${label}</span>
      <span class="key-number">${k.id + 1}</span>
    </div>`
}
