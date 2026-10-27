import QtQuick
import KubaOS.Components as K

// A small choice you click: a timer preset, a country, a weekday. A chip, `Selected` when picked
// and `Neutral` otherwise, that takes a click; its `tooltip` names it in a hint while the pointer
// is over it. Like the chips it replaces, it is no stop for Tab: the field or the row beside it
// is.
K.Pressable {
    id: root

    property string label
    property bool selected: false
    property string tooltip

    signal clicked

    // Above its neighbours while its hint shows, so the chips after it don't paint over the hint.
    z: root.pointerOver ? 1 : 0
    implicitWidth: chip.implicitWidth
    implicitHeight: chip.implicitHeight
    activeFocusOnTab: false
    onActivated: root.clicked()

    K.Chip {
        id: chip

        text: root.label
        kind: root.selected ? "Selected" : "Neutral"
    }

    K.StateLayer {
        radius: K.Tokens.radius.control
        content: root.selected ? K.Tokens.color.on_secondary_container : K.Tokens.color.on_surface
        hovered: root.hovered
        pressed: root.down
    }

    K.Hint {
        text: root.tooltip
        shown: root.tooltip !== "" && root.pointerOver
    }
}
