import QtQuick

// Stand-in for IconImage: a square image drawn at its size in device pixels, as the real one is.
Image {
    property real implicitSize: 0
    readonly property real actualSize: implicitSize

    width: implicitSize
    height: implicitSize
    readonly property real ratio: Window.window?.devicePixelRatio ?? Screen.devicePixelRatio

    sourceSize.width: Math.round(implicitSize * ratio)
    sourceSize.height: Math.round(implicitSize * ratio)
    fillMode: Image.PreserveAspectFit
    smooth: true
    mipmap: true
}
