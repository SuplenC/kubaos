pragma Singleton
import QtQuick

// Stand-in for Bt: headphones connected, a mouse pairing, a keyboard paired and a speaker nearby.
// The drivers set `prompt` for the passkey state.
QtObject {
    id: root

    function device(name: string, connected: bool, paired: bool, battery: real): QtObject {
        return Qt.createQmlObject(`import QtQuick; QtObject { property string name: "${name}"; property string address: "00:11:22:33:44:55"; property string dbusPath: "/org/bluez/hci0/${name.replace(/\W/g, "_")}"; property string icon: ""; property bool connected: ${connected}; property bool paired: ${paired}; property bool trusted: ${paired}; property bool batteryAvailable: ${battery >= 0}; property real battery: ${battery}; property int state: ${connected ? 1 : 0}; function connect() {} function disconnect() {} }`, root);
    }

    readonly property bool available: true
    property bool enabled: true
    readonly property bool blocked: false
    property bool scanning: false
    readonly property bool agentRunning: true
    property var prompt: null
    readonly property int promptId: prompt ? 1 : 0
    readonly property string blockedText: "Blocked by rfkill (airplane mode). Unblock it: rfkill unblock bluetooth"
    property string problem: ""
    readonly property int powerOnWait: 4000
    readonly property var devices: [root.device("WH-1000XM4", true, true, 0.8), root.device("MX Master 3S", false, true, -1), root.device("Keychron K3", false, true, -1), root.device("JBL Flip 6", false, false, -1)]
    readonly property int connectedCount: devices.filter(d => d.connected).length
    readonly property string icon: !enabled ? "bluetooth_disabled" : connectedCount > 0 ? "bluetooth_connected" : "bluetooth"

    signal reveal

    function toggle(): void {
        root.enabled = !root.enabled;
    }

    function powerOn(): void {
        root.enabled = true;
    }

    function setScanning(on: bool): void {
        root.scanning = on;
    }

    function show(scan: bool): void {
        root.reveal();
    }

    function isPairing(device: var): bool {
        return device.name === "MX Master 3S";
    }

    function pairError(device: var): string {
        return "";
    }

    function activate(device: var): void {
    }

    function answer(accept: bool, value: string): void {
    }

    function stateText(device: var): string {
        if (isPairing(device))
            return "Pairing…";
        const battery = device.batteryAvailable ? ` · ${Math.round(device.battery * 100)}%` : "";
        if (device.paired || device.connected)
            return (device.connected ? "Connected" : "Disconnected") + battery;
        return "Not paired";
    }

    function run(args: var, report: bool): void {
    }
}
