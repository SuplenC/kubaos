import QtQuick
import QtQuick as Q
import KubaOS.Components as K

// Key hints side by side, 16 apart: each combination's keys as `Keycap`s 4 apart, then what it
// does in a `caption` line 8 after them. `kubaos-ui`'s `keycap::key_hints`.
//
// The hints are `hints`, a list of `[combination, what]` pairs, or `text` in the form Kotodama
// and the menu write them, `↵ Open · Tab Enter app · ⌃K Actions`: each part's first word the
// combination, the rest what it does. A combination is written `Ctrl+K`; each of its keys is a
// keycap of its own, and so is each modifier or arrow glyph written together (`⌃K`, `↑↓`);
// only `+` parts the rest, so `Left Alt` is one key. A hint that would pass `room` is left out
// whole, and so is every hint after it, so none is cut through a keycap.
//
// HAZARD: no layout primitives in here; each box is placed with token arithmetic, and the
// delegates go by count. A keystroke that moves the Command menu's selection changes its footer's
// and its detail pane's hints, and the menu must answer on the next frame.
Item {
    id: root

    // `[combination, what]` pairs; taken from `text` when that is set.
    property var hints: []
    // The hints in Kotodama's form; empty for `hints`.
    property string text: ""
    property color color: Tokens.color.on_surface_variant
    // The widest the hints may take; hints past it are left out.
    property real room: Infinity

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // The pairs shown, from `text` when there is one.
    readonly property var shown: root.text === "" ? root.hints : root.parse(root.text)
    readonly property var probe: ({
            role: "component",
            name: "KeyHints",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    // `text`'s pairs: parts split at ` · `, each one's first word its combination.
    function parse(text: string): var {
        return text.split(" · ").filter(part => part !== "").map(part => {
            const space = part.indexOf(" ");
            return space < 0 ? [part, ""] : [part.slice(0, space), part.slice(space + 1)];
        });
    }

    // The keys of `combination`, one keycap each, as `keycap::keys` splits them.
    function keys(combination: string): var {
        if (combination === "+")
            return ["+"];
        const glyphs = "⌃⇧⌥⌘↑↓←→↵⌫⌦";
        const found = [];
        for (const part of combination.split("+")) {
            let word = "";
            const push = text => {
                const trimmed = text.trim();
                if (trimmed !== "")
                    found.push(trimmed);
            };
            for (const c of part) {
                if (glyphs.includes(c)) {
                    push(word);
                    push(c);
                    word = "";
                } else {
                    word += c;
                }
            }
            push(word);
        }
        return found;
    }

    implicitWidth: {
        let end = 0;
        for (let i = 0; i < hints.count; i++) {
            const hint = hints.itemAt(i);
            if (hint && hint.visible)
                end = hint.x + hint.width;
        }
        return end;
    }
    implicitHeight: Tokens.size.keycap

    // By count, not by the pairs themselves: a new text with as many hints keeps the same
    // delegates and only rebinds their words, rather than making them all again.
    Repeater {
        id: hints

        model: root.shown.length

        Item {
            id: hint

            required property int index
            readonly property var pair: root.shown[hint.index] ?? ["", ""]
            readonly property var caps: root.keys(hint.pair[0])
            readonly property string does: hint.pair[1]
            readonly property Item before: hint.index > 0 ? hints.itemAt(hint.index - 1) : null

            x: hint.before ? hint.before.x + hint.before.width + Tokens.space.s16 : 0
            width: keys.width + (hint.does === "" ? 0 : Tokens.space.s8 + what.width)
            height: Tokens.size.keycap
            visible: (!hint.before || hint.before.visible) && hint.x + hint.width <= root.room

            Q.Row {
                id: keys

                spacing: Tokens.space.s4

                Repeater {
                    model: hint.caps.length

                    K.Keycap {
                        required property int index

                        key: hint.caps[index] ?? ""
                    }
                }
            }

            K.Text {
                id: what

                x: keys.width + Tokens.space.s8
                y: K.Look.snap((Tokens.size.keycap - Tokens.type.caption.line) / 2, root.dpr)
                visible: hint.does !== ""
                text: hint.does
                style: Tokens.type.caption
                color: root.color
            }
        }
    }
}
