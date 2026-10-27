import QtQuick
import KubaOS.Components as K

// A value from 0 to 1, or one of a few stops, picked along a track: `kubaos-ui`'s
// `slider::slider`. 24 tall: a progress bar's track, filled up to a 16 thumb, a pill and a circle;
// the focus ring is a pill round it all. A slider with stops marks each one the fill hasn't
// reached and snaps to them.
//
// It only reports what the user does, through `moved` and `stepped`, so `value` and `index` can
// stay bound to a live service. Left and Right (h and l) step by a twentieth or a stop, Home and
// End go to the ends, the wheel steps, and a drag follows the pointer: on a slider with stops the
// fill shows the stop under the pointer and picks it on release.
Item {
    id: root

    // A continuous slider moved to `value`, 0 to 1.
    signal moved(real value)
    // A slider with stops moved to stop `index`.
    signal stepped(int index)

    property real value: 0
    // How many stops it snaps to; under 2, it doesn't.
    property int stops: 0
    // The stop it is on, from 0.
    property int index: 0
    // Whether the keyboard is on it; Qt's focus by default, or the app's where it owns focus.
    property bool focused: root.activeFocus
    // A state shown whatever the pointer does, `hovered` or `pressed`: the specimen's view of it.
    property string preview: ""

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property bool stepping: root.stops > 1
    readonly property int last: root.stops - 1
    // The stop the fill shows: the one under the pointer while a drag holds it.
    readonly property int shownIndex: pointer.pressed && pointer.dragIndex >= 0 ? pointer.dragIndex : root.index
    readonly property real fraction: root.stepping ? root.shownIndex / root.last : Math.max(0, Math.min(1, root.value))
    // How far the thumb's centre travels: the track less the thumb.
    readonly property real travel: Math.max(0, root.width - Tokens.size.slider_thumb)
    readonly property bool hovered: root.enabled && (root.preview === "hovered" || pointer.containsMouse)
    readonly property bool held: root.enabled && (root.preview === "pressed" || pointer.pressed)
    // The state layer over the fill and the thumb.
    readonly property real stateLayer: root.held ? Tokens.opacity.press : root.hovered ? Tokens.opacity.hover : 0
    // What the fill brightens toward: the palette's lighter end, so a held fill lights up in both
    // modes.
    readonly property color bright: Tokens.mode === "light" ? Tokens.color.surface : Tokens.color.on_surface
    readonly property real dim: root.enabled ? 1 : Tokens.opacity.disabled_content
    // A step of a continuous slider from the keyboard or the wheel.
    readonly property real nudge: 0.05
    readonly property var probe: ({
            role: "component",
            name: "Slider",
            variant: root.stepping ? "stepped" : "",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: Tokens.size.slider
    activeFocusOnTab: root.enabled

    // The left edge of the thumb at `fraction` of the way, on a whole device pixel.
    function thumbAt(fraction: real): real {
        return K.Look.snap(root.travel * fraction, root.dpr);
    }

    // The stop nearest `x`, a point along the slider.
    function stopAt(x: real): int {
        if (root.travel <= 0)
            return root.index;
        const at = Math.round((x - Tokens.size.slider_thumb / 2) / root.travel * root.last);
        return Math.max(0, Math.min(root.last, at));
    }

    function request(v: real): void {
        root.moved(Math.max(0, Math.min(1, v)));
    }

    function step(to: int): void {
        const clamped = Math.max(0, Math.min(root.last, to));
        if (clamped !== root.index)
            root.stepped(clamped);
    }

    function follow(x: real): void {
        if (root.stepping)
            pointer.dragIndex = root.stopAt(x);
        else
            root.request((x - Tokens.size.slider_thumb / 2) / Math.max(1, root.travel));
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

    Item {
        id: rail

        y: K.Look.snap((root.height - Tokens.size.progress) / 2, root.dpr)
        width: root.width
        height: Tokens.size.progress

        Rectangle {
            anchors.fill: parent
            radius: Tokens.radius.full
            color: K.Look.alpha(K.Look.alpha(Tokens.color.surface_container_highest, Tokens.opacity.track), root.dim)
        }

        // The fill takes the state layer too, toward the palette's bright end: it lights up while
        // pressed or dragged, as a progress bar's fill glows.
        Rectangle {
            width: thumb.x + Tokens.size.slider_thumb / 2
            height: parent.height
            radius: Tokens.radius.full
            color: K.Look.alpha(K.Look.mix(Tokens.color.primary, root.bright, root.stateLayer), root.dim)

            Behavior on color {
                ColorAnimation {
                    duration: Tokens.duration.micro
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: Tokens.easing.standard
                }
            }
        }

        // A mark at each stop the fill hasn't reached, centred where the thumb stands at it.
        Repeater {
            model: root.stepping ? root.stops : 0

            Rectangle {
                required property int index

                visible: index > root.shownIndex
                x: K.Look.snap(root.travel * index / root.last + (Tokens.size.slider_thumb - Tokens.size.slider_stop) / 2, root.dpr)
                y: K.Look.snap((Tokens.size.progress - Tokens.size.slider_stop) / 2, root.dpr)
                width: Tokens.size.slider_stop
                height: Tokens.size.slider_stop
                radius: Tokens.radius.full
                color: K.Look.alpha(Tokens.color.on_surface_variant, root.dim)
            }
        }
    }

    Item {
        id: thumb

        // It glides to a stop, or to a value set from outside; a continuous drag it follows at once,
        // so the thumb never trails the pointer.
        x: root.thumbAt(root.fraction)
        y: K.Look.snap((root.height - Tokens.size.slider_thumb) / 2, root.dpr)
        width: Tokens.size.slider_thumb
        height: Tokens.size.slider_thumb

        Behavior on x {
            enabled: root.stepping || !pointer.pressed

            NumberAnimation {
                duration: Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }

        Rectangle {
            anchors.fill: parent
            radius: Tokens.radius.full
            color: K.Look.alpha(Tokens.color.primary, root.dim)

            K.StateLayer {
                radius: parent.radius
                content: Tokens.color.on_primary
                active: root.enabled
                hovered: root.hovered
                pressed: root.held
            }
        }
    }

    K.FocusRing {
        shown: root.focused && root.enabled
        controlRadius: Tokens.radius.full
    }

    MouseArea {
        id: pointer

        // The stop under the pointer during a press, -1 when none.
        property int dragIndex: -1
        // The wheel's turn not yet a whole notch: a touchpad sends it in pieces.
        property real wheelTurn: 0

        anchors.fill: parent
        enabled: root.enabled
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        preventStealing: true
        onPressed: mouse => root.follow(mouse.x)
        onPositionChanged: mouse => {
            if (pressed)
                root.follow(mouse.x);
        }
        onReleased: {
            if (root.stepping && dragIndex >= 0)
                root.step(dragIndex);
            dragIndex = -1;
        }
        onCanceled: dragIndex = -1
        onWheel: wheel => {
            if (!root.stepping) {
                root.request(root.value + (wheel.angleDelta.y > 0 ? root.nudge : -root.nudge));
                return;
            }
            // A wheel's notch is 120 (Qt's QWheelEvent::angleDelta).
            wheelTurn += wheel.angleDelta.y;
            const notches = Math.trunc(wheelTurn / 120);
            if (notches !== 0) {
                wheelTurn -= notches * 120;
                root.step(root.index + notches);
            }
        }
    }
}
