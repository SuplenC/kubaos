import QtQuick

// Stand-in for a layer-shell PanelWindow. Anchored to two opposite edges, it spans its screen
// between them.
QsWindow {
    id: panel

    property Edges anchors: Edges {}
    property Edges margins: Edges {}
    property int exclusionMode
    property int exclusiveZone
    property bool aboveWindows: true
    property bool focusable: false

    readonly property ShellScreen fallback: ShellScreen {}
    readonly property var shownOn: panel.screen ?? panel.fallback

    width: panel.anchors.left && panel.anchors.right || panel.implicitWidth <= 0 ? panel.shownOn.width : panel.implicitWidth
    height: panel.anchors.top && panel.anchors.bottom || panel.implicitHeight <= 0 ? panel.shownOn.height : panel.implicitHeight
}
