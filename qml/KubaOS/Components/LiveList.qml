import QtQuick
import KubaOS.Components as K

// A list a scan updates while it shows, which never jumps (docs/DESIGN.md, "Motion"): `ListRows`
// whose rows come from `rows`, each `{ key, kind, folded, ... }` with the same roles, kept in step
// by key (`Keyed`), so only what changed changes. Its delegates are `LiveRow`s: a row that arrives
// opens its room and fades in, one that leaves fades and gives its room back, taking no press
// while it goes, and one that changes place glides there. The keyboard skips the rows whose
// `kind` is neither "row" nor "disclosure" (section labels, a note), and the drawer while it is
// closed.
//
// The `folded` rows, which come last, are a drawer under the "disclosure" line before them,
// open while `unfolded`. Opening, its bottom edge slides down from the line over `medium`,
// uncovering rows that stay where they are, and the rows fade in together, a `micro` after the
// edge starts; closing, the edge slides back up to the line as the rows fade out together. With
// reduced motion it opens and closes at once. The rows stay in the model either way, so the
// drawer is no stream of arrivals and leaves, and nothing above its edge moves.
//
// It scrolls only when the person scrolls it, or moves the selection with the keys: a scan moves
// the current row as it re-sorts, and a view that followed it would carry the Known rows out of
// sight.
//
// Each row carries the 4 after it, so a leaving row gives back its gap with its room, and the
// list's bottom margin is the well's 12 less that 4.
K.ListRows {
    id: root

    property var rows: []
    // Whether changes animate: only while the list shows, so an update nobody sees draws nothing.
    readonly property bool animated: root.visible
    // Whether the drawer is open.
    property bool unfolded: true
    // Where the drawer's edge is, from 0 at its line to 1 under its last row.
    property real drawerEdge: 1
    // How much the drawer's rows show, which runs a little behind the edge as it opens.
    property real drawerShown: 1
    // The drawer's first row and how many it holds, which place its edge on its rows.
    readonly property int drawerStart: root.rows.findIndex(row => row.folded === true)
    readonly property int drawerCount: root.rows.reduce((count, row) => count + (row.folded === true ? 1 : 0), 0)
    // Room after the list's end, where the person had scrolled into a drawer that then closed: the
    // view would otherwise be pulled down as the drawer shrinks, its line with it. It goes as they
    // scroll back up.
    property real slack: 0

    // How far the view reaches past the list's end, its bottom margin aside.
    function overshoot(): real {
        return root.contentY + root.height - root.originY - root.contentHeight - K.Tokens.space.s8;
    }

    // The slack a drawer closing from here needs: as far as the view reaches past where the list
    // ends once the drawer's rows have given back their room. A drawer with no row made is below
    // the view, and needs none.
    function slackToClose(): real {
        const row = root.contentItem.children.find(item => item.folds === true && item.uncovered === 1);
        const drawer = row ? root.drawerEdge * root.drawerCount * row.height : 0;
        return Math.max(0, root.overshoot() + drawer);
    }

    function openDrawer(): void {
        closing.stop();
        root.slack = 0;
        if (root.animated && !K.Motion.reduced) {
            opening.restart();
        } else {
            opening.stop();
            root.drawerEdge = 1;
            root.drawerShown = 1;
        }
    }

    function closeDrawer(): void {
        opening.stop();
        if (root.animated && !K.Motion.reduced) {
            root.slack = root.slackToClose();
            closing.restart();
        } else {
            closing.stop();
            root.drawerEdge = 0;
            root.drawerShown = 0;
        }
    }

    function folded(index: int): bool {
        return rowModel.get(index).folded === true && !root.unfolded;
    }

    spacing: 0
    highlightFollowsCurrentItem: false
    topMargin: K.Tokens.space.s12
    bottomMargin: K.Tokens.space.s8 + root.slack
    selectable: index => index >= 0 && index < rowModel.count && ["row", "disclosure"].includes(rowModel.get(index).kind) && !root.folded(index)
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

    // The drawer's moves are started on `unfolded`, not run as Behaviors: a Behavior's curve and
    // delay bound to `unfolded` may still hold the last move's when it starts.
    ParallelAnimation {
        id: opening

        NumberAnimation {
            target: root
            property: "drawerEdge"
            to: 1
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_decel
        }

        SequentialAnimation {
            // The rows wait for the edge to start uncovering them.
            PauseAnimation {
                duration: K.Tokens.duration.micro
            }

            NumberAnimation {
                target: root
                property: "drawerShown"
                to: 1
                duration: K.Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: K.Tokens.easing.standard
            }
        }
    }

    ParallelAnimation {
        id: closing

        NumberAnimation {
            target: root
            property: "drawerEdge"
            to: 0
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_accel
        }

        NumberAnimation {
            target: root
            property: "drawerShown"
            to: 0
            duration: K.Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_accel
        }
    }

    onUnfoldedChanged: {
        if (root.unfolded)
            root.openDrawer();
        else
            root.closeDrawer();
        // The selection leaves a drawer that closes, for its line.
        if (!root.selectable(root.currentIndex))
            root.jump(root.currentIndex);
    }
    onContentYChanged: root.slack = Math.max(0, Math.min(root.slack, root.overshoot()))

    onRowsChanged: {
        K.Keyed.apply(rowModel, root.rows);
        if (!root.selectable(root.currentIndex))
            root.jump(root.currentIndex);
    }
    Component.onCompleted: {
        root.drawerEdge = root.unfolded ? 1 : 0;
        root.drawerShown = root.drawerEdge;
        K.Keyed.apply(rowModel, root.rows);
    }
}
