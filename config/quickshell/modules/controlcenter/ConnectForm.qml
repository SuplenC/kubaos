import QtQuick
import KubaOS.Components as K

// Connecting to one network or device, in place of its list (docs/DESIGN.md, "A form takes its
// list's place"), centred in the well: the item on a large tile, its name, a line on what it is
// or what to do, then what it asks for (a field, or a code to type on the device), a status line
// (what is happening, or the error, which keeps the form open), and Cancel and the action. Enter
// does the action and Esc cancels; Tab goes field, Cancel, action. The Wi-Fi and Bluetooth tabs
// share it.
FocusScope {
    id: root

    property string icon
    property string title
    property string subtitle
    property string subtitleIcon: ""
    // What it asks for: "" (nothing), "field" (`placeholder`, bullets when `secret`) or "code" (a
    // code shown to type or check on the device).
    property string ask: ""
    property string placeholder
    property bool secret: false
    property string code
    // What is happening (Connecting…), or why it didn't.
    property string status
    property bool failed: false
    property string action: "Connect"
    // Whether the action can be taken: not while it is already under way.
    property bool ready: true
    readonly property string value: field.text

    // The action, with what was typed.
    signal submitted(string value)
    signal cancelled

    // A new item to connect to: an empty field, and the keyboard on what comes first.
    function begin(): void {
        field.text = "";
        Qt.callLater(root.focusFirst);
    }

    function focusFirst(): void {
        if (root.ask === "field")
            field.forceActiveFocus();
        else if (root.ready)
            actionButton.forceActiveFocus();
        else
            cancelButton.forceActiveFocus();
    }

    function submit(): void {
        if (root.ready)
            root.submitted(field.text);
    }

    Keys.onEscapePressed: root.cancelled()
    Keys.onReturnPressed: root.submit()
    Keys.onEnterPressed: root.submit()

    K.Stack {
        id: form

        y: K.Look.snap(Math.max(0, (root.height - height) / 2), dpr)
        width: root.width
        room: width
        centred: true

        readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

        K.IconTile {
            icon: root.icon
            size: "Lg"
            accent: true
        }

        K.Gap {
            space: K.Tokens.space.s12
        }

        K.Text {
            room: form.width
            text: root.title
            style: K.Tokens.type.headline
            maximumLines: 1
            horizontalAlignment: Text.AlignHCenter
        }

        K.Gap {
            space: K.Tokens.space.s4
        }

        K.Row {
            gap: K.Tokens.space.s4
            align: "centre"

            K.Icon {
                visible: root.subtitleIcon !== ""
                name: root.subtitleIcon
                size: K.Tokens.type.small.icon
                color: K.Tokens.color.on_surface_variant
            }

            K.Text {
                room: form.width - (root.subtitleIcon === "" ? 0 : K.Tokens.type.small.icon + K.Tokens.space.s4)
                text: root.subtitle
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
                horizontalAlignment: Text.AlignHCenter
            }
        }

        K.Gap {
            visible: root.ask !== ""
            space: K.Tokens.space.s16
        }

        K.Field {
            id: field

            visible: root.ask === "field"
            icon: root.secret ? "key" : "passkey"
            secret: root.secret
            placeholder: root.placeholder
            status: root.failed ? "Invalid" : "Normal"
            onAccepted: root.submit()
        }

        K.Text {
            visible: root.ask === "code"
            text: root.code
            style: K.Tokens.type.headline
            color: K.Tokens.color.primary
        }

        K.Gap {
            space: K.Tokens.space.s8
        }

        // The status line keeps its room while empty, so a status arriving moves nothing.
        Item {
            implicitWidth: form.width
            implicitHeight: Math.max(K.Tokens.type.small.line, note.implicitHeight)

            K.Text {
                id: note

                width: parent.width
                room: parent.width
                text: root.status
                style: K.Tokens.type.small
                color: root.failed ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
                horizontalAlignment: Text.AlignHCenter
            }
        }

        K.Gap {
            space: K.Tokens.space.s16
        }

        K.Row {
            fill: true
            gap: K.Tokens.space.s8
            align: "centre"

            K.Button {
                id: cancelButton

                text: "Cancel"
                kind: "Text"
                onActivated: root.cancelled()
            }

            K.Fill {
            }

            K.Button {
                id: actionButton

                text: root.action
                kind: "Filled"
                enabled: root.ready
                onActivated: root.submit()
            }
        }
    }
}
