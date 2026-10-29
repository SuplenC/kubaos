import QtQuick
import KubaOS.Components as K
import "keyed.js" as Keyed

// A list a scan updates while it shows, which never jumps (docs/DESIGN.md, "Motion"): its rows
// come from `rows`, each `{ key, kind, ... }` with the same roles, kept in step by key
// (`keyed.js`), so only what changed changes. Its delegates are `LiveRow`s: a row that arrives
// opens its room and fades in, one that leaves fades and gives its room back, taking no press
// while it goes, and one that changes place glides there. The keyboard skips the rows whose
// `kind` is neither "row" nor "action" (section labels, a note), and the folded block while
// it is folded away.
//
// The rows marked `folded` are one block, which opens while `unfolded` and folds away while not:
// every one of them grows and fades in together, or fades and collapses together, over `medium`,
// at once with reduced motion. They stay in the model either way, so a fold is no stream of
// arrivals and leaves; the rows after the block follow its edge, and the rows before it don't
// move.
//
// It scrolls only when the person scrolls it, or moves the selection with the keys: a scan moves
// the current row as it re-sorts, and a view that followed it would carry the Known rows out of
// sight.
//
// Each row carries the 4 after it, so a leaving row gives back its gap with its room, and the
// list's bottom margin is the well's 12 less that 4.
NavList {
    id: root

    property var rows: []
    // Whether changes animate: only while the list shows, so an update nobody sees draws nothing.
    readonly property bool animated: root.visible
    // Whether the folded block shows.
    property bool unfolded: true
    // How far the folded block is open, from 0 to 1, which its rows take their room and their
    // opacity from.
    property real unfold: root.unfolded ? 1 : 0

    function folded(index: int): bool {
        return rowModel.get(index).folded === true && !root.unfolded;
    }

    spacing: 0
    highlightFollowsCurrentItem: false
    topMargin: K.Tokens.space.s12
    bottomMargin: K.Tokens.space.s8
    selectable: index => index >= 0 && index < rowModel.count && ["row", "action"].includes(rowModel.get(index).kind) && !root.folded(index)
    model: ListModel {
        id: rowModel
    }

    // Arriving and leaving rows animate their own room (`LiveRow`), which the rows around them
    // follow as the view lays out; a view transition on them would fight it.
    displaced: null
    move: Transition {
        NumberAnimation {
            property: "y"
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_decel
        }
    }
    // HAZARD: a move that reaches a row mid-fade stops the fade where it was; the moving rows
    // finish it, or a scan that re-sorts the list strands rows near-invisible, as gaps.
    // https://doc.qt.io/qt-6/qml-qtquick-viewtransition.html#handling-interrupted-animations
    moveDisplaced: Transition {
        NumberAnimation {
            property: "y"
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_decel
        }
        NumberAnimation {
            property: "shown"
            to: 1
            duration: K.Tokens.duration.medium
        }
    }

    Behavior on unfold {
        enabled: root.animated && !K.Motion.reduced

        NumberAnimation {
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: root.unfolded ? K.Tokens.easing.standard_decel : K.Tokens.easing.standard_accel
        }
    }

    // The selection leaves a block that folds away, for the line before it.
    onUnfoldedChanged: if (!root.selectable(root.currentIndex))
        root.jump(root.currentIndex)

    onRowsChanged: {
        Keyed.apply(rowModel, root.rows);
        if (!root.selectable(root.currentIndex))
            root.jump(root.currentIndex);
    }
    Component.onCompleted: Keyed.apply(rowModel, root.rows)
}
