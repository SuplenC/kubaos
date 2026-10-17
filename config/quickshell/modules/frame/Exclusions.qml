import QtQuick
import Quickshell
import Quickshell.Wayland
import qs.theme

// Exclusive zones for the frame: one empty, click-through window per edge,
// so tiled windows sit inside the frame. Kept apart from Frame.qml so that
// window never has to be re-placed by the compositor.
Scope {
    id: root

    required property ShellScreen screen

    component Edge: PanelWindow {
        screen: root.screen
        color: "transparent"
        implicitWidth: 1
        implicitHeight: 1
        mask: Region {}
        WlrLayershell.namespace: "qs-exclusion"
        WlrLayershell.layer: WlrLayer.Bottom
    }

    Edge {
        anchors.top: true
        anchors.left: true
        anchors.right: true
        exclusiveZone: Theme.bar.height
    }

    Edge {
        anchors.bottom: true
        anchors.left: true
        anchors.right: true
        exclusiveZone: Theme.frame.thickness
    }

    Edge {
        anchors.left: true
        anchors.top: true
        anchors.bottom: true
        exclusiveZone: Theme.frame.thickness
    }

    Edge {
        anchors.right: true
        anchors.top: true
        anchors.bottom: true
        exclusiveZone: Theme.frame.thickness
    }
}
