import QtQuick
import KubaOS.Components as K
import "keyed.js" as Keyed

// A list a scan updates while it shows, which never jumps (docs/DESIGN.md, "Motion"): its rows
// come from `rows`, each `{ key, kind, ... }` with the same roles, kept in step by key
// (`keyed.js`), so only what changed changes. Its delegates are `LiveRow`s: a row that arrives
// opens its room and fades in, one that leaves fades and gives its room back, taking no press
// while it goes, and one that changes place glides there. The keyboard skips the rows whose
// `kind` is neither "row" nor "action" (section labels, a note).
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

    spacing: 0
    highlightFollowsCurrentItem: false
    topMargin: K.Tokens.space.s12
    bottomMargin: K.Tokens.space.s8
    selectable: index => index >= 0 && index < rowModel.count && ["row", "action"].includes(rowModel.get(index).kind)
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
            property: "opacity"
            to: 1
            duration: K.Tokens.duration.medium
        }
    }

    onRowsChanged: {
        Keyed.apply(rowModel, root.rows);
        if (!root.selectable(root.currentIndex))
            root.jump(root.currentIndex);
    }
    Component.onCompleted: Keyed.apply(rowModel, root.rows)
}
