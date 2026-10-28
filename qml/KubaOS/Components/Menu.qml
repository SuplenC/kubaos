import QtQuick
import KubaOS.Components as K

// A few actions or choices, one under the other: `kubaos-ui`'s `menu::menu`, on a `Popover`.
// `entries` are `{ text, icon, shortcut, checkable, checked, danger, enabled, separator }`, every
// key but `text` optional. The keyboard moves with ↑ ↓ (and j k, Ctrl+J/K/N/P), Home and End over
// the items that can be taken, Enter takes one (`activated`, once per press), and Esc leaves it
// (`dismissed`).
Item {
    id: root

    property var entries: []
    // The item the keyboard is on, or -1.
    property int currentIndex: -1

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
            if (!event.isAutoRepeat && root.takes(root.currentIndex))
                root.activated(root.currentIndex);
        } else if (k === Qt.Key_Escape)
            root.dismissed();
        else
            return;
        event.accepted = true;
    }

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

    onCurrentIndexChanged: {
        const item = items.itemAt(root.currentIndex);
        if (!item)
            return;
        if (item.y < scroller.contentY)
            scroller.contentY = item.y;
        else if (item.y + item.height > scroller.contentY + scroller.height)
            scroller.contentY = item.y + item.height - scroller.height;
    }

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
                    separator: modelData.separator ?? false
                    enabled: !(modelData.separator ?? false) && modelData.enabled !== false
                    highlighted: index === root.currentIndex
                    onActivated: root.activated(index)
                }
            }
        }
    }
}
