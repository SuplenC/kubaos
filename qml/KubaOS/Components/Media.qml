import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// A picture that never moves what is around it: `kubaos-ui`'s `media::media`. As wide as it is
// given and as tall as `aspect` makes it, on a whole device pixel, whether or not the picture has
// loaded; until it has, the box holds the placeholder, the high container.
//
// The picture is `source`, cut to cover the box at `radius`; or the content put in it, laid over
// the placeholder to fill it (a video, a picture that clips itself), which says it has loaded
// through `ready`.
Item {
    id: root

    // Width to height: one of `Tokens.aspect`.
    property list<int> aspect: Tokens.aspect.wallpaper
    property url source: ""
    property int radius: Tokens.radius.control
    // Whether the picture is in: the source's, or the content's when it sets this.
    property bool ready: picture.status === Image.Ready
    default property alias content: slot.data

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

    // The box's height at `width`, on a whole device pixel.
    function heightAt(width: real): real {
        const [across, down] = root.aspect;
        return K.Look.snap(width * down / across, root.dpr);
    }

    Rectangle {
        anchors.fill: parent
        radius: root.radius
        color: Tokens.color.surface_container_high
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
