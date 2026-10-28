import QtQuick

// Stand-in for PopupWindow: never shown in a shot.
Item {
    property PopupAnchor anchor: PopupAnchor {}
    property color color
    property var mask
    property real implicitWidth: 0
    property real implicitHeight: 0

    visible: false
}
