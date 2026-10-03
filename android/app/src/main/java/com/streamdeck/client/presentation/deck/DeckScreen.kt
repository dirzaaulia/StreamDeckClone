// [LINE BUDGET AUDIT] 110/250
package com.streamdeck.client.presentation.deck

import android.view.HapticFeedbackConstants
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalView
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.streamdeck.client.core.model.DeckKey
import com.streamdeck.client.core.theme.Spacing
import com.streamdeck.client.data.net.ConnectionStatus

@Composable
fun DeckRoute(
    viewModel: DeckViewModel = viewModel(),
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val view = LocalView.current

    LaunchedEffect(Unit) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is DeckUiEffect.TriggerHaptic -> {
                    view.performHapticFeedback(HapticFeedbackConstants.KEYBOARD_TAP)
                }
                is DeckUiEffect.ShowToast -> Unit
            }
        }
    }

    if (state.connectionStatus is ConnectionStatus.Connected) {
        DeckActiveScreen(
            state = state,
            onAction = viewModel::onAction,
            modifier = modifier,
        )
    } else {
        ConnectionScreen(
            state = state,
            onAction = viewModel::onAction,
            modifier = modifier,
        )
    }
}

@Composable
fun DeckActiveScreen(
    state: DeckUiState,
    onAction: (DeckUiAction) -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier.fillMaxSize(),
        topBar = {
            ConnectedTopBar(
                status = state.connectionStatus,
                activeProfile = state.activeProfileId,
                onDisconnect = { onAction(DeckUiAction.OnDisconnectClicked) },
            )
        },
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
        ) {
            DeckAdaptiveGrid(
                keys = state.keys,
                columns = state.columns,
                rows = state.rows,
                onPress = { onAction(DeckUiAction.OnKeyPress(it)) },
                onRelease = { onAction(DeckUiAction.OnKeyRelease(it)) },
                modifier = Modifier.weight(1f),
            )
        }
    }
}

@Composable
fun DeckAdaptiveGrid(
    keys: List<DeckKey>,
    columns: Int,
    rows: Int,
    onPress: (Int) -> Unit,
    onRelease: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val safeCols = columns.coerceAtLeast(1)
    val safeRows = rows.coerceAtLeast(1)

    BoxWithConstraints(modifier = modifier.fillMaxSize().padding(Spacing.small)) {
        val tileWidth = (maxWidth - (Spacing.small * (safeCols - 1))) / safeCols
        val tileHeight = (maxHeight - (Spacing.small * (safeRows - 1))) / safeRows

        LazyVerticalGrid(
            columns = GridCells.Fixed(safeCols),
            userScrollEnabled = false,
            horizontalArrangement = Arrangement.spacedBy(Spacing.small),
            verticalArrangement = Arrangement.spacedBy(Spacing.small),
        ) {
            items(keys, key = { it.id.value }) { keyItem ->
                DeckKeyTile(
                    keyItem = keyItem,
                    onPress = { onPress(keyItem.id.value) },
                    onRelease = { onRelease(keyItem.id.value) },
                    modifier = Modifier.size(tileWidth, tileHeight),
                )
            }
        }
    }
}
