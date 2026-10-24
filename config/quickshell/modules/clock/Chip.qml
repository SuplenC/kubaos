import QtQuick
import qs.theme
import qs.components

// A small rounded choice: a timer preset, a country, a weekday. `selected`
// fills it with the secondary container, as the tab indicator is.
Rectangle {
    id: root

    property string label
    property bool selected: false
    property string tooltip

    signal clicked

    implicitWidth: labelText.implicitWidth + Theme.spacing.md * 2
    implicitHeight: 28
    radius: Theme.rounding.full
    color: selected ? Theme.m3secondaryContainer : Theme.m3surfaceContainerHigh

    Behavior on color {
        CAnim {}
    }

    StyledText {
        id: labelText

        anchors.centerIn: parent
        text: root.label
        font.pixelSize: Theme.font.sm
        font.weight: root.selected ? Font.DemiBold : Font.Medium
        color: root.selected ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
    }

    StateLayer {
        id: hover

        color: root.selected ? Theme.m3onSecondaryContainer : Theme.m3onSurface
        onClicked: root.clicked()
    }

    Tooltip {
        text: root.tooltip
        shown: root.tooltip !== "" && hover.containsMouse
    }
}
