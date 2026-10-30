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
    // Whether rows are drawn away from where their view lays them, as a dragged row is, so they
    // may reach the well's corners while the list fits.
    property bool adrift: false
    // Whether what the well holds reaches its rounded corners, and is drawn through its shape.
    // HAZARD: draw through the shape only then. The shape is a layer the well's whole content is
    // drawn into and masked: with the software renderer, a list that fits and redraws (a scan's
    // pulse) takes 11.2 CPU s over 8 s this way against 6.1 s drawn as it is.
    readonly property bool shaped: root.adrift || Array.from(holder.children).some(item => root.reaches(item))

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property bool fillWidth: true
    // How far in from the well's top and bottom its corners cut what it holds, at the content's
    // sides: where the corner's arc meets them, and a pixel of the mask's edge.
    readonly property real cornerDepth: K.Tokens.radius.surface - Math.sqrt(K.Tokens.radius.surface ** 2 - (K.Tokens.radius.surface - root.inset) ** 2) + 1

    // Whether `item` draws into the corners' depth at the top or bottom. A view draws its rows
    // where they are, moving ones included, cut to its own box when it clips.
    function reaches(item: Item): bool {
        if (!item.visible || item.opacity === 0)
            return false;
        let top = item.y;
        let bottom = item.y + item.height;
        if (item instanceof Flickable) {
            // HAZARD: the rows' own boxes, not the content's `childrenRect`, which works itself out
            // as it is read and loops this binding.
            let rowsTop = Infinity;
            let rowsBottom = -Infinity;
            for (const row of item.contentItem.children) {
                if (!row.visible)
                    continue;
                rowsTop = Math.min(rowsTop, row.y);
                rowsBottom = Math.max(rowsBottom, row.y + row.height);
            }
            const shift = item.y + item.contentItem.y;
            top = item.clip ? Math.max(top, shift + rowsTop) : shift + rowsTop;
            bottom = item.clip ? Math.min(bottom, shift + rowsBottom) : shift + rowsBottom;
        }
        return bottom > top && (top < root.cornerDepth || bottom > root.height - root.cornerDepth);
    }

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
        layer.enabled: root.shaped
    }

    // The rows drawn through the well's shape. A layer, not a hidden source, so they still take
    // the pointer.
    Item {
        anchors.fill: parent
        layer.enabled: root.shaped
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
