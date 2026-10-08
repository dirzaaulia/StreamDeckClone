package com.streamdeck.client.core.util

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class PairingQrParserTest {
    private val fingerprint = "ab".repeat(32)

    @Test fun privateAndLoopbackAddresses() {
        for (host in listOf("192.168.1.100", "127.0.0.1", "10.0.5.20", "172.16.0.1", "169.254.12.34", "100.71.216.2", "100.100.100.100")) {
            val result = PairingQrParser.parse("streamdeck-pair:v2:$host:4455:000123:$fingerprint")
            assertTrue(result is QrParseResult.Success)
            val payload = (result as QrParseResult.Success).payload
            assertEquals(host, payload.host)
            assertEquals(4455, payload.port)
            assertEquals("000123", payload.code)
            assertEquals(fingerprint, payload.fingerprint)
        }
    }

    @Test fun rejectsLegacyAndInvalidFingerprints() {
        assertEquals(QrParseError.INVALID_FORMAT, (PairingQrParser.parse("streamdeck-pair:v1:192.168.1.1:4455:123456") as QrParseResult.Error).error)
        assertEquals(QrParseError.UNSUPPORTED_VERSION, (PairingQrParser.parse("streamdeck-pair:v3:192.168.1.1:4455:123456:$fingerprint") as QrParseResult.Error).error)
        assertEquals(QrParseError.INVALID_FINGERPRINT, (PairingQrParser.parse("streamdeck-pair:v2:192.168.1.1:4455:123456:bad") as QrParseResult.Error).error)
    }

    @Test fun rejectsUntrustedOrMalformedInput() {
        assertEquals(QrParseError.UNTRUSTED_IP, (PairingQrParser.parse("streamdeck-pair:v2:8.8.8.8:4455:123456:$fingerprint") as QrParseResult.Error).error)
        assertEquals(QrParseError.INVALID_IP, (PairingQrParser.parse("streamdeck-pair:v2:192.168.01.3:4455:123456:$fingerprint") as QrParseResult.Error).error)
        assertEquals(QrParseError.INVALID_PORT, (PairingQrParser.parse("streamdeck-pair:v2:192.168.1.1:70000:123456:$fingerprint") as QrParseResult.Error).error)
        assertEquals(QrParseError.INVALID_CODE, (PairingQrParser.parse("streamdeck-pair:v2:192.168.1.1:4455:１２３４５６:$fingerprint") as QrParseResult.Error).error)
        assertEquals(QrParseError.INVALID_FORMAT, (PairingQrParser.parse("x".repeat(161)) as QrParseResult.Error).error)
        assertEquals(QrParseError.EMPTY_PAYLOAD, (PairingQrParser.parse(null) as QrParseResult.Error).error)
    }
}
