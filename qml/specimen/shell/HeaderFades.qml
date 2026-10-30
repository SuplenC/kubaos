import QtQuick
import KubaOS.Components as K

// The "Shell only" page's header fades: a panel's header over its scroll, at its top and scrolled.
K.Section {
    title: "HeaderFade"
    description: "The gap under a panel's header, laid over the scroll beneath it: at its top nothing shows; scrolled, the gap stays clear and the rows fade into the surface under it, stopping short of the thumb."

    K.Row {
        gap: K.Tokens.space.s32

        Panel {
            scrolledBy: 0
        }

        Panel {
            scrolledBy: K.Tokens.size.row / 2
        }
    }

    // A panel's header, and the rows that scroll under it `scrolledBy` from their top.
    component Panel: Item {
        id: panel

        property real scrolledBy: 0

        implicitWidth: K.Tokens.size.panel
        implicitHeight: K.Tokens.size.log

        K.Text {
            id: title

            text: "Processes"
            style: K.Tokens.type.heading
        }

        K.Scroll {
            id: scroll

            y: title.height + K.Tokens.space.s16
            width: panel.width
            height: panel.height - y
            roomAbove: K.Tokens.space.s16
            roomBelow: 0
            contentY: panel.scrolledBy

            K.Stack {
                width: scroll.width
                fill: true
                gap: K.Tokens.space.s4

                Repeater {
                    model: ["firefox", "Hyprland", "quickshell", "ghostty", "nvim", "rust-analyzer"]

                    K.ListRow {
                        required property string modelData

                        title: modelData
                        icon: "memory"
                        detail: "2.1 GB"
                    }
                }
            }
        }

        K.HeaderFade {
            x: scroll.x
            y: title.height
            width: scroll.width
            scrolled: panel.scrolledBy > 0
        }
    }
}
