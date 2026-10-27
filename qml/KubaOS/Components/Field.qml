import QtQuick
import KubaOS.Components as K

// One line of text: `kubaos-ui`'s `field::field`. As tall as a regular button, the control
// radius, a hairline plus the focus ring, `body_large` text. An optional 20 icon leads it, the
// text starting past it. A placeholder is a hint, never a value that looks typed.
//
// States: `Normal`; `Invalid`, in the error colour with the reason in the note under it; and
// `Locked`, read-only, dimmed and never focused.
FocusScope {
    id: root

    // Enter in the field: what was typed is done.
    signal accepted
    // A key pressed while the field has the keyboard, before the field takes it: accept it to keep
    // it from the text, as a search box over a list does with its arrows and Ctrl keys.
    signal keyPressed(var event)

    property alias text: input.text
    property alias cursorPosition: input.cursorPosition
    property string placeholder
    // A symbol inside the field, before the text, or none.
    property string icon: ""
    // Bullets instead of the value: a password.
    property bool secret: false
    // `Normal`, `Invalid` or `Locked`.
    property string status: "Normal"
    // Whether the keyboard is on it; Qt's focus by default, or the app's where it owns focus.
    property bool focused: input.activeFocus
    // Items that see each key before the text does: a list under a search, which takes the
    // arrows, Ctrl+J/K/N/P and Enter.
    property list<Item> forwardTo

    readonly property bool fillWidth: true
    readonly property bool locked: root.status === "Locked"
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // Where the text starts: 12 in, or past the icon and 8 more.
    readonly property real textStart: root.icon === "" ? Tokens.space.s12 : Tokens.space.s12 + Tokens.icon.s20 + Tokens.space.s8
    readonly property var probe: ({
            role: "component",
            name: "Field",
            variant: root.icon === "" ? "" : "icon",
            fixed: {
                width: false,
                height: true
            }
        })

    implicitHeight: Tokens.size.control_md
    activeFocusOnTab: !root.locked

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.control
        border.width: K.Look.hairline(root.dpr)
        border.color: {
            const c = Tokens.color;
            if (root.locked)
                return K.Look.alpha(c.outline_variant, Tokens.opacity.locked);
            if (root.status === "Invalid")
                return c.error;
            if (root.focused)
                return c.primary;
            return hover.hovered ? c.outline : c.outline_variant;
        }
        color: {
            const c = Tokens.color;
            if (root.locked)
                return K.Look.alpha(c.surface_container_high, Tokens.opacity.locked);
            if (root.status === "Normal" && (root.focused || hover.hovered))
                return c.surface_container_highest;
            return c.surface_container_high;
        }

        Behavior on border.color {
            ColorAnimation {
                duration: Tokens.duration.micro
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }
    }

    // The icon in a box 12 wider than itself on each side, so it centres on the field's height on
    // a whole device pixel.
    Item {
        readonly property var probe: ({
                role: "box"
            })

        visible: root.icon !== ""
        width: Tokens.space.s12 * 2 + Tokens.icon.s20
        height: root.height

        K.Icon {
            x: Tokens.space.s12
            y: K.Look.snap((root.height - Tokens.icon.s20) / 2, root.dpr)
            name: root.icon
            size: Tokens.icon.s20
            color: Tokens.color.on_surface_variant
        }
    }

    TextInput {
        id: input

        x: root.textStart
        y: Tokens.space.s8
        width: root.width - root.textStart - Tokens.space.s12
        height: Tokens.type.body_large.line
        focus: true
        readOnly: root.locked
        enabled: !root.locked
        clip: true
        echoMode: root.secret ? TextInput.Password : TextInput.Normal
        // iced's bullet; Qt's default is the heavier black circle.
        passwordCharacter: "•"
        verticalAlignment: TextInput.AlignVCenter
        color: root.locked ? Tokens.color.on_surface_variant : Tokens.color.on_surface
        selectionColor: K.Look.alpha(Tokens.color.primary, Tokens.opacity.selection)
        selectedTextColor: Tokens.color.on_surface
        font.family: Tokens.type.body_large.family
        font.weight: Tokens.type.body_large.weight
        font.pixelSize: Tokens.type.body_large.size
        font.kerning: false
        font.preferShaping: false
        font.hintingPreference: Font.PreferNoHinting
        onAccepted: root.accepted()
        // The `forwardTo` items see a key first, then a `keyPressed` handler, then the text.
        Keys.forwardTo: root.forwardTo
        Keys.onPressed: event => {
            event.accepted = false;
            root.keyPressed(event);
        }

        Text {
            anchors.fill: parent
            visible: input.text === "" && input.preeditText === ""
            text: root.placeholder
            verticalAlignment: Text.AlignVCenter
            color: K.Look.alpha(Tokens.color.on_surface_variant, Tokens.opacity.placeholder)
            font: input.font
        }
    }

    HoverHandler {
        id: hover

        enabled: !root.locked
        cursorShape: Qt.IBeamCursor
    }

    K.FocusRing {
        shown: root.focused && !root.locked
        controlRadius: Tokens.radius.control
    }
}
