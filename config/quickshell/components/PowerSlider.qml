import QtQuick
import KubaOS.Components as K
import qs.services

// The power stops as a slider, Super Saver to Max Performance: the stop's symbol before it and its
// name after, in a box as wide as the widest name, so the track keeps one length whichever stop
// it is on. The control center's and the system monitor's.
K.Row {
    id: root

    readonly property real widest: {
        names.count;
        let widest = 0;
        for (let i = 0; i < names.count; i++)
            widest = Math.max(widest, names.itemAt(i)?.implicitWidth ?? 0);
        return widest;
    }

    gap: K.Tokens.space.s12
    align: "centre"

    K.Icon {
        name: Power.icon
        size: K.Tokens.icon.s20
        color: K.Tokens.color.on_surface_variant
    }

    K.Slider {
        stops: Power.stops.length
        index: Power.index
        Accessible.name: qsTr("Power")
        onStepped: index => Power.set(index)
    }

    Item {
        implicitWidth: root.widest
        implicitHeight: shown.implicitHeight

        K.Text {
            id: shown

            text: Power.titles[Power.index] ?? ""
            style: K.Tokens.type.label
            color: K.Tokens.color.on_surface_variant
        }
    }

    // Every stop's name, measured and never drawn.
    Item {
        visible: false

        Repeater {
            id: names

            model: Power.titles

            K.Text {
                required property string modelData

                text: modelData
                style: K.Tokens.type.label
            }
        }
    }
}
