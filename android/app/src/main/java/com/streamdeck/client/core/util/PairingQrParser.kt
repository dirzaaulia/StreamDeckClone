// [LINE BUDGET AUDIT] target: ≤ 100 lines
package com.streamdeck.client.core.util

import com.streamdeck.client.R

data class PairingQrPayload(
    val host: String,
    val port: Int,
    val code: String,
    val fingerprint: String,
)

enum class QrParseError(val messageResId: Int) {
    EMPTY_PAYLOAD(R.string.deck_qr_error_empty),
    INVALID_FORMAT(R.string.deck_qr_error_format),
    UNSUPPORTED_VERSION(R.string.deck_qr_error_version),
    INVALID_IP(R.string.deck_qr_error_ip),
    UNTRUSTED_IP(R.string.deck_qr_error_untrusted_ip),
    INVALID_PORT(R.string.deck_qr_error_port),
    INVALID_CODE(R.string.deck_qr_error_code),
    INVALID_FINGERPRINT(R.string.deck_qr_error_fingerprint),
}

sealed interface QrParseResult {
    data class Success(val payload: PairingQrPayload) : QrParseResult
    data class Error(val error: QrParseError) : QrParseResult
}

object PairingQrParser {
    fun parse(rawInput: String?): QrParseResult {
        if (rawInput != null && rawInput.length > 160) return QrParseResult.Error(QrParseError.INVALID_FORMAT)
        val trimmed = rawInput?.trim().orEmpty()
        if (trimmed.isEmpty()) return QrParseResult.Error(QrParseError.EMPTY_PAYLOAD)
        if (!trimmed.startsWith("streamdeck-pair:")) return QrParseResult.Error(QrParseError.INVALID_FORMAT)
        val parts = trimmed.split(":")
        if (parts.size != 6) return QrParseResult.Error(QrParseError.INVALID_FORMAT)
        if (parts[1] != "v2") return QrParseResult.Error(QrParseError.UNSUPPORTED_VERSION)
        val octets = parseIPv4Octets(parts[2]) ?: return QrParseResult.Error(QrParseError.INVALID_IP)
        if (!isTrustedPrivateOrLoopbackIp(octets)) return QrParseResult.Error(QrParseError.UNTRUSTED_IP)
        val port = parts[3].toIntOrNull() ?: return QrParseResult.Error(QrParseError.INVALID_PORT)
        if (port !in 1..65535) return QrParseResult.Error(QrParseError.INVALID_PORT)
        if (parts[4].length != 6 || !parts[4].all { it in '0'..'9' }) {
            return QrParseResult.Error(QrParseError.INVALID_CODE)
        }
        if (!Regex("[0-9a-fA-F]{64}").matches(parts[5])) {
            return QrParseResult.Error(QrParseError.INVALID_FINGERPRINT)
        }
        return QrParseResult.Success(PairingQrPayload(parts[2], port, parts[4], parts[5].lowercase()))
    }

    private fun parseIPv4Octets(ipStr: String): IntArray? {
        val parts = ipStr.split(".")
        if (parts.size != 4) return null
        val octets = IntArray(4)
        for (i in 0..3) {
            val part = parts[i]
            if (part.isEmpty() || part.length > 3 || !part.all { it in '0'..'9' }) return null
            if (part.length > 1 && part.startsWith("0")) return null
            val value = part.toIntOrNull() ?: return null
            if (value !in 0..255) return null
            octets[i] = value
        }
        return octets
    }

    private fun isTrustedPrivateOrLoopbackIp(octets: IntArray): Boolean {
        val (o1, o2, _, _) = octets
        return when {
            o1 == 127 -> true
            o1 == 10 -> true
            o1 == 172 && o2 in 16..31 -> true
            o1 == 192 && o2 == 168 -> true
            o1 == 169 && o2 == 254 -> true
            else -> false
        }
    }
}
