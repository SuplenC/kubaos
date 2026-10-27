import QtQuick
import KubaOS.Components as K
import "words.js" as Words

// Key hints in Kotodama's form, `↵ Open · Tab Enter app · ⌃K Actions`, as `K.KeyHints` draws
// them: each key a keycap 4 from the next, what it does in `caption` 8 after them, hints 16 apart.
// A hint that would pass `room` is left out whole, so none is cut through a keycap.
//
// HAZARD: no `K.Text` and no layout primitives in here, and the delegates go by count: a keystroke
// that moves the selection changes the footer's and the pane's hints, and the menu must answer on
// the next frame. Measured headless at 1x, 70 keystrokes over 14 rows, keystroke to frame: MEASURE.
Item {
    id: root

    property string text
    property color color: K.Tokens.color.on_surface_variant
    // The widest the hints may take; hints past it are left out.
    property real room: Infinity

    readonly property var shown: Words.hints(root.text)
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    implicitWidth: {
        let end = 0;
        for (let i = 0; i < hints.count; i++) {
            const hint = hints.itemAt(i);
            if (hint && hint.visible)
                end = hint.x + hint.width;
        }
        return end;
    }
    implicitHeight: K.Tokens.size.keycap

    Repeater {
        id: hints

        model: root.shown.length

        Item {
            id: hint

            required property int index
            readonly property var pair: root.shown[hint.index] ?? [[], ""]
            readonly property Item before: hint.index > 0 ? hints.itemAt(hint.index - 1) : null

            x: hint.before ? hint.before.x + hint.before.width + K.Tokens.space.s16 : 0
            width: caps.width + (what.text === "" ? 0 : K.Tokens.space.s8 + what.width)
            height: K.Tokens.size.keycap
            // Left out whole past the room, and so is every hint after it.
            visible: (!hint.before || hint.before.visible) && hint.x + hint.width <= root.room

            Row {
                id: caps

                spacing: K.Tokens.space.s4

                Repeater {
                    model: hint.pair[0].length

                    Keycap {
                        required property int index

                        key: hint.pair[0][index] ?? ""
                    }
                }
            }

            Label {
                id: what

                x: caps.width + K.Tokens.space.s8
                y: K.Look.snap((K.Tokens.size.keycap - K.Tokens.type.caption.line) / 2, root.dpr)
                width: K.Look.snapUp(implicitWidth, root.dpr)
                height: K.Tokens.type.caption.line
                elide: Text.ElideNone
                type: K.Tokens.type.caption
                text: hint.pair[1]
                color: root.color
            }
        }
    }
}
