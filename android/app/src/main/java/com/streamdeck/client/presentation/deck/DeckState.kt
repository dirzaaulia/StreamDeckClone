// [LINE BUDGET AUDIT] 42/250
package com.streamdeck.client.presentation.deck

import com.streamdeck.client.core.model.DeckKey
import com.streamdeck.client.data.net.ConnectionStatus
import com.streamdeck.client.data.net.DiscoveredHost
import com.streamdeck.client.data.net.WifiState

data class DeckUiState(
    val connectionStatus: ConnectionStatus = ConnectionStatus.Disconnected,
    val hostAddress: String = "",
    val pairingCode: String = "",
    val fingerprint: String = "",
    val discoveredHosts: List<DiscoveredHost> = emptyList(),
    val isScanning: Boolean = false,
    val wifiState: WifiState = WifiState.Disconnected,
    val columns: Int = 3,
    val rows: Int = 3,
    val keys: List<DeckKey> = emptyList(),
    val latencyMs: Long = 0L,
    val activeProfileId: String = "default",
    val activePageId: String = "main",
)

sealed interface DeckUiAction {
    data class OnHostAddressChanged(val address: String) : DeckUiAction
    data class OnPairingCodeChanged(val code: String) : DeckUiAction
    data class OnFingerprintChanged(val fingerprint: String) : DeckUiAction
    data class OnDiscoveredHostSelected(val address: String) : DeckUiAction
    data object OnConnectClicked : DeckUiAction
    data object OnDisconnectClicked : DeckUiAction
    data object OnCancelConnectingClicked : DeckUiAction
    data object OnRescanClicked : DeckUiAction
    data class OnKeyPress(val slotIndex: Int) : DeckUiAction
    data class OnKeyRelease(val slotIndex: Int) : DeckUiAction
    data object OnUseLocalhostClicked : DeckUiAction
    data object OnRefreshWifiClicked : DeckUiAction
    data class OnQrCodeScanned(val rawCode: String) : DeckUiAction
    data class OnQrScanFailed(val errorMessage: String) : DeckUiAction
}

sealed interface DeckUiEffect {
    data class ShowToast(val message: String) : DeckUiEffect
    data object TriggerHaptic : DeckUiEffect
}
