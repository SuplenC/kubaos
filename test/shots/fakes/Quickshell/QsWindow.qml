import QtQuick

// Stand-in for QsWindow: a plain Qt window. `implicitWidth` and `implicitHeight` size it, as they
// size a layer surface.
Window {
    property var screen: null
    property var mask: null
    property real implicitWidth: 0
    property real implicitHeight: 0

    width: implicitWidth > 0 ? implicitWidth : (screen?.width ?? 0)
    height: implicitHeight > 0 ? implicitHeight : (screen?.height ?? 0)
    visible: true

    function itemPosition(item: Item): point {
        return item.mapToItem(contentItem, 0, 0);
    }

    function mapFromItem(item: Item, x: real, y: real): point {
        return item.mapToItem(contentItem, x, y);
    }
}
