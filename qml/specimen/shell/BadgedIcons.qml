import QtQuick
import KubaOS.Components as K

// The "Shell only" page's badged icons.
K.Section {
    title: "BadgedIcon"
    description: "A bar's symbol with a badge centred on its top right corner: none, a dot, a count, and a count past 99. While a badge shows, the box takes its half past the symbol on both sides."

    K.Row {
        gap: K.Tokens.space.s24
        align: "centre"

        K.BadgedIcon {
            name: "notifications"
        }

        K.BadgedIcon {
            name: "notifications"
            badged: true
        }

        K.BadgedIcon {
            name: "notifications"
            badged: true
            count: 3
        }

        K.BadgedIcon {
            name: "notifications"
            badged: true
            count: 128
        }
    }
}
