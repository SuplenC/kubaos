import QtQuick
import KubaOS.Components as K

// Key hints side by side, 16 apart: each combination's keys as `Keycap`s 4 apart, then what it
// does in a `caption` line 8 after them. `kubaos-ui`'s `keycap::key_hints`.
//
// The hints are `hints`, a list of `[combination, what]` pairs, or `text` in the form Kotodama
// and the menu write them, `↵ Open · Tab Enter app · ⌃K Actions`: each part's first word the
// combination, the rest what it does. A combination is written `Ctrl+K`, a `+` alone the plus key.
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

    // The keys of `combination`.
    function keys(combination: string): var {
        return combination === "+" ? ["+"] : combination.split("+").filter(key => key !== "");
    }

    implicitWidth: line.implicitWidth
    implicitHeight: line.implicitHeight

    K.Row {
        id: line

        gap: Tokens.space.s16
        align: "centre"

        Repeater {
            model: root.shown

            K.Row {
                id: hint

                required property var modelData

                gap: Tokens.space.s8
                align: "centre"

                K.Row {
                    gap: Tokens.space.s4
                    align: "centre"

                    Repeater {
                        model: root.keys(hint.modelData[0])

                        K.Keycap {
                            required property string modelData

                            key: modelData
                        }
                    }
                }

                K.Text {
                    visible: hint.modelData[1] !== ""
                    text: hint.modelData[1]
                    style: Tokens.type.caption
                    color: root.color
                }
            }
        }
    }
}
