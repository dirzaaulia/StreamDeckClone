package com.streamdeck.client.presentation.deck

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import coil3.compose.AsyncImage
import coil3.compose.rememberAsyncImagePainter
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.streamdeck.client.R
import com.streamdeck.client.core.model.DeckKey
import com.streamdeck.client.core.model.parseHexColor
import com.streamdeck.client.core.theme.KeyBadgeBg
import com.streamdeck.client.core.theme.KeyBadgeText
import com.streamdeck.client.core.theme.KeyTileBorder
import com.streamdeck.client.core.theme.KeyTileDefaultBg
import com.streamdeck.client.core.theme.KeyTilePressedBg
import com.streamdeck.client.core.theme.Spacing
import com.streamdeck.client.data.net.ConnectionStatus

@Composable
fun DeckKeyTile(
    keyItem: DeckKey,
    onPress: () -> Unit,
    onRelease: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var isPressed by remember { mutableStateOf(false) }
    val baseColor = remember(keyItem.backgroundColorHex) {
        parseHexColor(keyItem.backgroundColorHex, KeyTileDefaultBg)
    }
    val animatedBg by animateColorAsState(
        targetValue = if (isPressed) KeyTilePressedBg else baseColor,
        label = "TileBgAnimation",
    )
    val tileShape = RoundedCornerShape(12.dp)
    val desc = stringResource(R.string.deck_tile_content_desc, keyItem.title, keyItem.id.value)

    Box(
        modifier = modifier
            .sizeIn(minWidth = Spacing.minTouchTarget, minHeight = Spacing.minTouchTarget)
            .clip(tileShape)
            .background(animatedBg)
            .border(1.5.dp, KeyTileBorder, tileShape)
            .semantics { contentDescription = desc }
            .pointerInput(keyItem.id) {
                awaitEachGesture {
                    awaitFirstDown(requireUnconsumed = false)
                    isPressed = true
                    onPress()
                    val upOrCancel = waitForUpOrCancellation()
                    isPressed = false
                    if (upOrCancel != null) onRelease()
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        KeyTileContent(keyItem = keyItem)
    }
}

@Composable
private fun KeyTileContent(keyItem: DeckKey) {
    Column(
        modifier = Modifier.fillMaxSize().padding(Spacing.small),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        if (keyItem.badgeText.isNotBlank()) {
            KeyBadge(text = keyItem.badgeText)
        }
        if (keyItem.iconUrl.isNotBlank()) {
            AsyncImage(
                model = keyItem.iconUrl,
                contentDescription = null,
                modifier = Modifier.size(36.dp),
                contentScale = ContentScale.Fit,
            )
        }
        Text(
            text = keyItem.title.ifBlank { stringResource(R.string.deck_tile_empty) },
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.Bold,
            color = Color.White,
            textAlign = TextAlign.Center,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun KeyBadge(text: String) {
    Surface(
        shape = RoundedCornerShape(4.dp),
        color = KeyBadgeBg,
        modifier = Modifier.padding(bottom = Spacing.extraSmall),
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelSmall,
            color = KeyBadgeText,
            modifier = Modifier.padding(horizontal = Spacing.extraSmall, vertical = 2.dp),
        )
    }
}

@Composable
fun ConnectedTopBar(
    status: ConnectionStatus,
    activeProfile: String,
    onDisconnect: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .padding(horizontal = Spacing.medium, vertical = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(
                modifier = Modifier
                    .size(8.dp)
                    .background(Color(0xFF22C55E), shape = RoundedCornerShape(4.dp)),
            )
            Text(
                text = when (status) {
                    is ConnectionStatus.Connected -> stringResource(R.string.deck_host_connected)
                    else -> stringResource(R.string.deck_host_disconnected)
                },
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.padding(start = Spacing.small),
            )
        }
        Row(verticalAlignment = Alignment.CenterVertically) {
            Surface(
                shape = RoundedCornerShape(6.dp),
                color = MaterialTheme.colorScheme.primaryContainer,
                modifier = Modifier.padding(end = Spacing.small),
            ) {
                Text(
                    text = activeProfile,
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                    modifier = Modifier.padding(horizontal = Spacing.small, vertical = 2.dp),
                )
            }
            TextButton(onClick = onDisconnect) {
                Text(
                    text = stringResource(R.string.deck_disconnect_button),
                    style = MaterialTheme.typography.labelMedium,
                )
            }
        }
    }
}
