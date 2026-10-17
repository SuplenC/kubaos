import QtQuick
import qs.theme

// Horizontal slider for values in 0..1, a filled pill with the icon inside.
// It only reports user input through moved(value), so `value` can stay bound
// to a live service property. Keys: Left/Right or h/l step 5%.
Item {
    id: root

    property real value: 0
    property string icon

    signal moved(real value)

    implicitHeight: 36
    activeFocusOnTab: true

    function request(v: real): void {
        moved(Math.max(0, Math.min(1, v)));
    }

    Keys.onPressed: event => {
        if (event.key === Qt.Key_Left || event.key === Qt.Key_H)
            request(value - 0.05);
        else if (event.key === Qt.Key_Right || event.key === Qt.Key_L)
            request(value + 0.05);
        else
            return;
        event.accepted = true;
    }

    Rectangle {
        id: track

        anchors.fill: parent
        radius: height / 2
        color: Theme.m3surfaceContainerHighest

        Rectangle {
            id: fill

            height: parent.height
            width: Math.max(height, parent.width * Math.min(1, root.value))
            radius: height / 2
            color: Theme.m3primary

            Behavior on width {
                enabled: !drag.pressed

                Anim {
                    duration: Theme.anim.small
                    easing.bezierCurve: Theme.anim.emphasizedDecel
                }
            }

            // Handle: a short bar near the end of the fill.
            Rectangle {
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacing.sm
                anchors.verticalCenter: parent.verticalCenter
                width: 3
                height: parent.height * (drag.pressed ? 0.6 : 0.4)
                radius: 2
                color: Theme.m3onPrimary
                opacity: fill.width > fill.height * 1.6 ? 0.8 : 0

                Behavior on height {
                    Anim {
                        duration: Theme.anim.fast
                    }
                }
            }
        }

        Icon {
            anchors.left: parent.left
            anchors.leftMargin: (parent.height - width) / 2
            anchors.verticalCenter: parent.verticalCenter
            name: root.icon
            size: 20
            color: Theme.m3onPrimary
            filled: true
        }

        FocusRing {
            shown: root.activeFocus
        }
    }

    MouseArea {
        id: drag

        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onPressed: mouse => root.request(mouse.x / width)
        onPositionChanged: mouse => root.request(mouse.x / width)
        onWheel: wheel => root.request(root.value + (wheel.angleDelta.y > 0 ? 0.05 : -0.05))
    }
}
