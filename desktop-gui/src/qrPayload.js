// [LINE BUDGET AUDIT] target: ≤ 50 lines
// Formats pairing code and host network info for Android QR code scanner parsing.
// Target payload spec: streamdeck-pair:v2:<ip>:<port>:<6-digit-code>:<sha256-fingerprint>

export function formatPairingQrPayload(ip, port, code, fingerprint) {
  if (!ip || !port || !code || !fingerprint) return null
  const cleanIp = String(ip).trim()
  const cleanCode = String(code).trim()
  const numPort = Number(port)

  if (!/^\d{1,3}(\.\d{1,3}){3}$/.test(cleanIp)) return null
  if (cleanIp === '0.0.0.0') return null
  if (!/^\d{6}$/.test(cleanCode)) return null
  if (!Number.isInteger(numPort) || numPort <= 0 || numPort > 65535) return null
  const cleanFingerprint = String(fingerprint).trim().toLowerCase()
  if (!/^[0-9a-f]{64}$/.test(cleanFingerprint)) return null

  return `streamdeck-pair:v2:${cleanIp}:${numPort}:${cleanCode}:${cleanFingerprint}`
}
