import QtQuick
import qs.services

// The power stops as a slider, Super Saver to Max Performance, the stop's
// name in the track and its symbol in the fill: the control center's and the
// system monitor's.
StyledSlider {
    stops: Power.stops.length
    index: Power.index
    labels: Power.titles
    icon: Power.icon
    onStepped: index => Power.set(index)
}
