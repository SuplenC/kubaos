import QtQuick
import KubaOS.Components as K

// On or off, 44 by 24, the knob 12 off and 16 on, a hairline round the track while off:
// `kubaos-ui`'s `switch::switch`. It takes effect at once; Space, Enter or a click toggles it, and
// the knob moves over `short`.
K.Pressable {
    id: root

    signal toggled

    property bool checked: false
    // How far the knob is, 0 off to 1 on: follows `checked` over `short`, or is pinned.
    property real position: root.checked ? 1 : 0

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property real dim: root.enabled ? 1 : Tokens.opacity.disabled_content
    readonly property var probe: ({
            role: "component",
            name: "Switch",
            variant: "",
            fixed: {
                width: true,
                height: true
            }
        })

    implicitWidth: Tokens.size.switch_width
    implicitHeight: Tokens.size.switch_height
    onActivated: root.toggled()

    Behavior on position {
        NumberAnimation {
            duration: Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: Tokens.easing.standard
        }
    }

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.full
        color: {
            const c = Tokens.color;
            const track = K.Look.mix(c.surface_container_highest, c.primary, root.position);
            const hovered = root.enabled && root.pointerOver;
            return K.Look.alpha(hovered ? K.Look.mix(track, c.on_surface, Tokens.opacity.hover) : track, root.dim);
        }

        Rectangle {
            anchors.fill: parent
            visible: root.position < 1
            radius: Tokens.radius.full
            color: "transparent"
            border.width: K.Look.hairline(root.dpr)
            border.color: K.Look.alpha(Tokens.color.outline, root.dim * (1 - root.position))
        }
    }

    Rectangle {
        readonly property real diameter: Tokens.size.switch_knob_off + (Tokens.size.switch_knob_on - Tokens.size.switch_knob_off) * root.position

        x: root.height / 2 + (root.width - root.height) * root.position - diameter / 2
        y: (root.height - diameter) / 2
        width: diameter
        height: diameter
        radius: Tokens.radius.full
        color: K.Look.alpha(K.Look.mix(Tokens.color.outline, Tokens.color.on_primary, root.position), root.dim)
    }

    K.FocusRing {
        shown: root.focused && root.enabled
        controlRadius: Tokens.radius.full
    }
}
