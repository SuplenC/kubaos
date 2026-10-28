pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Networking
import "radios.js" as Radios

// NetworkManager state through Quickshell.Networking.
Singleton {
    id: root

    readonly property var wifiDevice: Networking.devices.values.find(d => d.type === DeviceType.Wifi) ?? null
    readonly property bool wired: Networking.devices.values.some(d => d.type === DeviceType.Wired && d.connected)
    readonly property bool wifiEnabled: Networking.wifiEnabled
    // The networks in range: NetworkManager drops one a few scans after it was last seen.
    readonly property var networks: wifiDevice ? wifiDevice.networks.values : []
    // Saved networks in range, the one in use first; then the rest, by signal (radios.js).
    readonly property var known: Radios.wifiKnown(networks)
    readonly property var nearby: Radios.wifiNearby(networks)
    readonly property var active: networks.find(n => n.connected) ?? null
    // The band the network in use is on (5 GHz), or empty while that isn't known.
    property string band: ""
    // Whether the Wi-Fi tab has NetworkManager scanning.
    readonly property bool scanning: wifiDevice?.scannerEnabled ?? false
    readonly property string icon: {
        if (wired)
            return "lan";
        if (!wifiEnabled)
            return "wifi_off";
        if (!active)
            return "signal_wifi_0_bar";
        return strengthIcon(active.signalStrength);
    }

    // The control center goes to its Wi-Fi tab.
    signal reveal

    function bars(strength: real): int {
        return Radios.bars(strength);
    }

    // KubaOS Symbols has a wedge per bar count, named as Material Symbols named them; the two ends
    // carry different prefixes.
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

    // How a secured network is secured, as the connection form names it.
    function securityText(network): string {
        switch (network.security) {
        case WifiSecurityType.Sae:
            return "WPA3";
        case WifiSecurityType.Wpa2Psk:
            return "WPA2";
        case WifiSecurityType.WpaPsk:
            return "WPA";
        case WifiSecurityType.StaticWep:
        case WifiSecurityType.DynamicWep:
            return "WEP";
        }
        return "Enterprise";
    }

    function setWifiEnabled(on: bool): void {
        Networking.wifiEnabled = on;
    }

    function setScanning(on: bool): void {
        if (wifiDevice)
            wifiDevice.scannerEnabled = on;
    }

    // The control center on the Wi-Fi tab.
    function show(): void {
        reveal();
        Panels.open("controlcenter");
    }

    // Quickshell's networks carry no frequency: `nmcli` reads the one in use, read-only, without a
    // rescan, each time it changes, and after each scan, which sees a roam between bands.
    function readBand(): void {
        frequency.running = false;
        if (root.active)
            frequency.running = true;
    }

    onActiveChanged: {
        root.band = "";
        root.readBand();
    }
    onScanningChanged: if (!root.scanning)
        root.readBand()

    Process {
        id: frequency

        command: ["nmcli", "--terse", "--fields", "ACTIVE,FREQ", "device", "wifi", "list", "--rescan", "no"]
        stdout: StdioCollector {
            onStreamFinished: {
                const line = text.split("\n").find(l => l.startsWith("yes:")) ?? "";
                root.band = Radios.band(parseInt(line.slice(4)) || 0);
            }
        }
    }
}
