package com.streamdeck.client.data.net

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Credentials are device-bound; a restored or invalidated key requires pairing again. */
class PairingCredentials(context: Context) {
    private val prefs = context.getSharedPreferences("pairing_credentials", Context.MODE_PRIVATE)
    private val legacy = context.getSharedPreferences("deck_prefs", Context.MODE_PRIVATE)
    private val alias = "streamdeck-pairing-v2"

    init {
        // Never migrate plaintext credentials: the old transport did not protect them.
        legacy.edit().apply {
            legacy.all.keys.filter { it.startsWith("paired_token_") }.forEach(::remove)
        }.apply()
    }

    private fun key(): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(alias, null) as? SecretKey)?.let { return it }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(
            KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return generator.generateKey()
    }

    fun get(host: String): Pair<String, String>? {
        val encoded = prefs.getString(host, null) ?: return null
        return try {
            val data = Base64.decode(encoded, Base64.NO_WRAP)
            require(data.size > 12)
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, data.copyOfRange(0, 12)))
            cipher.updateAAD(host.toByteArray(Charsets.UTF_8))
            val plain = String(cipher.doFinal(data.copyOfRange(12, data.size)), Charsets.UTF_8)
            val parts = plain.split(':', limit = 2)
            require(parts.size == 2 && parts[0].length == 64)
            parts[0] to parts[1]
        } catch (_: Exception) {
            prefs.edit().remove(host).apply()
            null
        }
    }

    fun save(host: String, fingerprint: String, token: String) {
        require(Regex("[0-9a-fA-F]{64}").matches(fingerprint))
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key())
        cipher.updateAAD(host.toByteArray(Charsets.UTF_8))
        val data = cipher.iv + cipher.doFinal("${fingerprint.lowercase()}:$token".toByteArray(Charsets.UTF_8))
        check(prefs.edit().putString(host, Base64.encodeToString(data, Base64.NO_WRAP)).commit())
    }
}
