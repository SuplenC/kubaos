import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services

// Paired and discovered Bluetooth devices, and the pairing agent's prompt: the code to type on
// a keyboard, a passkey to confirm, or a PIN to enter. Enter on a device connects, disconnects,
// pairs, or cancels its pairing. A connected device carries the check.
Item {
    id: root

    readonly property var prompt: Bt.prompt
    readonly property string kind: prompt?.kind ?? ""
    // The prompts that take a PIN or passkey typed here.
    readonly property bool asksCode: kind === "passkey" || kind === "pin-code"
    // The prompts with a yes: the rest only show a code, and Cancel ends their pairing.
    readonly property bool asksYes: asksCode || kind === "confirm" || kind === "authorize"

    function promptText(p): string {
        switch (p.kind) {
        case "display-passkey":
        case "display-pin-code":
            return `Type this on ${p.name}`;
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

    function submitCode(): void {
        Bt.answer(true, code.text);
        code.text = "";
        list.forceActiveFocus();
    }

    onPromptChanged: if (asksCode && !code.focused) {
        code.text = "";
        code.forceActiveFocus();
    }

    K.Stack {
        id: top

        width: root.width
        room: root.width
        fill: true
        gap: K.Tokens.space.s8

        K.Row {
            fill: true
            gap: K.Tokens.space.s8
            align: "centre"

            // Takes the room the buttons leave, and wraps there.
            K.Stack {
                fill: true

                K.Text {
                    text: Bt.problem || (Bt.blocked ? Bt.blockedText : Bt.scanning ? "Searching for devices…" : `${Bt.devices.length} devices`)
                    style: K.Tokens.type.body
                    color: Bt.problem || Bt.blocked ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
                }
            }

            K.IconButton {
                visible: Bt.available && !Bt.enabled && !Bt.blocked
                icon: "power_settings_new"
                label: "Turn Bluetooth on"
                kind: "Tonal"
                size: "Sm"
                onActivated: Bt.powerOn()
            }

            K.IconButton {
                enabled: Bt.enabled
                icon: "radar"
                label: Bt.scanning ? "Stop searching" : "Search for devices"
                kind: Bt.scanning ? "Filled" : "Tonal"
                size: "Sm"
                onActivated: Bt.setScanning(!Bt.scanning)
            }
        }

        // Enter on a device that is pairing stops it, as its row's action.
        K.KeyHints {
            visible: root.prompt === null && Bt.devices.some(device => Bt.isPairing(device))
            hints: [["Enter", "cancel the pairing"]]
        }

        K.Stack {
            visible: root.prompt !== null
            fill: true
            gap: K.Tokens.space.s4

            K.Text {
                text: root.prompt ? root.promptText(root.prompt) : ""
                style: K.Tokens.type.body
            }

            K.Row {
                fill: true
                gap: K.Tokens.space.s8
                align: "centre"

                K.Text {
                    visible: (root.prompt?.code ?? null) !== null
                    text: root.prompt?.code ?? ""
                    style: K.Tokens.type.headline
                    color: K.Tokens.color.primary
                }

                K.Text {
                    visible: (root.prompt?.entered ?? null) !== null
                    text: `${root.prompt?.entered ?? 0} of ${root.prompt?.code?.length ?? 6} typed`
                    style: K.Tokens.type.body
                    color: K.Tokens.color.on_surface_variant
                }

                K.Fill {
                }

                K.IconButton {
                    visible: root.asksYes && !root.asksCode
                    icon: "check"
                    label: "Pair"
                    kind: "Filled"
                    size: "Sm"
                    onActivated: Bt.answer(true, "")
                }

                K.IconButton {
                    icon: "close"
                    label: "Cancel"
                    kind: "Tonal"
                    size: "Sm"
                    onActivated: Bt.answer(false, "")
                }
            }

            K.Field {
                id: code

                visible: root.asksCode
                icon: "key"
                placeholder: root.kind === "passkey" ? "Six digits" : "The PIN"
                Keys.onEscapePressed: {
                    Bt.answer(false, "");
                    list.forceActiveFocus();
                }
                Keys.onReturnPressed: root.submitCode()
                Keys.onEnterPressed: root.submitCode()
            }

            // The code shown is typed on the device, then its own Enter.
            K.KeyHints {
                visible: root.kind === "display-passkey" || root.kind === "display-pin-code"
                hints: [["Enter", `on ${root.prompt?.name ?? "it"}, after the code`]]
            }

            K.KeyHints {
                visible: root.asksCode
                hints: [["Enter", "pair"], ["Esc", "cancel"]]
            }
        }
    }

    Well {
        y: top.height + K.Tokens.space.s12
        width: root.width
        height: root.height - y
        focused: list.activeFocus

        NavList {
            id: list

            anchors.fill: parent
            // The well's padding above the first row and below the last, scrolled with them.
            topMargin: K.Tokens.space.s12
            bottomMargin: K.Tokens.space.s12
            visible: count > 0
            model: ScriptModel {
                values: Bt.devices
            }
            highlightShown: activeFocus
            onActivated: index => Bt.activate(Bt.devices[index])

            delegate: K.ListRow {
                required property var modelData
                required property int index

                width: ListView.view.width
                icon: modelData.connected ? "bluetooth_connected" : "bluetooth"
                title: modelData.name
                subtitle: Bt.stateText(modelData)
                checkable: true
                checked: modelData.connected
                selected: ListView.isCurrentItem && ListView.view.highlightShown
                onActivated: Bt.activate(modelData)
            }
        }

        K.EmptyState {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width
            room: parent.width
            visible: list.count === 0
            icon: Bt.enabled ? "bluetooth_searching" : "bluetooth_disabled"
            title: !Bt.available ? "No Bluetooth adapter" : Bt.blocked ? "Bluetooth is blocked" : Bt.enabled ? "No devices" : "Bluetooth is off"
            body: !Bt.available ? "Nothing to pair with on this machine." : Bt.blocked ? "Unblock it, then turn it on." : Bt.enabled ? "Search, and devices in pairing mode show up here." : "The Bluetooth tile turns it on."
        }
    }
}
