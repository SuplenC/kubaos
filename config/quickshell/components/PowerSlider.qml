import QtQuick
import KubaOS.Components as K
import qs.services

// The power stops as a slider, Super Saver to Max Performance: the stop's symbol in the icon's
// circle and its name at the end. Its icon steps from high to low and back. The control center's
// and the system monitor's.
K.Slider {
    icon: Power.icon
    stops: Power.stops.length
    index: Power.index
    names: Power.titles
    iconLabel: qsTr("Next power stop")
    Accessible.name: qsTr("Power")
    onStepped: index => Power.set(index)
}
