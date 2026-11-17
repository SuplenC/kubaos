import QtQuick
import KubaOS.Components as K

// The "Shell only" page's scrolling lists: `ListRows` in a `ScrollingList`'s well.
K.Section {
    title: "ScrollingList and ListRows"
    description: "Rows that scroll inside their well, cut by its own rounded shape: at rest, and with the keyboard on the list, its edge lit and the current row selected."

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s24

        Well {
        }

        Well {
            focused: true
            current: 1
        }
    }

    // A well of outputs, `current` selected while `focused`.
    component Well: K.ScrollingList {
        id: well

        property int current: -1

        implicitHeight: K.Tokens.size.log

        K.ListRows {
            anchors.fill: parent
            topMargin: well.inset
            bottomMargin: well.inset
            currentIndex: well.current
            highlightShown: well.focused
            model: [
                { title: "Speakers", connection: "Built-in", icon: "speaker" },
                { title: "Headphones", connection: "Built-in", icon: "speaker" },
                { title: "LG UltraFine", connection: "HDMI", icon: "tv" },
                { title: "USB audio", connection: "USB", icon: "usb" },
                { title: "WH-1000XM4", connection: "Bluetooth", icon: "bluetooth" }
            ]

            delegate: K.ListRow {
                required property var modelData
                required property int index

                width: ListView.view.width
                icon: modelData.icon
                title: modelData.title
                subtitle: index === 0 ? `${modelData.connection} · Default output` : modelData.connection
                checkable: true
                checked: index === 0
                selected: ListView.isCurrentItem && ListView.view.highlightShown
            }
        }
    }
}
