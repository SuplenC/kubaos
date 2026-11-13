import QtQuick
import KubaOS.Components as K

// A few actions or choices, one under the other: `kubaos-ui`'s `menu::menu`, on a `Popover`.
// `entries` are `{ text, icon, shortcut, checkable, checked, danger, held, enabled, separator }`,
// every key but `text` optional; a sample may pin one `preview`: "hovered" or "pressed". The keyboard moves with ↑ ↓ (and j k, Ctrl+J/K/N/P), Home and End
// over the items that can be taken, Enter takes one (`activated`, once per press) or, kept down,
// holds a held one until it is taken, and Esc leaves it (`dismissed`). A held entry's `reach` pins
// its sweep part of the way: a sample.
Item {
    id: root

    property var entries: []
    // The item the keyboard is on, or -1.
    property int currentIndex: -1
    // Enter is down on a held item: it holds the item the keyboard is on, and lets go when the
    // keyboard moves off it.
    property bool keyHeld: false

    signal activated(int index)
    signal dismissed

    function takes(index: int): bool {
        const entry = root.entries[index];
        return !!entry && !entry.separator && entry.enabled !== false;
    }

    // To the next item that can be taken `delta` away, wrapping round.
    function step(delta: int): void {
        const count = root.entries.length;
        let index = root.currentIndex;
        for (let i = 0; i < count; i++) {
            index = (index + delta + count) % count;
            if (root.takes(index)) {
                root.currentIndex = index;
                return;
            }
        }
    }

    // To the first item that can be taken from `from` on, in `delta`'s direction.
    function jump(from: int, delta: int): void {
        for (let index = from; index >= 0 && index < root.entries.length; index += delta) {
            if (root.takes(index)) {
                root.currentIndex = index;
                return;
            }
        }
    }

    activeFocusOnTab: true
    Keys.onPressed: event => {
        const k = event.key;
        const ctrl = event.modifiers & Qt.ControlModifier;
        if (k === Qt.Key_Down || (!ctrl && k === Qt.Key_J) || (ctrl && (k === Qt.Key_J || k === Qt.Key_N)))
            root.step(1);
        else if (k === Qt.Key_Up || (!ctrl && k === Qt.Key_K) || (ctrl && (k === Qt.Key_K || k === Qt.Key_P)))
            root.step(-1);
        else if (k === Qt.Key_Home)
            root.jump(0, 1);
        else if (k === Qt.Key_End)
            root.jump(root.entries.length - 1, -1);
        else if (k === Qt.Key_Return || k === Qt.Key_Enter) {
            if (!event.isAutoRepeat && root.takes(root.currentIndex)) {
                if (root.entries[root.currentIndex].held)
                    root.keyHeld = true;
                else
                    root.activated(root.currentIndex);
            }
        } else if (k === Qt.Key_Escape)
            root.dismissed();
        else
            return;
        event.accepted = true;
    }
    Keys.onReleased: event => {
        if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter) && !event.isAutoRepeat)
            root.keyHeld = false;
    }
    onActiveFocusChanged: root.keyHeld = false

    readonly property bool fillWidth: true
    readonly property var probe: ({
            role: "component",
            name: "Menu",
            variant: "",
            fixed: {
                width: false,
                height: false
            }
        })

    // The tallest it grows; past it, its items scroll, the one the keyboard is on kept in view.
    property real maxHeight: Infinity

    implicitHeight: Math.min(column.implicitHeight, root.maxHeight)

    function relayout(): void {
        column.relayout();
    }

    // Scrolls the item the keyboard is on into view. Laid out only while shown, the items have no
    // place before the menu shows, so it waits for one.
    function reveal(): void {
        const item = items.itemAt(root.currentIndex);
        if (!item || !root.visible || scroller.height <= 0 || column.implicitHeight <= 0)
            return;
        if (item.y < scroller.contentY)
            scroller.contentY = item.y;
        else if (item.y + item.height > scroller.contentY + scroller.height)
            scroller.contentY = item.y + item.height - scroller.height;
    }

    onCurrentIndexChanged: {
        root.keyHeld = false;
        root.reveal();
    }
    onEntriesChanged: scroller.contentY = 0
    // Each time it shows, and once its items have their places, the keyboard's item is in view.
    onVisibleChanged: {
        if (root.visible)
            Qt.callLater(root.reveal);
    }
    onHeightChanged: Qt.callLater(root.reveal)

    Flickable {
        id: scroller

        width: root.width
        height: root.height
        contentWidth: root.width
        contentHeight: column.implicitHeight
        interactive: column.implicitHeight > root.maxHeight
        clip: interactive
        boundsBehavior: Flickable.StopAtBounds

        K.Stack {
            id: column

            width: root.width
            fill: true
            onImplicitHeightChanged: Qt.callLater(root.reveal)

            Repeater {
                id: items

                model: root.entries

                K.MenuItem {
                    required property var modelData
                    required property int index

                    text: modelData.text ?? ""
                    icon: modelData.icon ?? ""
                    shortcut: modelData.shortcut ?? []
                    checkable: modelData.checkable ?? false
                    checked: modelData.checked ?? false
                    danger: modelData.danger ?? false
                    held: modelData.held ?? false
                    hold: modelData.reach ?? 0
                    preview: modelData.preview ?? ""
                    keyHolding: root.keyHeld && index === root.currentIndex
                    separator: modelData.separator ?? false
                    enabled: !(modelData.separator ?? false) && modelData.enabled !== false
                    highlighted: index === root.currentIndex
                    onActivated: root.activated(index)
                }
            }
        }
    }
}
