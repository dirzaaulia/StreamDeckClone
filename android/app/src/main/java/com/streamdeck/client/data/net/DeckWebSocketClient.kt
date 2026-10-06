package com.streamdeck.client.data.net

import com.streamdeck.client.core.model.DeckMessage
import com.streamdeck.client.core.model.HandshakeRequest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.protobuf.ProtoBuf
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import okio.ByteString

sealed interface ConnectionStatus {
    data object Disconnected : ConnectionStatus
    data class Connecting(val address: String) : ConnectionStatus
    data class Connected(val address: String) : ConnectionStatus
    data class PairingRequired(val message: String) : ConnectionStatus
    data class Error(val message: String) : ConnectionStatus
}

@OptIn(ExperimentalSerializationApi::class)
class DeckWebSocketClient(
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    @Volatile private var activeWebSocket: WebSocket? = null
    private var pairingCode: String = ""
    private var deviceId: String = ""
    private var savedToken: String = ""
    private var onPaired: ((String) -> Unit)? = null

    private val _status = MutableStateFlow<ConnectionStatus>(ConnectionStatus.Disconnected)
    val status: StateFlow<ConnectionStatus> = _status.asStateFlow()

    private val _incoming = MutableSharedFlow<DeckMessage>(extraBufferCapacity = 64)
    val incoming: SharedFlow<DeckMessage> = _incoming.asSharedFlow()

    fun connect(host: String, port: Int, fingerprint: String, deviceId: String, token: String, pairingCode: String, onPaired: ((String) -> Unit)?) {
        disconnect()
        this.deviceId = deviceId
        this.savedToken = token
        this.pairingCode = pairingCode
        this.onPaired = onPaired
        val address = "$host:$port"
        _status.value = ConnectionStatus.Connecting(address)
        val request = Request.Builder().url("wss://$address/").build()
        activeWebSocket = PinnedTls.client(fingerprint).newWebSocket(request, createListener(address))
    }

    fun disconnect() {
        activeWebSocket?.close(1000, "User disconnected")
        activeWebSocket = null
        _status.value = ConnectionStatus.Disconnected
    }

    fun sendMessage(msg: DeckMessage) {
        val socket = activeWebSocket ?: return
        if (_status.value !is ConnectionStatus.Connected) return
        scope.launch(Dispatchers.IO) {
            if (activeWebSocket !== socket || _status.value !is ConnectionStatus.Connected) return@launch
            val bytes = ProtoBuf.encodeToByteArray(DeckMessage.serializer(), msg)
            socket.send(ByteString.of(*bytes))
        }
    }

    private fun createListener(address: String): WebSocketListener = object : WebSocketListener() {
        override fun onOpen(webSocket: WebSocket, response: Response) {
            android.util.Log.i("StreamDeckClient", "WebSocket onOpen connected to $address")
            if (activeWebSocket !== webSocket) return
            sendHandshake(webSocket)
        }

        override fun onMessage(webSocket: WebSocket, bytes: ByteString) {
            if (bytes.size > 65536) {
                webSocket.close(1009, "Message too large")
                return
            }
            runCatching {
                val decoded = ProtoBuf.decodeFromByteArray(
                    DeckMessage.serializer(),
                    bytes.toByteArray(),
                )
                if (activeWebSocket !== webSocket) return@runCatching
                decoded.handshakeRes?.let { response ->
                    if (response.success) {
                        if (response.sessionToken.isNotBlank() && response.sessionToken != savedToken) {
                            val stored = runCatching { onPaired?.invoke(response.sessionToken) }.isSuccess
                            if (!stored) {
                                _status.value = ConnectionStatus.Error("Could not securely store pairing")
                                webSocket.close(1000, "Pairing storage failed")
                                return@runCatching
                            }
                            savedToken = response.sessionToken
                        }
                        _status.value = ConnectionStatus.Connected(address)
                    } else {
                        _status.value = ConnectionStatus.PairingRequired(response.message)
                        webSocket.close(1000, "Pairing required")
                    }
                }
                if (_status.value is ConnectionStatus.Connected) scope.launch { _incoming.emit(decoded) }
            }.onFailure {
                android.util.Log.e("StreamDeckClient", "Protobuf decode error", it)
            }
        }

        override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
            android.util.Log.e("StreamDeckClient", "WebSocket onFailure: ${t.javaClass.name}: ${t.message}", t)
            if (activeWebSocket === webSocket) {
                activeWebSocket = null
                if (_status.value !is ConnectionStatus.PairingRequired) {
                    _status.value = ConnectionStatus.Error(t.localizedMessage ?: "Connection failure")
                }
            }
        }

        override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
            android.util.Log.i("StreamDeckClient", "WebSocket onClosed: code=$code, reason=$reason")
            if (activeWebSocket === webSocket) {
                activeWebSocket = null
                if (_status.value !is ConnectionStatus.PairingRequired && _status.value !is ConnectionStatus.Error) {
                    _status.value = ConnectionStatus.Disconnected
                }
            }
        }
    }

    private fun sendHandshake(webSocket: WebSocket) {
        val handshake = DeckMessage(
            timestamp = System.currentTimeMillis(),
            handshakeReq = HandshakeRequest(
                deviceId = deviceId,
                deviceName = android.os.Build.MODEL ?: "Android Device",
                clientVersion = "0.1.0",
                authToken = pairingCode.ifBlank { savedToken },
            ),
        )
        val bytes = ProtoBuf.encodeToByteArray(DeckMessage.serializer(), handshake)
        webSocket.send(ByteString.of(*bytes))
    }
}
