import QtQuick
import KubaOS.Components as K
import "samples.js" as Samples

// The components page's "Icon tiles and badges" section, drawn with `KubaOS.Components`: the section
// `crates/kubaos-specimen/src/pages/components/icon_tiles.rs` draws with `kubaos-ui`, box for box.
K.Section {
    title: "Icon tiles and badges"
    description: "Every row's icon on one tile, a symbol until a picture loads, so rows read the same size: small, regular and large, an app's icon, a photo, a critical one. Then badges, a count or a dot over an icon's corner."

    K.Row {
        gap: K.Tokens.space.s24
        align: "centre"

        K.IconTile {
            icon: "terminal"
            size: "Sm"
        }

        K.IconTile {
            icon: "terminal"
        }

        K.IconTile {
            icon: "terminal"
            size: "Lg"
        }

        K.IconTile {
            photo: Samples.picture(K.Tokens.color.primary, K.Tokens.color.cold)
        }

        K.IconTile {
            icon: "warning"
            danger: true
        }

        K.Badge {
            count: 3
        }

        K.Badge {
            count: 128
        }

        K.Badge {
        }

        Badged {
            icon: "notifications"
            count: 3
        }

        Badged {
            icon: "system_update_alt"
        }
    }

    // A 24 symbol in a small control's box, a badge over its top right corner, as the iced page
    // stacks them.
    component Badged: Item {
        id: badged

        property string icon
        property int count: 0
        readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

        implicitWidth: K.Tokens.size.control_sm
        implicitHeight: K.Tokens.size.control_sm

        K.Icon {
            x: K.Look.snap((badged.width - width) / 2, badged.dpr)
            y: K.Look.snap((badged.height - height) / 2, badged.dpr)
            name: badged.icon
            size: K.Tokens.icon.s24
            color: K.Tokens.color.on_surface_variant
        }

        K.Badge {
            x: badged.width - width
            count: badged.count
        }
    }
}
