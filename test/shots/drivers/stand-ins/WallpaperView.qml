import QtQuick

// Stand-in for the shell's WallpaperView: the still, or a video's poster, never playing.
Item {
    id: root

    property string path
    property bool playing: false
    property size sourceSize
    property int fillMode: Image.PreserveAspectCrop
    readonly property bool video: false
    readonly property bool ready: still.status === Image.Ready
    readonly property int status: still.status
    readonly property bool moving: false

    Image {
        id: still

        anchors.fill: parent
        source: root.path ? `file://${root.path}` : ""
        fillMode: root.fillMode
        sourceSize: root.sourceSize
        asynchronous: false
    }
}
