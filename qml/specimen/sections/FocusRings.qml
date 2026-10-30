import QtQuick
import KubaOS.Components as K

// The components page's "Focus" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/focus_rings.rs` draws with `kubaos-ui`, box for
// box.
K.Section {
    title: "Focus"
    description: "The neon tube on the edge of whatever the keyboard is on: the accent, the cold one where the accent means picked, the error colour on a wrong value, and the hot core outside a fill."

    K.Row {
        gap: K.Tokens.space.s32
        align: "centre"

        Target {
            words: "A control"

            K.FocusRing {
                shown: true
            }
        }

        Target {
            words: "A picked card"
            fill: K.Tokens.color.secondary_container
            ink: K.Tokens.color.on_secondary_container
            corner: K.Tokens.radius.surface

            K.FocusRing {
                shown: true
                cold: true
                controlRadius: K.Tokens.radius.surface
            }
        }

        Target {
            words: "A wrong value"

            K.FocusRing {
                shown: true
                error: true
            }
        }

        Target {
            words: "On a fill"
            fill: K.Tokens.color.primary
            ink: K.Tokens.color.on_primary

            K.FocusRing {
                shown: true
                filled: true
            }
        }
    }

    // A box reading `words` in `ink` on `fill`, at `corner`: what the tube lights.
    component Target: Rectangle {
        id: target

        property string words
        property color fill: K.Tokens.color.surface_container_high
        property color ink: K.Tokens.color.on_surface
        property int corner: K.Tokens.radius.control

        readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

        implicitWidth: K.Look.snapUp(label.implicitWidth, target.dpr) + K.Tokens.space.s16 * 2
        implicitHeight: K.Look.snapUp(label.implicitHeight, target.dpr) + K.Tokens.space.s8 * 2
        radius: target.corner
        color: target.fill

        K.Text {
            id: label

            x: K.Tokens.space.s16
            y: K.Tokens.space.s8
            text: target.words
            color: target.ink
        }
    }
}
