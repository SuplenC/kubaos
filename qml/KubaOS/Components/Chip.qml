import QtQuick
import KubaOS.Components as K

// A short fact or pick, 24 tall at the control radius, which on a chip is a pill: `kubaos-ui`'s
// `chip::chip`. The label starts where the curve ends; an icon sits closer in. A removable chip
// ends in a close button, a circle as tall as the chip, so its hover layer shares the chip's curve.
// A pressable chip takes a press of its own (a pick made the default, a setting turned on), with
// the state layer, and the keyboard, with the focus tube, unless it is no stop for Tab. Disabled
// (`enabled: false`), its content is at the disabled opacity and the chip at its own, and neither
// its press nor its close button takes the pointer or the keyboard.
Item {
    id: root

    signal removed
    signal activated

    property string text
    // `Neutral` (a fact), `Selected` (one of several picks), `Accent` (a tag that matters),
    // `Cold` (rarely: a demo, a preview) or `Filled` (the accent's own fill: an action among
    // states, Reboot beside Done).
    property string kind: "Neutral"
    // A symbol before the label, or none.
    property string icon: ""
    // Whether it ends in a close button that sends `removed`.
    property bool removable: false
    // Whether a press on it sends `activated`.
    property bool pressable: false
    // Whether Tab stops on a pressable chip. Off where the field or the row beside the chips is
    // the stop and a click picks one (the clock's timer presets and settings).
    property bool tabStop: true
    // A pressable chip's name, in a hint under it while the pointer or the keyboard is on it.
    property string tooltip: ""

    // Whether the keyboard, or the pointer, is on a pressable chip: what a tooltip on it follows.
    readonly property bool focused: root.pressable && press.focused
    readonly property bool pointerOver: root.pressable && press.pointerOver

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var colours: {
        const c = Tokens.color;
        if (!root.enabled)
            return [K.Look.alpha(c.on_surface, Tokens.opacity.disabled_container), K.Look.alpha(c.on_surface, Tokens.opacity.disabled_content)];
        switch (root.kind) {
        case "Selected":
            return [c.secondary_container, c.on_secondary_container];
        case "Accent":
            return [K.Look.alpha(c.primary, Tokens.opacity.tint), c.primary];
        case "Cold":
            return [K.Look.alpha(c.cold, Tokens.opacity.tint), c.cold];
        case "Filled":
            return [c.primary, c.on_primary];
        default:
            return [c.surface_container_high, c.on_surface];
        }
    }
    readonly property real leftPadding: root.icon === "" ? Tokens.space.s12 : Tokens.space.s8
    readonly property real rightPadding: root.removable ? 0 : Tokens.space.s12
    readonly property var probe: ({
            role: "component",
            name: "Chip",
            variant: [root.kind, root.icon === "" ? "" : "icon", root.removable ? "removable" : ""].filter(word => word !== "").join(" "),
            fixed: {
                width: false,
                height: true
            }
        })

    implicitWidth: K.Look.snapUp(root.leftPadding + outer.implicitWidth + root.rightPadding, root.dpr)
    implicitHeight: Tokens.size.chip

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.control
        color: root.colours[0]
    }

    K.Pressable {
        id: press

        anchors.fill: parent
        visible: root.pressable
        activeFocusOnTab: root.tabStop && press.enabled
        Accessible.name: root.text
        Accessible.role: Accessible.Button
        onActivated: root.activated()

        K.StateLayer {
            radius: Tokens.radius.control
            content: root.colours[1]
            hovered: parent.hovered
            pressed: parent.down
        }

        K.FocusRing {
            shown: press.focused
            filled: root.kind === "Filled"
            controlRadius: Tokens.radius.control
        }
    }

    K.Row {
        id: outer

        x: root.leftPadding
        y: K.Look.snap((Tokens.size.chip - outer.implicitHeight) / 2, root.dpr)
        align: "centre"

        K.Row {
            gap: Tokens.space.s4
            align: "centre"

            K.Icon {
                visible: root.icon !== ""
                name: root.icon
                size: Tokens.type.label.icon
                color: root.colours[1]
            }

            K.Text {
                text: root.text
                style: Tokens.type.label
                color: root.colours[1]
            }

            K.Pressable {
                visible: root.removable
                implicitWidth: Tokens.size.chip
                implicitHeight: Tokens.size.chip
                Accessible.name: qsTr("Remove")
                onActivated: root.removed()

                K.StateLayer {
                    radius: Tokens.radius.full
                    content: root.colours[1]
                    hovered: parent.hovered
                    pressed: parent.down
                }

                K.Icon {
                    x: Tokens.space.s4
                    y: Tokens.space.s4
                    name: "close"
                    size: Tokens.type.label.icon
                    color: root.colours[1]
                }
            }
        }
    }

    // Made only for a chip with a tooltip, so a row of plain chips builds no hints.
    Loader {
        anchors.fill: parent
        active: root.pressable && root.tooltip !== ""

        sourceComponent: K.Hint {
            objectName: "hint"
            text: root.tooltip
            shown: root.pointerOver || root.focused
        }
    }
}
