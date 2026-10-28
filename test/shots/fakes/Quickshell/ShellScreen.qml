import QtQuick

QtObject {
    property string name: "HEADLESS-1"
    property string model: "Headless"
    property int x: 0
    property int y: 0
    // design-allow(size): the shots' screen in logical pixels, room for every panel.
    property int width: 1600
    // design-allow(size): the shots' screen in logical pixels, room for every panel.
    property int height: 1000
    property real devicePixelRatio: 1
}
