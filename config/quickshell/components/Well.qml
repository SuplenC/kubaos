import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// A list whose rows scroll inside its well: `kubaos-ui`'s `list::scrolling_list`. The well of a
// `List`, as tall as its parent makes it, holding a `ListRows` (or a `LiveList`), which lays out
// and scrolls the rows a well alone can't, and whatever else the well holds beside it (a toolbar,
// a form in the list's place). The focus tube lights the well's edge while `focused`.
//
// What it holds fills the well across its rows' room, 12 in at the sides, and down its whole
// height: rows scroll right to the well's edge and are cut by its own rounded shape, not by a
// rectangle 12 in. The 12 above the first row and below the last is the view's scroll padding
// (`inset`, a ListView's `topMargin` and `bottomMargin`), so at rest they sit where `List` puts
// them.
Item {
    id: root

    default property alias view: holder.data
    // Whether the list has the keyboard.
    property bool focused: false
    // The well's padding: the view's sides, and its scroll padding above and below.
    readonly property int inset: K.Tokens.space.s12

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property bool fillWidth: true

    Rectangle {
        anchors.fill: parent
        radius: K.Tokens.radius.surface
        color: K.Look.alpha(K.Tokens.color.surface_container_low, K.Tokens.opacity.well)
    }

    Rectangle {
        id: shape

        anchors.fill: parent
        radius: K.Tokens.radius.surface
        visible: false
        layer.enabled: true
    }

    // The rows drawn through the well's shape. A layer, not a hidden source, so they still take
    // the pointer.
    Item {
        anchors.fill: parent
        layer.enabled: true
        layer.effect: MultiEffect {
            maskEnabled: true
            maskSource: shape
            maskThresholdMin: 0.5
            maskSpreadAtMin: 1
        }

        Item {
            id: holder

            x: root.inset
            width: parent.width - root.inset * 2
            height: parent.height
        }
    }

    // The hairline over the rows, so a row scrolled to the edge never covers it.
    Rectangle {
        anchors.fill: parent
        radius: K.Tokens.radius.surface
        color: "transparent"
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Look.alpha(K.Tokens.color.outline_variant, K.Tokens.opacity.outline_soft)
    }

    K.FocusRing {
        shown: root.focused
        controlRadius: K.Tokens.radius.surface
    }
}
