import QtQuick
import qs.theme

// The text cursor of the menu's inputs, a cursorDelegate. It blinks on a
// Timer, not an animation, as the shell's SearchField does: a running
// animation repaints the window every frame, a Timer twice a second.
Rectangle {
    id: caret

    property bool on: true

    width: 2
    radius: 1
    color: Theme.m3primary
    visible: (parent?.activeFocus ?? false) && on

    Timer {
        id: blink

        // The blink of components/SearchField.qml.
        interval: 530
        repeat: true
        running: caret.parent?.activeFocus ?? false
        onRunningChanged: caret.on = true
        onTriggered: caret.on = !caret.on
    }

    Connections {
        target: caret.parent

        function onCursorPositionChanged(): void {
            caret.on = true;
            blink.restart();
        }
    }
}
