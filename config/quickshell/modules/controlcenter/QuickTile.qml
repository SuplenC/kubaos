import QtQuick
import QtQuick.Layouts
import qs.theme
import qs.components

// Toggle tile: icon, title and a status line. Enter or Space activates.
// On, it fills with primary and its corners round further (M3 shape morph).
Rectangle {
    id: root

    property string icon
    property string title
    property string subtitle
    property bool checked: false

    signal clicked

    implicitHeight: 58
    radius: stateLayer.pressed ? Theme.rounding.md : checked ? Theme.rounding.xxl : Theme.rounding.lg
    color: checked ? Theme.m3primary : Theme.m3surfaceContainerHigh
    activeFocusOnTab: true

    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()
    Keys.onSpacePressed: clicked()

    Behavior on color {
        CAnim {}
    }

    Behavior on radius {
        Anim {
            duration: Theme.anim.large
            easing.bezierCurve: Theme.anim.spatial
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.spacing.lg
        anchors.rightMargin: Theme.spacing.md
        spacing: Theme.spacing.md

        Icon {
            name: root.icon
            size: 22
            filled: root.checked
            color: root.checked ? Theme.m3onPrimary : Theme.m3onSurfaceVariant
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 1

            StyledText {
                Layout.fillWidth: true
                text: root.title
                font.weight: Font.DemiBold
                color: root.checked ? Theme.m3onPrimary : Theme.m3onSurface
            }

            StyledText {
                Layout.fillWidth: true
                text: root.subtitle
                font.pixelSize: Theme.font.sm
                color: root.checked ? Theme.alpha(Theme.m3onPrimary, 0.8) : Theme.m3onSurfaceVariant
            }
        }
    }

    StateLayer {
        id: stateLayer

        color: root.checked ? Theme.m3onPrimary : Theme.m3onSurface
        onClicked: root.clicked()
    }

    FocusRing {}
}
