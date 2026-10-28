import QtQuick
import KubaOS.Components as K

// The neon focus on its parent, a control whose corners have radius `controlRadius`: the control's
// own edge lit as a tube `Tokens.focus.tube` wide, inside its box and over its outline, so there is
// only ever one line and nothing dark between the tube and its glow. The glow is outside the
// control, bands from its edge out fading from the tube (`kubaos-ui`'s `focus::rings`). Nothing
// takes layout space, so focus moving never moves anything. Focus is a state of the control, drawn
// as part of it: the glow is always its full size, clipped by whatever clips the control and
// covered by whatever is drawn over it.
//
// On a control filled with the focus colour (`filled`: a `Filled` button, a tile that is on) the
// fill is the tube, and the tube's hot core (`primary_core`, `cold_core`) rides just outside it,
// the thin stroke wide, with the glow beyond: `kubaos-ui`'s `focus::focus_ring_on_fill`.
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
    // The control is filled with the focus colour: the core rides outside the fill.
    property bool filled: false

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property color tint: root.error ? Tokens.color.error : root.cold ? Tokens.color.cold : Tokens.color.primary
    readonly property color core: root.error ? Tokens.color.error : root.cold ? Tokens.color.cold_core : Tokens.color.primary_core
    // How far outside the control the glow starts: past the core on a fill.
    readonly property real inset: root.filled ? K.Look.thin(root.dpr) : 0
    // How lit the tube is while it isn't striking, 0 to 1.
    property real level: 0
    // How far through the strike, 0 to its steps.
    property real step: 0
    // Lit, lighting or fading: the bands stay through the strike's dark steps, so they aren't
    // built again on each.
    readonly property bool lit: root.shown || root.level > 0 || strike.running || fade.running
    // How many device pixels out the glow reaches.
    readonly property int bands: Math.ceil(Tokens.focus.far * root.dpr - 1e-3)

    // The glow's opacity `d` logical pixels outside the tube: `kubaos-ui`'s `focus::glow`.
    function glow(d: real): real {
        const fade = (reach, opacity) => d >= reach ? 0 : opacity * Math.pow(1 - d / reach, 2);
        const near = fade(Tokens.focus.near, Tokens.focus.near_opacity);
        const far = fade(Tokens.focus.far, Tokens.focus.far_opacity);
        return 1 - (1 - near) * (1 - far);
    }

    function light(): void {
        const now = root.opacity;
        strike.stop();
        fade.stop();
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
    Component.onCompleted: root.level = root.shown ? 1 : 0

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

    // The glow as bands that all start at the control's edge, each a device pixel wider:
    // `kubaos-ui`'s `focus::rings`. Laid over one another they make `glow` at every device pixel
    // out, and no two edges meet along a curve, so the corners show no seams.
    Repeater {
        model: root.lit ? root.bands : 0

        Rectangle {
            required property int index
            // The band's reach outside where the glow starts.
            readonly property real out: (index + 1) / root.dpr
            readonly property real from: root.inset + out
            // The glow at this device pixel and at the next one out, which the wider bands give.
            readonly property real here: root.glow((index + 0.5) / root.dpr)
            readonly property real next: index + 1 < root.bands ? root.glow((index + 1.5) / root.dpr) : 0

            x: -from
            y: -from
            width: root.width + from * 2
            height: root.height + from * 2
            radius: K.Look.outset(root.controlRadius, from)
            color: "transparent"
            border.width: out
            border.color: K.Look.alpha(root.tint, 1 - (1 - here) / (1 - next))
        }
    }

    Rectangle {
        x: -root.inset
        y: -root.inset
        width: root.width + root.inset * 2
        height: root.height + root.inset * 2
        radius: K.Look.outset(root.controlRadius, root.inset)
        color: "transparent"
        border.width: root.filled ? root.inset : K.Look.stroke(Tokens.focus.tube, root.dpr)
        border.color: root.filled ? root.core : root.tint
    }
}
