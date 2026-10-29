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
//
// Sheets in the same place are twins, as the system monitor, the notifications panel and the
// control center are. One asked for while a twin is open takes the surface over where it is, and
// only the content changes: the twin's slides away from this one's button and fades over `micro`,
// and this one's slides in from its button's side over `short`, starting half a beat later and
// fading in once the twin's is gone, so the two are never drawn over each other. With reduce
// motion the content swaps at once.
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
    // The sheets that open in this one's place, and where this one's button is along the bar
    // against theirs: a twin with a lower `side` has its button further left.
    property var twins: []
    property int side: 0
    // The twin this one is taking the surface over from, and which way this one's content moves
    // in: 1 from the right, -1 from the left.
    property K.Sheet from: null
    readonly property int toward: root.from ? Math.sign(root.side - root.from.side) : 0
    // Handing the surface over to its twin: the content is on its way out, and the twin draws the
    // surface.
    property bool leaving: false
    // How far the content is moved sideways, and how much of it shows, while it is swapped.
    property real shift: 0
    property real presence: 1
    readonly property int travel: Tokens.space.s24
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
    readonly property string path: active && !leaving ? K.SheetPath.build(frame, edge, u0, u1, reveal, Tokens.radius.surface, Tokens.radius.surface, flushStart, flushEnd) : ""
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

    // The twin on screen to take the surface over from: the one drawing it, else one on its way
    // out with it, or null.
    function holder(): K.Sheet {
        return root.twins.find(twin => twin.active && !twin.leaving) ?? root.twins.find(twin => twin.active) ?? null;
    }

    // Takes the surface over from `twin`, which is open or on its way in or out: the surface
    // carries on from where the twin's was, and the twin's content leaves as this one's arrives.
    // A third still leaving from the switch before goes at once, so only two ever swap.
    function takeOver(twin: K.Sheet): void {
        // Asked for again before the twin's content has begun to show: this one's content turns
        // back from where it is instead of leaving the screen empty for a beat.
        const turningBack = root.leaving && twin.presence === 0;
        swap.stop();
        back.stop();
        for (const other of root.twins) {
            if (other !== twin && other.leaving)
                other.settle();
        }
        root.from = twin;
        root.leaving = false;
        root.progress = twin.progress;
        twin.yieldTo(root);
        if (K.Motion.reduced) {
            twin.settle();
            root.shift = 0;
            root.presence = 1;
            return;
        }
        if (turningBack) {
            twin.settle();
            back.start();
            return;
        }
        root.shift = root.toward * root.travel;
        root.presence = 0;
        swap.start();
    }

    // Hands the surface over to `to`, which runs the swap: this sheet stays as open as `to` is
    // while its content leaves.
    function yieldTo(to: Item): void {
        swap.stop();
        back.stop();
        motion.stop();
        root.leaving = true;
        root.progress = Qt.binding(() => to.progress);
    }

    // Ends a hand-over to the twin: the content is gone, and so is this sheet.
    function settle(): void {
        swap.stop();
        back.stop();
        root.leaving = false;
        root.progress = 0;
        root.shift = 0;
        root.presence = 1;
    }

    onShownChanged: {
        // A twin taking over has already set this one leaving; it goes when its content has.
        if (!shown && leaving)
            return;
        const twin = shown ? holder() : null;
        if (twin)
            takeOver(twin);
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

    // The content swap, run by the sheet taking over: the twin's content leaves away from this
    // one's button, then this one's comes in from it.
    //
    //   0       micro/2 micro                   micro/2 + short
    //   |---------------|                       the twin's content moves out and fades
    //           |-------------------------------| this one's moves in
    //                   |-----------------------| and fades in
    ParallelAnimation {
        id: swap

        onFinished: root.from.settle()

        SequentialAnimation {
            ParallelAnimation {
                NumberAnimation {
                    target: root.from
                    property: "shift"
                    to: -root.toward * root.travel
                    duration: Tokens.duration.micro
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: Tokens.easing.emphasized_accel
                }

                NumberAnimation {
                    target: root.from
                    property: "presence"
                    to: 0
                    duration: Tokens.duration.micro
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: Tokens.easing.standard_accel
                }
            }

            NumberAnimation {
                target: root
                property: "presence"
                to: 1
                duration: Tokens.duration.short - Tokens.duration.micro / 2
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }

        SequentialAnimation {
            PauseAnimation {
                duration: Tokens.duration.micro / 2
            }

            NumberAnimation {
                target: root
                property: "shift"
                to: 0
                duration: Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.emphasized_decel
            }
        }
    }

    // This sheet's content returning to rest from part-way out, when it is asked for again before
    // its twin's has begun to show.
    ParallelAnimation {
        id: back

        NumberAnimation {
            target: root
            property: "shift"
            to: 0
            duration: Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.emphasized_decel
        }

        NumberAnimation {
            target: root
            property: "presence"
            to: 1
            duration: Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.standard
        }
    }

    // Swallow clicks on the sheet so they don't reach the click-away catcher. Content on its way
    // out to the twin takes no clicks, so none lands on what is leaving over what arrives.
    MouseArea {
        anchors.fill: parent
        enabled: !root.leaving
    }

    FocusScope {
        id: body

        focus: true
        x: (root.edge === "left" ? root.reveal - root.depth : 0) + root.shift
        y: root.edge === "top" ? root.reveal - root.depth : 0
        width: root.horizontal ? root.u1 - root.u0 : root.depth
        height: root.horizontal ? root.depth : root.u1 - root.u0
        opacity: root.contentOpacity * root.presence
        enabled: !root.leaving
    }
}
