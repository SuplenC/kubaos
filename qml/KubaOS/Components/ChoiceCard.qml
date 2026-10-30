import QtQuick
import KubaOS.Components as K

// One of a few cards to pick from, at the surface radius: `kubaos-ui`'s `card::choice_card`. The
// picked one takes the secondary container and an accent hairline; its focus tube is the cold
// one, since the accent already means picked here. Disabled (`enabled: false`), what it holds is at
// the disabled opacity and the card is the disabled container, with no hover, press or focus.
K.Pressable {
    id: root

    signal picked

    default property alias content: slot.data
    property alias room: slot.room
    property bool checked: false

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "ChoiceCard",
            variant: "",
            fixed: {
                width: false,
                height: false,
                holds: true
            }
        })

    function relayout(): void {
        slot.relayout();
    }

    implicitHeight: slot.implicitHeight
    onActivated: root.picked()

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.surface
        color: {
            const c = Tokens.color;
            if (!root.enabled)
                return K.Look.alpha(c.on_surface, Tokens.opacity.disabled_container);
            if (root.checked)
                return c.secondary_container;
            if (root.enabled && root.pointerOver)
                return K.Look.mix(c.surface_container_low, c.on_surface, Tokens.opacity.hover);
            return K.Look.alpha(c.surface_container_low, Tokens.opacity.well);
        }
        border.width: K.Look.hairline(root.dpr)
        border.color: root.checked && root.enabled ? Tokens.color.primary : Tokens.color.outline_variant

        Behavior on color {
            ColorAnimation {
                duration: Tokens.duration.micro
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }
    }

    K.Slot {
        id: slot

        width: root.width
        fill: true
        opacity: root.enabled ? 1 : Tokens.opacity.disabled_content
        leftPadding: Tokens.space.s16
        rightPadding: Tokens.space.s16
        topPadding: Tokens.space.s16
        bottomPadding: Tokens.space.s16
    }

    K.FocusRing {
        shown: root.focused && root.enabled
        controlRadius: Tokens.radius.surface
        cold: true
    }
}
