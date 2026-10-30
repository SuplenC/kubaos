import QtQuick
import KubaOS.Components as K

// The "Shell only" page's figures.
K.Section {
    title: "Figure"
    description: "A number that ticks, in a box as wide as the same text with every digit a 0, so what sits beside it holds still: the clock's time, a timer, a stopwatch."

    K.Row {
        gap: K.Tokens.space.s32
        align: "end"

        K.Figure {
            text: "21:47"
            style: K.Tokens.type.hero_number
        }

        K.Figure {
            text: "4:59"
            style: K.Tokens.type.headline
        }

        K.Figure {
            text: "1:02:11.4"
            style: K.Tokens.type.body_strong
        }
    }
}
