// [LINE BUDGET AUDIT] 235/250
package com.streamdeck.client.data.net

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.net.wifi.WifiManager
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

data class DiscoveredHost(
    val name: String,
    val address: String,
)

class DeckDiscoveryService(private val context: Context) {
    private val nsdManager = context.getSystemService(Context.NSD_SERVICE) as NsdManager
    private val wifiManager = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
    private val serviceType = "_streamdeck._tcp"
    private val tag = "DeckDiscovery"

    private var multicastLock: WifiManager.MulticastLock? = null
    private var scanScope: CoroutineScope? = null
    private val _discovered = MutableStateFlow<List<DiscoveredHost>>(emptyList())
    val discovered: StateFlow<List<DiscoveredHost>> = _discovered.asStateFlow()

    private val _isScanning = MutableStateFlow(false)
    val isScanning: StateFlow<Boolean> = _isScanning.asStateFlow()

    private var discoveryListener: NsdManager.DiscoveryListener? = null
    private var isResolving = false
    private val pendingResolves = ArrayDeque<NsdServiceInfo>()

    fun startDiscovery() {
        if (discoveryListener != null) return
        acquireMulticastLock()
        _discovered.value = emptyList()
        _isScanning.value = true

        startSubnetProbe()

        discoveryListener = object : NsdManager.DiscoveryListener {
            override fun onDiscoveryStarted(regType: String) {
                Log.i(tag, "Service discovery started")
                _isScanning.value = true
            }

            override fun onServiceFound(service: NsdServiceInfo) {
                Log.i(tag, "Service found: ${service.serviceName}, type: ${service.serviceType}")
                if (service.serviceType?.contains("_streamdeck._tcp") == true) {
                    queueResolve(service)
                }
            }

            override fun onServiceLost(service: NsdServiceInfo) {
                Log.i(tag, "Service lost: ${service.serviceName}")
                _discovered.update { current -> current.filterNot { it.name == service.serviceName } }
            }

            override fun onDiscoveryStopped(serviceType: String) {
                Log.i(tag, "Discovery stopped: $serviceType")
                _isScanning.value = false
            }

            override fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {
                Log.e(tag, "Discovery failed: Error code:$errorCode")
                stopDiscovery()
            }

            override fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {
                Log.e(tag, "Stop Discovery failed: Error code:$errorCode")
                stopDiscovery()
            }
        }

        try {
            nsdManager.discoverServices(
                serviceType,
                NsdManager.PROTOCOL_DNS_SD,
                discoveryListener,
            )
        } catch (e: Exception) {
            Log.e(tag, "Failed to start service discovery", e)
            _isScanning.value = false
        }
    }

    private fun queueResolve(serviceInfo: NsdServiceInfo) {
        synchronized(pendingResolves) {
            if (isResolving) {
                pendingResolves.addLast(serviceInfo)
            } else {
                isResolving = true
                resolveNext(serviceInfo)
            }
        }
    }

    private fun resolveNext(serviceInfo: NsdServiceInfo) {
        nsdManager.resolveService(serviceInfo, object : NsdManager.ResolveListener {
            override fun onResolveFailed(info: NsdServiceInfo, errorCode: Int) {
                Log.e(tag, "Resolve failed for ${info.serviceName}: $errorCode")
                onResolveComplete()
            }

            override fun onServiceResolved(info: NsdServiceInfo) {
                val host = info.host?.hostAddress?.removePrefix("/")
                val port = info.port
                if (!host.isNullOrBlank()) {
                    val address = "$host:$port"
                    val hostItem = DiscoveredHost(
                        name = info.serviceName ?: "StreamDeck Host",
                        address = address,
                    )
                    _discovered.update { current ->
                        if (current.none { it.address == address }) current + hostItem else current
                    }
                }
                onResolveComplete()
            }
        })
    }

    private fun onResolveComplete() {
        synchronized(pendingResolves) {
            val next = pendingResolves.removeFirstOrNull()
            if (next != null) {
                resolveNext(next)
            } else {
                isResolving = false
            }
        }
    }

    private fun acquireMulticastLock() {
        try {
            if (multicastLock == null) {
                multicastLock = wifiManager?.createMulticastLock("streamdeck_mdns_lock")?.apply {
                    setReferenceCounted(true)
                }
            }
            multicastLock?.let {
                if (!it.isHeld) it.acquire()
            }
        } catch (e: Exception) {
            Log.w(tag, "Failed to acquire MulticastLock: ${e.message}")
        }
    }

    private fun releaseMulticastLock() {
        try {
            multicastLock?.let {
                if (it.isHeld) it.release()
            }
        } catch (e: Exception) {
            Log.w(tag, "Failed to release MulticastLock: ${e.message}")
        }
    }

    fun stopDiscovery() {
        _isScanning.value = false
        scanScope?.cancel()
        scanScope = null
        releaseMulticastLock()
        discoveryListener?.let {
            try {
                nsdManager.stopServiceDiscovery(it)
            } catch (e: Exception) {
                Log.e(tag, "Error stopping discovery", e)
            }
            discoveryListener = null
        }
        synchronized(pendingResolves) {
            pendingResolves.clear()
            isResolving = false
        }
    }

    private fun probeHost(host: String, port: Int, displayName: String) {
        try {
            java.net.Socket().use { s ->
                s.connect(java.net.InetSocketAddress(host, port), 250)
                val address = "$host:$port"
                val item = DiscoveredHost(displayName, address)
                _discovered.update { list -> if (list.none { it.address == address }) list + item else list }
            }
        } catch (_: Exception) {}
    }

    private fun startSubnetProbe() {
        scanScope?.cancel()
        val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
        scanScope = scope
        scope.launch {
            probeHost("127.0.0.1", 4455, "StreamDeck (ADB Loopback)")
            val prefs = context.getSharedPreferences("deck_prefs", Context.MODE_PRIVATE)
            prefs.getString("last_host", null)?.let { saved ->
                if (saved.isNotBlank() && !saved.startsWith("127.0.0.1")) {
                    val parts = saved.split(":")
                    val host = parts.getOrNull(0).orEmpty()
                    val port = parts.getOrNull(1)?.toIntOrNull() ?: 4455
                    if (host.isNotEmpty()) probeHost(host, port, "StreamDeck ($host)")
                }
            }
            val ipInt = wifiManager?.connectionInfo?.ipAddress ?: 0
            if (ipInt != 0) {
                val b1 = ipInt and 0xff
                val b2 = (ipInt shr 8) and 0xff
                val b3 = (ipInt shr 16) and 0xff
                val prefix = "$b1.$b2.$b3"
                kotlinx.coroutines.coroutineScope {
                    (1..254).map { hostNum ->
                        async { probeHost("$prefix.$hostNum", 4455, "StreamDeck ($prefix.$hostNum)") }
                    }.awaitAll()
                }
            }
        }
    }

    fun rescan() {
        stopDiscovery()
        startDiscovery()
    }
}
