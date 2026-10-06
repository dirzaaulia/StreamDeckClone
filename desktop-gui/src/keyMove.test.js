import { test } from 'node:test'
import assert from 'node:assert/strict'
import { swapKeyContents } from './keyMove.js'

test('moves label, action and icon while keeping physical slot IDs', () => {
  const keys = [
    { id: 0, label: 'Mute', action: 'vol_mute', icon: 'M' },
    { id: 1, label: 'Copy', action: 'ctrl_c', icon: 'C' },
    { id: 2, label: 'Paste', action: 'ctrl_v', icon: 'P' },
  ]
  assert.deepEqual(swapKeyContents(keys, 0, 1), [
    { id: 0, label: 'Copy', action: 'ctrl_c', icon: 'C' },
    { id: 1, label: 'Mute', action: 'vol_mute', icon: 'M' },
    { id: 2, label: 'Paste', action: 'ctrl_v', icon: 'P' },
  ])
  assert.equal(keys[0].action, 'vol_mute')
  assert.equal(swapKeyContents(keys, 0, 0), null)
  assert.equal(swapKeyContents(keys, 0, 9), null)
  assert.equal(swapKeyContents(keys, -1, 1), null)
})
