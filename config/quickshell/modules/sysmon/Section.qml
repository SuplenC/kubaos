import QtQuick
import qs.theme
import qs.components

// One part of the system monitor's popup: an icon and a title, the headline
// reading on the right, and what it holds below.
Column {
    id: root

    property string icon
    property string title
    property string value
    property color valueColor: Theme.m3onSurface
    default property alias body: content.data

    spacing: Theme.spacing.sm

    Item {
        width: parent.width
        height: Math.max(heading.implicitHeight, reading.implicitHeight)

        Row {
            id: heading

            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.spacing.sm

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: root.icon
                size: Theme.font.xl
                color: Theme.m3primary
            }

            StyledText {
                anchors.verticalCenter: parent.verticalCenter
                text: root.title
                font.weight: Font.DemiBold
            }
        }

        StyledText {
            id: reading

            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, parent.width - heading.width - Theme.spacing.md)
            horizontalAlignment: Text.AlignRight
            text: root.value
            color: root.valueColor
            font.pixelSize: Theme.font.lg
            font.weight: Font.DemiBold
            font.features: Theme.font.tabular
        }
    }

    Column {
        id: content

        width: parent.width
        spacing: Theme.spacing.xs
    }
}
