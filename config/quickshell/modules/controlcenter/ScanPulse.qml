import QtQuick

// When a radio scans for what is in range: while `active` (the control center open on the radio's
// tab) a scan starts at once, runs for `window`, and starts again every `every`; inactive, none
// runs and none is due. The owner turns the radio's scanning on and off with `scanning`.
QtObject {
    id: root

    property bool active: false
    // From one scan's start to the next's, in ms.
    property int every: 30000
    // How long each scan runs, in ms.
    property int window: 5000
    readonly property bool scanning: root.on

    property bool on: false

    property Timer windowTimer: Timer {
        interval: root.window
        onTriggered: root.on = false
    }

    property Timer periodTimer: Timer {
        interval: root.every
        repeat: true
        onTriggered: root.begin()
    }

    function begin(): void {
        root.on = true;
        root.windowTimer.restart();
    }

    onActiveChanged: {
        if (root.active) {
            root.begin();
            root.periodTimer.restart();
        } else {
            root.periodTimer.stop();
            root.windowTimer.stop();
            root.on = false;
        }
    }

    Component.onCompleted: if (root.active) {
        root.begin();
        root.periodTimer.restart();
    }
}
