import QtQuick
import KubaOS.Components as K
import qs.services
import "counters.js" as Logic

// A counter's menu, a `Menu` on a `Popover` floating over the window at the pointer: Rename, Reset
// now, Auto reset, History, then Hold to remove, held to confirm. Auto reset opens in its place:
// Back, the box a rule is typed into (a `Prompt` in the menu, saying what it reads and when the
// next reset is as it is typed), then Never and When it reaches N, which asks for N in a prompt
// under it. Esc steps back from Auto reset, then closes it; a press anywhere else closes it.
Item {
    id: root

    // The counter it is open for, or null while it is closed.
    property var counter: null
    // "menu", or "auto" for Auto reset in its place.
    property string view: "menu"
    // What had the keyboard before the menu took it, given back when it closes.
    property Item returnTo: null
    // Where the pointer opened it, in the window's content coordinates.
    property point at: Qt.point(0, 0)

    readonly property bool open: root.counter !== null

    // Rename, History or a reset: the sheet does it, once the menu has closed.
    signal renameAsked(var counter)
    signal historyAsked(var counter)

    // Opens `counter`'s menu with its corner at `at`, in the window's content coordinates.
    function show(counter: var, at: point): void {
        const window = root.Window.window;
        if (!window)
            return;
        root.returnTo = window.activeFocusItem;
        root.parent = window.contentItem;
        root.counter = counter;
        root.at = at;
        root.showMenu();
    }

    function showMenu(): void {
        root.view = "menu";
        menu.currentIndex = 0;
        menu.forceActiveFocus();
    }

    function showAuto(): void {
        root.view = "auto";
        rule.text = root.counter.rule;
        Counters.check(rule.text);
        rule.focusField();
    }

    function close(): void {
        stop.hide();
        root.counter = null;
        const back = root.returnTo;
        root.returnTo = null;
        if (back)
            back.forceActiveFocus();
    }

    // HAZARD: read what was picked before closing. Closing clears the counter.
    function take(index: int): void {
        const counter = root.counter;
        const what = Logic.MENU[index];
        if (what === "auto") {
            root.showAuto();
            return;
        }
        root.close();
        if (what === "rename")
            root.renameAsked(counter);
        else if (what === "reset")
            Counters.reset(counter.name);
        else if (what === "history")
            root.historyAsked(counter);
        else if (what === "remove")
            Counters.remove(counter.name);
    }

    anchors.fill: parent
    visible: root.open
    z: 998

    // The counter changed while its menu shows (its value, its rule): the menu shows it as it is
    // now. Removed, it takes the menu with it.
    Connections {
        target: Counters

        function onListChanged(): void {
            if (!root.counter)
                return;
            const now = Counters.list.find(c => c.id === root.counter.id);
            if (now)
                root.counter = now;
            else
                root.close();
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onPressed: root.close()
    }

    K.Popover {
        id: popover

        readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

        floating: true
        fixedWidth: K.Tokens.size.menu
        // At the pointer, kept inside the window `space.s8` from its edges.
        x: K.Look.snap(Math.max(K.Tokens.space.s8, Math.min(root.at.x, root.width - width - K.Tokens.space.s8)), dpr)
        y: K.Look.snap(Math.max(K.Tokens.space.s8, Math.min(root.at.y, root.height - height - K.Tokens.space.s8)), dpr)

        K.Stack {
            fill: true

            K.Menu {
                id: menu

                visible: root.view === "menu"
                entries: root.counter ? Logic.menuEntries(root.counter) : []
                onActivated: index => root.take(index)
                onDismissed: root.close()
            }

            K.Menu {
                id: back

                visible: root.view === "auto"
                entries: [
                    {
                        text: "Back",
                        icon: "arrow_back"
                    }
                ]
                onActivated: root.showMenu()
                onDismissed: root.showMenu()
            }

            K.MenuItem {
                visible: root.view === "auto"
                separator: true
            }

            K.Prompt {
                id: rule

                visible: root.view === "auto"
                inMenu: true
                label: "When it resets"
                check: typed => Logic.checkRule(typed, Counters.reading)
                note: typed => Logic.ruleNote(typed, Counters.reading)
                onTextChanged: {
                    if (root.view === "auto")
                        Counters.check(rule.text);
                }
                onSaved: typed => {
                    Counters.setRule(root.counter.name, typed);
                    root.close();
                }
                onCancelled: root.showMenu()
            }

            K.MenuItem {
                visible: root.view === "auto"
                separator: true
            }

            K.Menu {
                id: choices

                visible: root.view === "auto"
                entries: root.counter ? Logic.autoEntries(root.counter) : []
                onActivated: index => {
                    if (index === 0) {
                        Counters.setRule(root.counter.name, "never");
                        root.close();
                    } else {
                        stop.text = String(Logic.stopOf(root.counter));
                        // Under the menu as it is laid out once Auto reset has taken its place.
                        Qt.callLater(() => stop.show(choices, "end"));
                    }
                }
                onDismissed: root.showMenu()
            }
        }
    }

    // N for When it reaches, under the menu's last item.
    K.Prompt {
        id: stop

        floating: true
        label: "When it reaches"
        check: typed => Logic.checkStop(typed)
        onSaved: typed => {
            Counters.setStop(root.counter.name, Number(typed));
            root.close();
        }
        onCancelled: choices.forceActiveFocus()
    }
}
