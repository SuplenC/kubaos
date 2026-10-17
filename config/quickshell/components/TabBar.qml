import QtQuick
import qs.theme

// Row of pill tabs with an indicator that slides to the selected one.
// The owner keeps the state: bind currentIndex, handle picked().
// Focusable: h/l or Left/Right pick the neighbouring tab.
Item {
    id: root

    property list<string> labels
    property int currentIndex: 0

    signal picked(int index)

    implicitWidth: row.implicitWidth
    implicitHeight: 32
    activeFocusOnTab: true

    function step(delta: int): void {
        picked((currentIndex + delta + labels.length) % labels.length);
    }

    Keys.onPressed: event => {
        if (event.key === Qt.Key_Left || event.key === Qt.Key_H)
            step(-1);
        else if (event.key === Qt.Key_Right || event.key === Qt.Key_L)
            step(1);
        else
            return;
        event.accepted = true;
    }

    Rectangle {
        id: indicator

        // itemAt() is not reactive; reading count re-evaluates it once the tabs exist.
        readonly property Item target: repeater.count > 0 ? repeater.itemAt(Math.min(root.currentIndex, repeater.count - 1)) : null

        x: target?.x ?? 0
        width: target?.width ?? 0
        height: root.height
        radius: Theme.rounding.full
        color: Theme.m3secondaryContainer

        Behavior on x {
            Anim {
                duration: Theme.anim.large
                easing.bezierCurve: Theme.anim.spatial
            }
        }

        Behavior on width {
            Anim {
                duration: Theme.anim.large
                easing.bezierCurve: Theme.anim.spatial
            }
        }

        FocusRing {
            shown: root.activeFocus
        }
    }

    Row {
        id: row

        spacing: Theme.spacing.xxs

        Repeater {
            id: repeater

            model: root.labels

            delegate: Item {
                id: tab

                required property string modelData
                required property int index
                readonly property bool selected: root.currentIndex === index
                readonly property real radius: Theme.rounding.full

                implicitWidth: label.implicitWidth + Theme.spacing.lg * 2
                implicitHeight: root.height

                StyledText {
                    id: label

                    anchors.centerIn: parent
                    text: tab.modelData
                    color: tab.selected ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
                    font.weight: Font.Medium

                    Behavior on color {
                        CAnim {}
                    }
                }

                StateLayer {
                    onClicked: root.picked(tab.index)
                }
            }
        }
    }
}
