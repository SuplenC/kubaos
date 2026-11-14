import QtQuick
import KubaOS.Components as K

// One item of a menu, or the hairline between two groups: `kubaos-ui`'s `menu::menu_item` and
// `menu::menu_separator`. An item is a small control tall, 12 inside: an optional symbol, its
// words on one line, an optional value of its own at their end (When it reaches 10), the keys that
// do the same as keycaps, and a check whose room is kept when unchecked. Its corners are concentric inside the popover. The item the keyboard is on takes
// the secondary container. One that destroys something (Kill) is in the error colour, and lights
// in the error itself with its words `on_error`, pressed to `error_pressed`, so lit it reads as what
// it is.
//
// A held item destroys something that can't be taken back (Hold to remove), so it is taken only
// by holding it for `duration.hold`: the pointer pressed on it, or Enter while the keyboard is on
// it (`keyHolding`, which its menu sets). While it is held, the error colour sweeps across it from
// its start on `linear`, its words `on_error` under the sweep, and it is taken when the sweep
// reaches its end; let go or left early, the sweep runs back over `short`. It is in the error
// colour lit or not, and lit it lies on the error at `opacity.tint`, so the sweep has room to
// show. The sweep is a timer, not a move: reduce motion keeps it.
K.Pressable {
    id: root

    property string text
    property string icon: ""
    // The keys that do the same, one keycap each.
    property list<string> shortcut
    // A figure the item is about, set apart at the end of its words; a prompt that changes it opens
    // under it (`valueItem`).
    property string value: ""
    // Whether it shows a check at its end, and whether it is ticked.
    property bool checkable: false
    property bool checked: false
    property bool danger: false
    property bool highlighted: false
    // A hairline between two groups, in place of an item.
    property bool separator: false
    // Taken only once held for `duration.hold`; a held item is a danger item.
    property bool held: false
    // How far the sweep reaches across it, 0 to 1. The hold moves it; a sample pins it.
    property real hold: 0
    // Enter is down while the keyboard is on it: its menu holds it from the keyboard.
    property bool keyHolding: false

    readonly property bool dangerous: root.danger || root.held
    // The value's box, for a prompt to open under; null without a value.
    readonly property Item valueItem: root.value === "" ? null : line.valueItem
    // A danger item lit by the keyboard: it lies on the error, and its press is a fill of its own,
    // not the state layer.
    readonly property bool dangerLit: root.danger && !root.held && root.highlighted && root.enabled
    // Held by the pointer only while the pointer stays on it: leaving it lets go.
    readonly property bool holding: root.held && root.enabled && (root.keyHolding || (root.pointerDown && root.pointerOver))
    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property color ink: {
        if (!root.enabled)
            return K.Look.alpha(Tokens.color.on_surface, Tokens.opacity.disabled_content);
        if (root.held)
            return Tokens.color.error;
        if (root.dangerLit)
            return Tokens.color.on_error;
        if (root.highlighted)
            return Tokens.color.on_secondary_container;
        return root.danger ? Tokens.color.error : Tokens.color.on_surface;
    }
    readonly property color litFill: {
        if (root.held)
            return K.Look.alpha(Tokens.color.error, Tokens.opacity.tint);
        if (root.danger)
            return root.down ? Tokens.color.error_pressed : Tokens.color.error;
        return Tokens.color.secondary_container;
    }
    readonly property var probe: root.separator ? null : {
        role: "component",
        name: "MenuItem",
        variant: [root.icon === "" ? "" : "icon", root.checkable ? "check" : "", root.dangerous ? "danger" : ""].filter(word => word !== "").join(" "),
        fixed: {
            width: false,
            height: true
        }
    }

    function relayout(): void {
        if (root.separator)
            rule.relayout();
        else
            line.relayout();
    }

    implicitHeight: root.separator ? rule.implicitHeight : Tokens.size.control_sm
    activeFocusOnTab: false
    instant: !root.held

    // Pressed on, the sweep goes on to the end over what is left of `duration.hold`; let go, it
    // runs back.
    onHoldingChanged: {
        sweeping.stop();
        if (root.holding) {
            sweeping.to = 1;
            sweeping.duration = Tokens.duration.hold * (1 - root.hold);
            sweeping.easing.bezierCurve = Tokens.easing.linear;
        } else if (root.hold > 0) {
            sweeping.to = 0;
            sweeping.duration = Tokens.duration.short;
            sweeping.easing.bezierCurve = Tokens.easing.standard;
        } else {
            return;
        }
        sweeping.start();
    }

    NumberAnimation {
        id: sweeping

        target: root
        property: "hold"
        easing.type: Easing.BezierSpline
        onFinished: {
            if (root.holding && root.hold >= 1)
                root.activated();
        }
    }

    Rectangle {
        anchors.fill: parent
        visible: !root.separator
        radius: K.Look.nested(Tokens.radius.surface, Tokens.space.s12)
        color: root.highlighted && root.enabled ? root.litFill : "transparent"

        K.StateLayer {
            radius: parent.radius
            content: root.ink
            active: root.enabled
            hovered: root.hovered
            pressed: root.down && !root.dangerLit
        }
    }

    K.Stack {
        id: rule

        visible: root.separator
        width: root.width
        fill: true
        topPadding: Tokens.space.s4
        bottomPadding: Tokens.space.s4

        K.Divider {
        }
    }

    Content {
        id: line

        visible: !root.separator
        x: Tokens.space.s12
        width: root.width - Tokens.space.s12 * 2
        fixedHeight: root.height
        ink: root.ink
        text: root.text
        icon: root.icon
        value: root.value
        shortcut: root.shortcut
        checkable: root.checkable
        checked: root.checked
    }

    // The sweep, from the item's start to `hold` of its width: the error colour with the item's
    // content again in `on_error`, cut at its edge, so the words read on either side of it.
    Item {
        // Paint over the item, not a part of the page: the geometry check reads the item's own
        // content.
        readonly property bool overlay: true

        visible: root.held && root.hold > 0
        width: K.Look.snap(root.width * root.hold, root.dpr)
        height: root.height
        clip: true

        Rectangle {
            width: root.width
            height: root.height
            radius: K.Look.nested(Tokens.radius.surface, Tokens.space.s12)
            color: Tokens.color.error

            Content {
                x: Tokens.space.s12
                width: root.width - Tokens.space.s12 * 2
                fixedHeight: root.height
                ink: Tokens.color.on_error
                text: root.text
                icon: root.icon
                value: root.value
                shortcut: root.shortcut
                checkable: root.checkable
                checked: root.checked
            }
        }
    }

    // An item's symbol, words, value, keys and check in `ink`, across `width`.
    component Content: K.Row {
        id: content

        property color ink
        property string text
        property string icon
        property string value
        readonly property Item valueItem: figure
        property list<string> shortcut
        property bool checkable
        property bool checked

        gap: Tokens.space.s12
        align: "centre"

        K.Icon {
            visible: content.icon !== ""
            name: content.icon
            size: Tokens.type.body.icon
            color: content.ink
        }

        K.Stack {
            fill: true

            K.Text {
                text: content.text
                style: Tokens.type.body
                color: content.ink
                maximumLines: 1
            }
        }

        K.Text {
            id: figure

            visible: content.value !== ""
            text: content.value
            style: Tokens.type.mono_large
            color: content.ink
        }

        K.Row {
            visible: content.shortcut.length > 0
            gap: Tokens.space.s4
            align: "centre"

            Repeater {
                model: content.shortcut

                K.Keycap {
                    required property string modelData

                    key: modelData
                }
            }
        }

        Item {
            visible: content.checkable
            implicitWidth: Tokens.type.body.icon
            implicitHeight: Tokens.type.body.icon

            K.Icon {
                visible: content.checked
                name: "check"
                size: Tokens.type.body.icon
                color: content.ink
            }
        }
    }
}
