import QtQuick
import KubaOS.Components as K

// A panel that grows out of an inner edge of the screen frame: `kubaos-ui`'s `sheet::sheet`, and
// every shell panel. The owner draws the sheet's surface from `path` in the same layer as the
// frame, so sheet and frame read as one surface (the shell's `modules/frame/Frame.qml`); this item
// only places and clips the content.
//
// Opening animates `progress` 0 -> 1 over `long` on emphasized decelerate, as every panel
// arrives; closing goes back over `medium` on emphasized accelerate, as it leaves. The body grows
// to progress * depth and the content rides on its far edge, so it slides out from under the frame
// instead of fading in place. The sheet meets the frame through concave fillets of the surface
// radius, the frame's own inner corner.
FocusScope {
    id: root

    // The frame's inner edges in the owner's coordinates: `{ l, t, r, b }`.
    required property QtObject frame
    // Which edge it grows out of: `top`, `bottom`, `left` or `right`.
    property string edge: "top"
    // Where along the edge it sits, 0..1; 0.5 centres it.
    property real align: 0.5
    // Where the body starts along the edge, in place of `align`; NaN leaves it to `align`.
    property real start: NaN
    // The body touches the perpendicular frame edge at its start, or at its end.
    property bool flushStart: false
    property bool flushEnd: false
    property bool shown: false
    // Whether it takes the keyboard when shown.
    property bool takesFocus: true
    // The content's size, from its implicit size.
    property real contentWidth: 0
    property real contentHeight: 0
    // Whether content that grows or shrinks while it is open settles into its new size.
    property bool animateSize: true
    default property alias content: body.data

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property bool horizontal: edge === "top" || edge === "bottom"
    // Along the edge, and away from it.
    readonly property real span: horizontal ? contentWidth : contentHeight
    readonly property real depth: horizontal ? contentHeight : contentWidth
    readonly property real edgeStart: horizontal ? frame.l : frame.t
    readonly property real edgeEnd: horizontal ? frame.r : frame.b
    // Where the body starts along the edge, on a whole device pixel: centred as a stack centres
    // its children, an odd spare pixel after it, and kept a fillet's room from either end.
    readonly property real u0: {
        if (!isNaN(start))
            return start;
        if (flushStart)
            return edgeStart;
        if (flushEnd)
            return edgeEnd - span;
        const spare = edgeEnd - edgeStart - span;
        const offset = align === 0.5 ? K.Look.centre(spare, root.dpr) : K.Look.snap(spare * align, root.dpr);
        const min = edgeStart + Tokens.radius.surface;
        const max = edgeEnd - span - Tokens.radius.surface;
        return Math.max(min, Math.min(max, edgeStart + offset));
    }
    readonly property real u1: flushStart && flushEnd ? edgeEnd : u0 + span
    property real progress: 0
    readonly property real reveal: Math.max(0, depth * progress)
    readonly property bool active: progress > 0.001
    // The rectangle the body covers once fully open, in the owner's coordinates.
    readonly property rect area: {
        if (edge === "top")
            return Qt.rect(u0, frame.t, u1 - u0, depth);
        if (edge === "bottom")
            return Qt.rect(u0, frame.b - depth, u1 - u0, depth);
        if (edge === "left")
            return Qt.rect(frame.l, u0, depth, u1 - u0);
        return Qt.rect(frame.r - depth, u0, depth, u1 - u0);
    }
    // Content fades in a beat after the surface starts moving and out before it has fully
    // retracted.
    readonly property real contentOpacity: Math.max(0, Math.min(1, (progress - 0.2) / 0.55))
    readonly property string path: active ? K.SheetPath.build(frame, edge, u0, u1, reveal, Tokens.radius.surface, Tokens.radius.surface, flushStart, flushEnd) : ""
    readonly property var probe: ({
            role: "component",
            name: "Sheet",
            variant: "",
            fixed: {
                width: false,
                height: false,
                holds: true
            }
        })

    // Emitted once the sheet starts opening; give keyboard focus here.
    signal opened

    onShownChanged: {
        motion.stop();
        motion.to = shown ? 1 : 0;
        motion.duration = shown ? Tokens.duration.long : Tokens.duration.medium;
        motion.easing.bezierCurve = shown ? Tokens.easing.emphasized_decel : Tokens.easing.emphasized_accel;
        motion.start();
        if (shown) {
            if (takesFocus)
                body.forceActiveFocus();
            opened();
        }
    }

    x: edge === "right" ? frame.r - reveal : horizontal ? u0 : frame.l
    y: edge === "bottom" ? frame.b - reveal : horizontal ? frame.t : u0
    width: horizontal ? u1 - u0 : reveal
    height: horizontal ? reveal : u1 - u0
    visible: active
    clip: true

    // Content that grows or shrinks while the sheet is open settles into its new size, as a list
    // reordering does.
    Behavior on contentWidth {
        enabled: root.animateSize && root.active

        NumberAnimation {
            duration: Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.standard_decel
        }
    }

    Behavior on contentHeight {
        enabled: root.animateSize && root.active

        NumberAnimation {
            duration: Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.standard_decel
        }
    }

    NumberAnimation {
        id: motion

        target: root
        property: "progress"
        easing.type: Easing.BezierSpline
    }

    // Swallow clicks on the sheet so they don't reach the click-away catcher.
    MouseArea {
        anchors.fill: parent
    }

    FocusScope {
        id: body

        focus: true
        x: root.edge === "left" ? root.reveal - root.depth : 0
        y: root.edge === "top" ? root.reveal - root.depth : 0
        width: root.horizontal ? root.u1 - root.u0 : root.depth
        height: root.horizontal ? root.depth : root.u1 - root.u0
        opacity: root.contentOpacity
    }
}
