import QtQuick
import KubaOS.Components as K

// Key hints side by side, 16 apart: each combination's keys as `Keycap`s 4 apart, then what it
// does in a `caption` line 8 after them. `kubaos-ui`'s `keycap::key_hints`.
//
// The hints are `hints`, a list of `[combination, what]` pairs, or `text` in the form Kotodama
// and the menu write them, `↵ Open · Tab Enter app · ⌃K Actions`: each part's first word the
// combination, the rest what it does. A combination is written `Ctrl+K`; each of its keys is a
// keycap of its own, and so is each modifier or arrow glyph written together (`⌃K`, `↑↓`).
Item {
    id: root

    // `[combination, what]` pairs; taken from `text` when that is set.
    property var hints: []
    // The hints in Kotodama's form; empty for `hints`.
    property string text: ""
    property color color: Tokens.color.on_surface_variant

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
        for (const part of combination.split(/[+\s]/)) {
            let word = "";
            for (const c of part) {
                if (glyphs.includes(c)) {
                    if (word !== "")
                        found.push(word);
                    found.push(c);
                    word = "";
                } else {
                    word += c;
                }
            }
            if (word !== "")
                found.push(word);
        }
        return found;
    }

    implicitWidth: line.implicitWidth
    implicitHeight: line.implicitHeight

    K.Row {
        id: line

        gap: Tokens.space.s16
        align: "centre"

        // By count, not by the pairs themselves: a new text with as many hints keeps the same
        // delegates and only rebinds their words, rather than making them all again.
        Repeater {
            model: root.shown.length

            K.Row {
                id: hint

                required property int index
                readonly property var pair: root.shown[hint.index] ?? ["", ""]
                readonly property var caps: root.keys(hint.pair[0])

                gap: Tokens.space.s8
                align: "centre"

                K.Row {
                    gap: Tokens.space.s4
                    align: "centre"

                    Repeater {
                        model: hint.caps.length

                        K.Keycap {
                            required property int index

                            key: hint.caps[index] ?? ""
                        }
                    }
                }

                K.Text {
                    visible: hint.pair[1] !== ""
                    text: hint.pair[1]
                    style: Tokens.type.caption
                    color: root.color
                }
            }
        }
    }
}
