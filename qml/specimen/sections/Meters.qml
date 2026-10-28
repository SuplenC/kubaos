import QtQuick
import KubaOS.Components as K

// The components page's "Meters and sparklines" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/meters.rs` draws with `kubaos-ui`, box for box.
K.Section {
    id: root

    // A minute of a reading, a sample every two seconds, as [t, value]: the wave the iced page
    // draws.
    function wave(level: real, swing: real, phase: real): var {
        const points = [];
        for (let i = 0; i < 30; i++)
            points.push([i * 2000, level + swing * Math.sin(i * 0.4 + phase) + swing * 0.4 * Math.sin(i * 1.3 + phase)]);
        return points;
    }

    title: "Meters and sparklines"
    description: "Readings at a glance: a reading in parts on a progress bar's track, and the last minute of one as a line."

    K.Row {
        gap: K.Tokens.space.s48

        K.Stack {
            fill: true
            gap: K.Tokens.space.s16

            K.Meter {
                parts: [
                    {
                        value: 0.55,
                        color: K.Tokens.color.primary
                    },
                    {
                        value: 0.15,
                        color: K.Tokens.color.tertiary
                    }
                ]
            }

            K.Meter {
                parts: [
                    {
                        value: 0.92,
                        color: K.Tokens.color.warning
                    }
                ]
            }

            K.Meter {
                parts: [
                    {
                        value: 0.3,
                        color: K.Tokens.color.secondary
                    }
                ]
            }
        }

        K.Stack {
            fill: true
            gap: K.Tokens.space.s16

            // The waves' 30 samples, the first at the left edge and the last at
            // the right, as the iced page spaces them.
            K.Sparkline {
                points: root.wave(40, 20, 0)
                span: 58000
            }

            K.Sparkline {
                points: root.wave(1200, 600, 1)
                second: root.wave(300, 150, 2)
                span: 58000
                max: 0
            }
        }
    }
}
