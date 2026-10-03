// [LINE BUDGET AUDIT] 80/250
package com.streamdeck.client.data.net

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.wifi.WifiInfo
import android.net.wifi.WifiManager
import android.os.Build
import androidx.core.content.ContextCompat

sealed interface WifiState {
    data class Connected(val ssid: String) : WifiState
    data object Disconnected : WifiState
    data object PermissionRequired : WifiState
}

object WifiInfoHelper {

    fun hasWifiPermission(context: Context): Boolean {
        val fineLocation = ContextCompat.checkSelfPermission(
            context,
            Manifest.permission.ACCESS_FINE_LOCATION,
        ) == PackageManager.PERMISSION_GRANTED

        val coarseLocation = ContextCompat.checkSelfPermission(
            context,
            Manifest.permission.ACCESS_COARSE_LOCATION,
        ) == PackageManager.PERMISSION_GRANTED

        val nearbyDevices = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.NEARBY_WIFI_DEVICES,
            ) == PackageManager.PERMISSION_GRANTED
        } else {
            false
        }

        return fineLocation || coarseLocation || nearbyDevices
    }

    fun getWifiState(context: Context): WifiState {
        val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            ?: return WifiState.Disconnected
        val activeNet = cm.activeNetwork ?: return WifiState.Disconnected
        val caps = cm.getNetworkCapabilities(activeNet) ?: return WifiState.Disconnected

        if (!caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)) {
            return WifiState.Disconnected
        }

        if (!hasWifiPermission(context)) {
            return WifiState.PermissionRequired
        }

        val wifiManager = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
        val capsSsid = (caps.transportInfo as? WifiInfo)?.ssid?.removeSurrounding("\"")
        val directSsid = wifiManager?.connectionInfo?.ssid?.removeSurrounding("\"")

        val rawSsid = when {
            !capsSsid.isNullOrBlank() && capsSsid != "<unknown ssid>" -> capsSsid
            !directSsid.isNullOrBlank() && directSsid != "<unknown ssid>" -> directSsid
            else -> null
        }

        return if (!rawSsid.isNullOrBlank()) {
            WifiState.Connected(rawSsid)
        } else {
            WifiState.Connected(ssid = "Wi-Fi (Active)")
        }
    }
}
