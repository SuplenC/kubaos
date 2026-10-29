import QtQuick
import KubaOS.Components as K

// A control's name in a small bubble under it, while the pointer is over it or the keyboard is on
// it: `kubaos-ui`'s `hint::hint`. The inverse colours, `label` type, 8 by 4 inside, the small
// radius, centred under the control 8 below it, on whole device pixels.
//
// The bubble floats over the whole window, as iced's overlay does: it is drawn on the window's
// own content item, so neither a clipping panel nor what is laid out after the control covers it.
// It shows only while its control shows whole: a control a scroll has cut, or taken out of view,
// takes its bubble with it. It is no part of the page's layout; leave it room below the control,
// as the specimen's rows do.
Item {
    id: root

    property string text
    property bool shown: false
    // Whether the control shows whole where it is: one scrolled out of its view, even part way,
    // shows no bubble, which would float over whatever is there instead. Read as it is placed.
    property bool inView: true

    // What a geometry dump leaves out: the bubble floats, as iced's overlay does.
    readonly property bool overlay: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // Where the bubble floats: the window's content item, or the control when there is no window.
    readonly property Item host: Window.window ? Window.window.contentItem : null

    // Puts the floating box over the control, on whole device pixels. A place in the window isn't
    // a binding, so it is worked out after each frame drawn while the bubble shows: it follows a
    // panel sliding in, and costs nothing while nothing moves.
    function place(): void {
        if (!root.host || !root.shown)
            return;
        root.inView = K.Look.unclipped(root);
        const at = root.mapToItem(root.host, 0, 0);
        floating.x = K.Look.snap(at.x, root.dpr);
        floating.y = K.Look.snap(at.y, root.dpr);
        // Centred under the control, but held inside the window, so a control at a screen's edge
        // keeps its bubble whole.
        const left = floating.x + bubble.centred;
        const inside = Math.max(0, Math.min(left, root.host.width - bubble.width));
        floating.shift = K.Look.snap(inside - left, root.dpr);
    }

    anchors.fill: root.parent
    z: 1
    onShownChanged: root.place()
    Component.onCompleted: Qt.callLater(root.place)

    Connections {
        target: root.shown ? root.Window.window : null

        function onFrameSwapped() {
            root.place();
        }
    }

    Item {
        id: floating

        readonly property bool overlay: true
        // How far the bubble moves aside from centred to stay inside the window.
        property real shift: 0

        parent: root.host ?? root
        z: 1
        width: root.width
        height: root.height
        // Its control's visibility, which it no longer inherits on the window's content item. It
        // reads the bubble's opacity, not its `visible`: that is the bubble's own and this item's
        // together, so once hidden it would keep this item hidden for good.
        visible: bubble.opacity > 0 && root.visible

        Rectangle {
            id: bubble

            readonly property real centred: K.Look.snap((floating.width - width) / 2, root.dpr)

            x: bubble.centred + floating.shift
            y: floating.height + Tokens.space.s8
            width: K.Look.snapUp(label.implicitWidth + Tokens.space.s8 * 2, root.dpr)
            height: K.Look.snapUp(label.implicitHeight + Tokens.space.s4 * 2, root.dpr)
            radius: Tokens.radius.small
            color: Tokens.color.on_surface
            opacity: root.shown && root.inView ? 1 : 0
            visible: opacity > 0

            Behavior on opacity {
                NumberAnimation {
                    duration: Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: Tokens.easing.standard
                }
            }

            K.Text {
                id: label

                x: Tokens.space.s8
                y: Tokens.space.s4
                text: root.text
                style: Tokens.type.label
                color: Tokens.color.surface
            }
        }
    }
}
