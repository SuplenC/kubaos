import QtQuick
import qs.theme

// ListView driven from the keyboard: arrows, j/k, Ctrl+j/k, Ctrl+n/p, Enter.
// Plain j/k are skipped when `vimKeys` is false (e.g. a text field forwards its keys here).
// The selection is a highlight that slides between rows. playEnter() makes
// the rows cascade in (ListRow reads enterSerial).
ListView {
    id: root

    property bool vimKeys: true
    property bool highlightShown: true
    property int enterSerial: 0

    signal activated(int index)

    function playEnter(): void {
        enterSerial++;
    }

    clip: true
    activeFocusOnTab: true
    boundsBehavior: Flickable.StopAtBounds
    highlightFollowsCurrentItem: false
    spacing: Theme.spacing.xxs

    highlight: Rectangle {
        width: root.width
        height: root.currentItem?.height ?? 0
        y: root.currentItem?.y ?? 0
        radius: Theme.rounding.md
        color: Theme.m3secondaryContainer
        opacity: root.highlightShown && root.count > 0 ? 1 : 0

        Behavior on y {
            Anim {
                duration: Theme.anim.small
                easing.bezierCurve: Theme.anim.emphasizedDecel
            }
        }

        Behavior on opacity {
            Anim {
                duration: Theme.anim.fast
            }
        }
    }

    // HAZARD: no `add` transition on opacity; rows fade themselves in (ListRow, on `ListView.onAdd`).
    // A model change that reaches a row mid-fade cancels the view's transition and leaves the
    // opacity where the fade was. While a Wi-Fi scan re-sorts the list that strands rows
    // near-invisible, as gaps.
    // https://doc.qt.io/qt-6/qml-qtquick-viewtransition.html#handling-interrupted-animations
    displaced: Transition {
        Anim {
            property: "y"
            duration: Theme.anim.small
            easing.bezierCurve: Theme.anim.emphasizedDecel
        }
    }

    function step(delta: int): void {
        if (count > 0)
            currentIndex = (currentIndex + delta + count) % count;
    }

    // Returns true when the key was handled, so callers can set event.accepted.
    function handleKey(event: KeyEvent): bool {
        const ctrl = event.modifiers & Qt.ControlModifier;
        const k = event.key;
        if (k === Qt.Key_Down || (ctrl && (k === Qt.Key_J || k === Qt.Key_N)) || (vimKeys && !ctrl && k === Qt.Key_J))
            step(1);
        else if (k === Qt.Key_Up || (ctrl && (k === Qt.Key_K || k === Qt.Key_P)) || (vimKeys && !ctrl && k === Qt.Key_K))
            step(-1);
        else if ((k === Qt.Key_Return || k === Qt.Key_Enter) && currentIndex >= 0 && count > 0)
            activated(currentIndex);
        else
            return false;
        return true;
    }

    Keys.onPressed: event => event.accepted = handleKey(event)
}
