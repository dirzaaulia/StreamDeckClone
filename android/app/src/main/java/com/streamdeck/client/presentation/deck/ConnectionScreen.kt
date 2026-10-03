// [LINE BUDGET AUDIT] 153/250
package com.streamdeck.client.presentation.deck

import android.Manifest
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import com.streamdeck.client.R
import com.streamdeck.client.core.theme.Spacing
import com.streamdeck.client.data.net.ConnectionStatus

@Composable
fun ConnectionScreen(
    state: DeckUiState,
    onAction: (DeckUiAction) -> Unit,
    modifier: Modifier = Modifier,
) {
    var showRationale by remember { mutableStateOf(false) }
    val permLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions(),
    ) {
        onAction(DeckUiAction.OnRefreshWifiClicked)
        onAction(DeckUiAction.OnRescanClicked)
    }

    val requestPerms: () -> Unit = {
        val perms = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            arrayOf(
                Manifest.permission.ACCESS_FINE_LOCATION,
                Manifest.permission.ACCESS_COARSE_LOCATION,
                Manifest.permission.NEARBY_WIFI_DEVICES,
            )
        } else {
            arrayOf(
                Manifest.permission.ACCESS_FINE_LOCATION,
                Manifest.permission.ACCESS_COARSE_LOCATION,
            )
        }
        permLauncher.launch(perms)
    }

    if (showRationale) {
        WifiPermissionRationaleDialog(
            onDismiss = { showRationale = false },
            onGrant = {
                showRationale = false
                requestPerms()
            },
        )
    }

    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = Spacing.large, vertical = Spacing.medium),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            ConnectionHeader()
            Spacer(modifier = Modifier.height(Spacing.medium))

            WifiNetworkBanner(
                wifiState = state.wifiState,
                onRequestPermission = requestPerms,
                onLearnMore = { showRationale = true },
            )
            Spacer(modifier = Modifier.height(Spacing.medium))

            when (val status = state.connectionStatus) {
                is ConnectionStatus.Connecting -> ConnectingBanner(
                    address = status.address,
                    onCancel = { onAction(DeckUiAction.OnCancelConnectingClicked) },
                )
                is ConnectionStatus.Error -> ErrorBanner(message = status.message)
                else -> Unit
            }

            Spacer(modifier = Modifier.height(Spacing.small))
            DiscoveredHostsSection(
                hosts = state.discoveredHosts,
                isScanning = state.isScanning,
                onConnect = { onAction(DeckUiAction.OnDiscoveredHostSelected(it)) },
                onRescan = { onAction(DeckUiAction.OnRescanClicked) },
            )

            Spacer(modifier = Modifier.height(Spacing.large))
            ManualConnectionSection(
                address = state.hostAddress,
                onAddressChange = { onAction(DeckUiAction.OnHostAddressChanged(it)) },
                onConnect = { onAction(DeckUiAction.OnConnectClicked) },
                onUseLocalhost = { onAction(DeckUiAction.OnUseLocalhostClicked) },
            )

            Spacer(modifier = Modifier.height(Spacing.medium))
            ConnectionHelpFooter()
        }
    }
}

@Composable
private fun ConnectionHeader() {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(
            text = stringResource(R.string.deck_connect_screen_title),
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.primary,
        )
        Text(
            text = stringResource(R.string.deck_connect_screen_subtitle),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
private fun ConnectionHelpFooter() {
    Text(
        text = stringResource(R.string.deck_wifi_hint),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        textAlign = TextAlign.Center,
        modifier = Modifier.fillMaxWidth().padding(horizontal = Spacing.medium),
    )
}
