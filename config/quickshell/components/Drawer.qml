import QtQuick
import qs.theme
import "drawerpath.js" as DrawerPath

// A panel that grows out of an inner edge of the screen frame (see
// modules/frame/Frame.qml). The frame draws its background from `path`, in the
// same layer as the frame itself, so panel and frame read as one surface. This
// item only places and clips the content.
//
// Opening animates `progress` 0 -> 1 on the spatial curve (a slight
// overshoot); closing goes back on emphasized-accelerate, quicker. A panel
// may take other curves and durations (the clock popup glides, as the
// Command menu does). The body
// grows to progress * depth and the content rides on its far edge, so it
// slides out from under the frame instead of fading in place.
FocusScope {
    id: root

    required property QtObject frame            // { l, t, r, b, screen } from Frame.qml
    property string edge: "top"                  // top | bottom | left | right
    property real align: 0.5                     // position along the edge, 0..1 (centered)
    // Where the body starts along the edge, in place of `align`; NaN leaves it to `align`.
    property real start: NaN
    property bool flushStart: false              // body touches the perpendicular frame edge at its start
    property bool flushEnd: false                // ... at its end
    property bool shown: false
    property bool takesFocus: true               // grab keyboard focus when shown
    property real contentWidth: 0                // set from the content's implicit size
    property real contentHeight: 0
    property bool animateSize: true               // smooth content size changes while open
    property int enterDuration: Theme.anim.large
    property int exitDuration: Theme.anim.normal
    property list<real> enterCurve: Theme.anim.spatial
    property list<real> exitCurve: Theme.anim.emphasizedAccel
    default property alias data: content.data
    // Tooltips of the buttons inside open within this panel's edges.
    readonly property bool keepsTooltips: true

    readonly property bool horizontal: edge === "top" || edge === "bottom"
    readonly property real span: horizontal ? contentWidth : contentHeight   // along the edge
    readonly property real depth: horizontal ? contentHeight : contentWidth  // away from the edge
    readonly property real edgeStart: horizontal ? frame.l : frame.t
    readonly property real edgeEnd: horizontal ? frame.r : frame.b
    readonly property real u0: {
        if (!isNaN(start))
            return start;
        if (flushStart)
            return edgeStart;
        if (flushEnd)
            return edgeEnd - span;
        const min = edgeStart + Theme.frame.flare;
        const max = edgeEnd - span - Theme.frame.flare;
        return Math.round(Math.max(min, Math.min(max, edgeStart + (edgeEnd - edgeStart - span) * align)));
    }
    readonly property real u1: flushStart && flushEnd ? edgeEnd : u0 + span
    property real progress: 0
    readonly property real reveal: Math.max(0, depth * progress)
    readonly property bool active: progress > 0.001
    // The rectangle the body covers once fully open, in window coordinates.
    readonly property rect area: {
        if (edge === "top")
            return Qt.rect(u0, frame.t, u1 - u0, depth);
        if (edge === "bottom")
            return Qt.rect(u0, frame.b - depth, u1 - u0, depth);
        if (edge === "left")
            return Qt.rect(frame.l, u0, depth, u1 - u0);
        return Qt.rect(frame.r - depth, u0, depth, u1 - u0);
    }
    // Content fades in a beat after the surface starts moving and out
    // before it has fully retracted.
    readonly property real contentOpacity: Math.max(0, Math.min(1, (progress - 0.2) / 0.55))
    readonly property string path: active ? DrawerPath.build(frame, edge, u0, u1, reveal, Theme.frame.flare, Theme.frame.panelRounding, flushStart, flushEnd) : ""

    // Emitted once the panel starts opening; give keyboard focus here.
    signal opened

    onShownChanged: {
        motion.stop();
        motion.to = shown ? 1 : 0;
        motion.duration = shown ? enterDuration : exitDuration;
        motion.easing.bezierCurve = shown ? enterCurve : exitCurve;
        motion.start();
        if (shown) {
            if (takesFocus)
                content.forceActiveFocus();
            opened();
        }
    }

    x: edge === "right" ? frame.r - reveal : horizontal ? u0 : frame.l
    y: edge === "bottom" ? frame.b - reveal : horizontal ? frame.t : u0
    width: horizontal ? u1 - u0 : reveal
    height: horizontal ? reveal : u1 - u0
    visible: active
    clip: true

    Behavior on contentWidth {
        enabled: root.animateSize && root.active

        Anim {
            duration: Theme.anim.large
            easing.bezierCurve: Theme.anim.spatial
        }
    }

    Behavior on contentHeight {
        enabled: root.animateSize && root.active

        Anim {
            duration: Theme.anim.large
            easing.bezierCurve: Theme.anim.spatial
        }
    }

    NumberAnimation {
        id: motion

        target: root
        property: "progress"
        easing.type: Easing.BezierSpline
    }

    // Swallow clicks on the panel so they don't reach the click-away catcher.
    MouseArea {
        anchors.fill: parent
    }

    FocusScope {
        id: content

        focus: true
        x: root.edge === "left" ? root.reveal - root.depth : 0
        y: root.edge === "top" ? root.reveal - root.depth : 0
        width: root.horizontal ? root.u1 - root.u0 : root.depth
        height: root.horizontal ? root.depth : root.u1 - root.u0
        opacity: root.contentOpacity
    }
}
