import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Networking
import qs.theme
import qs.components
import qs.services

// Nearby Wi-Fi networks. Enter connects; secured networks without saved
// credentials (or whose saved ones were rejected) ask for a password below.
ColumnLayout {
    id: root

    property var pending: null

    spacing: Theme.spacing.sm

    function choose(network): void {
        if (network.connected)
            return;
        if (network.known || !Net.isSecured(network))
            network.connect();
        else
            askPassword(network);
    }

    function askPassword(network): void {
        pending = network;
        password.text = "";
        password.focusInput();
    }

    function statusText(network): string {
        if (network.connected)
            return "Connected";
        if (network.stateChanging)
            return ConnectionState.toString(network.state);
        if (network.known)
            return "Saved";
        return Net.isSecured(network) ? "Secured" : "Open";
    }

    NavList {
        id: list

        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: count > 0
        model: ScriptModel {
            values: Net.networks
        }
        highlightShown: activeFocus
        onActivated: index => root.choose(Net.networks[index])

        delegate: ListRow {
            id: row

            required property var modelData
            required property int index

            width: ListView.view.width
            icon: Net.strengthIcon(modelData.signalStrength)
            title: modelData.name
            subtitle: root.statusText(modelData)
            active: modelData.connected
            current: ListView.isCurrentItem && ListView.view.activeFocus
            onClicked: root.choose(modelData)

            Icon {
                name: "lock"
                size: 16
                visible: Net.isSecured(row.modelData)
                color: Theme.m3outline
            }

            Connections {
                target: row.modelData

                function onConnectionFailed(reason) {
                    if (reason === ConnectionFailReason.NoSecrets)
                        root.askPassword(row.modelData);
                }
            }
        }
    }

    Item {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: list.count === 0

        EmptyState {
            anchors.centerIn: parent
            icon: Net.wifiEnabled ? "wifi_find" : "wifi_off"
            text: Net.wifiEnabled ? "Scanning…" : "Wi-Fi is off"
        }
    }

    SearchField {
        id: password

        Layout.fillWidth: true
        visible: root.pending !== null
        icon: "key"
        echoMode: TextInput.Password
        placeholder: `Password for ${root.pending?.name ?? ""}`
        catchEscape: true
        onEscaped: {
            root.pending = null;
            list.forceActiveFocus();
        }
        onAccepted: {
            root.pending.connectWithPsk(text);
            root.pending = null;
            list.forceActiveFocus();
        }
    }
}
