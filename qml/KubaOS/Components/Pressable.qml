import QtQuick

// What every pressable component shares: the pointer over it and pressing it, the keyboard on it,
// and pressing it with Enter or Space once per press, never on key repeat. Disabled
// (`enabled: false`), it takes no input and no focus.
Item {
    id: root

    // Pressed by a click, Enter or Space.
    signal activated

    // A state shown whatever the pointer does, `hovered` or `pressed`: the specimen's view of them.
    property string preview: ""
    // Whether the keyboard is on it; Qt's focus by default, or the app's where it owns focus.
    property bool focused: root.activeFocus
    // Whether Space presses it; a field's text takes Space instead.
    property bool spacePresses: true

    // The pointer itself over it, whatever the preview.
    readonly property bool pointerOver: pointer.containsMouse
    readonly property bool hovered: root.enabled && (root.preview === "hovered" || root.pointerOver)
    readonly property bool down: root.enabled && (root.preview === "pressed" || pointer.pressed || root.keyDown)
    property bool keyDown: false

    activeFocusOnTab: root.enabled

    function presses(event: KeyEvent): bool {
        return event.key === Qt.Key_Return || event.key === Qt.Key_Enter || (root.spacePresses && event.key === Qt.Key_Space);
    }

    Keys.onPressed: event => {
        if (!root.presses(event))
            return;
        event.accepted = true;
        if (event.isAutoRepeat)
            return;
        root.keyDown = true;
        root.activated();
    }
    Keys.onReleased: event => {
        if (!root.presses(event) || event.isAutoRepeat)
            return;
        event.accepted = true;
        root.keyDown = false;
    }
    onActiveFocusChanged: root.keyDown = false

    MouseArea {
        id: pointer

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.activated()
    }
}
