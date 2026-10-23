pragma Singleton

import Quickshell
import Quickshell.Networking

// NetworkManager state through Quickshell.Networking.
Singleton {
    id: root

    readonly property var wifiDevice: Networking.devices.values.find(d => d.type === DeviceType.Wifi) ?? null
    readonly property bool wired: Networking.devices.values.some(d => d.type === DeviceType.Wired && d.connected)
    readonly property bool wifiEnabled: Networking.wifiEnabled
    // Connected network first, then by signal bars, then by name. Sorting on bars rather than the raw
    // strength keeps the order still while a scan nudges every strength by a point or two.
    readonly property var networks: wifiDevice ? wifiDevice.networks.values.slice().sort((a, b) => (b.connected - a.connected) || (bars(b.signalStrength) - bars(a.signalStrength)) || a.name.localeCompare(b.name)) : []
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

    // Signal bars from 0 to 4 for a `signalStrength` (0 to 1), at the thresholds `nmcli` draws its
    // BARS column with, so the icons agree with `nmcli device wifi`.
    // https://github.com/NetworkManager/NetworkManager/blob/main/src/libnm-core-impl/nm-utils.c
    // (`nm_utils_wifi_strength_bars`)
    function bars(strength: real): int {
        // NetworkManager reports whole percents; rounding undoes the float division (0.55 * 100
        // is 55.00000000000001).
        const percent = Math.round(strength * 100);
        if (percent > 80)
            return 4;
        if (percent > 55)
            return 3;
        if (percent > 30)
            return 2;
        return percent > 5 ? 1 : 0;
    }

    // Material Symbols has a wedge per bar count; the two ends carry different prefixes.
    function strengthIcon(strength: real): string {
        const n = bars(strength);
        if (n === 0)
            return "signal_wifi_0_bar";
        if (n === 4)
            return "signal_wifi_4_bar";
        return `network_wifi_${n}_bar`;
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
