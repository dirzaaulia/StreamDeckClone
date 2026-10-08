// [LINE BUDGET AUDIT] 54/250
package com.streamdeck.client

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.streamdeck.client.core.theme.StreamDeckTheme
import com.streamdeck.client.presentation.deck.DeckRoute

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        // Keep screen on while controlling PC
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        // Immersive full-screen stream deck experience
        WindowCompat.setDecorFitsSystemWindows(window, false)
        val insetsController = WindowCompat.getInsetsController(window, window.decorView)
        insetsController.systemBarsBehavior =
            WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        insetsController.hide(WindowInsetsCompat.Type.systemBars())

        handleLaunchIntent(intent)

        setContent {
            StreamDeckTheme {
                DeckRoute()
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleLaunchIntent(intent)
    }

    private fun handleLaunchIntent(intent: Intent?) {
        val host = intent?.getStringExtra("host_address")
        val fingerprint = intent?.getStringExtra("fingerprint")
        if (!host.isNullOrBlank() || !fingerprint.isNullOrBlank()) {
            val prefs = getSharedPreferences("deck_prefs", Context.MODE_PRIVATE)
            prefs.edit().apply {
                if (!host.isNullOrBlank()) putString("last_host", host.trim())
                if (!fingerprint.isNullOrBlank()) putString("last_fingerprint", fingerprint.trim().lowercase())
            }.apply()
        }
    }
}
