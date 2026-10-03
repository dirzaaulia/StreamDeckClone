// [LINE BUDGET AUDIT] 137/150
package com.streamdeck.client.presentation.deck

import android.app.Application
import android.content.Context
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.streamdeck.client.core.model.DeckMessage
import com.streamdeck.client.core.model.KeyEvent
import com.streamdeck.client.core.model.toDomain
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
        }
    }

    fun refreshWifiState() {
        val state = WifiInfoHelper.getWifiState(getApplication())
        _uiState.update { it.copy(wifiState = state) }
    }

    private fun connectToHost() {
        val raw = _uiState.value.hostAddress
            .trim().removePrefix("ws://").removePrefix("http://").trimEnd('/')
        if (raw.isBlank()) return
        val parts = raw.split(":")
        val host = parts.getOrNull(0)?.ifBlank { "127.0.0.1" } ?: "127.0.0.1"
        val port = parts.getOrNull(1)?.toIntOrNull() ?: 4455
        client.connect(host, port)
    }

    private fun handleKeyEvent(slotIndex: Int, eventType: Int) {
        if (eventType == 0) _effects.trySend(DeckUiEffect.TriggerHaptic)
        val event = KeyEvent(
            profileId = _uiState.value.activeProfileId,
            pageId = _uiState.value.activePageId,
            keyIndex = slotIndex,
            eventType = eventType,
        )
        client.sendMessage(DeckMessage(timestamp = System.currentTimeMillis(), keyEvent = event))
    }

    private fun observeNetwork() {
        viewModelScope.launch {
            client.status.collect { status ->
                _uiState.update { it.copy(connectionStatus = status) }
                if (status is ConnectionStatus.Connected) {
                    prefs.edit().putString("last_host", status.address).apply()
                }
            }
        }
        viewModelScope.launch { client.incoming.collect { handleMessage(it) } }
    }

    private fun observeDiscovery() {
        viewModelScope.launch {
            discoveryService.discovered.collect { hosts ->
                _uiState.update { it.copy(discoveredHosts = hosts) }
            }
        }
        viewModelScope.launch {
            discoveryService.isScanning.collect { scanning ->
                _uiState.update { it.copy(isScanning = scanning) }
            }
        }
    }

    private fun handleMessage(msg: DeckMessage) {
        msg.layoutUpdate?.let { u ->
            _uiState.update { it.copy(columns = u.columns, rows = u.rows, keys = u.keys.map { k -> k.toDomain() }) }
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
