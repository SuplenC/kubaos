import QtQuick
import QtQuick.Layouts
import qs.theme

// Row of pill tabs with an indicator that slides to the selected one.
// The owner keeps the state: bind currentIndex, handle picked().
// Focusable: h/l or Left/Right pick the neighbouring tab.
//
// With `icons` (Material Symbols names, one per label) a tab shows its icon,
// the selected one its label beside it, and a tooltip names each.
//
// Its implicit width is every tab with full padding. Given more, the tabs
// share the extra; given less, they give up padding first and then elide
// their labels, so no tab or indicator ever reaches past the bar's edges.
Item {
    id: root

    property list<string> labels
    property list<string> icons
    property int currentIndex: 0
    readonly property int iconSize: 18

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

    RowLayout {
        id: row

        anchors.fill: parent
        spacing: Theme.spacing.xxs

        Repeater {
            id: repeater

            model: root.labels

            delegate: Item {
                id: tab

                required property string modelData
                required property int index
                readonly property bool selected: root.currentIndex === index
                readonly property string icon: root.icons[index] ?? ""
                readonly property bool iconShown: icon !== ""
                readonly property bool labelShown: !iconShown || selected
                readonly property real iconWidth: iconShown ? root.iconSize : 0
                readonly property real gap: iconShown && labelShown ? Theme.spacing.xs : 0
                readonly property color ink: selected ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
                readonly property real radius: Theme.rounding.full

                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredWidth: iconWidth + gap + (labelShown ? label.implicitWidth : 0) + Theme.spacing.lg * 2
                Layout.minimumWidth: Math.max(iconWidth, Theme.spacing.sm) + Theme.spacing.sm * 2

                Row {
                    anchors.centerIn: parent
                    spacing: tab.gap

                    Icon {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: tab.iconShown
                        name: tab.icon
                        size: root.iconSize
                        filled: tab.selected
                        color: tab.ink

                        Behavior on color {
                            CAnim {}
                        }
                    }

                    StyledText {
                        id: label

                        anchors.verticalCenter: parent.verticalCenter
                        visible: tab.labelShown
                        width: Math.max(0, Math.min(implicitWidth, tab.width - Theme.spacing.sm * 2 - tab.iconWidth - tab.gap))
                        horizontalAlignment: Text.AlignHCenter
                        text: tab.modelData
                        color: tab.ink
                        font.weight: Font.Medium

                        Behavior on color {
                            CAnim {}
                        }
                    }
                }

                StateLayer {
                    id: pointer

                    onClicked: root.picked(tab.index)
                }

                Tooltip {
                    text: tab.modelData
                    shown: tab.iconShown && pointer.containsMouse && !pointer.pressed
                }
            }
        }
    }
}
