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
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import okio.ByteString
import java.util.concurrent.TimeUnit

sealed interface ConnectionStatus {
    data object Disconnected : ConnectionStatus
    data class Connecting(val address: String) : ConnectionStatus
    data class Connected(val address: String) : ConnectionStatus
    data class Error(val message: String) : ConnectionStatus
}

@OptIn(ExperimentalSerializationApi::class)
class DeckWebSocketClient(
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    private val client = OkHttpClient.Builder()
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .pingInterval(5, TimeUnit.SECONDS)
        .retryOnConnectionFailure(true)
        .build()

    private var activeWebSocket: WebSocket? = null

    private val _status = MutableStateFlow<ConnectionStatus>(ConnectionStatus.Disconnected)
    val status: StateFlow<ConnectionStatus> = _status.asStateFlow()

    private val _incoming = MutableSharedFlow<DeckMessage>(extraBufferCapacity = 64)
    val incoming: SharedFlow<DeckMessage> = _incoming.asSharedFlow()

    fun connect(host: String, port: Int = 4455) {
        disconnect()
        val clean = host.trim().removePrefix("ws://").removePrefix("http://").trimEnd('/')
        val address = if (clean.contains(":")) clean else "$clean:$port"
        android.util.Log.i("StreamDeckClient", "Initiating WebSocket connection to ws://$address/")
        _status.value = ConnectionStatus.Connecting(address)

        val request = Request.Builder()
            .url("ws://$address/")
            .build()

        activeWebSocket = client.newWebSocket(request, createListener(address))
    }

    fun disconnect() {
        activeWebSocket?.close(1000, "User disconnected")
        activeWebSocket = null
        _status.value = ConnectionStatus.Disconnected
    }

    fun sendMessage(msg: DeckMessage) {
        val socket = activeWebSocket ?: return
        scope.launch(Dispatchers.IO) {
            val bytes = ProtoBuf.encodeToByteArray(DeckMessage.serializer(), msg)
            socket.send(ByteString.of(*bytes))
        }
    }

    private fun createListener(address: String): WebSocketListener = object : WebSocketListener() {
        override fun onOpen(webSocket: WebSocket, response: Response) {
            android.util.Log.i("StreamDeckClient", "WebSocket onOpen connected to $address")
            _status.value = ConnectionStatus.Connected(address)
            sendHandshake(webSocket)
        }

        override fun onMessage(webSocket: WebSocket, bytes: ByteString) {
            android.util.Log.i("StreamDeckClient", "WebSocket onMessage received ${bytes.size} bytes")
            runCatching {
                val decoded = ProtoBuf.decodeFromByteArray(
                    DeckMessage.serializer(),
                    bytes.toByteArray(),
                )
                scope.launch { _incoming.emit(decoded) }
            }.onFailure {
                android.util.Log.e("StreamDeckClient", "Protobuf decode error", it)
            }
        }

        override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
            android.util.Log.e("StreamDeckClient", "WebSocket onFailure: ${t.javaClass.name}: ${t.message}", t)
            _status.value = ConnectionStatus.Error(t.localizedMessage ?: "Connection failure")
        }

        override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
            android.util.Log.i("StreamDeckClient", "WebSocket onClosed: code=$code, reason=$reason")
            _status.value = ConnectionStatus.Disconnected
        }
    }

    private fun sendHandshake(webSocket: WebSocket) {
        val handshake = DeckMessage(
            timestamp = System.currentTimeMillis(),
            handshakeReq = HandshakeRequest(
                deviceId = "android-${android.os.Build.MODEL}",
                deviceName = android.os.Build.MODEL ?: "Android Device",
                clientVersion = "0.1.0",
            ),
        )
        val bytes = ProtoBuf.encodeToByteArray(DeckMessage.serializer(), handshake)
        webSocket.send(ByteString.of(*bytes))
    }
}
