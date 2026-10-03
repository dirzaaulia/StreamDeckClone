package com.streamdeck.client.core.model

import androidx.compose.ui.graphics.Color

@JvmInline
value class KeyId(val value: Int)

data class DeckKey(
    val id: KeyId,
    val title: String,
    val iconUrl: String = "",
    val backgroundColorHex: String = "",
    val badgeText: String = "",
    val isDial: Boolean = false,
)

internal fun KeySlotConfig.toDomain(): DeckKey = DeckKey(
    id = KeyId(index),
    title = title,
    iconUrl = iconUrl,
    backgroundColorHex = backgroundColor,
    badgeText = badgeText,
    isDial = isDial,
)

internal fun parseHexColor(hex: String, fallback: Color): Color {
    if (hex.isBlank()) return fallback
    return runCatching {
        val clean = hex.removePrefix("#")
        val parsed = clean.toLong(16)
        if (clean.length == 6) {
            Color(parsed or 0xFF000000)
        } else {
            Color(parsed)
        }
    }.getOrDefault(fallback)
}
