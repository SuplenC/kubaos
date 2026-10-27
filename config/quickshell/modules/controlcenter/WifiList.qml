import QtQuick
import Quickshell
import Quickshell.Networking
import KubaOS.Components as K
import qs.components
import qs.services

// Nearby Wi-Fi networks. Enter connects; secured networks without saved
// credentials (or whose saved ones were rejected) ask for a password below.
// The connected one carries the check; whether one is secured is its subtitle.
Item {
    id: root

    property var pending: null

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
        password.forceActiveFocus();
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

    function backToList(): void {
        root.pending = null;
        list.forceActiveFocus();
    }

    Well {
        id: well

        width: root.width
        height: root.height - (password.visible ? password.height + K.Tokens.space.s12 : 0)
        focused: list.activeFocus

        NavList {
            id: list

            anchors.fill: parent
            // The well's padding above the first row and below the last, scrolled with them.
            topMargin: K.Tokens.space.s12
            bottomMargin: K.Tokens.space.s12
            visible: count > 0
            model: ScriptModel {
                values: Net.networks
            }
            highlightShown: activeFocus
            onActivated: index => root.choose(Net.networks[index])

            delegate: K.ListRow {
                id: row

                required property var modelData
                required property int index

                width: ListView.view.width
                icon: Net.strengthIcon(modelData.signalStrength)
                title: modelData.name
                subtitle: root.statusText(modelData)
                checkable: true
                checked: modelData.connected
                selected: ListView.isCurrentItem && ListView.view.highlightShown
                onActivated: root.choose(modelData)

                Connections {
                    target: row.modelData

                    function onConnectionFailed(reason) {
                        if (reason === ConnectionFailReason.NoSecrets)
                            root.askPassword(row.modelData);
                    }
                }
            }
        }

        K.EmptyState {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width
            room: parent.width
            visible: list.count === 0
            icon: Net.wifiEnabled ? "wifi_find" : "wifi_off"
            title: Net.wifiEnabled ? "Scanning…" : "Wi-Fi is off"
            body: Net.wifiEnabled ? "Networks show up here as they're found." : "The Wi-Fi tile turns it on."
        }
    }

    K.Field {
        id: password

        y: root.height - height
        width: root.width
        visible: root.pending !== null
        icon: "key"
        secret: true
        placeholder: `Password for ${root.pending?.name ?? ""}`
        Keys.onEscapePressed: root.backToList()
        Keys.onReturnPressed: {
            root.pending.connectWithPsk(text);
            root.backToList();
        }
        Keys.onEnterPressed: {
            root.pending.connectWithPsk(text);
            root.backToList();
        }
    }
}
