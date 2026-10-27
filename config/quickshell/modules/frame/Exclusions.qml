import QtQuick
import Quickshell
import Quickshell.Wayland
import KubaOS.Components as K

// Exclusive zones for the frame: one empty, click-through window per edge,
// so tiled windows sit inside the frame. Kept apart from Frame.qml so that
// window never has to be re-placed by the compositor.
Scope {
    id: root

    required property ShellScreen screen

    component Edge: PanelWindow {
        screen: root.screen
        color: "transparent"
        // design-allow(size): a layer surface can't be empty; this one only reserves its zone.
        implicitWidth: 1
        // design-allow(size): a layer surface can't be empty; this one only reserves its zone.
        implicitHeight: 1
        mask: Region {}
        WlrLayershell.namespace: "qs-exclusion"
        WlrLayershell.layer: WlrLayer.Bottom
    }

    Edge {
        anchors.top: true
        anchors.left: true
        anchors.right: true
        exclusiveZone: K.Tokens.size.bar
    }

    Edge {
        anchors.bottom: true
        anchors.left: true
        anchors.right: true
        exclusiveZone: K.Tokens.space.s8
    }

    Edge {
        anchors.left: true
        anchors.top: true
        anchors.bottom: true
        exclusiveZone: K.Tokens.space.s8
    }

    Edge {
        anchors.right: true
        anchors.top: true
        anchors.bottom: true
        exclusiveZone: K.Tokens.space.s8
    }
}
