package com.streamdeck.client.data.net

import okhttp3.OkHttpClient
import java.net.Socket
import java.security.MessageDigest
import java.security.cert.CertificateException
import java.security.cert.X509Certificate
import java.util.concurrent.TimeUnit
import javax.net.ssl.SSLEngine
import javax.net.ssl.X509ExtendedTrustManager
import javax.net.ssl.SSLContext

internal object PinnedTls {
    fun client(fingerprint: String): OkHttpClient {
        require(Regex("[0-9a-fA-F]{64}").matches(fingerprint))
        val expected = fingerprint.lowercase()
        val trust = object : X509ExtendedTrustManager() {
            private fun check(chain: Array<out X509Certificate>) {
                if (chain.size != 1) throw CertificateException("Unexpected certificate chain")
                chain[0].checkValidity()
                val actual = MessageDigest.getInstance("SHA-256").digest(chain[0].encoded)
                    .joinToString("") { "%02x".format(it) }
                if (!MessageDigest.isEqual(actual.toByteArray(), expected.toByteArray())) {
                    throw CertificateException("Host fingerprint mismatch; verify the PC and pair again")
                }
            }
            override fun checkServerTrusted(chain: Array<out X509Certificate>, authType: String) = check(chain)
            override fun checkServerTrusted(chain: Array<out X509Certificate>, authType: String, socket: Socket) = check(chain)
            override fun checkServerTrusted(chain: Array<out X509Certificate>, authType: String, engine: SSLEngine) = check(chain)
            override fun checkClientTrusted(chain: Array<out X509Certificate>, authType: String) = throw CertificateException("Client TLS not supported")
            override fun checkClientTrusted(chain: Array<out X509Certificate>, authType: String, socket: Socket) = throw CertificateException("Client TLS not supported")
            override fun checkClientTrusted(chain: Array<out X509Certificate>, authType: String, engine: SSLEngine) = throw CertificateException("Client TLS not supported")
            override fun getAcceptedIssuers(): Array<X509Certificate> = emptyArray()
        }
        val context = SSLContext.getInstance("TLS").apply { init(null, arrayOf(trust), null) }
        return OkHttpClient.Builder()
            .sslSocketFactory(context.socketFactory, trust)
            .hostnameVerifier { _, _ -> true } // Host identity is the explicit, verified certificate pin.
            .readTimeout(0, TimeUnit.MILLISECONDS)
            .pingInterval(5, TimeUnit.SECONDS)
            .build()
    }
}
