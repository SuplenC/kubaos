import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.theme
import qs.components
import qs.services

// The keyboard layouts, the default first, and "Add a language…" with a
// search over every layout and variant xkb knows. Every change runs
// `kubaos keyboard`, the one writer of the layouts, which reloads Hyprland;
// services/Keyboard.qml sees the reload and this list reads them again.
// Keys in the list: Enter types with a layout, d makes it the default,
// Shift+J/K moves it, x or Delete removes it. In the search, Enter adds the
// selected match and Esc goes back.
//
// The search is Kotodama's, as every search over a list in the shell is: a
// `KotodamaSearch` of its own enters the "Add a keyboard layout" app
// (crates/kotodama-kubaos/layouts/), so typing matches the way the Command
// menu does, accents and typos included, and this list draws its Results.
// The add view is made anew each time it opens, so nothing from the last
// time (a pick, a query, the selected row) shows over the new one.
//
// A click never adds a match: it picks it, and the Add button under the list
// adds it. "Add a language…" opens the matches where the row was, so the
// second click of a double-click lands on one. For the same reason a click
// on anything that changes the layouts is ignored for a double-click's
// interval after the view switches or the rows move.
ColumnLayout {
    id: root

    // "us", "us(intl)": the layouts as `kubaos keyboard show` spells them.
    property var layouts: []
    // Every layout and variant's name by its spelling.
    property var names: ({})
    property bool accents: true
    property bool adding: false
    property bool busy: false
    property string problem: ""
    readonly property bool settling: settle.running
    // `c_keyboard`'s probe (test/lib/e2e.sh), with KUBAOS_KEYBOARD_PROBE=1 only.
    readonly property bool probed: Quickshell.env("KUBAOS_KEYBOARD_PROBE") === "1"
    // The most a keymap holds (kubaos_core::keyboard::MAX_LAYOUTS).
    readonly property int maxLayouts: 4

    spacing: Theme.spacing.sm

    function nameOf(spelling: string): string {
        return names[spelling] ?? spelling;
    }

    // `kubaos keyboard ARGS`; its stderr is the problem when it fails.
    function change(args: var): void {
        if (busy)
            return;
        busy = true;
        call.createObject(root, {
            command: ["kubaos", "keyboard", ...args]
        });
    }

    // A change from a click, unless the click may be the second of a
    // double-click meant for what was under the pointer a moment ago.
    function changeOnClick(args: var): void {
        if (!settling)
            change(args);
    }

    function move(index: int, delta: int): void {
        const to = index + delta;
        if (to >= 0 && to < layouts.length) {
            change(["move", layouts[index], String(to + 1)]);
            list.currentIndex = to;
        }
    }

    function startAdding(): void {
        if (layouts.length >= maxLayouts)
            return;
        problem = "";
        adding = true;
    }

    function stopAdding(): void {
        adding = false;
        list.forceActiveFocus();
    }

    function add(spelling: string): void {
        change(["add", spelling]);
        stopAdding();
    }

    // `kubaos keyboard` says "the keyboard layouts weren't changed: …";
    // the tab shows it as a sentence.
    function sentence(stderr: string): string {
        const line = stderr.trim().split("\n").pop().replace(/^kubaos: /, "").trim();
        if (line === "")
            return "The keyboard layouts couldn't be changed.";
        const capital = line.charAt(0).toUpperCase() + line.slice(1);
        return /[.!?]$/.test(capital) ? capital : `${capital}.`;
    }

    function probe(line: string): void {
        if (probed)
            console.info(`keyboard probe: ${line}`);
    }

    onAddingChanged: {
        settle.restart();
        probe(`adding ${adding}`);
    }
    onLayoutsChanged: settle.restart()

    Timer {
        id: settle

        interval: Qt.styleHints.mouseDoubleClickInterval
    }

    // Where the probe's pointer clicks: the centre of each target on screen,
    // logged when it moves (the drawer slides in).
    Timer {
        property string last: ""

        running: root.probed && root.visible
        repeat: true
        interval: 200
        onTriggered: {
            const at = (name, item) => {
                if (!item.visible)
                    return "";
                const p = item.mapToItem(null, item.width / 2, item.height / 2);
                return `${name} x=${Math.round(p.x)} y=${Math.round(p.y)}`;
            };
            const button = adder.item?.button;
            const matches = adder.item?.list;
            const first = matches?.visible ? matches.itemAtIndex(0) : null;
            const now = [at("add-row", addRow), first ? at("first-match", first) : "", button ? at("add-button", button) : ""].filter(s => s !== "").join(" ");
            if (now !== last) {
                last = now;
                root.probe(`at ${now}`);
            }
        }
    }

    Component.onCompleted: {
        show.running = true;
        catalogueRead.running = true;
        accentsRead.running = true;
    }

    // `kubaos keyboard` reloads Hyprland after a change, and the layouts
    // Hyprland reports change with it.
    Connections {
        target: Keyboard

        function onLayoutsChanged(): void {
            show.running = true;
            accentsRead.running = true;
        }
    }

    Process {
        id: show

        command: ["kubaos", "keyboard", "show"]
        stdout: StdioCollector {
            onStreamFinished: root.layouts = text.split("\n").filter(l => l !== "")
        }
    }

    Process {
        id: catalogueRead

        command: ["kubaos", "keyboard", "list"]
        stdout: StdioCollector {
            onStreamFinished: {
                const names = {};
                for (const line of text.split("\n")) {
                    const tab = line.indexOf("\t");
                    if (tab >= 0)
                        names[line.slice(0, tab)] = line.slice(tab + 1);
                }
                root.names = names;
            }
        }
    }

    Process {
        id: accentsRead

        command: ["kubaos", "keyboard", "accents"]
        stdout: StdioCollector {
            onStreamFinished: root.accents = text.trim() !== "off"
        }
    }

    Component {
        id: call

        Process {
            id: proc

            property int code: -1
            property bool streamed: false

            function finish(): void {
                if (code < 0 || !streamed)
                    return;
                root.problem = code === 0 ? "" : root.sentence(errors.text);
                root.busy = false;
                show.running = true;
                accentsRead.running = true;
                proc.destroy();
            }

            running: true
            stderr: StdioCollector {
                id: errors

                onStreamFinished: {
                    proc.streamed = true;
                    proc.finish();
                }
            }
            onExited: exitCode => {
                proc.code = exitCode;
                proc.finish();
            }
        }
    }

    // The layouts.
    NavList {
        id: list

        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: !root.adding
        model: ScriptModel {
            values: root.layouts
        }
        highlightShown: activeFocus
        onActivated: index => Quickshell.execDetached(["hyprctl", "switchxkblayout", "all", String(index)])

        // Plain j/k are this handler's, so Shift+J/K can move a layout;
        // NavList's own still takes the arrows, Ctrl+j/k and Enter.
        vimKeys: false
        Keys.onPressed: event => {
            const i = currentIndex;
            const shift = event.modifiers & Qt.ShiftModifier;
            if (event.modifiers & Qt.ControlModifier || i < 0 || i >= root.layouts.length)
                return;
            if (event.key === Qt.Key_J)
                shift ? root.move(i, 1) : step(1);
            else if (event.key === Qt.Key_K)
                shift ? root.move(i, -1) : step(-1);
            else if (event.key === Qt.Key_D && i > 0) {
                root.change(["default", root.layouts[i]]);
                currentIndex = 0;
            } else if ((event.key === Qt.Key_X || event.key === Qt.Key_Delete) && root.layouts.length > 1)
                root.change(["remove", root.layouts[i]]);
            else
                return;
            event.accepted = true;
        }

        delegate: ListRow {
            id: row

            required property string modelData
            required property int index

            width: ListView.view.width
            icon: "keyboard"
            title: root.nameOf(modelData)
            subtitle: modelData + (index === 0 ? " · default" : "") + (Keyboard.active === index ? " · typing" : "")
            active: Keyboard.active === index
            current: ListView.isCurrentItem && ListView.view.activeFocus
            onClicked: Quickshell.execDetached(["hyprctl", "switchxkblayout", "all", String(index)])

            IconButton {
                size: 30
                icon: "star"
                tooltip: "Make the default"
                visible: row.index > 0
                onClicked: root.changeOnClick(["default", row.modelData])
            }

            IconButton {
                size: 30
                icon: "arrow_upward"
                tooltip: "Move up"
                visible: row.index > 0
                onClicked: if (!root.settling)
                    root.move(row.index, -1)
            }

            IconButton {
                size: 30
                icon: "arrow_downward"
                tooltip: "Move down"
                visible: row.index < root.layouts.length - 1
                onClicked: if (!root.settling)
                    root.move(row.index, 1)
            }

            IconButton {
                size: 30
                icon: "close"
                tooltip: "Remove"
                visible: root.layouts.length > 1
                onClicked: root.changeOnClick(["remove", row.modelData])
            }
        }
    }

    ListRow {
        id: addRow

        Layout.fillWidth: true
        visible: !root.adding
        icon: "add"
        title: "Add a language…"
        subtitle: root.layouts.length >= root.maxLayouts ? `${root.maxLayouts} is the most a keyboard holds; remove one first` : ""
        onClicked: root.startAdding()
    }

    ListRow {
        Layout.fillWidth: true
        visible: !root.adding
        icon: "keyboard_option_key"
        title: "Mac accents on Left Alt"
        subtitle: root.accents ? "On · Left Alt + e, then e: é" : "Off"
        active: root.accents
        onClicked: root.changeOnClick(["accents", root.accents ? "off" : "on"])
    }

    // Adding: made while it shows, and gone once it closes.
    Loader {
        id: adder

        Layout.fillWidth: true
        Layout.fillHeight: true
        active: root.adding
        visible: active
        sourceComponent: KeyboardAddView {
            onAdd: spelling => {
                if (!root.settling)
                    root.add(spelling);
            }
            onAddAtOnce: spelling => root.add(spelling)
            onCancelled: root.stopAdding()
            onPickedChanged: root.probe(`picked ${picked?.spelling ?? ""}`)
            onEnteredChanged: root.probe(`entered ${entered}`)
            onFailed: message => root.problem = message
        }
    }

    StyledText {
        Layout.fillWidth: true
        visible: root.problem !== ""
        text: root.problem
        color: Theme.m3error
        wrapMode: Text.Wrap
    }
}
