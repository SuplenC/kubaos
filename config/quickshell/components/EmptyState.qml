import QtQuick
import qs.theme

// Centered icon and message for an empty list.
Column {
    property string icon
    property string text

    spacing: Theme.spacing.sm

    Icon {
        anchors.horizontalCenter: parent.horizontalCenter
        name: parent.icon
        size: 36
        color: Theme.m3outline
    }

    StyledText {
        anchors.horizontalCenter: parent.horizontalCenter
        text: parent.text
        color: Theme.m3onSurfaceVariant
    }
}
