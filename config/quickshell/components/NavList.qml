import QtQuick
import qs.theme

// ListView driven from the keyboard, with the keys of docs/DESIGN.md ("The keyboard"): the arrows
// along it, h j k l, Ctrl+j/k, Ctrl+n/p, Home and End, Page Up and Page Down, and Enter once per
// press. Plain letters are skipped when `vimKeys` is false, and Home and End when `fieldKeys` is
// true: a text field that forwards its keys here keeps both.
// The selection is a highlight that slides between rows. playEnter() makes
// the rows cascade in (ListRow reads enterSerial).
ListView {
    id: root

    // Whether plain h j k l move the selection.
    property bool vimKeys: true
    // Whether a text field forwards its keys here, which keeps Home and End for its caret.
    property bool fieldKeys: false
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

    // To row `index`, held to the list's ends: Home, End and a page never wrap round.
    function jump(index: int): void {
        if (count > 0)
            currentIndex = Math.max(0, Math.min(count - 1, index));
    }

    // How many rows a page is: as many as the view shows whole, measured on the selected row, or
    // the first when none is. One while no row has a size yet, so a page never jumps to the end.
    function page(): int {
        const vertical = orientation === ListView.Vertical;
        const item = currentItem ?? itemAtIndex(0);
        const row = (vertical ? item?.height : item?.width) ?? 0;
        if (row <= 0)
            return 1;
        const room = vertical ? height : width;
        return Math.max(1, Math.floor((room + spacing) / (row + spacing)));
    }

    // Returns true when the key was handled, so callers can set event.accepted.
    function handleKey(event: KeyEvent): bool {
        const ctrl = event.modifiers & Qt.ControlModifier;
        const plain = vimKeys && !ctrl;
        const k = event.key;
        const vertical = orientation === ListView.Vertical;
        const next = vertical ? (k === Qt.Key_Down || (plain && k === Qt.Key_J)) : (k === Qt.Key_Right || (plain && k === Qt.Key_L));
        const previous = vertical ? (k === Qt.Key_Up || (plain && k === Qt.Key_K)) : (k === Qt.Key_Left || (plain && k === Qt.Key_H));
        if (next || (ctrl && (k === Qt.Key_J || k === Qt.Key_N)))
            step(1);
        else if (previous || (ctrl && (k === Qt.Key_K || k === Qt.Key_P)))
            step(-1);
        else if (!fieldKeys && k === Qt.Key_Home)
            jump(0);
        else if (!fieldKeys && k === Qt.Key_End)
            jump(count - 1);
        else if (k === Qt.Key_PageDown)
            jump(currentIndex + page());
        else if (k === Qt.Key_PageUp)
            jump(currentIndex - page());
        else if ((k === Qt.Key_Return || k === Qt.Key_Enter) && currentIndex >= 0 && count > 0) {
            if (!event.isAutoRepeat)
                activated(currentIndex);
        } else
            return false;
        return true;
    }

    Keys.onPressed: event => event.accepted = handleKey(event)
}
