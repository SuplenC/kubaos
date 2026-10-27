import QtQuick
import KubaOS.Components as K

// The pane right of the list: the selected row, in full, on a card. A row the selection moved to
// crossfades in over the last, one the Results brought swaps at once, and the same row changing in
// place just updates.
K.Card {
    id: pane

    // Which of the two contents shows the entry now.
    property bool flipped: false
    readonly property PaneContent shown: flipped ? second : first
    // Whether the last flip crossfades. A fade still running when the next key comes would make
    // that frame wait on the screen's refresh.
    property bool fading: false

    // Shows `entry`, an object as `Menu.qml`'s `paneEntry` builds it, or null for no row,
    // crossfading if `fade`.
    function show(entry: var, fade: bool): void {
        if ((entry?.key ?? "") === (shown.entry?.key ?? "")) {
            shown.entry = entry;
            return;
        }
        (flipped ? first : second).entry = entry;
        fading = fade;
        flipped = !flipped;
    }

    // The card's room for content, as tall as the card less its padding; both contents fill it
    // and crossfade in place.
    Item {
        id: room

        readonly property bool fillWidth: true

        implicitHeight: pane.height - K.Tokens.space.s16 * 2

        PaneContent {
            id: first

            anchors.fill: parent
            opacity: pane.flipped ? 0 : 1
            visible: opacity > 0 && entry !== null

            Behavior on opacity {
                enabled: pane.fading

                NumberAnimation {
                    duration: K.Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.standard
                }
            }
        }

        PaneContent {
            id: second

            anchors.fill: parent
            opacity: pane.flipped ? 1 : 0
            visible: opacity > 0 && entry !== null

            Behavior on opacity {
                enabled: pane.fading

                NumberAnimation {
                    duration: K.Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.standard
                }
            }
        }

        // Laid out once, while the menu is built, and shown or hidden by its opacity, so showing
        // it never waits on a layout pass.
        K.EmptyState {
            y: K.Look.snap((room.height - implicitHeight) / 2, pane.dpr)
            width: room.width
            opacity: pane.shown.entry === null ? 1 : 0
            icon: "manage_search"
            title: "Nothing selected"
            body: "Pick a row and it shows up here."

            Behavior on opacity {
                enabled: pane.fading

                NumberAnimation {
                    duration: K.Tokens.duration.short
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: K.Tokens.easing.standard
                }
            }
        }
    }
}
