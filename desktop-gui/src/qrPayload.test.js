import { test } from 'node:test'
import assert from 'node:assert/strict'
import { formatPairingQrPayload } from './qrPayload.js'

const fingerprint = 'ab'.repeat(32)

test('formats QR v2 with certificate fingerprint', () => {
  assert.equal(
    formatPairingQrPayload('192.168.1.10', 4455, '123456', fingerprint),
    `streamdeck-pair:v2:192.168.1.10:4455:123456:${fingerprint}`
  )
  assert.equal(
    formatPairingQrPayload('10.0.0.5', '4455', '001234', fingerprint.toUpperCase()),
    `streamdeck-pair:v2:10.0.0.5:4455:001234:${fingerprint}`
  )
})

test('rejects invalid inputs for QR payload', () => {
  assert.equal(formatPairingQrPayload('', 4455, '123456', fingerprint), null)
  assert.equal(formatPairingQrPayload('0.0.0.0', 4455, '123456', fingerprint), null)
  assert.equal(formatPairingQrPayload('invalid-ip', 4455, '123456', fingerprint), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 4455, '12345', fingerprint), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 4455, '1234567', fingerprint), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 4455, 'abcdef', fingerprint), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 0, '123456', fingerprint), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 70000, '123456', fingerprint), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 4455, '123456', ''), null)
  assert.equal(formatPairingQrPayload('192.168.1.1', 4455, '123456', 'a'.repeat(63)), null)
})
