import QtQuick
import KubaOS.Components as K
import "samples.js" as Samples

// The components page's "Media" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/media.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Media"
    description: "Pictures in a box of a fixed aspect, reserved before they load, so nothing moves when they arrive: the placeholder, the picture, then two pictures to pick, picked with its name under it and focused with its name over its foot."

    K.Grid {
        columns: 4
        gap: K.Tokens.space.s16

        K.Media {
        }

        K.Media {
            source: Samples.picture(K.Tokens.color.primary, K.Tokens.color.cold)
        }

        K.PictureChoice {
            source: Samples.picture(K.Tokens.color.primary, K.Tokens.color.cold)
            name: "Shinjuku, night"
            picked: true
            markIcon: "check"
            markText: "Current"
        }

        K.PictureChoice {
            source: Samples.picture(K.Tokens.color.primary, K.Tokens.color.cold)
            name: "Golden Gai"
            namePlace: "over"
            badgeIcon: "play_arrow"
            badgeText: "Video"
            focused: true
        }
    }
}
