pragma Singleton
import QtQuick

// Stand-in for Bt: headphones and a mouse connected with their batteries, a keyboard and a phone
// paired, and three devices in range at three distances. The drivers set `prompt`, `pairing` and
// `errors` for the pairing states.
QtObject {
    id: root

    function device(name: string, icon: string, connected: bool, paired: bool, battery: real): QtObject {
        return Qt.createQmlObject(`import QtQuick; QtObject { property string name: "${name}"; property string address: "00:11:22:33:44:55"; property string dbusPath: "/org/bluez/hci0/${name.replace(/\W/g, "_")}"; property string icon: "${icon}"; property bool connected: ${connected}; property bool paired: ${paired}; property bool trusted: ${paired}; property bool batteryAvailable: ${battery >= 0}; property real battery: ${battery}; property int state: ${connected ? 1 : 0}; function connect() {} function disconnect() {} function forget() {} }`, root);
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
    property var devices: [root.device("WH-1000XM4", "audio-headset", true, true, 0.8), root.device("MX Master 3S", "input-mouse", true, true, 0.45), root.device("Keychron K3", "input-keyboard", false, true, -1), root.device("Pixel 9", "phone", false, true, -1), root.device("JBL Flip 6", "audio-card", false, false, -1), root.device("MX Keys Mini", "input-keyboard", false, false, -1), root.device("Galaxy Watch", "", false, false, -1)]
    property var rssi: ({
            "/org/bluez/hci0/JBL_Flip_6": -48,
            "/org/bluez/hci0/MX_Keys_Mini": -70,
            "/org/bluez/hci0/Galaxy_Watch": -86
        })
    // The object paths being paired, and each one's last failure.
    property var pairing: []
    property var errors: ({})
    // As radios.js orders them, the fixture's RSSIs already nearest first; written out here, as the
    // tree before radios.js existed has none.
    readonly property var known: devices.filter(d => d.paired).sort((a, b) => (b.connected - a.connected) || a.name.localeCompare(b.name))
    readonly property var nearby: devices.filter(d => !d.paired).sort((a, b) => (rssi[b.dbusPath] ?? -999) - (rssi[a.dbusPath] ?? -999))
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
    }

    function show(): void {
        root.reveal();
    }

    function isPairing(device: var): bool {
        return root.pairing.includes(device.dbusPath);
    }

    function pairError(device: var): string {
        return root.errors[device.dbusPath] ?? "";
    }

    function activate(device: var): void {
    }

    // The tree before the tabs' redesign reads its rows' lines here.
    function stateText(device: var): string {
        if (isPairing(device))
            return "Pairing…";
        const battery = device.batteryAvailable ? ` · ${Math.round(device.battery * 100)}%` : "";
        if (device.paired || device.connected)
            return (device.connected ? "Connected" : "Disconnected") + battery;
        return "Not paired";
    }

    function answer(accept: bool, value: string): void {
    }

    function run(args: var, report: bool): void {
    }
}
