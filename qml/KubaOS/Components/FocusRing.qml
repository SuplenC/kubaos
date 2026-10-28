import QtQuick
import KubaOS.Components as K

// The neon focus on its parent, a control whose corners have radius `controlRadius`: the control's
// own edge lit as a tube `Tokens.focus.tube` wide, inside its box and over its outline, so there is
// only ever one line and nothing dark between the tube and its glow. The glow is outside the
// control, bands from its edge out fading from the tube (`kubaos-ui`'s `focus::rings`), and
// reaches only as far as the room the control has before the nearest ancestor that clips, so it is
// never cut. Nothing takes layout space, so focus moving never moves anything.
//
// Focus arriving strikes the tube on through `Tokens.neon.strike` over `medium`, as the neon sign
// does, unless `Motion.strikes` is off; then it fades in over `short`. Focus leaving fades it out
// over `short`.
Item {
    id: root

    // Whether the keyboard is on the control.
    property bool shown: false
    // The control's corner radius.
    property int controlRadius: Tokens.radius.control
    // The cold accent, on cards where the accent already means picked.
    property bool cold: false
    // The error colour, on a control whose value is wrong, so focus never hides that.
    property bool error: false

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property color tint: root.error ? Tokens.color.error : root.cold ? Tokens.color.cold : Tokens.color.primary
    // How lit the tube is while it isn't striking, 0 to 1.
    property real level: 0
    // How far through the strike, 0 to its steps.
    property real step: 0
    // Lit, lighting or fading: the bands and the watch on the room stay through the strike's dark
    // steps, so they aren't built again on each.
    readonly property bool lit: root.shown || root.level > 0 || strike.running || fade.running
    // The room the control has before what clips it, in logical pixels.
    property real room: Tokens.focus.far
    // What clips the control: its nearest ancestor that clips, or the window.
    property Item area: null
    // How many device pixels out the glow reaches.
    readonly property int bands: Math.max(0, Math.ceil(Math.min(Tokens.focus.far, root.room) * root.dpr - 1e-3))

    // The glow's opacity `d` logical pixels outside the tube: `kubaos-ui`'s `focus::glow`.
    function glow(d: real): real {
        const room = Math.max(0, root.room);
        const fade = (reach, opacity) => {
            const r = Math.min(reach, room);
            return d >= r ? 0 : opacity * Math.pow(1 - d / r, 2);
        };
        const near = fade(Tokens.focus.near, Tokens.focus.near_opacity);
        const far = fade(Tokens.focus.far, Tokens.focus.far_opacity);
        return 1 - (1 - near) * (1 - far);
    }

    // Finds what clips the control and how far inside it the control stands at its nearest side.
    function measure(): void {
        const control = root.parent;
        let clipper = control;
        while (clipper && !clipper.clip)
            clipper = clipper.parent;
        root.area = clipper ?? (Window.window ? Window.window.contentItem : null);
        if (!control || !root.area) {
            root.room = Tokens.focus.far;
            return;
        }
        if (root.area === control) {
            root.room = 0;
            return;
        }
        const at = control.mapToItem(root.area, 0, 0);
        root.room = Math.min(at.x, at.y, root.area.width - at.x - control.width, root.area.height - at.y - control.height);
    }

    function light(): void {
        const now = root.opacity;
        strike.stop();
        fade.stop();
        if (root.shown)
            root.measure();
        if (root.shown && K.Motion.strikes) {
            root.level = 1;
            strike.start();
            return;
        }
        root.level = now;
        fade.to = root.shown ? 1 : 0;
        fade.start();
    }

    x: 0
    y: 0
    width: root.parent ? root.parent.width : 0
    height: root.parent ? root.parent.height : 0
    opacity: strike.running ? Tokens.neon.strike[Math.min(Tokens.neon.strike.length - 1, Math.floor(root.step))] : root.level
    visible: root.opacity > 0

    onShownChanged: root.light()
    Component.onCompleted: {
        root.level = root.shown ? 1 : 0;
        if (root.shown)
            root.measure();
    }

    NumberAnimation {
        id: strike

        target: root
        property: "step"
        from: 0
        to: Tokens.neon.strike.length
        duration: Tokens.duration.medium
    }

    NumberAnimation {
        id: fade

        target: root
        property: "level"
        duration: Tokens.duration.short
        easing.type: Easing.BezierSpline
        easing.bezierCurve: Tokens.easing.standard
    }

    // Anything that moves or sizes the control while it is lit (a panel sliding in, a dialog
    // laying out after focus moved into it) moves its room: measured again after each frame that
    // animated, and after the scene settles.
    Connections {
        target: root.lit ? root.Window.window : null

        function onAfterAnimating(): void {
            root.measure();
        }

        function onFrameSwapped(): void {
            if (!settled.running)
                settled.restart();
        }
    }

    // The frame after the last one that drew: whatever laid out late has its place by then.
    Timer {
        id: settled

        interval: 0
        onTriggered: root.measure()
    }

    // A scroll moving the control moves its room.
    Connections {
        target: root.lit ? root.area : null
        ignoreUnknownSignals: true

        function onContentXChanged(): void {
            root.measure();
        }

        function onContentYChanged(): void {
            root.measure();
        }

        function onWidthChanged(): void {
            root.measure();
        }

        function onHeightChanged(): void {
            root.measure();
        }
    }

    // The glow as bands that all start at the control's edge, each a device pixel wider:
    // `kubaos-ui`'s `focus::rings`. Laid over one another they make `glow` at every device pixel
    // out, and no two edges meet along a curve, so the corners show no seams.
    Repeater {
        model: root.lit ? root.bands : 0

        Rectangle {
            required property int index
            // The band's reach outside the control.
            readonly property real out: (index + 1) / root.dpr
            // The glow at this device pixel and at the next one out, which the wider bands give.
            readonly property real here: root.glow((index + 0.5) / root.dpr)
            readonly property real next: index + 1 < root.bands ? root.glow((index + 1.5) / root.dpr) : 0

            x: -out
            y: -out
            width: root.width + out * 2
            height: root.height + out * 2
            radius: K.Look.outset(root.controlRadius, out)
            color: "transparent"
            border.width: out
            border.color: K.Look.alpha(root.tint, 1 - (1 - here) / (1 - next))
        }
    }

    Rectangle {
        anchors.fill: parent
        radius: root.controlRadius
        color: "transparent"
        border.width: K.Look.stroke(Tokens.focus.tube, root.dpr)
        border.color: root.tint
    }
}
