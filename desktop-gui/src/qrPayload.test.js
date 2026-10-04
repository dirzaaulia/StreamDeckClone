import { test } from 'node:test'
import assert from 'node:assert/strict'
import { formatPairingQrPayload } from './qrPayload.js'

test('formats QR payload correctly for valid pairing code and IP', () => {
  assert.equal(
    formatPairingQrPayload('192.168.1.10', 4455, '123456'),
    'streamdeck-pair:v1:192.168.1.10:4455:123456'
  )
  assert.equal(
    formatPairingQrPayload('10.0.0.5', '4455', '001234'),
    'streamdeck-pair:v1:10.0.0.5:4455:001234'
  )
})

test('rejects invalid inputs for QR payload', () => {
  assert.equal(formatPairingQrPayload('', 4455, '123456'), null)
  assert.equal(formatPairingQrPayload('0.0.0.0', 4455, '123456'), null)
  assert.equal(formatPairingQrPayload('invalid-ip', 4455, '123456'), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 4455, '12345'), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 4455, '1234567'), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 4455, 'abcdef'), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 0, '123456'), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 70000, '123456'), null)
})
