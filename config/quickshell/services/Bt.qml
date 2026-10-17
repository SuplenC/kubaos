pragma Singleton

import Quickshell
import Quickshell.Bluetooth

// BlueZ adapter and devices through Quickshell.Bluetooth.
Singleton {
    id: root

    readonly property BluetoothAdapter adapter: Bluetooth.defaultAdapter
    readonly property bool available: adapter !== null
    readonly property bool enabled: adapter?.enabled ?? false
    readonly property bool scanning: adapter?.discovering ?? false
    // Connected first, then paired, then by name. Unnamed discoveries are noise.
    readonly property var devices: Bluetooth.devices.values.filter(d => d.paired || d.name !== d.address).sort((a, b) => (b.connected - a.connected) || (b.paired - a.paired) || a.name.localeCompare(b.name))
    readonly property int connectedCount: devices.filter(d => d.connected).length
    readonly property string icon: !enabled ? "bluetooth_disabled" : connectedCount > 0 ? "bluetooth_connected" : "bluetooth"

    function toggle(): void {
        if (adapter)
            adapter.enabled = !adapter.enabled;
    }

    function setScanning(on: bool): void {
        if (adapter && adapter.enabled)
            adapter.discovering = on;
    }

    // Enter on a device: disconnect, connect, or pair (pairing new devices also trusts them).
    function activate(device: BluetoothDevice): void {
        if (device.connected) {
            device.disconnect();
        } else if (device.paired) {
            device.connect();
        } else {
            device.trusted = true;
            device.pair();
        }
    }

    function stateText(device: BluetoothDevice): string {
        if (device.pairing)
            return "Pairing…";
        const state = BluetoothDeviceState.toString(device.state);
        const battery = device.batteryAvailable ? ` · ${Math.round(device.battery * 100)}%` : "";
        return (device.paired || device.connected ? state : "Not paired") + battery;
    }
}
