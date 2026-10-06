// [LINE BUDGET AUDIT] 147/150
package com.streamdeck.client.presentation.deck

import android.app.Application
import android.content.Context
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.streamdeck.client.R
import com.streamdeck.client.core.model.DeckMessage
import com.streamdeck.client.core.model.KeyEvent
import com.streamdeck.client.core.model.toDomain
import com.streamdeck.client.core.util.PairingQrParser
import com.streamdeck.client.core.util.QrParseResult
import com.streamdeck.client.data.net.ConnectionStatus
import com.streamdeck.client.data.net.DeckDiscoveryService
import com.streamdeck.client.data.net.DeckWebSocketClient
import com.streamdeck.client.data.net.WifiInfoHelper
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class DeckViewModel(
    application: Application,
) : AndroidViewModel(application) {

    private val prefs = application.getSharedPreferences("deck_prefs", Context.MODE_PRIVATE)
    private val credentials = com.streamdeck.client.data.net.PairingCredentials(application)
    private val client = DeckWebSocketClient()
    private val discoveryService = DeckDiscoveryService(application)

    private val _uiState = MutableStateFlow(
        DeckUiState(hostAddress = prefs.getString("last_host", "") ?: ""),
    )
    val uiState: StateFlow<DeckUiState> = _uiState.asStateFlow()

    private val _effects = Channel<DeckUiEffect>(Channel.BUFFERED)
    val effects = _effects.receiveAsFlow()

    init {
        observeNetwork()
        observeDiscovery()
        refreshWifiState()
        discoveryService.startDiscovery()
        val saved = _uiState.value.hostAddress
        if (saved.isNotBlank()) connectToHost()
    }

    fun onAction(action: DeckUiAction) {
        when (action) {
            is DeckUiAction.OnHostAddressChanged -> _uiState.update { it.copy(hostAddress = action.address) }
            is DeckUiAction.OnPairingCodeChanged -> _uiState.update { it.copy(pairingCode = action.code.trim()) }
            is DeckUiAction.OnFingerprintChanged -> _uiState.update { it.copy(fingerprint = action.fingerprint.trim()) }
            is DeckUiAction.OnDiscoveredHostSelected -> {
                _uiState.update { it.copy(hostAddress = action.address) }
                connectToHost()
            }
            is DeckUiAction.OnUseLocalhostClicked -> {
                _uiState.update { it.copy(hostAddress = "127.0.0.1:4455") }
                connectToHost()
            }
            is DeckUiAction.OnConnectClicked -> connectToHost()
            is DeckUiAction.OnDisconnectClicked, is DeckUiAction.OnCancelConnectingClicked -> client.disconnect()
            is DeckUiAction.OnRescanClicked -> {
                refreshWifiState()
                discoveryService.rescan()
            }
            is DeckUiAction.OnRefreshWifiClicked -> refreshWifiState()
            is DeckUiAction.OnKeyPress -> handleKeyEvent(action.slotIndex, 0)
            is DeckUiAction.OnKeyRelease -> handleKeyEvent(action.slotIndex, 1)
            is DeckUiAction.OnQrCodeScanned -> handleQrCodeScanned(action.rawCode)
            is DeckUiAction.OnQrScanFailed -> handleQrScanFailed(action.errorMessage)
        }
    }

    fun refreshWifiState() {
        _uiState.update { it.copy(wifiState = WifiInfoHelper.getWifiState(getApplication())) }
    }

    private fun handleQrCodeScanned(rawCode: String) {
        val app = getApplication<Application>()
        when (val result = PairingQrParser.parse(rawCode)) {
            is QrParseResult.Success -> {
                _uiState.update {
                    it.copy(
                        hostAddress = "${result.payload.host}:${result.payload.port}",
                        pairingCode = result.payload.code,
                        fingerprint = result.payload.fingerprint,
                        connectionStatus = ConnectionStatus.Disconnected,
                    )
                }
            }
            is QrParseResult.Error -> {
                _uiState.update { it.copy(connectionStatus = ConnectionStatus.Error(app.getString(result.error.messageResId))) }
            }
        }
    }

    private fun handleQrScanFailed(errorMessage: String) {
        val msg = getApplication<Application>().getString(R.string.deck_qr_scanner_failed, errorMessage)
        _uiState.update { it.copy(connectionStatus = ConnectionStatus.Error(msg)) }
    }

    private fun connectToHost() {
        val raw = _uiState.value.hostAddress.trim().removePrefix("ws://").removePrefix("http://").trimEnd('/')
        if (raw.isBlank()) return
        val parts = raw.split(":")
        val host = parts.getOrNull(0)?.ifBlank { "127.0.0.1" } ?: "127.0.0.1"
        val port = parts.getOrNull(1)?.toIntOrNull() ?: 4455
        val deviceId = prefs.getString("device_id", null) ?: java.util.UUID.randomUUID().toString().also {
            prefs.edit().putString("device_id", it).apply()
        }
        val address = "$host:$port"
        val saved = credentials.get(address)
        val supplied = _uiState.value.fingerprint.lowercase()
        val fingerprint = supplied.ifBlank { saved?.first.orEmpty() }
        if (!Regex("[0-9a-f]{64}").matches(fingerprint)) {
            _uiState.update { it.copy(connectionStatus = ConnectionStatus.Error(getApplication<Application>().getString(R.string.deck_fingerprint_required))) }
            return
        }
        if (saved != null && saved.first != fingerprint) {
            _uiState.update { it.copy(connectionStatus = ConnectionStatus.Error(getApplication<Application>().getString(R.string.deck_fingerprint_changed))) }
            return
        }
        client.connect(host, port, fingerprint, deviceId, saved?.second.orEmpty(), _uiState.value.pairingCode) { pairedToken ->
            credentials.save(address, fingerprint, pairedToken)
            _uiState.update { it.copy(pairingCode = "", fingerprint = fingerprint) }
        }
    }

    private fun handleKeyEvent(slotIndex: Int, eventType: Int) {
        if (eventType == 0) _effects.trySend(DeckUiEffect.TriggerHaptic)
        val event = KeyEvent(_uiState.value.activeProfileId, _uiState.value.activePageId, slotIndex, eventType)
        client.sendMessage(DeckMessage(timestamp = System.currentTimeMillis(), keyEvent = event))
    }

    private fun observeNetwork() {
        viewModelScope.launch {
            launch {
                client.status.collect { status ->
                    _uiState.update { it.copy(connectionStatus = status) }
                    if (status is ConnectionStatus.Connected) prefs.edit().putString("last_host", status.address).apply()
                }
            }
            launch { client.incoming.collect { handleMessage(it) } }
        }
    }

    private fun observeDiscovery() {
        viewModelScope.launch {
            launch { discoveryService.discovered.collect { h -> _uiState.update { it.copy(discoveredHosts = h) } } }
            launch { discoveryService.isScanning.collect { s -> _uiState.update { it.copy(isScanning = s) } } }
        }
    }

    private fun handleMessage(msg: DeckMessage) {
        msg.layoutUpdate?.let { u ->
            _uiState.update {
                it.copy(
                    activeProfileId = u.profileId,
                    activePageId = u.pageId,
                    columns = u.columns,
                    rows = u.rows,
                    keys = u.keys.map { k -> k.toDomain() },
                )
            }
        }
        msg.handshakeRes?.let { r ->
            _uiState.update { it.copy(activeProfileId = r.activeProfileId, activePageId = r.activePageId) }
        }
    }

    override fun onCleared() {
        super.onCleared()
        client.disconnect()
        discoveryService.stopDiscovery()
    }
}
