import { HOTKEY_ACTIONS, escapeHtml } from '../constants.js'
import { state } from '../state.js'

export function renderInspector() {
  const k = state.keys.find(x => x.id === state.selectedKey)
  if (!k) {
    return `<aside class="inspector" aria-label="Button editor">
      <span class="eyebrow">STEP 3 · PERSONALIZE</span>
      <h2>Pick a button to get started</h2>
      <p class="inspector-empty">Tap any button in the grid to give it a shortcut.</p>
    </aside>`
  }
  const options = [{ id: '', label: 'Choose a shortcut' }, ...HOTKEY_ACTIONS].map(a =>
    `<option value="${a.id}" ${k.action === a.id ? 'selected' : ''}>${a.label}</option>`
  ).join('')
  return `<aside class="inspector" aria-label="Edit button ${k.id + 1}">
    <span class="eyebrow">PERSONALIZE</span>
    <h2>Button ${k.id + 1}</h2>
    <p class="inspector-empty">What should this button do?</p>
    <div class="field">
      <label for="inp-action">Shortcut</label>
      <select id="inp-action">${options}</select>
    </div>
    <div class="field">
      <label for="inp-label">Name on button</label>
      <input id="inp-label" type="text" value="${escapeHtml(k.label)}" placeholder="e.g. Play music" />
    </div>
    <details class="advanced-options">
      <summary>Customize icon</summary>
      <div class="field">
        <label for="inp-icon">Emoji or image URL</label>
        <input id="inp-icon" type="text" value="${escapeHtml(k.icon)}" placeholder="🎵" />
      </div>
    </details>
    <div class="inspector-actions">
      <button class="btn btn-primary" id="btn-save" ${state.connected && !state.pending ? '' : 'disabled'}>Save button</button>
      <button class="btn btn-outline" id="btn-clear" ${state.connected && !state.pending ? '' : 'disabled'}>Clear</button>
    </div>
  </aside>`
}
