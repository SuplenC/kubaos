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
ColumnLayout {
    id: root

    // "us", "us(intl)": the layouts as `kubaos keyboard show` spells them.
    property var layouts: []
    // Every layout and variant, { spelling, name }, sorted by name.
    property var catalogue: []
    property var names: ({})
    property bool accents: true
    property bool adding: false
    property bool busy: false
    property string problem: ""
    // The most a keymap holds (kubaos_core::keyboard::MAX_LAYOUTS).
    readonly property int maxLayouts: 4
    readonly property var matches: {
        const query = search.text.trim().toLowerCase();
        return catalogue.filter(c => !layouts.includes(c.spelling) && (query === "" || c.name.toLowerCase().includes(query) || c.spelling.includes(query)));
    }

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
        adding = true;
        search.text = "";
        search.focusInput();
    }

    function stopAdding(): void {
        adding = false;
        list.forceActiveFocus();
    }

    function add(spelling: string): void {
        change(["add", spelling]);
        stopAdding();
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
                const catalogue = [];
                for (const line of text.split("\n")) {
                    const tab = line.indexOf("\t");
                    if (tab < 0)
                        continue;
                    const entry = {
                        spelling: line.slice(0, tab),
                        name: line.slice(tab + 1)
                    };
                    names[entry.spelling] = entry.name;
                    catalogue.push(entry);
                }
                root.names = names;
                root.catalogue = catalogue;
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
                root.problem = code === 0 ? "" : errors.text.trim().replace(/^kubaos: /, "") || "kubaos keyboard failed";
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
                onClicked: root.change(["default", row.modelData])
            }

            IconButton {
                size: 30
                icon: "arrow_upward"
                tooltip: "Move up"
                visible: row.index > 0
                onClicked: root.move(row.index, -1)
            }

            IconButton {
                size: 30
                icon: "arrow_downward"
                tooltip: "Move down"
                visible: row.index < root.layouts.length - 1
                onClicked: root.move(row.index, 1)
            }

            IconButton {
                size: 30
                icon: "close"
                tooltip: "Remove"
                visible: root.layouts.length > 1
                onClicked: root.change(["remove", row.modelData])
            }
        }
    }

    ListRow {
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
        onClicked: root.change(["accents", root.accents ? "off" : "on"])
    }

    // Adding: every layout and variant not in the list, by name.
    SearchField {
        id: search

        Layout.fillWidth: true
        visible: root.adding
        placeholder: "Search languages and layouts"
        catchEscape: true
        forwardTo: [matchList]
        onEscaped: root.stopAdding()
    }

    NavList {
        id: matchList

        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: root.adding
        vimKeys: false
        model: ScriptModel {
            values: root.matches
        }
        onActivated: index => root.add(root.matches[index].spelling)

        delegate: ListRow {
            required property var modelData
            required property int index

            width: ListView.view.width
            title: modelData.name
            subtitle: modelData.spelling
            current: ListView.isCurrentItem
            onClicked: root.add(modelData.spelling)
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
