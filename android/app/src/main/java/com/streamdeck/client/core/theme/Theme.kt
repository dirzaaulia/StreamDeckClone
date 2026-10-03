package com.streamdeck.client.core.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val DarkColorScheme = darkColorScheme(
    primary = DeckPrimary,
    onPrimary = DeckOnPrimary,
    primaryContainer = DeckPrimaryContainer,
    onPrimaryContainer = Color.White,
    background = DeckBackgroundDark,
    onBackground = Color.White,
    surface = DeckSurfaceDark,
    onSurface = Color.White,
    surfaceVariant = DeckSurfaceVariantDark,
    onSurfaceVariant = Color(0xFF94A3B8),
)

@Composable
fun StreamDeckTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = DarkColorScheme,
        content = content,
    )
}
