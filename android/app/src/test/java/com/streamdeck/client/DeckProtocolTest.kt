package com.streamdeck.client

import com.streamdeck.client.core.model.DeckMessage
import com.streamdeck.client.core.model.HandshakeRequest
import com.streamdeck.client.core.model.KeyEvent
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.protobuf.ProtoBuf
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test

@OptIn(ExperimentalSerializationApi::class)
class DeckProtocolTest {

    @Test
    fun testHandshakeSerialization() {
        val msg = DeckMessage(
            timestamp = 123456L,
            handshakeReq = HandshakeRequest(
                deviceId = "test-device",
                deviceName = "Test Device",
                clientVersion = "1.0.0",
            ),
        )

        val bytes = ProtoBuf.encodeToByteArray(DeckMessage.serializer(), msg)
        assertNotNull(bytes)

        val decoded = ProtoBuf.decodeFromByteArray(DeckMessage.serializer(), bytes)
        assertEquals(123456L, decoded.timestamp)
        assertEquals("test-device", decoded.handshakeReq?.deviceId)
    }

    @Test
    fun testKeyEventSerialization() {
        val msg = DeckMessage(
            timestamp = 99999L,
            keyEvent = KeyEvent(
                profileId = "default",
                pageId = "main",
                keyIndex = 3,
                eventType = 0,
            ),
        )

        val bytes = ProtoBuf.encodeToByteArray(DeckMessage.serializer(), msg)
        val decoded = ProtoBuf.decodeFromByteArray(DeckMessage.serializer(), bytes)
        assertEquals(3, decoded.keyEvent?.keyIndex)
    }
}
