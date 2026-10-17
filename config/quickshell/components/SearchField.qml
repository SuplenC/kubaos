import QtQuick
import qs.theme

// Pill-shaped single-line text input with a leading icon.
// Keys go to `forwardTo` items first (e.g. a NavList for Up/Down/Enter).
// With `catchEscape`, Esc emits escaped() instead of closing the surrounding panel.
Rectangle {
    id: root

    property list<Item> forwardTo

    property alias text: input.text
    property alias input: input
    property alias echoMode: input.echoMode
    property string placeholder
    property string icon: "search"
    property color iconColor: Theme.m3onSurfaceVariant
    property bool catchEscape: false

    signal accepted
    signal escaped

    implicitHeight: 42
    radius: Theme.rounding.full
    color: Theme.m3surfaceContainerHigh
    border.width: input.activeFocus ? 1 : 0
    border.color: Theme.alpha(Theme.m3primary, 0.5)

    function focusInput(): void {
        input.forceActiveFocus();
    }

    Icon {
        id: leading

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacing.lg
        anchors.verticalCenter: parent.verticalCenter
        name: root.icon
        color: root.iconColor

        Behavior on color {
            CAnim {}
        }
    }

    TextInput {
        id: input

        anchors.left: leading.right
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacing.md
        anchors.rightMargin: Theme.spacing.lg
        anchors.verticalCenter: parent.verticalCenter
        color: Theme.m3onSurface
        selectionColor: Theme.m3primary
        selectedTextColor: Theme.m3onPrimary
        font.family: Theme.font.sans
        font.pixelSize: Theme.font.lg
        renderType: Text.NativeRendering
        passwordCharacter: "•"
        clip: true
        Keys.forwardTo: root.forwardTo
        Keys.onEscapePressed: event => {
            event.accepted = root.catchEscape;
            if (root.catchEscape)
                root.escaped();
        }
        onAccepted: root.accepted()

        // Blinks on a Timer, not an animation: a running animation repaints the
        // whole layer every frame, a Timer only twice a second. Hidden panels
        // keep activeFocus, so it also stops when the field is not visible.
        cursorDelegate: Rectangle {
            id: cursor

            property bool on: true

            width: 2
            radius: 1
            color: Theme.m3primary
            visible: input.activeFocus && on

            Timer {
                id: blink

                interval: 530
                repeat: true
                running: input.activeFocus && root.visible
                onRunningChanged: cursor.on = true
                onTriggered: cursor.on = !cursor.on
            }

            Connections {
                target: input

                function onCursorPositionChanged() {
                    cursor.on = true;
                    blink.restart();
                }
            }
        }

        StyledText {
            anchors.fill: parent
            text: root.placeholder
            color: Theme.m3onSurfaceVariant
            font.pixelSize: Theme.font.lg
            visible: !input.text && !input.preeditText
        }
    }
}
