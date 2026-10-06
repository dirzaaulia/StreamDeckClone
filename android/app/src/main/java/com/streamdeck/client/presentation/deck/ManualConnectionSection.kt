package com.streamdeck.client.presentation.deck

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import com.streamdeck.client.R
import com.streamdeck.client.core.theme.Spacing

@Composable
fun ManualConnectionSection(
    address: String,
    pairingCode: String,
    fingerprint: String,
    onPairingCodeChange: (String) -> Unit,
    onFingerprintChange: (String) -> Unit,
    onAddressChange: (String) -> Unit,
    onConnect: () -> Unit,
    onUseLocalhost: () -> Unit,
    onScanQr: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            text = stringResource(R.string.deck_manual_section),
            style = MaterialTheme.typography.titleSmall,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.padding(bottom = Spacing.small),
        )
        OutlinedButton(onClick = onScanQr, modifier = Modifier.fillMaxWidth()) {
            Text(stringResource(R.string.deck_scan_qr_button))
        }
        Spacer(modifier = Modifier.height(Spacing.small))
        OutlinedTextField(
            value = address,
            onValueChange = onAddressChange,
            modifier = Modifier.fillMaxWidth(),
            label = { Text(stringResource(R.string.deck_host_address_label)) },
            placeholder = { Text(stringResource(R.string.deck_host_address_hint)) },
            singleLine = true,
        )
        Spacer(modifier = Modifier.height(Spacing.small))
        OutlinedTextField(
            value = pairingCode,
            onValueChange = onPairingCodeChange,
            modifier = Modifier.fillMaxWidth(),
            label = { Text(stringResource(R.string.deck_pairing_code)) },
            placeholder = { Text(stringResource(R.string.deck_pairing_code_hint)) },
            singleLine = true,
        )
        Spacer(modifier = Modifier.height(Spacing.small))
        OutlinedTextField(
            value = fingerprint,
            onValueChange = onFingerprintChange,
            modifier = Modifier.fillMaxWidth(),
            label = { Text(stringResource(R.string.deck_fingerprint_label)) },
            placeholder = { Text(stringResource(R.string.deck_fingerprint_hint)) },
            singleLine = true,
        )
        Spacer(modifier = Modifier.height(Spacing.small))
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(Spacing.small),
        ) {
            Button(onClick = onConnect, modifier = Modifier.weight(1f)) {
                Text(stringResource(R.string.deck_connect_button))
            }
            OutlinedButton(onClick = onUseLocalhost, modifier = Modifier.weight(1f)) {
                Text(stringResource(R.string.deck_quick_localhost))
            }
        }
    }
}
