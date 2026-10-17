pragma Singleton

import Quickshell
import Quickshell.Networking

// NetworkManager state through Quickshell.Networking.
Singleton {
    id: root

    readonly property var wifiDevice: Networking.devices.values.find(d => d.type === DeviceType.Wifi) ?? null
    readonly property bool wired: Networking.devices.values.some(d => d.type === DeviceType.Wired && d.connected)
    readonly property bool wifiEnabled: Networking.wifiEnabled
    // Connected network first, then by signal strength.
    readonly property var networks: wifiDevice ? wifiDevice.networks.values.slice().sort((a, b) => (b.connected - a.connected) || (b.signalStrength - a.signalStrength)) : []
    readonly property var active: networks.find(n => n.connected) ?? null
    readonly property string icon: {
        if (wired)
            return "lan";
        if (!wifiEnabled)
            return "wifi_off";
        if (!active)
            return "signal_wifi_0_bar";
        return strengthIcon(active.signalStrength);
    }

    function strengthIcon(strength: real): string {
        if (strength > 0.75)
            return "signal_wifi_4_bar";
        return `network_wifi_${Math.max(1, Math.ceil(strength * 4) - 1)}_bar`;
    }

    function isSecured(network): bool {
        return network.security !== WifiSecurityType.Open && network.security !== WifiSecurityType.Owe;
    }

    function setWifiEnabled(on: bool): void {
        Networking.wifiEnabled = on;
    }

    function setScanning(on: bool): void {
        if (wifiDevice)
            wifiDevice.scannerEnabled = on;
    }
}
