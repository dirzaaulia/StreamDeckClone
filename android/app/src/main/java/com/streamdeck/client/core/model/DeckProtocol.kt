package com.streamdeck.client.core.model

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.Serializable
import kotlinx.serialization.protobuf.ProtoNumber

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class DeckMessage(
    @ProtoNumber(1) val timestamp: Long = 0L,
    @ProtoNumber(2) val handshakeReq: HandshakeRequest? = null,
    @ProtoNumber(3) val handshakeRes: HandshakeResponse? = null,
    @ProtoNumber(4) val keyEvent: KeyEvent? = null,
    @ProtoNumber(5) val layoutUpdate: PageLayoutUpdate? = null,
    @ProtoNumber(6) val stateUpdate: KeyStateUpdate? = null,
    @ProtoNumber(7) val heartbeat: Heartbeat? = null,
)

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class HandshakeRequest(
    @ProtoNumber(1) val deviceId: String = "",
    @ProtoNumber(2) val deviceName: String = "",
    @ProtoNumber(3) val clientVersion: String = "0.1.0",
    @ProtoNumber(4) val screenWidthDp: Int = 0,
    @ProtoNumber(5) val screenHeightDp: Int = 0,
    @ProtoNumber(6) val authToken: String = "",
)

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class HandshakeResponse(
    @ProtoNumber(1) val success: Boolean = false,
    @ProtoNumber(2) val serverVersion: String = "",
    @ProtoNumber(3) val message: String = "",
    @ProtoNumber(4) val activeProfileId: String = "",
    @ProtoNumber(5) val activePageId: String = "",
    @ProtoNumber(6) val sessionToken: String = "",
)

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class KeyEvent(
    @ProtoNumber(1) val profileId: String = "default",
    @ProtoNumber(2) val pageId: String = "main",
    @ProtoNumber(3) val keyIndex: Int = 0,
    @ProtoNumber(4) val eventType: Int = 0, // 0 = DOWN, 1 = UP, 2 = LONG_PRESS, 3 = DIAL_ROTATE
    @ProtoNumber(5) val deltaValue: Int = 0,
)

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class PageLayoutUpdate(
    @ProtoNumber(1) val profileId: String = "",
    @ProtoNumber(2) val pageId: String = "",
    @ProtoNumber(3) val columns: Int = 3,
    @ProtoNumber(4) val rows: Int = 3,
    @ProtoNumber(5) val keys: List<KeySlotConfig> = emptyList(),
)

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class KeySlotConfig(
    @ProtoNumber(1) val index: Int = 0,
    @ProtoNumber(2) val title: String = "",
    @ProtoNumber(3) val iconUrl: String = "",
    @ProtoNumber(4) val iconBytes: ByteArray = byteArrayOf(),
    @ProtoNumber(5) val backgroundColor: String = "",
    @ProtoNumber(6) val badgeText: String = "",
    @ProtoNumber(7) val badgeColor: String = "",
    @ProtoNumber(8) val toggleState: Int = 0,
    @ProtoNumber(9) val isDial: Boolean = false,
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false
        other as KeySlotConfig
        return index == other.index && title == other.title
    }

    override fun hashCode(): Int {
        return index.hashCode() * 31 + title.hashCode()
    }
}

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class KeyStateUpdate(
    @ProtoNumber(1) val keyIndex: Int = 0,
    @ProtoNumber(2) val title: String? = null,
    @ProtoNumber(3) val badgeText: String? = null,
    @ProtoNumber(4) val badgeColor: String? = null,
    @ProtoNumber(5) val toggleState: Int? = null,
    @ProtoNumber(6) val iconChecksum: String? = null,
)

@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class Heartbeat(
    @ProtoNumber(1) val ping: Long = 0L,
)
