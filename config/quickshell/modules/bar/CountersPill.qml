import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// The counters in the bar: a pill with the tally marks alone, while there is a counter. A click
// opens the counters' sheet, which grows down from the bar's corner, or closes it.
Pill {
    visible: Counters.list.length > 0
    interactive: true
    padding: K.Tokens.space.s8
    tooltip: "Counters"
    onClicked: Panels.toggle("counters")

    K.Icon {
        name: "tallymarks"
        size: K.Tokens.icon.s16
        color: K.Tokens.color.on_surface_variant
    }
}
