import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services
import "counters.js" as Logic

// The counters' sheet, growing down from the bar's corner as the bell's peek does: a row per
// counter, flush on the sheet (`CounterRow.qml`), then under a divider New counter, which asks for
// a name. A row's name or value is changed in a prompt under it, its menu (`CounterMenu.qml`)
// resets it, sets its auto reset, shows its history in the sheet's place (`HistoryPage.qml`) or
// removes it. Every change goes through `kubaos-counters` (`services/Counters.qml`).
//
// Keys on the list: ↑ ↓ (j k, Ctrl+J/K/N/P) move round the rows; + or = adds 1 (at its stop,
// resets); - takes 1 away; F2 asks for the name, Shift+F2 for the value; Menu or Shift+F10 opens
// the row's menu under it. Tab goes on to New counter. Esc leaves the history, then closes.
K.Sheet {
    id: root

    readonly property int padding: K.Tokens.space.s8
    // "list", or "history" while a counter's history shows in the list's place.
    property string page: "list"
    // The counters' rows past which the list scrolls.
    readonly property int mostRows: 8
    // What the prompt asks for: "name", "value" or "new"; the counter it changes for the first two.
    property string asking: ""
    property var askedFor: null
    // A key has been used in the sheet since it opened: the list's tube and the keyboard's row
    // light only then. A row clicked is lit quietly, with no tube.
    property bool keyed: false
    property bool clicked: false

    // Asks for `counter`'s name or value (`what`) in a prompt under it.
    function ask(what: string, counter: var, anchor: Item): void {
        root.asking = what;
        root.askedFor = counter;
        if (what === "name") {
            prompt.label = "Rename";
            prompt.text = counter.name;
            prompt.check = typed => Logic.checkName(typed, Counters.list, counter.name);
        } else {
            prompt.label = "Set value";
            prompt.text = String(counter.value);
            prompt.check = typed => Logic.checkCount(typed);
        }
        prompt.show(anchor, what === "value" ? "end" : "start");
    }

    function askNew(): void {
        root.asking = "new";
        root.askedFor = null;
        prompt.label = "New counter";
        prompt.text = Logic.nextName(Counters.list);
        prompt.check = typed => Logic.checkName(typed, Counters.list, null);
        prompt.show(newRow, "start");
    }

    // What the prompt asked for, saved.
    function save(typed: string): void {
        const counter = root.askedFor;
        if (root.asking === "name")
            Counters.rename(counter.name, typed.trim());
        else if (root.asking === "value")
            Counters.set(counter.name, Number(typed));
        else if (root.asking === "new")
            Counters.create(typed.trim());
        root.land();
    }

    // The keyboard back where it was before a prompt or the history: the list, or New counter.
    function land(): void {
        if (root.asking === "new")
            newRow.forceActiveFocus();
        else
            list.forceActiveFocus();
        root.asking = "";
        root.askedFor = null;
    }

    function showList(): void {
        root.page = "list";
        list.forceActiveFocus();
    }

    function showHistory(counter: var): void {
        root.page = "history";
        history.showFor(counter);
    }

    // +1, or at its stop, back to 0.
    function plus(counter: var): void {
        if (counter.stop > 0 && counter.value >= counter.stop)
            Counters.reset(counter.name);
        else
            Counters.add(counter.name);
    }

    function minus(counter: var): void {
        if (counter.value > 0)
            Counters.sub(counter.name);
    }

    // The row's menu under it, as the keyboard opens it.
    function menuUnder(index: int): void {
        const row = list.itemAtIndex(index);
        if (!row)
            return;
        menu.show(row.counter, row.mapToItem(null, row.width / 2, row.height));
    }

    edge: "top"
    flushEnd: true
    contentWidth: K.Tokens.size.panel
    contentHeight: (root.page === "history" ? history.implicitHeight : listPage.implicitHeight) + root.padding * 2
    onOpened: {
        root.keyed = false;
        root.clicked = false;
        root.page = "list";
        list.currentIndex = Math.min(Math.max(list.currentIndex, 0), Counters.list.length - 1);
        list.forceActiveFocus();
    }
    onShownChanged: {
        if (!root.shown) {
            prompt.hide();
            menu.close();
        }
    }

    // The last counter removed: nothing is left to show, and the pill has gone.
    Connections {
        target: Counters

        function onListChanged(): void {
            if (root.shown && Counters.list.length === 0)
                Panels.close();
        }
    }

    K.Stack {
        id: listPage

        x: root.padding
        y: root.padding
        width: root.contentWidth - root.padding * 2
        visible: root.page === "list"
        fill: true
        gap: K.Tokens.space.s8

        K.List {
            flush: true
            focused: list.activeFocus && root.keyed

            // The rows' room: a layout lays out no view of its own, as it takes one for a Repeater
            // (it has `itemAt`), so the view fills this.
            Item {
                readonly property bool fillWidth: true

                implicitHeight: Math.max(0, Math.min(list.count, root.mostRows) * (K.Tokens.size.row_two_line + list.spacing) - list.spacing)

                NavList {
                    id: list

                    readonly property string widest: Logic.widest(Counters.list)

                    anchors.fill: parent
                    highlightShown: list.activeFocus && (root.keyed || root.clicked)
                    // Kept by id, so a counter that changes keeps its row and its value rolls.
                    model: ScriptModel {
                        values: Counters.list
                        objectProp: "id"
                    }
                    KeyNavigation.tab: newRow

                    Keys.onPressed: event => {
                        root.keyed = true;
                        const row = list.currentItem;
                        if (!row || event.modifiers & (Qt.ControlModifier | Qt.AltModifier))
                            return;
                        const shift = event.modifiers & Qt.ShiftModifier;
                        if (event.key === Qt.Key_Plus || event.key === Qt.Key_Equal)
                            root.plus(row.counter);
                        else if (event.key === Qt.Key_Minus)
                            root.minus(row.counter);
                        else if (event.key === Qt.Key_F2 && shift)
                            root.ask("value", row.counter, row.valueAnchor);
                        else if (event.key === Qt.Key_F2)
                            root.ask("name", row.counter, row.nameAnchor);
                        else if (event.key === Qt.Key_Menu || (event.key === Qt.Key_F10 && shift))
                            root.menuUnder(list.currentIndex);
                        else
                            return;
                        event.accepted = true;
                    }

                    delegate: CounterRow {
                        id: row

                        widest: list.widest
                        lit: ListView.isCurrentItem && (ListView.view.highlightShown || menu.counter?.id === row.counter.id)
                        onPicked: {
                            root.clicked = true;
                            ListView.view.currentIndex = row.index;
                            list.forceActiveFocus();
                        }
                        onPlus: root.plus(row.counter)
                        onMinus: root.minus(row.counter)
                        onRenameAsked: {
                            ListView.view.currentIndex = row.index;
                            root.ask("name", row.counter, row.nameAnchor);
                        }
                        onSetAsked: {
                            ListView.view.currentIndex = row.index;
                            root.ask("value", row.counter, row.valueAnchor);
                        }
                        onMenuAsked: at => {
                            ListView.view.currentIndex = row.index;
                            menu.show(row.counter, at);
                        }
                    }
                }
            }
        }

        K.Divider {}

        K.List {
            flush: true
            focused: newRow.activeFocus && root.keyed

            K.ListRow {
                id: newRow

                title: "New counter"
                icon: "add"
                selected: newRow.activeFocus && root.keyed
                Keys.onPressed: root.keyed = true
                activeFocusOnTab: true
                KeyNavigation.backtab: list
                onActivated: root.askNew()
            }
        }
    }

    HistoryPage {
        id: history

        x: root.padding
        y: root.padding
        width: root.contentWidth - root.padding * 2
        visible: root.page === "history"
        onBack: root.showList()
    }

    CounterMenu {
        id: menu

        onRenameAsked: counter => {
            const row = list.itemAtIndex(Counters.list.findIndex(c => c.id === counter.id));
            if (row)
                root.ask("name", counter, row.nameAnchor);
        }
        onHistoryAsked: counter => root.showHistory(counter)
    }

    // A counter's name or value, or a new counter's name, under what it changes.
    K.Prompt {
        id: prompt

        floating: true
        onSaved: typed => root.save(typed)
        onCancelled: root.land()
    }
}
