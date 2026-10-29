import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services
import "feed.js" as Feed

// The notifications panel, out of the frame's right edge from the bar to the bottom edge, as the
// control center is, and never beside it (`Panels` opens one panel at a time). A header with the
// title, a line saying how many are new or why they are silenced, and two quiet buttons: Do not
// disturb (filled with the accent while on, its hint saying why) and Clear all. Under it the
// notifications, New then Earlier, several from one app in a group (`CardGroup.qml`), which a
// click, Enter or → opens and ← or Esc closes, one card at a time unfolded to its whole text; the
// keys button floats in the bottom right corner, its tooltip listing the keys, and the list keeps
// room at its end to scroll clear of it.
//
// What was unread when the panel opened stays under New while it shows, so no card changes
// section as it is marked read or its action runs; closing the panel marks everything read.
// Opening it is seeing the popups too: they leave as it opens and are read with the rest.
K.Sheet {
    id: root

    // The groups opened in the list, by their rows' keys.
    property var expanded: ({})
    // The card the keyboard is on in an opened group, by its place in the group.
    property int inner: 0
    // The notification whose card shows its whole text, null for none.
    property var unfolded: null
    // The ids unread when the panel opened, as a set.
    property var fresh: ({})
    readonly property var sections: Feed.sections(Notifs.history, n => root.fresh[n.id] === true || Notifs.isUnread(n))
    readonly property var rows: Feed.rows(root.sections)
    readonly property var current: list.currentIndex >= 0 ? root.rows[list.currentIndex] ?? null : null
    readonly property int padding: K.Tokens.space.s16

    function setExpanded(key: string, open: bool): void {
        const next = Object.assign({}, root.expanded);
        if (open)
            next[key] = true;
        else
            delete next[key];
        root.expanded = next;
    }

    // The group of the row the keyboard is on, null on none.
    function currentGroup(): CardGroup {
        return list.itemAtIndex(list.currentIndex)?.group ?? null;
    }

    // Opens or closes the current row's group, the keyboard on its first card.
    function openCurrent(open: bool): bool {
        const group = root.currentGroup();
        if (!group?.stacked || group.isOpen === open)
            return false;
        root.setExpanded(root.current.key, open);
        root.inner = 0;
        return true;
    }

    // Moves the keyboard `delta` cards within an opened group; whether there was a card to go to.
    function moveInside(delta: int): bool {
        const group = root.currentGroup();
        const next = root.inner + delta;
        if (!group?.isOpen || next < 0 || next >= group.count)
            return false;
        root.inner = next;
        root.keepInView(group.cardRect(next), group);
        return true;
    }

    // Scrolls the list just enough to show `box`, in `group`'s coordinates.
    function keepInView(box: rect, group: Item): void {
        const top = group.mapToItem(list.contentItem, 0, box.y).y;
        const low = top - list.topMargin;
        const high = top + box.height + list.bottomMargin - list.height;
        const least = list.originY - list.topMargin;
        const most = Math.max(least, list.originY + list.contentHeight + list.bottomMargin - list.height);
        const to = list.contentY > low ? low : list.contentY < high ? high : list.contentY;
        list.contentY = Math.max(least, Math.min(most, to));
    }

    // Dismisses the card the keyboard is on: one of an opened group, or the whole group.
    function dismissCurrent(): void {
        const at = list.currentIndex;
        const group = root.currentGroup();
        if (!group)
            return;
        if (group.isOpen) {
            Notifs.dismissAll([group.items[root.inner]]);
            root.inner = Math.max(0, Math.min(root.inner, group.count - 2));
        } else {
            Notifs.dismissAll(group.items);
        }
        Qt.callLater(() => list.jump(at));
    }

    // Which way a key moves along the list: 1, -1, or 0 for a key that doesn't, as `NavList` reads
    // them.
    function stepOf(event: KeyEvent): int {
        const ctrl = event.modifiers & Qt.ControlModifier;
        const plain = !ctrl && !(event.modifiers & Qt.AltModifier);
        const k = event.key;
        if (k === Qt.Key_Down || (plain && k === Qt.Key_J) || (ctrl && (k === Qt.Key_J || k === Qt.Key_N)))
            return 1;
        if (k === Qt.Key_Up || (plain && k === Qt.Key_K) || (ctrl && (k === Qt.Key_K || k === Qt.Key_P)))
            return -1;
        return 0;
    }

    edge: "right"
    flushStart: true
    flushEnd: true
    contentWidth: K.Tokens.size.panel
    onOpened: {
        list.jump(0);
        list.forceActiveFocus();
    }
    onActiveChanged: {
        if (!root.active)
            Notifs.letGoSeen();
    }
    onShownChanged: {
        if (root.shown) {
            root.fresh = Object.assign({}, Notifs.unreadIds);
            return;
        }
        Notifs.markAllRead();
        root.fresh = {};
        root.expanded = {};
        root.inner = 0;
        root.unfolded = null;
    }

    // The header, the container padding in at the sides and 16 above.
    K.Container {
        id: head

        // Over the list's fade, so a tooltip hanging under the header is never faded.
        z: 1
        width: root.contentWidth

        K.Stack {
            fill: true
            topPadding: K.Tokens.space.s16

            K.Row {
                fill: true
                gap: K.Tokens.space.s8
                align: "centre"

                K.Stack {
                    fill: true

                    K.Text {
                        text: "Notifications"
                        style: K.Tokens.type.heading
                        maximumLines: 1
                    }

                    K.Text {
                        text: Feed.status(Notifs.unreadCount, Notifs.count, Notifs.dndReason)
                        style: K.Tokens.type.small
                        color: K.Tokens.color.on_surface_variant
                        maximumLines: 1
                    }
                }

                K.IconButton {
                    id: dnd

                    readonly property var tip: Feed.dndTip(Notifs.dndReason, Notifs.dndState.sharing)

                    icon: "dark_mode"
                    label: dnd.tip.title
                    kind: Notifs.dnd ? "Filled" : "Text"
                    size: "Sm"
                    explained: true
                    onActivated: Notifs.toggleDnd()

                    // Below the button: the header leaves no room above it.
                    K.Tooltip {
                        title: dnd.tip.title
                        body: dnd.tip.body
                        within: panelBox
                        shown: dnd.pointerOver || dnd.focused
                    }
                }

                K.IconButton {
                    id: clearAll

                    visible: Notifs.count > 0
                    icon: "clear_all"
                    label: "Clear all"
                    kind: "Text"
                    size: "Sm"
                    explained: true
                    onActivated: Notifs.clear()

                    K.Tooltip {
                        title: "Clear all"
                        body: "Dismiss every notification, new and earlier."
                        within: panelBox
                        shown: clearAll.pointerOver || clearAll.focused
                    }
                }
            }
        }
    }

    // The list, in the room under the header. It reaches past the container padding at the sides
    // as far as a glow does, so the focused card keeps its glow, and is cut at the header. A
    // group's preview floats over the header's fade too.
    NavList {
        id: list

        readonly property int reach: K.Tokens.focus.far
        // The group whose preview shows.
        property CardGroup lifted: null

        x: root.padding - list.reach
        y: head.height
        width: root.contentWidth - (root.padding - list.reach) * 2
        height: Math.max(0, root.height - y)
        z: list.lifted ? 1 : 0
        visible: root.rows.length > 0
        topMargin: K.Tokens.space.s16
        // Room to scroll the last card clear of the keys button.
        bottomMargin: K.Tokens.space.s16 * 2 + K.Tokens.size.control_sm
        spacing: K.Tokens.space.s8
        highlightShown: list.activeFocus
        selectable: index => root.rows[index]?.kind === "card"
        model: ScriptModel {
            values: root.rows
            objectProp: "key"
        }
        onActivated: root.currentGroup()?.press()
        onCurrentIndexChanged: root.inner = 0

        Keys.onPressed: event => {
            const step = root.stepOf(event);
            if (step !== 0) {
                if (root.moveInside(step)) {
                    event.accepted = true;
                    return;
                }
                event.accepted = list.handleKey(event);
                // Up into an opened group lands on its last card.
                const group = root.currentGroup();
                if (step < 0 && group?.isOpen)
                    root.inner = group.count - 1;
                return;
            }
            if (event.key === Qt.Key_Right || event.key === Qt.Key_L)
                root.openCurrent(true);
            else if (event.key === Qt.Key_Left || event.key === Qt.Key_H)
                root.openCurrent(false);
            else if (event.key === Qt.Key_Delete)
                root.dismissCurrent();
            else if (event.key === Qt.Key_D && event.modifiers === Qt.NoModifier)
                Notifs.toggleDnd();
            else if (event.key === Qt.Key_Space) {
                if (!event.isAutoRepeat)
                    root.currentGroup()?.press();
            } else if (event.key === Qt.Key_Escape && (root.unfolded !== null || root.currentGroup()?.isOpen)) {
                // Esc folds an unfolded card first, then closes an opened group; with neither it
                // goes on to close the panel.
                if (root.unfolded !== null)
                    root.unfolded = null;
                else
                    root.openCurrent(false);
            } else {
                event.accepted = list.handleKey(event);
                return;
            }
            event.accepted = true;
        }

        delegate: Item {
            id: row

            required property var modelData
            required property int index
            readonly property bool label: row.modelData.kind === "label"
            readonly property CardGroup group: row.label ? null : loader.item

            // The list places its rows down its whole width; the row's content sits the glow's reach
            // in from either side, at the container padding.
            width: list.width
            height: row.label ? K.Tokens.type.label.line + (row.index > 0 ? K.Tokens.space.s8 : 0) : (loader.item?.implicitHeight ?? 0)
            z: row.group?.raised ? 1 : 0
            // Every other row sits back behind a group's preview.
            opacity: list.lifted && list.lifted !== row.group ? 1 - K.Tokens.opacity.recede * list.lifted.recede : 1

            K.Text {
                x: list.reach
                y: row.height - height
                visible: row.label
                text: row.modelData.text ?? ""
                style: K.Tokens.type.label
                color: K.Tokens.color.on_surface_variant
            }

            Loader {
                id: loader

                x: list.reach
                width: row.width - list.reach * 2
                active: !row.label

                sourceComponent: CardGroup {
                    id: group

                    items: row.modelData.items
                    open: root.expanded[row.modelData.key] === true
                    selected: list.currentIndex === row.index && list.highlightShown
                    current: list.currentIndex === row.index ? root.inner : -1
                    onOpenRequested: open => {
                        root.setExpanded(row.modelData.key, open);
                        list.currentIndex = row.index;
                        root.inner = 0;
                    }
                    onRaisedChanged: {
                        if (group.raised)
                            list.lifted = group;
                        else if (list.lifted === group)
                            list.lifted = null;
                    }
                    Component.onDestruction: {
                        if (list.lifted === group)
                            list.lifted = null;
                    }

                    delegate: NotificationCard {
                        required property int index

                        notification: group.items[index] ?? null
                        items: group.standing[index] ?? []
                        unfolded: root.unfolded !== null && root.unfolded === notification
                        onUnfoldRequested: open => root.unfolded = open ? notification : null
                        onMenuRequested: at => menu.open(notification, at)
                    }
                }
            }
        }
    }

    HeaderFade {
        y: head.height
        width: root.contentWidth
        scrolled: !list.atYBeginning
    }

    // Nothing at all: one quiet line in the middle of the list's room.
    K.Text {
        x: K.Look.centre(root.contentWidth - width, root.dpr)
        y: head.height + K.Look.snap((root.height - head.height - height) / 2, root.dpr)
        visible: root.rows.length === 0
        text: "Nothing new."
        style: K.Tokens.type.body
        color: K.Tokens.color.on_surface_variant
    }

    // The panel's own box, which its tooltips keep inside.
    Item {
        id: panelBox

        anchors.fill: parent
    }

    // The keys, floating in the bottom right corner over the list.
    K.IconButton {
        id: keys

        x: root.contentWidth - root.padding - width
        y: root.height - root.padding - height
        icon: "keyboard"
        label: "Keys"
        kind: "Tonal"
        size: "Sm"
        explained: true
        activeFocusOnTab: false

        K.Tooltip {
            title: "Keys"
            within: panelBox
            shown: keys.pointerOver

            K.Stack {
                gap: K.Tokens.space.s8

                K.KeyHints {
                    hints: [["↑ ↓", "Move"], ["↵", "Unfold or open"]]
                }

                K.KeyHints {
                    hints: [["→ ←", "Open or close a group"]]
                }

                K.KeyHints {
                    hints: [["Del", "Dismiss"], ["D", "Do not disturb"]]
                }

                K.KeyHints {
                    hints: [["Esc", "Fold, close or leave"]]
                }
            }
        }
    }

    NotificationMenu {
        id: menu
    }
}
