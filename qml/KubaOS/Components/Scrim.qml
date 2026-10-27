import QtQuick
import KubaOS.Components as K

// Everything behind a dialog, dimmed, with the dialog centred over it on whole device pixels:
// `kubaos-ui`'s `dialog::scrim`. Fills all the room it is given.
Rectangle {
    id: root

    default property alias dialog: middle.data
    property alias room: middle.room

    readonly property bool fillWidth: true

    function relayout(): void {
        middle.relayout();
    }

    color: K.Look.alpha(Tokens.color.scrim, Tokens.opacity.scrim)

    // Takes every click, hover and scroll over the dimmed screen, so none reaches what is behind.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        hoverEnabled: true
        onWheel: wheel => wheel.accepted = true
    }

    K.Stack {
        id: middle

        probe: null
        centred: true
        width: root.width
        y: K.Look.snap((root.height - middle.implicitHeight) / 2, middle.dpr)
    }
}
