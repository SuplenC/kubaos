import QtQuick
import qs.theme
import qs.components

// A key as the menu teaches it, such as `↵`, `Tab` or `⌃K`.
Rectangle {
    id: cap

    property MenuLook look
    property string key
    property color tint: Theme.m3onSurfaceVariant

    implicitHeight: look.keyHeight
    implicitWidth: look.snap(Math.max(implicitHeight, label.implicitWidth + 2 * look.tight))
    radius: look.keyRadius
    color: Theme.alpha(tint, 0.08)
    border.width: 1
    border.color: Theme.alpha(tint, 0.22)

    StyledText {
        id: label

        anchors.centerIn: parent
        text: cap.key
        color: cap.tint
        font.pixelSize: cap.look.font.small
        font.weight: Font.Medium
    }
}
