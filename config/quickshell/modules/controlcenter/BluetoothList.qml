import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services

// Paired and discovered Bluetooth devices, and the pairing agent's prompt: the code to type on
// a keyboard, a passkey to confirm, or a PIN to enter. Enter on a device connects, disconnects,
// pairs, or cancels its pairing.
ColumnLayout {
    id: root

    readonly property var prompt: Bt.prompt
    readonly property string kind: prompt?.kind ?? ""
    // The prompts that take a PIN or passkey typed here.
    readonly property bool asksCode: kind === "passkey" || kind === "pin-code"
    // The prompts with a yes: the rest only show a code, and Cancel ends their pairing.
    readonly property bool asksYes: asksCode || kind === "confirm" || kind === "authorize"

    spacing: Theme.spacing.sm

    function promptText(p): string {
        switch (p.kind) {
        case "display-passkey":
        case "display-pin-code":
            return `Type this on ${p.name}, then Enter`;
        case "confirm":
            return `Pair with ${p.name}? It should show this code`;
        case "authorize":
            return `${p.name} wants to pair`;
        case "passkey":
            return `Enter the passkey ${p.name} shows`;
        case "pin-code":
            return `Enter the PIN for ${p.name}`;
        }
        return "";
    }

    onPromptChanged: if (asksCode && !code.input.activeFocus) {
        code.text = "";
        code.focusInput();
    }

    RowLayout {
        Layout.fillWidth: true

        StyledText {
            Layout.fillWidth: true
            text: Bt.problem || (Bt.blocked ? Bt.blockedText : Bt.scanning ? "Searching for devices…" : `${Bt.devices.length} devices`)
            color: Bt.problem || Bt.blocked ? Theme.m3error : Theme.m3onSurfaceVariant
            wrapMode: Text.Wrap
            elide: Text.ElideNone
        }

        IconButton {
            icon: "power_settings_new"
            tooltip: "Turn Bluetooth on"
            visible: Bt.available && !Bt.enabled && !Bt.blocked
            onClicked: Bt.powerOn()
        }

        IconButton {
            icon: "radar"
            tooltip: Bt.scanning ? "Stop searching" : "Search for devices"
            toggled: Bt.scanning
            enabled: Bt.enabled
            onClicked: Bt.setScanning(!Bt.scanning)
        }
    }

    ColumnLayout {
        Layout.fillWidth: true
        visible: root.prompt !== null
        spacing: Theme.spacing.xs

        StyledText {
            Layout.fillWidth: true
            text: root.prompt ? root.promptText(root.prompt) : ""
            wrapMode: Text.Wrap
            elide: Text.ElideNone
        }

        RowLayout {
            Layout.fillWidth: true

            StyledText {
                Layout.fillWidth: true
                visible: (root.prompt?.code ?? null) !== null
                text: root.prompt?.code ?? ""
                font.pixelSize: Theme.font.xxl
                font.features: Theme.font.tabular
                color: Theme.m3primary
            }

            StyledText {
                visible: (root.prompt?.entered ?? null) !== null
                text: `${root.prompt?.entered ?? 0} of ${root.prompt?.code?.length ?? 6} typed`
                color: Theme.m3onSurfaceVariant
            }

            Item {
                Layout.fillWidth: true
                visible: (root.prompt?.code ?? null) === null
            }

            IconButton {
                icon: "check"
                tooltip: "Pair"
                visible: root.asksYes && !root.asksCode
                onClicked: Bt.answer(true, "")
            }

            IconButton {
                icon: "close"
                tooltip: "Cancel"
                onClicked: Bt.answer(false, "")
            }
        }

        SearchField {
            id: code

            Layout.fillWidth: true
            visible: root.asksCode
            icon: "key"
            placeholder: root.kind === "passkey" ? "Six digits, then Enter" : "The PIN, then Enter"
            catchEscape: true
            onEscaped: {
                Bt.answer(false, "");
                list.forceActiveFocus();
            }
            onAccepted: {
                Bt.answer(true, text);
                text = "";
                list.forceActiveFocus();
            }
        }
    }

    NavList {
        id: list

        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: count > 0
        model: ScriptModel {
            values: Bt.devices
        }
        highlightShown: activeFocus
        onActivated: index => Bt.activate(Bt.devices[index])

        delegate: ListRow {
            required property var modelData
            required property int index

            width: ListView.view.width
            icon: modelData.connected ? "bluetooth_connected" : "bluetooth"
            title: modelData.name
            subtitle: Bt.stateText(modelData)
            active: modelData.connected
            current: ListView.isCurrentItem && ListView.view.activeFocus
            onClicked: Bt.activate(modelData)
        }
    }

    Item {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: list.count === 0

        EmptyState {
            anchors.centerIn: parent
            icon: Bt.enabled ? "bluetooth_searching" : "bluetooth_disabled"
            text: !Bt.available ? "No Bluetooth adapter" : Bt.blocked ? "Bluetooth is blocked" : Bt.enabled ? "No devices" : "Bluetooth is off"
        }
    }
}
