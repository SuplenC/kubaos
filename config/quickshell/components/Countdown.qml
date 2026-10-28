import QtQuick

// A popup's time left, from 1 down to 0 over `duration` ms while `running`, stopped where it is
// while `held` (the pointer on the popup, or its menu open) and going on from there after.
QtObject {
    id: root

    property bool running: false
    property bool held: false
    property int duration: 0
    property real remaining: 1

    signal finished

    property NumberAnimation animation: NumberAnimation {
        target: root
        property: "remaining"
        from: 1
        to: 0
        duration: root.duration
        running: root.running
        paused: running && root.held
        onFinished: root.finished()
    }

    onRunningChanged: if (!root.running)
        root.remaining = 1
}
