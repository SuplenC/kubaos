pragma Singleton
import QtQuick

// Stand-in for Net: Wi-Fi on Shinjuku-5G at 5 GHz, one more saved network and three in range; a
// scan's driver empties the list, and the live-list drivers change it.
QtObject {
    id: root

    function network(name: string, strength: real, connected: bool, known: bool, secured: bool): QtObject {
        return Qt.createQmlObject(`import QtQuick; QtObject { property string name: "${name}"; property real signalStrength: ${strength}; property bool connected: ${connected}; property bool known: ${known}; property bool secured: ${secured}; property bool stateChanging: false; property int state: ${connected ? 3 : 1}; property int security: ${secured ? 1 : 0}; signal connectionFailed(int reason); function connect() {} function connectWithPsk(p) {} function disconnect() {} function forget() {} }`, root);
    }

    readonly property var wifiDevice: ({})
    property var networks: [root.network("Shinjuku-5G", 0.86, true, true, true), root.network("Golden Gai", 0.7, false, true, true), root.network("Omoide Yokocho", 0.52, false, false, true), root.network("7SPOT", 0.4, false, false, false), root.network("Kabukicho Free Wi-Fi", 0.2, false, false, false)]
    // As radios.js orders them; written out here, as the tree before radios.js existed has none.
    readonly property var known: networks.filter(n => n.known).sort((a, b) => (b.connected - a.connected) || (bars(b.signalStrength) - bars(a.signalStrength)) || a.name.localeCompare(b.name))
    readonly property var nearby: networks.filter(n => !n.known).sort((a, b) => (bars(b.signalStrength) - bars(a.signalStrength)) || a.name.localeCompare(b.name))
    readonly property var active: networks.find(n => n.connected) ?? null
    property string band: "5 GHz"
    property bool scanning: false
    readonly property bool wired: false
    property bool wifiEnabled: true
    readonly property string icon: wired ? "lan" : !wifiEnabled ? "wifi_off" : !active ? "signal_wifi_0_bar" : strengthIcon(active.signalStrength)

    signal reveal

    function bars(strength: real): int {
        const percent = Math.round(strength * 100);
        if (percent > 80)
            return 4;
        if (percent > 55)
            return 3;
        if (percent > 30)
            return 2;
        return percent > 5 ? 1 : 0;
    }

    function strengthIcon(strength: real): string {
        const n = bars(strength);
        if (n === 0)
            return "signal_wifi_0_bar";
        if (n === 4)
            return "signal_wifi_4_bar";
        return `network_wifi_${n}_bar`;
    }

    function isSecured(network: var): bool {
        return network.secured;
    }

    function securityText(network: var): string {
        return "WPA2";
    }

    function setWifiEnabled(on: bool): void {
        root.wifiEnabled = on;
    }

    function setScanning(on: bool): void {
        root.scanning = on;
    }

    function show(): void {
        root.reveal();
    }
}
