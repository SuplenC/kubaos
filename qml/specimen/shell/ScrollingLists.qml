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
            model: ["Speakers", "Headphones", "HDMI 1 (LG UltraFine)", "USB audio", "Bluetooth (WH-1000XM4)"]

            delegate: K.ListRow {
                required property string modelData
                required property int index

                width: ListView.view.width
                icon: index === 0 ? "speaker" : "speaker_group"
                title: modelData
                subtitle: index === 0 ? "Default output" : ""
                checkable: true
                checked: index === 0
                selected: ListView.isCurrentItem && ListView.view.highlightShown
            }
        }
    }
}
