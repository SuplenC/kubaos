import QtQuick
import Quickshell
import Quickshell.Io
import KubaOS.Components as K
import qs.components
import qs.services

// The keyboard layouts in one well: the rows, the default first and wearing a filled star, and a
// toolbar at the well's foot with the Mac accents chip, ↑ ↓ to move the focused row, + to add a
// layout and × to remove one. + swaps the rows for the add view (KeyboardAddView.qml) in the same
// well. Every change runs `kubaos keyboard`, the one writer of the layouts, which reloads
// Hyprland; services/Keyboard.qml sees the reload and this list reads them again.
//
// A click on a row types with it and makes it the focused row the toolbar acts on. Dragging any
// row reorders: past the drag threshold it follows the pointer, a drop line shows where it lands,
// and the drop runs `kubaos keyboard move` once; dragged to the top, it is the default.
// Keys in the list: ↑ ↓ (and Ctrl+j/k) move the focus, Enter types with the focused layout,
// Alt+↑ ↓ or Shift+J/K move it, d makes it the default, x or Delete removes it. Tab goes on to
// the toolbar.
//
// A click on anything that changes the layouts is ignored for a double-click's interval after
// the view switches or the rows move, so the second click of a double-click lands on nothing new.
Item {
    id: root

    // The room between the tab and the tabs above it, and the panel's edge below it, which its
    // scroll reaches into.
    property alias roomAbove: pane.roomAbove
    property alias roomBelow: pane.roomBelow

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
    // The sheet the tab is in has stopped moving, so what the probe logs is where things stay.
    property bool settled: true
    // The most a keymap holds (kubaos_core::keyboard::MAX_LAYOUTS).
    readonly property int maxLayouts: 4

    // The rows the well is round: the layouts, or one while they are read.
    readonly property int rows: Math.max(1, root.layouts.length)
    // What the toolbar acts on: the focused layout.
    readonly property int selected: list.currentIndex
    // The row being dragged, how far the pointer has taken it, and the place it lands on if
    // dropped now; -1 while no row is dragged.
    property int dragFrom: -1
    property real dragOffset: 0
    property int dropAt: -1
    // The layout typed with while a move waits for Hyprland's reload: Hyprland's index of it
    // (`Keyboard.active`) is the old order's until then. Empty while none waits.
    property string typing: ""
    // The Mac accents tooltip shown whatever the pointer does: the screenshots' view of it.
    property bool explaining: false

    function nameOf(spelling: string): string {
        return names[spelling] ?? spelling;
    }

    // `kubaos keyboard ARGS`; its stderr is the problem when it fails.
    function change(args: var): void {
        if (busy)
            return;
        busy = true;
        probe(`change ${args.join(" ")}`);
        call.createObject(root, {
            command: ["kubaos", "keyboard", ...args]
        });
    }

    // A change from a click, unless the click may be the second of a double-click meant for what
    // was under the pointer a moment ago.
    function changeOnClick(args: var): void {
        if (!settling)
            change(args);
    }

    // Moves the layout at `from` to `to` at once on screen, then for good: the rows don't wait for
    // the reload, and the next `show` puts them back if the change fails.
    function move(from: int, to: int): void {
        if (busy || from === to || from < 0 || to < 0 || from >= layouts.length || to >= layouts.length)
            return;
        const spelling = layouts[from];
        if (typing === "")
            typing = layouts[Keyboard.active] ?? "";
        change(["move", spelling, String(to + 1)]);
        const order = layouts.slice();
        order.splice(from, 1);
        order.splice(to, 0, spelling);
        layouts = order;
        list.currentIndex = to;
    }

    // Where a row dragged from `from` by `dy` lands: as many places on as whole rows it has passed
    // half of, held to the list's ends.
    function landing(from: int, dy: real): int {
        const pitch = K.Tokens.size.row_two_line + list.spacing;
        return Math.max(0, Math.min(layouts.length - 1, from + Math.round(dy / pitch)));
    }

    // Types with the layout at `index` of the list. Hyprland counts its layouts in its own
    // order, which is the old one while a move waits for its reload, so it waits too.
    function typeWith(index: int): void {
        if (!busy && typing === "")
            Quickshell.execDetached(["hyprctl", "switchxkblayout", "all", String(index)]);
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

    // `kubaos keyboard` says "the keyboard layouts weren't changed: …"; the tab shows it as a
    // sentence.
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
    // The dragged row is the focused one, the toolbar's.
    onDragFromChanged: if (dragFrom >= 0)
        list.currentIndex = dragFrom

    Timer {
        id: settle

        interval: Qt.styleHints.mouseDoubleClickInterval
    }

    // Where the probe's pointer clicks: the centre of each target on screen, logged when it moves,
    // once the sheet has stopped sliding in.
    Timer {
        property string last: ""

        running: root.probed && root.visible && root.settled
        repeat: true
        interval: 200
        onTriggered: {
            const at = (name, item) => {
                if (!item || !item.visible)
                    return "";
                const p = item.mapToItem(null, item.width / 2, item.height / 2);
                return `${name} x=${Math.round(p.x)} y=${Math.round(p.y)}`;
            };
            const matches = adder.item?.list;
            const first = matches?.visible ? matches.itemAtIndex(0) : null;
            const rows = root.adding ? [] : root.layouts.map((_, i) => at(`row-${i}`, list.itemAtIndex(i)));
            const now = [root.adding ? "" : at("add-button", addButton), first ? at("first-match", first) : "", ...rows].filter(s => s !== "").join(" ");
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

    // `kubaos keyboard` reloads Hyprland after a change, and the layouts Hyprland reports change
    // with it.
    Connections {
        target: Keyboard

        function onLayoutsChanged(): void {
            root.typing = "";
            show.running = true;
            accentsRead.running = true;
        }
    }

    Process {
        id: show

        command: ["kubaos", "keyboard", "show"]
        stdout: StdioCollector {
            onStreamFinished: {
                const read = text.split("\n").filter(l => l !== "");
                // The same order again keeps the model, so a row just dropped doesn't flicker.
                if (read.join(",") !== root.layouts.join(","))
                    root.layouts = read;
            }
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
                // Refused, the move never reaches Hyprland, whose order stays the one it counts in.
                if (code !== 0)
                    root.typing = "";
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

    // The tab's content, scrolled when a short screen leaves it less room than two layouts and the
    // toolbar need; otherwise exactly the room it has.
    K.Scroll {
        id: pane

        // The well round `rows` rows: its padding, the rows, the toolbar, and the gaps between them.
        function around(rows: int): real {
            return K.Tokens.space.s12 * 2 + (K.Tokens.size.row_two_line + K.Tokens.space.s4) * rows + K.Tokens.size.control_sm;
        }

        // The well at its smallest: two rows.
        readonly property real fewest: pane.around(2)
        // The problem under the well, and the gap above it, when there is one.
        readonly property real noteRoom: note.visible ? K.Tokens.space.s12 + note.height : 0
        // The room the well may take, above the problem: all the tab has, or two rows' worth.
        readonly property real room: Math.max(pane.height, pane.fewest + pane.noteRoom) - pane.noteRoom

        anchors.fill: parent

        Well {
            id: well

            width: root.width
            // As tall as the layouts and the toolbar, so nothing hangs empty between them; the add
            // view takes all the room, for its matches.
            height: Math.max(0, root.adding ? pane.room : Math.min(pane.room, pane.around(root.rows)))
            focused: list.activeFocus || (adder.item?.listFocused ?? false)

            // The layouts, down to the toolbar.
            NavList {
                id: list

                visible: !root.adding
                width: parent.width
                height: Math.max(0, toolbar.y - K.Tokens.space.s4)
                // The well's padding above the first row, scrolled with them.
                topMargin: K.Tokens.space.s12
                // A mouse drag reorders the rows; the wheel still scrolls them.
                acceptedButtons: Qt.NoButton
                model: ScriptModel {
                    values: root.layouts
                }
                onActivated: index => root.typeWith(index)

                // Plain j/k are this handler's, so Shift+J/K can move a layout; NavList's own
                // still takes the arrows, Ctrl+j/k and Enter.
                vimKeys: false
                Keys.onPressed: event => {
                    const i = currentIndex;
                    const shift = event.modifiers & Qt.ShiftModifier;
                    const alt = event.modifiers & Qt.AltModifier;
                    if (event.modifiers & Qt.ControlModifier || i < 0 || i >= root.layouts.length)
                        return;
                    if (alt && event.key === Qt.Key_Up)
                        root.move(i, i - 1);
                    else if (alt && event.key === Qt.Key_Down)
                        root.move(i, i + 1);
                    else if (event.key === Qt.Key_J)
                        shift ? root.move(i, i + 1) : step(1);
                    else if (event.key === Qt.Key_K)
                        shift ? root.move(i, i - 1) : step(-1);
                    else if (event.key === Qt.Key_D && i > 0)
                        root.move(i, 0);
                    else if ((event.key === Qt.Key_X || event.key === Qt.Key_Delete) && root.layouts.length > 1)
                        root.change(["remove", root.layouts[i]]);
                    else
                        return;
                    event.accepted = true;
                }

                delegate: K.ListRow {
                    id: row

                    required property string modelData
                    required property int index
                    readonly property bool dragged: root.dragFrom === row.index

                    width: ListView.view.width
                    z: row.dragged ? 1 : 0
                    icon: row.index === 0 ? "star_filled" : "keyboard"
                    tile: true
                    accent: row.index === 0
                    title: root.nameOf(row.modelData)
                    subtitle: row.modelData
                    checkable: true
                    checked: root.typing !== "" ? root.typing === row.modelData : Keyboard.active === row.index
                    selected: row.dragged || (ListView.isCurrentItem && ListView.view.highlightShown)
                    // Lifted: the row it passes over shows through, as the place it leaves.
                    opacity: row.dragged ? K.Tokens.opacity.subdued : 1
                    Accessible.name: row.index === 0 ? `${row.title}, the default` : row.title
                    onActivated: {
                        ListView.view.currentIndex = row.index;
                        root.typeWith(row.index);
                    }

                    // The row follows the pointer while it is dragged; a translation is drawn,
                    // not laid out, so the list keeps 60 fps under it.
                    transform: Translate {
                        y: row.dragged ? root.dragOffset : 0
                    }

                    // Over the row, so its handler sees a press before the row's own click does,
                    // and takes it over once the pointer passes the drag threshold.
                    Item {
                        anchors.fill: parent

                        DragHandler {
                            id: grab

                            target: null
                            xAxis.enabled: false
                            enabled: !root.busy && root.layouts.length > 1
                            grabPermissions: PointerHandler.CanTakeOverFromAnything
                            cursorShape: Qt.ClosedHandCursor
                            // HAZARD: how far the pointer has come since the press, not
                            // `activeTranslation`, which counts from the move that passed the drag
                            // threshold. A busy window gets a drag's moves merged into one, and
                            // when that one move is the whole drag the row drops back where it was.
                            readonly property real travel: centroid.scenePosition.y - centroid.scenePressPosition.y

                            onActiveChanged: {
                                root.probe(`drag ${row.index} ${active}`);
                                if (active) {
                                    root.dragOffset = travel;
                                    root.dropAt = root.landing(row.index, travel);
                                    root.dragFrom = row.index;
                                    return;
                                }
                                const from = root.dragFrom;
                                const to = root.dropAt;
                                root.dragFrom = -1;
                                root.dropAt = -1;
                                root.dragOffset = 0;
                                root.move(from, to);
                            }
                            // A drag taken away mid-way (the layouts changed under it) lands nowhere.
                            onCanceled: {
                                root.dragFrom = -1;
                                root.dropAt = -1;
                                root.dragOffset = 0;
                            }
                            onTravelChanged: {
                                if (!active)
                                    return;
                                root.dragOffset = travel;
                                root.dropAt = root.landing(row.index, travel);
                            }
                        }
                    }
                }

                // Where the dragged row lands: a thin line in the gap above the row it goes
                // before, or below the one it goes after, inset as the rows' words are.
                Rectangle {
                    readonly property real pitch: K.Tokens.size.row_two_line + list.spacing
                    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

                    parent: list.contentItem
                    visible: root.dragFrom >= 0 && root.dropAt !== root.dragFrom
                    x: K.Tokens.space.s12
                    width: list.width - K.Tokens.space.s12 * 2
                    height: K.Look.thin(dpr)
                    radius: K.Tokens.radius.full
                    color: K.Tokens.color.primary
                    y: {
                        const gap = root.dropAt < root.dragFrom ? root.dropAt * pitch - list.spacing : root.dropAt * pitch + K.Tokens.size.row_two_line;
                        return K.Look.snap(gap + (list.spacing - height) / 2, dpr);
                    }
                }
            }

            // At the well's foot, its padding in from the edge, as the rows are.
            K.Row {
                id: toolbar

                visible: !root.adding
                y: well.height - K.Tokens.space.s12 - height
                width: parent.width
                room: parent.width
                fill: true
                gap: K.Tokens.space.s4
                align: "centre"

                K.Chip {
                    id: accentsChip

                    text: "Mac accents"
                    icon: "keyboard_option_key"
                    kind: root.accents ? "Selected" : "Neutral"
                    pressable: true
                    onActivated: root.changeOnClick(["accents", root.accents ? "off" : "on"])

                    K.Tooltip {
                        title: "Mac accents on Left Alt"
                        body: `${root.accents ? "On" : "Off"}. Type accents as a Mac does: hold Left Alt with the accent's key, then type the letter.`
                        shown: root.explaining || accentsChip.focused || accentsChip.pointerOver

                        K.Stack {
                            gap: K.Tokens.space.s8

                            K.KeyHints {
                                hints: [["Left Alt+E", "then"], ["E", "types é"]]
                            }

                            K.KeyHints {
                                hints: [["Left Alt+U", "then"], ["O", "types ö"]]
                            }
                        }
                    }
                }

                K.Fill {
                }

                K.IconButton {
                    enabled: root.selected > 0
                    icon: "arrow_upward"
                    label: "Move up"
                    kind: "Text"
                    size: "Sm"
                    onActivated: if (!root.settling)
                        root.move(root.selected, root.selected - 1)
                }

                K.IconButton {
                    enabled: root.selected >= 0 && root.selected < root.layouts.length - 1
                    icon: "arrow_downward"
                    label: "Move down"
                    kind: "Text"
                    size: "Sm"
                    onActivated: if (!root.settling)
                        root.move(root.selected, root.selected + 1)
                }

                // Beside ×, so add and remove read as a pair.
                K.IconButton {
                    id: addButton

                    enabled: root.layouts.length < root.maxLayouts
                    icon: "add"
                    label: root.layouts.length < root.maxLayouts ? "Add a language" : `${root.maxLayouts} is the most a keyboard holds`
                    kind: "Tonal"
                    size: "Sm"
                    onActivated: root.startAdding()
                }

                K.IconButton {
                    enabled: root.selected >= 0 && root.layouts.length > 1
                    icon: "close"
                    label: "Remove"
                    kind: "Text"
                    size: "Sm"
                    onActivated: root.changeOnClick(["remove", root.layouts[root.selected]])
                }
            }

            // Adding: in the same well, made while it shows and gone once it closes.
            Loader {
                id: adder

                width: parent.width
                height: parent.height
                active: root.adding
                visible: active
                sourceComponent: KeyboardAddView {
                    onAdd: spelling => {
                        if (!root.settling)
                            root.add(spelling);
                    }
                    onAddAtOnce: spelling => root.add(spelling)
                    onCancelled: root.stopAdding()
                    onEnteredChanged: root.probe(`entered ${entered}`)
                    onFailed: message => root.problem = message
                }
            }
        }

        K.Text {
            id: note

            y: well.height + K.Tokens.space.s12
            visible: root.problem !== ""
            text: root.problem
            room: root.width
            style: K.Tokens.type.body
            color: K.Tokens.color.error
        }
    }
}
