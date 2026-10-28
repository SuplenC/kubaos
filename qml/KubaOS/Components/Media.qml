import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// A picture that never moves what is around it: `kubaos-ui`'s `media::media`. As wide as it is
// given and as tall as `aspect` makes it, down to the 4 px grid, whether or not the picture has
// loaded; until it has, the box holds the placeholder, a block skeleton of the high container.
//
// The picture is `source`, cut to cover the box at `radius`; or the content put in it, laid over
// the placeholder to fill it (a video, a picture that clips itself, fading in over it).
Item {
    id: root

    // Width to height: one of `Tokens.aspect`.
    property list<int> aspect: Tokens.aspect.wallpaper
    property url source: ""
    property int radius: Tokens.radius.control
    default property alias content: slot.data

    // Whether `source` has loaded, and covers the placeholder.
    readonly property bool ready: picture.status === Image.Ready

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Media",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    implicitHeight: root.heightAt(root.width)

    // The box's height at `width`, down to the 4 px grid: whole device pixels at every scale, and
    // one height for boxes side by side whose widths differ by a device pixel.
    function heightAt(width: real): real {
        const [across, down] = root.aspect;
        return Math.floor(width * down / across / Tokens.rules.grid) * Tokens.rules.grid;
    }

    // The placeholder: a block skeleton, the box's own, so it breathes until the picture covers
    // it, then stops.
    K.Skeleton {
        anchors.fill: parent
        bone: "block"
        radius: root.radius
        reported: false
        visible: !root.ready
    }

    Image {
        id: picture

        anchors.fill: parent
        visible: false
        source: root.source
        asynchronous: true
        fillMode: Image.PreserveAspectCrop
        // Decoded at the size it is shown, in device pixels: crisp, and no larger than it must be.
        sourceSize: Qt.size(Math.ceil(root.width * root.dpr), Math.ceil(root.height * root.dpr))
    }

    Rectangle {
        id: corners

        anchors.fill: parent
        radius: root.radius
        visible: false
        layer.enabled: picture.status === Image.Ready
    }

    MultiEffect {
        anchors.fill: parent
        visible: picture.status === Image.Ready
        source: picture
        maskEnabled: true
        maskSource: corners
        maskThresholdMin: 0.5
        maskSpreadAtMin: 1
    }

    Item {
        id: slot

        anchors.fill: parent
    }
}
