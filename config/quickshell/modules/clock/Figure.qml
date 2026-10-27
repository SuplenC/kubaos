import QtQuick
import KubaOS.Components as K

// A number that ticks: a time, a timer, a stopwatch. Geist's figures are proportional, so its box
// is as wide as the same text with every digit a 0, the widest; the text starts at the box's left
// edge, and what sits beside or around the box holds still as the digits change.
Item {
    id: root

    property alias text: shown.text
    property alias style: shown.style
    property alias color: shown.color

    implicitWidth: widest.implicitWidth
    implicitHeight: shown.implicitHeight

    K.Text {
        id: shown
    }

    K.Text {
        id: widest

        visible: false
        text: shown.text.replace(/\d/g, "0")
        style: shown.style
    }
}
