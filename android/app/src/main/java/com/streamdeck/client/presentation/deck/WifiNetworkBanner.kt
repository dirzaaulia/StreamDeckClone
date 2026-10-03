// [LINE BUDGET AUDIT] 120/250
package com.streamdeck.client.presentation.deck

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.streamdeck.client.R
import com.streamdeck.client.core.theme.Spacing
import com.streamdeck.client.data.net.WifiState

@Composable
fun WifiNetworkBanner(
    wifiState: WifiState,
    onRequestPermission: () -> Unit,
    onLearnMore: () -> Unit,
    modifier: Modifier = Modifier,
) {
    when (wifiState) {
        is WifiState.Connected -> WifiConnectedCard(ssid = wifiState.ssid, modifier = modifier)
        is WifiState.PermissionRequired -> WifiPermissionCard(
            onRequestPermission = onRequestPermission,
            onLearnMore = onLearnMore,
            modifier = modifier,
        )
        is WifiState.Disconnected -> WifiDisconnectedCard(modifier = modifier)
    }
}

@Composable
private fun WifiConnectedCard(
    ssid: String,
    modifier: Modifier = Modifier,
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
        shape = RoundedCornerShape(12.dp),
    ) {
        Column(modifier = Modifier.padding(Spacing.medium)) {
            Text(
                text = stringResource(R.string.deck_wifi_connected_title),
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
            Spacer(modifier = Modifier.height(Spacing.extraSmall))
            Text(
                text = stringResource(R.string.deck_wifi_connected_desc, ssid),
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onPrimaryContainer,
            )
        }
    }
}

@Composable
private fun WifiPermissionCard(
    onRequestPermission: () -> Unit,
    onLearnMore: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
        shape = RoundedCornerShape(12.dp),
    ) {
        Column(modifier = Modifier.padding(Spacing.medium)) {
            Text(
                text = stringResource(R.string.deck_wifi_perm_needed_title),
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Spacer(modifier = Modifier.height(Spacing.extraSmall))
            Text(
                text = stringResource(R.string.deck_wifi_perm_needed_desc),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.height(Spacing.small))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Button(onClick = onRequestPermission) {
                    Text(stringResource(R.string.deck_wifi_grant_button))
                }
                TextButton(onClick = onLearnMore) {
                    Text(stringResource(R.string.deck_wifi_learn_more))
                }
            }
        }
    }
}

@Composable
private fun WifiDisconnectedCard(modifier: Modifier = Modifier) {
    Card(
        modifier = modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer),
        shape = RoundedCornerShape(12.dp),
    ) {
        Column(modifier = Modifier.padding(Spacing.medium)) {
            Text(
                text = stringResource(R.string.deck_wifi_disconnected_title),
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onErrorContainer,
            )
            Text(
                text = stringResource(R.string.deck_wifi_disconnected_desc),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onErrorContainer,
                modifier = Modifier.padding(top = Spacing.extraSmall),
            )
        }
    }
}

@Composable
fun WifiPermissionRationaleDialog(
    onDismiss: () -> Unit,
    onGrant: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                text = stringResource(R.string.deck_wifi_dialog_title),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
            )
        },
        text = {
            Text(
                text = stringResource(R.string.deck_wifi_dialog_desc),
                style = MaterialTheme.typography.bodyMedium,
            )
        },
        confirmButton = {
            Button(onClick = onGrant) {
                Text(stringResource(R.string.deck_wifi_dialog_positive))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(stringResource(R.string.deck_wifi_dialog_negative))
            }
        },
    )
}
