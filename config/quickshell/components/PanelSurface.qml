import QtQuick
import qs.theme

// Floating card for dialogs that don't belong to the frame (polkit).
// Swallows clicks so they don't reach the backdrop.
Rectangle {
    color: Theme.m3surfaceContainer
    radius: Theme.rounding.xxl
    border.width: 1
    border.color: Theme.alpha(Theme.m3outlineVariant, 0.6)

    MouseArea {
        anchors.fill: parent
    }
}
