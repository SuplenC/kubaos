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
// notifications, New then Earlier, several from one app stacked into one card, one card at a time
// unfolded to its whole text; the keys button floats in the bottom right corner, its tooltip
// listing the keys, and the list keeps room at its end to scroll clear of it.
//
// What was unread when the panel opened stays under New while it shows, so no card changes
// section as it is marked read or its action runs; closing the panel marks everything read.
// Opening it is seeing the popups too: they leave as it opens and are read with the rest.
K.Sheet {
    id: root

    // The stacks open in the list, by their rows' keys.
    property var expanded: ({})
    // The notification whose card shows its whole text, null for none.
    property var unfolded: null
    // The ids unread when the panel opened, as a set.
    property var fresh: ({})
    readonly property var sections: Feed.sections(Notifs.history, n => root.fresh[n.id] === true || Notifs.isUnread(n))
    readonly property var rows: Feed.rows(root.sections, root.expanded)
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

    // Folds the current card's stack and keeps the keyboard on it.
    function fold(): void {
        const key = root.current?.key ?? "";
        if (key === "" || root.expanded[key] !== true)
            return;
        root.setExpanded(key, false);
        list.currentIndex = root.rows.findIndex(r => r.key === key);
    }

    function openStack(): void {
        if (root.current?.stacked)
            root.setExpanded(root.current.key, true);
    }

    function dismissCurrent(): void {
        const at = list.currentIndex;
        if (root.current?.kind === "card")
            Notifs.dismissAll(root.current.items);
        Qt.callLater(() => list.jump(at));
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
    // as far as a glow does, so the focused card keeps its glow, and is cut at the header.
    NavList {
        id: list

        readonly property int reach: K.Tokens.focus.far

        x: root.padding - list.reach
        y: head.height
        width: root.contentWidth - (root.padding - list.reach) * 2
        height: Math.max(0, root.height - y)
        visible: root.rows.length > 0
        topMargin: K.Tokens.space.s16
        // Room to scroll the last card clear of the keys button.
        bottomMargin: K.Tokens.space.s16 * 2 + K.Tokens.size.control_sm
        spacing: K.Tokens.space.s8
        highlightShown: list.activeFocus
        selectable: index => root.rows[index]?.kind === "card"
        model: ScriptModel {
            values: root.rows
        }
        onActivated: index => cardAt(index)?.press()

        function cardAt(index: int): var {
            const item = list.itemAtIndex(index);
            return item?.card ?? null;
        }

        Keys.onPressed: event => {
            if (event.key === Qt.Key_Right || event.key === Qt.Key_L)
                root.openStack();
            else if (event.key === Qt.Key_Left || event.key === Qt.Key_H)
                root.fold();
            else if (event.key === Qt.Key_Delete)
                root.dismissCurrent();
            else if (event.key === Qt.Key_D && event.modifiers === Qt.NoModifier)
                Notifs.toggleDnd();
            else if (event.key === Qt.Key_Space) {
                if (!event.isAutoRepeat)
                    list.cardAt(list.currentIndex)?.press();
            } else if (event.key === Qt.Key_Escape && root.unfolded !== null) {
                // Esc folds an unfolded card first; with none it goes on to close the panel.
                root.unfolded = null;
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
            readonly property var card: row.label ? null : loader.item

            // The list places its rows down its whole width; the row's content sits the glow's reach
            // in from either side, at the container padding.
            width: list.width
            height: row.label ? K.Tokens.type.label.line + (row.index > 0 ? K.Tokens.space.s8 : 0) : (loader.item?.implicitHeight ?? 0)

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

                sourceComponent: NotificationCard {
                    notification: row.modelData.items[0]
                    items: row.modelData.items
                    opens: true
                    unfolded: root.unfolded === row.modelData.items[0]
                    selected: list.currentIndex === row.index && list.highlightShown
                    onExpandRequested: {
                        root.setExpanded(row.modelData.key, true);
                        list.currentIndex = row.index;
                    }
                    onUnfoldRequested: open => root.unfolded = open ? row.modelData.items[0] : null
                    onMenuRequested: at => menu.open(row.modelData.items[0], at)
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
                    hints: [["↑ ↓", "Move"], ["↵", "Unfold"]]
                }

                K.KeyHints {
                    hints: [["→", "Expand a stack"], ["Del", "Dismiss"]]
                }

                K.KeyHints {
                    hints: [["D", "Do not disturb"], ["Esc", "Fold, close"]]
                }
            }
        }
    }

    NotificationMenu {
        id: menu
    }
}
