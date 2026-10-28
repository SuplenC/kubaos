import QtQuick
import KubaOS.Components as K

// A value from 0 to 1, or one of a few stops, as a pill: `kubaos-ui`'s `slider::slider`. The fill
// is the value: its round end marks it, the value being that end circle's centre, and at 0 the
// fill is the circle behind the icon. The value, or a name for it, sits at the far end, in the
// track's colours over the track and the fill's over the fill. Regular (36) or compact (28); a
// compact slider shows a number only under the pointer, while held or focused, and a name always.
//
// Steps show as dots on both sides of the fill, each in its side's colour. The mark the value
// rests on, a step or a sticky point it holds at, turns solid and pops in; no dot is drawn under
// the icon or the label, which mark that place already.
//
// It only reports what the user does, through `moved`, `stepped` and `iconActivated`, so `value`
// and `index` can stay bound to a live service. A drag on a slider with stops moves freely and
// settles on the nearest stop when released. Left and Right (h and l) step by a twentieth or a
// stop, Home and End go to the ends, the wheel steps. The icon is a button where `iconButton` is
// set (volume's mutes) and on a slider with stops, where each press steps from high to low and
// back, turning at either end.
Item {
    id: root

    // A continuous slider moved to `value`, 0 to 1.
    signal moved(real value)
    // A slider with stops moved to stop `index`.
    signal stepped(int index)
    // The icon of a continuous slider was pressed.
    signal iconActivated

    property string icon
    // "Regular" or "Compact".
    property string size: "Regular"
    property real value: 0
    // How many stops it snaps to; under 2, it doesn't.
    property int stops: 0
    // The stop it is on, from 0.
    property int index: 0
    // Each stop's name, the one nearest the value shown.
    property var names: []
    // A name shown in place of the value, always: "Muted", "Auto · 55%".
    property string words: ""
    // The number shown in place of the percentage, as the owner reads the value.
    property string number: ""
    // The widest the number reads: its box is this wide, so the number ends in one place.
    property string widest: "100%"
    // The fill in the softer, desaturated accent: a level something else picked, as Auto
    // brightness's.
    property bool soft: false
    // A sticky point, a fraction along the track, or -1 for none.
    property real sticky: -1
    // Whether the icon of a continuous slider is a button, sending `iconActivated`.
    property bool iconButton: false
    // What the icon's button does, for assistive technology: "Mute".
    property string iconLabel: ""
    // Whether the keyboard is on it; Qt's focus by default, or the app's where it owns focus.
    property bool focused: root.activeFocus
    // A state shown whatever the pointer does, `hovered` or `pressed`: the specimen's view of it.
    property string preview: ""

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property bool compact: root.size === "Compact"
    readonly property real side: root.compact ? Tokens.size.slider_compact : Tokens.size.slider
    readonly property real anchorSide: root.compact ? Tokens.size.slider_anchor_compact : Tokens.size.slider_anchor
    readonly property bool stepping: root.stops > 1
    readonly property int last: root.stops - 1
    readonly property bool iconPresses: root.enabled && (root.iconButton || root.stepping)
    // Where the value rests when nothing holds it: the stop, or the value.
    readonly property real rest: root.stepping ? root.index / root.last : Math.max(0, Math.min(1, root.value))
    // The fraction the fill shows: under the pointer while a drag holds it, else easing to rest.
    property real shown: 0
    readonly property real span: Math.max(1, root.width - root.side)
    readonly property real valueX: root.centre(root.shown)
    // Where the fill ends, on a whole device pixel.
    readonly property real fillEnd: Math.min(root.width, K.Look.snap(root.valueX + root.side / 2, root.dpr))
    readonly property bool hovered: root.enabled && (root.preview === "hovered" || (pointer.containsMouse && !iconArea.containsMouse))
    readonly property bool held: root.enabled && (root.preview === "pressed" || pointer.dragging)
    // How far the fill lights up toward the bright end: a little under the pointer, and plainly
    // while pressed or dragged.
    readonly property real lift: root.held ? Tokens.opacity.held : root.hovered ? Tokens.opacity.press : 0
    // What the fill brightens toward: the palette's lighter end, so a held fill lights up in both
    // modes.
    readonly property color bright: Tokens.mode === "light" ? Tokens.color.surface : Tokens.color.on_surface
    readonly property real dim: root.enabled ? 1 : Tokens.opacity.disabled_content
    readonly property color fillColor: K.Look.alpha(K.Look.mix(root.soft ? Tokens.color.secondary : Tokens.color.primary, root.bright, root.lift), root.dim)
    readonly property color onFill: K.Look.alpha(root.soft ? Tokens.color.on_secondary : Tokens.color.on_primary, root.dim)
    readonly property color onTrack: K.Look.alpha(Tokens.color.on_surface, root.dim)
    // What the label says: a name, a stop's name, or the number.
    readonly property bool numeric: root.words === "" && !(root.stepping && root.names.length > 0)
    readonly property string label: root.words !== "" ? root.words : root.stepping && root.names.length > 0 ? root.names[Math.max(0, Math.min(root.last, Math.round(root.shown * root.last)))] ?? "" : root.number !== "" ? root.number : `${Math.round(root.rest * 100)}%`
    readonly property QtObject labelStyle: root.numeric ? Tokens.type.label_number : Tokens.type.label
    // A compact slider's number shows under the pointer, while held or focused.
    readonly property bool labelShown: !root.compact || !root.numeric || root.preview !== "" || pointer.containsMouse || pointer.dragging || root.focused
    readonly property real pad: Tokens.space.s12
    readonly property real labelWidth: root.numeric ? Math.max(onTrackText.implicitWidth, widestText.implicitWidth) : onTrackText.implicitWidth
    // Where the label's box starts.
    readonly property real labelLeft: root.width - root.pad - root.labelWidth
    // The mark the value rests on, a fraction, or -1: a slider with stops at rest on its stop, or a
    // sticky point the value holds at.
    // Never ahead of the fill: a value set there counts once the fill has arrived.
    readonly property real resting: settle.running ? -1 : root.stepping ? (pointer.dragging ? -1 : root.rest) : root.sticky >= 0 && Math.abs(root.shown - root.sticky) < 1e-4 ? root.sticky : -1
    // Each step's fraction, or the sticky point's: what the dots stand for. It changes only with
    // the steps, so the dots aren't made again as the value moves.
    readonly property var points: {
        if (root.stepping) {
            const all = [];
            for (let i = 0; i < root.stops; i++)
                all.push(i / root.last);
            return all;
        }
        return root.sticky >= 0 ? [root.sticky] : [];
    }
    // A step of a continuous slider from the keyboard or the wheel.
    readonly property real nudge: 0.05
    // Which way the icon steps next: -1 toward the first stop, +1 toward the last.
    property int direction: -1
    readonly property var probe: ({
            role: "component",
            name: "Slider",
            variant: [root.compact ? "compact" : "regular", root.stepping ? "stepped" : ""].filter(word => word !== "").join(" "),
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: root.side
    activeFocusOnTab: root.enabled

    // Where the value `fraction` stands: the centre of the fill's round end.
    function centre(fraction: real): real {
        return root.side / 2 + root.span * Math.max(0, Math.min(1, fraction));
    }

    // The fraction whose centre is under `x`, held on the sticky point while near it.
    function fractionAt(x: real): real {
        const fraction = Math.max(0, Math.min(1, (x - root.side / 2) / root.span));
        const reach = Tokens.size.slider_hold / root.span;
        return root.sticky >= 0 && Math.abs(fraction - root.sticky) <= reach ? root.sticky : fraction;
    }

    // Whether a dot `size` across at `x` stays clear of the icon and the label, which mark that
    // place already. `kubaos-ui`'s `Track::marks` holds the same rules.
    function clear(x: real, size: real): bool {
        return x - size / 2 >= root.side + Tokens.space.s4 && x + size / 2 <= root.labelLeft - Tokens.space.s4;
    }

    // Whether the step at `point` shows: clear of the icon and the label, not the mark the value
    // rests on (its anchor shows there), not under the moving value's end, and not astride the
    // fill's edge, where it would take one side's colour over both.
    function stepShows(point: real): bool {
        const x = root.centre(point);
        const size = Tokens.size.slider_stop;
        const anchored = root.resting >= 0 && Math.abs(root.resting - point) < 1e-4;
        const fillEnd = root.valueX + root.side / 2;
        return root.clear(x, size) && !anchored && Math.abs(x - root.valueX) >= root.anchorSide / 2 && Math.abs(x - fillEnd) >= size / 2;
    }

    // Eases the fill from where it is to where the value rests.
    function settleTo(): void {
        settle.stop();
        if (Math.abs(root.shown - root.rest) < 1e-6)
            return;
        settle.from = root.shown;
        settle.to = root.rest;
        settle.start();
    }

    function request(v: real): void {
        root.moved(Math.max(0, Math.min(1, v)));
    }

    function step(to: int): void {
        const clamped = Math.max(0, Math.min(root.last, to));
        if (clamped !== root.index)
            root.stepped(clamped);
    }

    // The icon's press: a slider with stops steps from high to low and back, turning at either
    // end; a continuous one sends `iconActivated`.
    function pressIcon(): void {
        if (!root.stepping) {
            root.iconActivated();
            return;
        }
        if (root.index + root.direction < 0 || root.index + root.direction > root.last)
            root.direction = -root.direction;
        root.step(root.index + root.direction);
    }

    onRestChanged: if (!pointer.dragging)
        root.settleTo()
    onRestingChanged: if (root.resting >= 0)
        pop.restart()
    Component.onCompleted: root.shown = root.rest

    // The fill easing to rest: to a value set from outside, and onto a stop after a drag.
    NumberAnimation {
        id: settle

        target: root
        property: "shown"
        duration: Tokens.duration.short
        easing.type: Easing.BezierSpline
        easing.bezierCurve: root.stepping ? Tokens.easing.standard_decel : Tokens.easing.standard
    }

    Keys.onPressed: event => {
        const k = event.key;
        const back = k === Qt.Key_Left || k === Qt.Key_H;
        const forward = k === Qt.Key_Right || k === Qt.Key_L;
        if (k === Qt.Key_Home || k === Qt.Key_End) {
            const end = k === Qt.Key_End;
            if (root.stepping)
                root.step(end ? root.last : 0);
            else
                root.request(end ? 1 : 0);
        } else if (back || forward) {
            if (root.stepping)
                root.step(root.index + (forward ? 1 : -1));
            else
                root.request(root.value + (forward ? root.nudge : -root.nudge));
        } else {
            return;
        }
        event.accepted = true;
    }

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.full
        color: K.Look.alpha(K.Look.alpha(Tokens.color.surface_container_highest, Tokens.opacity.track), root.dim)
    }

    Rectangle {
        width: root.fillEnd
        height: root.height
        radius: Tokens.radius.full
        color: root.fillColor

        Behavior on color {
            ColorAnimation {
                duration: Tokens.duration.micro
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }
    }

    // The steps, each in its side's colour.
    Repeater {
        model: root.points

        Rectangle {
            required property real modelData
            readonly property real at: root.centre(modelData)

            visible: root.stepShows(modelData)
            x: at - width / 2
            y: (root.height - height) / 2
            width: Tokens.size.slider_stop
            height: Tokens.size.slider_stop
            radius: Tokens.radius.full
            color: at < root.valueX + root.side / 2 ? K.Look.alpha(root.onFill, Tokens.opacity.subdued) : K.Look.alpha(Tokens.color.on_surface_variant, root.dim)
        }
    }

    // The mark the value rests on, solid, popping in.
    Rectangle {
        id: anchorDot

        visible: root.resting >= 0 && root.clear(root.centre(root.resting), root.anchorSide)
        x: root.centre(Math.max(0, root.resting)) - width / 2
        y: (root.height - height) / 2
        width: root.anchorSide
        height: root.anchorSide
        radius: Tokens.radius.full
        color: root.onFill

        NumberAnimation {
            id: pop

            target: anchorDot
            property: "scale"
            from: 0
            to: 1
            duration: Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.pop
        }
    }

    MouseArea {
        id: pointer

        // Whether a press on the track holds the value, the fill following the pointer.
        property bool dragging: false
        // The wheel's turn not yet a whole notch: a touchpad sends it in pieces.
        property real wheelTurn: 0

        anchors.fill: parent
        enabled: root.enabled
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        preventStealing: true
        onPressed: mouse => {
            settle.stop();
            dragging = true;
            root.shown = root.fractionAt(mouse.x);
            if (!root.stepping)
                root.request(root.shown);
        }
        onPositionChanged: mouse => {
            if (!dragging)
                return;
            root.shown = root.fractionAt(mouse.x);
            if (!root.stepping)
                root.request(root.shown);
        }
        onReleased: {
            if (dragging && root.stepping)
                root.step(Math.round(root.shown * root.last));
            dragging = false;
            root.settleTo();
        }
        onCanceled: {
            dragging = false;
            root.settleTo();
        }
        onWheel: wheel => {
            // A wheel's notch is 120 (Qt's QWheelEvent::angleDelta); a sideways scroll is no turn.
            wheelTurn += wheel.angleDelta.y;
            const notches = Math.trunc(wheelTurn / 120);
            if (notches === 0)
                return;
            wheelTurn -= notches * 120;
            if (root.stepping)
                root.step(root.index + notches);
            else
                root.request(root.value + notches * root.nudge);
        }
    }

    // Drawn over the track by the slider itself, not laid out beside it, so the geometry check
    // sees the slider as one box, as iced's slider reports it.
    Item {
        readonly property bool overlay: true

        anchors.fill: parent

        K.Icon {
            x: K.Look.snap((root.side - width) / 2, root.dpr)
            y: K.Look.snap((root.side - height) / 2, root.dpr)
            name: root.icon
            size: Tokens.type.label.icon
            color: root.onFill
        }

        // The label over the track, from where the fill ends.
        Item {
            x: root.fillEnd
            width: Math.max(0, root.width - root.fillEnd)
            height: root.height
            clip: true
            opacity: labels.opacity

            K.Text {
                id: onTrackText

                x: root.width - root.pad - width - parent.x
                y: K.Look.snap((root.height - height) / 2, root.dpr)
                text: root.label
                style: root.labelStyle
                color: root.onTrack
            }
        }

        // The label over the fill, up to where it ends.
        Item {
            id: labels

            width: root.fillEnd
            height: root.height
            clip: true
            opacity: root.labelShown ? 1 : 0

            Behavior on opacity {
                NumberAnimation {
                    duration: Tokens.duration.micro
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: Tokens.easing.standard
                }
            }

            K.Text {
                x: root.width - root.pad - width
                y: K.Look.snap((root.height - height) / 2, root.dpr)
                text: root.label
                style: root.labelStyle
                color: root.onFill
            }
        }

        // The widest number, measured and never drawn.
        K.Text {
            id: widestText

            visible: false
            text: root.widest
            style: Tokens.type.label_number
        }
    }

    // The icon's button: over the drawn icon, so its state layer lies over it, as every state
    // layer lies over its content. The icon's circle is the fill's start, always on the fill.
    MouseArea {
        id: iconArea

        width: root.side
        height: root.side
        enabled: root.iconPresses
        visible: root.iconPresses
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        Accessible.role: Accessible.Button
        Accessible.name: root.iconLabel
        // The button is the circle: a press in the square's corners goes to the track.
        onPressed: mouse => mouse.accepted = Math.hypot(mouse.x - width / 2, mouse.y - height / 2) <= width / 2
        onClicked: root.pressIcon()

        K.StateLayer {
            radius: Tokens.radius.full
            content: root.onFill
            hovered: iconArea.containsMouse
            pressed: iconArea.pressed
        }
    }

    K.FocusRing {
        shown: root.focused && root.enabled
        controlRadius: Tokens.radius.full
    }
}
