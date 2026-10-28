import QtQuick
import KubaOS.Components as K

// One step of a stepper or a checklist: `kubaos-ui`'s `step::step`. An optional symbol, a
// `body_strong` title and a `small` subtitle that wraps to as many lines as it needs, then where
// the step stands: to do a plain chip, done a tonal one with a check, an action (Reboot) the
// accent with its symbol, since it is what to do next; the current step is the selected row. A
// narrow rail shows a `mark` instead: a check when done, the action's symbol in the accent. 44
// tall with one line, its parts centred down it; 56 with a subtitle's first line and a line taller
// for each line more. Its corners are concentric inside a list's well.
K.Pressable {
    id: root

    property string title
    property string subtitle
    property string icon: ""
    // `todo`, `current`, `done` or `action`.
    property string status: "todo"
    // The action's label and symbol, while the step waits on one.
    property string actionLabel: ""
    property string actionIcon: ""
    // Other words on the state's chip, in the state's look ("Waiting", "Not here"), and their
    // symbol.
    property string words: ""
    property string wordsIcon: ""
    // The state as a mark at the end in place of a chip.
    property bool mark: false
    // The row the keyboard is on, or the one picked.
    property bool selected: false

    readonly property bool fillWidth: true
    readonly property bool lit: root.selected || root.status === "current"
    readonly property color main: root.lit ? Tokens.color.on_secondary_container : Tokens.color.on_surface
    readonly property color muted: root.lit ? K.Look.alpha(Tokens.color.on_secondary_container, Tokens.opacity.subdued_text) : Tokens.color.on_surface_variant
    // One line is a list row's 44, its parts centred down it, so a chip moves nothing; a subtitle
    // takes 8 above and below.
    readonly property real padY: root.subtitle === "" ? 0 : Tokens.space.s8
    readonly property var probe: ({
            role: "component",
            name: "Step",
            variant: [root.status, root.icon === "" ? "" : "icon", root.mark ? "mark" : ""].filter(word => word !== "").join(" "),
            fixed: {
                width: false,
                height: false
            }
        })

    implicitHeight: root.subtitle === "" ? Tokens.size.row : line.implicitHeight + root.padY * 2
    activeFocusOnTab: false

    function relayout(): void {
        line.relayout();
    }

    Rectangle {
        anchors.fill: parent
        radius: K.Look.nested(Tokens.radius.surface, Tokens.space.s12)
        color: root.lit ? Tokens.color.secondary_container : "transparent"

        K.StateLayer {
            radius: parent.radius
            content: Tokens.color.on_surface
            active: root.enabled
            hovered: root.hovered
            pressed: root.down
        }
    }

    K.Row {
        id: line

        x: Tokens.space.s12
        y: root.padY
        width: root.width - Tokens.space.s12 * 2
        fixedHeight: root.subtitle === "" ? Tokens.size.row : 0
        fill: true
        gap: Tokens.space.s12
        align: "centre"

        K.Icon {
            visible: root.icon !== ""
            name: root.icon
            // Beside a title over a subtitle, the icon spans both lines.
            size: root.subtitle === "" ? Tokens.type.body_strong.icon : Tokens.icon.two_line
            color: root.main
        }

        K.Stack {
            fill: true
            gap: Tokens.space.s4

            K.Text {
                text: root.title
                style: Tokens.type.body_strong
                maximumLines: 1
                color: root.main
            }

            K.Text {
                visible: root.subtitle !== ""
                text: root.subtitle
                style: Tokens.type.small
                color: root.muted
            }
        }

        K.Chip {
            visible: !root.mark && root.status !== "current"
            // On a selected row done is plain with its check: the accent's tint over the row's
            // secondary container leaves its label under 4.5:1.
            kind: root.status === "done" && !root.lit ? "Accent" : root.status === "action" ? "Filled" : "Neutral"
            icon: root.words !== "" ? root.wordsIcon : root.status === "done" ? "check" : root.status === "action" ? root.actionIcon : ""
            text: root.words !== "" ? root.words : root.status === "done" ? "Done" : root.status === "action" ? root.actionLabel : "To do"
        }

        Item {
            visible: root.mark
            implicitWidth: Tokens.type.body_strong.icon
            implicitHeight: Tokens.type.body_strong.icon

            K.Icon {
                visible: root.status === "done" || root.status === "action"
                name: root.status === "done" ? "check" : root.actionIcon
                size: Tokens.type.body_strong.icon
                color: root.status === "action" ? Tokens.color.primary : root.main
            }
        }
    }
}
