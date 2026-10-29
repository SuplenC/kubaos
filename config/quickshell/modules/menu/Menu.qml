import QtQuick
import Quickshell
import Quickshell.Wayland
import Kotodama
import KubaOS.Components as K
import qs.components
import qs.services
import qs.settings

// The Command menu: KubaOS's own front end on Kotodama's `KotodamaSearch`, built from
// `KubaOS.Components`. A search field over the Daemon's Results, a detail pane beside them when
// the screen has room, the Views Extensions push, the jobs they run, and each Result's Action
// panel. Kotodama's `KotodamaMenu.qml` handles the same calls; this file follows it call for call.
//
// `kotodama toggle` (SUPER + Space, config/hypr/fallback.lua) opens it through the Daemon, so it
// has no IPC target of its own. Hyprland blurs what shows through its translucent panel (the
// `kotodama` layer, config/hypr/rules.lua), and leaves its motion to it.
//
// Speed comes first: a keystroke's Results reach the screen in the frame they arrive. Nothing waits
// on an animation, the rows are cheap (ResultRow.qml), and the detail pane updates only once a
// frame with the Results is on screen and typing has paused. The panel's frame (the field, the
// footer, the cards) is the library's components on its layout primitives, laid out while the menu
// is built; what changes with every key is in `Label`s on token arithmetic, which costs a keystroke
// nothing to lay out.
PanelWindow {
    id: menu

    // It opens on the focused screen, and stays there while it is open, even when the pointer
    // crosses to another screen and takes the focus with it.
    Binding on screen {
        value: Hypr.focusedScreen
        when: !menu.visible
        restoreMode: Binding.RestoreNone
    }

    // The search under a name its children's own `search` properties don't shadow.
    readonly property KotodamaSearch engine: search
    readonly property string placeholder: "Search apps, projects and windows. Sums work too"

    // The panel's measures on the screen it opens on. Every one is a token or a whole count of
    // one, so the panel's edges land on the 4 px grid, and so on whole device pixels at every
    // supported scale.
    readonly property int screenWidth: menu.screen?.width ?? 0
    readonly property int screenHeight: menu.screen?.height ?? 0
    readonly property real dpr: menu.screen?.devicePixelRatio ?? 1
    // The panel's padding, and the room between its parts: the field, the body and the footer, the
    // list and the pane. Rows 12 in nest their corners in the panel's.
    readonly property int gap: K.Tokens.space.s12
    // Half the screen's width, from one column to two.
    readonly property int panelWidth: Math.max(K.Tokens.size.column, Math.min(K.Tokens.size.column * 2, menu.onGrid(menu.screenWidth / 2)))
    // The detail pane shows on a panel as wide as a dialog beside a rail: a 1536 wide screen (1080p
    // at 1.25) has it, a 1280 wide one doesn't. It takes two fifths of the room, a rail at the
    // least.
    readonly property bool paneFits: menu.panelWidth >= K.Tokens.size.dialog + K.Tokens.size.rail
    readonly property real paneShare: 0.4
    readonly property int paneWidth: menu.paneFits ? Math.max(K.Tokens.size.rail, menu.onGrid((menu.panelWidth - menu.gap * 3) * menu.paneShare)) : 0
    // A row and the gap under it.
    readonly property int pitch: K.Tokens.size.row_two_line + K.Tokens.space.s4
    // Everything but the body: the gaps round the field, the body and the footer (the footer's
    // divider sits in one), and the field and the footer themselves.
    readonly property int chrome: menu.gap * 5 + K.Tokens.size.control_md + K.Tokens.size.control_sm
    // Rows fill about two thirds of the screen's height: a taller screen shows more of them. Eight
    // at the least, the e2e harness's 1280×800 included.
    readonly property int rows: Math.max(menu.minRows, Math.min(menu.maxRows, Math.floor((menu.screenHeight * 0.64 - menu.chrome) / menu.pitch)))
    readonly property int minRows: 8
    readonly property int maxRows: 14
    readonly property int bodyHeight: menu.rows * menu.pitch - K.Tokens.space.s4
    readonly property int panelHeight: menu.chrome + menu.bodyHeight
    // Two fifths of the height it leaves free above it, as the screen's optical centre.
    readonly property int top: menu.onGrid((menu.screenHeight - menu.panelHeight) * 0.4)

    // Whether things may move, not only fade.
    readonly property bool moves: Settings.menuMotion !== "fades"
    // How far the window drops in as it opens, and a View slides as it is pushed or popped.
    readonly property int openTravel: menu.moves ? K.Tokens.space.s8 : 0
    readonly property int viewTravel: menu.moves ? K.Tokens.space.s24 : 0

    // Why the last Action failed, until the next Action or opening.
    property string failure: ""
    // The top View's kind (`list`, `form`, `detail` or `progress`), or empty at the search.
    readonly property string kind: search.view.kind ?? ""
    // Whether the body is a list: the Results, or a list View's Items.
    readonly property bool listing: kind === "" || kind === "list"
    readonly property bool paneShown: menu.paneFits && listing
    // The Action panel of the selected row while Ctrl+K shows it: each entry
    // `{title, group, shortcut}`, as `KotodamaSearch.actions` gives them.
    property var actions: []
    readonly property bool actionsOpen: actions.length > 0
    // The keys the selected row takes, for the footer when there is no pane to show them. Reading
    // the row's markup and unavailable makes them follow a new Result in the same place, not only a
    // move of the selection.
    readonly property string hints: {
        const row = list.currentItem;
        if (menu.paneShown || !row || !menu.listing)
            return "";
        row.markup;
        row.unavailable;
        return search.hints(list.currentIndex);
    }
    // What Esc does now, as the footer names it.
    readonly property string escapeHint: {
        if (kind === "list" && filter.text !== "" || kind === "" && input.text !== "")
            return "Esc Clear";
        if (search.depth > 0)
            return "Esc Back";
        return search.scope !== "" ? "Esc Leave app" : "Esc Close";
    }
    readonly property string footerHints: {
        if (kind === "form")
            return `Tab Next field · ↵ ${search.view.submit ?? "Submit"} · ${escapeHint}`;
        if (!listing)
            return escapeHint;
        return paneShown ? `↑↓ Select · ⌃K Actions · ${escapeHint}` : hints;
    }

    // The selected row, by what it is: a new key is a new selection, which the pane crossfades to.
    readonly property string selectedKey: {
        const row = list.currentItem;
        return row && menu.listing ? `${row.extension}\n${row.title}\n${row.subtitle}` : "";
    }
    // The pane is behind the selection, until the next frame is on screen.
    property bool paneStale: true
    // Typing moved it there, so the pane swaps rather than crossfades.
    property bool paneTyped: false

    // 0 hidden, 1 open; the window fades and drops in between.
    property real openness: 0
    // How far the body has yet to slide, and how faded it still is, after a View was pushed or
    // popped.
    property real slide: 0
    property real settle: 1
    property int lastDepth: 0
    // Results a keystroke changed show at once. While any animation runs, each frame waits on the
    // screen's next refresh, so the rows and the highlight move only for changes nobody typed, and
    // the pane catches up when typing pauses: the selection's glide after the last key.
    readonly property bool typing: keystroke.running

    // The Daemon types or pastes once the window is gone and focus is back where the user came
    // from, so it is told only after the window unmaps.
    property bool closeAsked: false

    // `v` down to the 4 px grid.
    function onGrid(v: real): int {
        return Math.floor(v / K.Tokens.rules.grid) * K.Tokens.rules.grid;
    }

    function show(query: string): void {
        menu.failure = "";
        input.text = query;
        input.cursorPosition = query.length;
        // The box may show `query` already, in an app it has left.
        search.query = query;
        list.pointer = null;
        menu.paneStale = true;
        if (!visible || closing.running) {
            closing.stop();
            if (!visible)
                openness = 0;
            visible = true;
            opening.restart();
        }
        menu.focusView();
    }

    // Closing pops no View: Extensions aren't told the user left one, so their jobs keep running,
    // and the menu opens again where it was. Only Esc pops.
    function hide(): void {
        menu.actions = [];
        search.cancelConfirm();
        if (!visible || closing.running)
            return;
        opening.stop();
        closing.restart();
    }

    // Gives keys to the top View: the search field, a list's filter, the first field of a form,
    // or the menu itself for a detail or a progress View.
    function focusView(): void {
        if (menu.actionsOpen)
            actionPanel.forceActiveFocus();
        else if (menu.kind === "")
            input.forceActiveFocus();
        else if (menu.kind === "list")
            filter.forceActiveFocus();
        else if (menu.kind === "form" && form.item?.takeFirst())
            return;
        else
            viewKeys.forceActiveFocus();
    }

    // Where the keys go, in `focusView`'s order, once the compositor has given the window the
    // keyboard: the probe's `keys` line.
    function keysAt(): string {
        if (!panel.Window.window?.active)
            return "";
        if (actionPanel.activeFocus)
            return "actions";
        if (input.activeFocus)
            return "search";
        if (filter.activeFocus)
            return "filter";
        if (form.item?.activeFocus)
            return "form";
        return viewKeys.activeFocus ? "view" : "";
    }

    function openActions(): void {
        const actions = search.actions(list.currentIndex);
        if (actions.length > 0) {
            actionPanel.row = list.currentIndex;
            actionPanel.current = 0;
            menu.actions = actions;
            actionPanel.forceActiveFocus();
        }
    }

    function closeActions(): void {
        menu.actions = [];
        menu.focusView();
    }

    function runAction(n: int): void {
        menu.failure = "";
        search.runAction(actionPanel.row, n);
        menu.closeActions();
    }

    // The pane shows the selection, if it is behind and typing has paused.
    function catchUp(): void {
        if (paneStale && paneShown && !typing) {
            paneStale = false;
            pane.show(paneEntry(), !paneTyped);
            paneTyped = false;
        }
    }

    // What the detail pane shows of the selected row, or null for none.
    function paneEntry(): var {
        const row = list.currentItem;
        if (!row)
            return null;
        return {
            key: menu.selectedKey,
            title: row.title,
            subtitle: row.subtitle,
            icon: row.icon,
            extension: row.extension,
            alias: row.alias,
            unavailable: row.unavailable,
            actions: search.actions(list.currentIndex),
            hints: search.hints(list.currentIndex)
        };
    }

    // Ctrl and no other modifier: the menu's own Ctrl keys leave Ctrl+Shift+K and the like to the
    // Actions' key bindings.
    function ctrlOnly(event: KeyEvent): bool {
        return (event.modifiers & (Qt.ControlModifier | Qt.ShiftModifier | Qt.AltModifier | Qt.MetaModifier)) === Qt.ControlModifier;
    }

    // Whether `event` is a modifier pressed on its own, as on the way to a key binding.
    function modifierAlone(event: KeyEvent): bool {
        return [Qt.Key_Shift, Qt.Key_Control, Qt.Key_Alt, Qt.Key_Meta, Qt.Key_Super_L, Qt.Key_Super_R].includes(event.key);
    }

    // While the Toast shows, its key undoes what it offers to, whatever has the keys; otherwise
    // the key is the text field's own undo. True if `event` was it.
    function undoKey(event: KeyEvent): bool {
        if (search.toast === "" || !search.isUndoKey(event.key, event.modifiers))
            return false;
        search.undo();
        return true;
    }

    // The keys every View shares. True if `event` was one of them.
    function key(event: KeyEvent): bool {
        const ctrl = menu.ctrlOnly(event);
        const enter = event.key === Qt.Key_Return || event.key === Qt.Key_Enter;
        if (menu.undoKey(event))
            return true;
        // An Action that asks first waits in the footer: Enter runs it, Esc drops it, and any
        // other key drops it and does what it does.
        if (search.confirming !== "" && !menu.modifierAlone(event)) {
            if (enter) {
                menu.failure = "";
                search.confirm();
                return true;
            }
            search.cancelConfirm();
            if (event.key === Qt.Key_Escape)
                return true;
        }
        if (event.key === Qt.Key_Escape) {
            // One rule everywhere, so Esc never throws away more than the user can see: it clears
            // the text if there is any, else pops the View, else leaves the app for the whole
            // search, else closes the menu.
            if (menu.kind === "list" && filter.text !== "")
                filter.text = "";
            else if (menu.kind === "" && input.text !== "")
                input.text = "";
            else if (search.depth > 0)
                search.popView();
            else if (search.leaveApp())
                menu.failure = "";
            else
                menu.hide();
            return true;
        }
        if (menu.kind === "form") {
            if (enter) {
                menu.failure = "";
                search.submit();
            } else if (event.key === Qt.Key_Tab || event.key === Qt.Key_Down) {
                form.item?.move(1);
            } else if (event.key === Qt.Key_Backtab || event.key === Qt.Key_Up) {
                form.item?.move(-1);
            } else {
                return false;
            }
            return true;
        }
        if (!menu.listing)
            return false;
        if (ctrl && event.key === Qt.Key_K) {
            menu.openActions();
        } else if (event.key === Qt.Key_Tab && menu.kind === "" && search.enter(list.currentIndex)) {
            // Tab on an app's entry Item enters the app.
            menu.failure = "";
        } else if (enter) {
            menu.failure = "";
            search.run(list.currentIndex);
        } else if (event.key === Qt.Key_Down || (ctrl && event.key === Qt.Key_N)) {
            list.incrementCurrentIndex();
        } else if (event.key === Qt.Key_Up || (ctrl && event.key === Qt.Key_P)) {
            list.decrementCurrentIndex();
        } else if (event.key === Qt.Key_PageDown) {
            list.currentIndex = Math.min(list.count - 1, list.currentIndex + menu.rows);
        } else if (event.key === Qt.Key_PageUp) {
            list.currentIndex = Math.max(0, list.currentIndex - menu.rows);
        } else if ((event.modifiers & (Qt.ControlModifier | Qt.AltModifier | Qt.MetaModifier)) && search.runShortcut(list.currentIndex, event.key, event.modifiers)) {
            menu.failure = "";
        } else {
            return false;
        }
        return true;
    }

    visible: false
    // Anchored at the top only, so the compositor centres it horizontally.
    anchors.top: true
    margins.top: menu.top
    // The surface keeps one size for the screen, so typing never waits on a resize, and nothing
    // jumps as Results, the pane or a status come and go.
    implicitWidth: menu.panelWidth
    implicitHeight: menu.panelHeight
    exclusionMode: ExclusionMode.Ignore
    color: "transparent"
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.namespace: "kotodama"
    // Held only while the pointer is on the menu's screen, so the other screens stay usable while
    // it is open.
    WlrLayershell.keyboardFocus: visible && !closing.running && Hypr.focusedScreen === screen ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

    // However the window opened, the Daemon is told, so the Empty query it shows is ranked for
    // this time of day.
    onVisibleChanged: {
        if (visible)
            search.shown();
        if (!visible && closeAsked) {
            closeAsked = false;
            Qt.callLater(search.closed);
        }
    }
    onPaneShownChanged: paneStale = true
    onSelectedKeyChanged: paneStale = true
    onTypingChanged: catchUp()

    // `kotodama toggle` and `open` that reached another of the shell's searches.
    Connections {
        target: MenuRequests

        function onToggleRequested(): void {
            menu.visible && !closing.running ? menu.hide() : menu.show("");
        }

        function onOpenRequested(query: string): void {
            menu.show(query);
        }
    }

    KotodamaSearch {
        id: search

        highlightColor: K.Tokens.color.primary
        onOpenRequested: (query, view) => menu.show(query)
        onToggleRequested: menu.visible && !closing.running ? menu.hide() : menu.show("")
        onCloseRequested: {
            if (menu.visible) {
                menu.closeAsked = true;
                menu.hide();
            } else {
                search.closed();
            }
        }
        onActionFailed: message => menu.failure = message
        // The field shows the rest of the query once it is in an app, or what the Daemon typed.
        // The cursor keeps its place from the end.
        onQueryReplaced: text => {
            const fromEnd = input.text.length - input.cursorPosition;
            input.text = text;
            input.cursorPosition = Math.max(0, text.length - fromEnd);
        }
        // A list View's rows changed: its selected Item is at `row`, or the top row is selected,
        // as in the search, if it went away.
        onSelectRow: row => {
            list.currentIndex = row;
            follow.stop();
            if (row === 0)
                list.positionViewAtBeginning();
            else
                list.positionViewAtIndex(row, ListView.Contain);
        }
        // A View pushed slides in from the right, one popped from the left.
        onDepthChanged: {
            menu.actions = [];
            list.currentIndex = 0;
            follow.stop();
            list.positionViewAtBeginning();
            viewMove.from = search.depth > menu.lastDepth ? menu.viewTravel : -menu.viewTravel;
            menu.lastDepth = search.depth;
            if (menu.visible)
                viewIn.restart();
        }
        // A View that is pushed, popped or changed takes the keys; one whose job progressed keeps
        // them where they are. A list's filter shows what was typed into it, which is nothing for
        // a new one.
        onViewChanged: {
            const typed = search.view.filter ?? "";
            if (filter.text !== typed)
                filter.text = typed;
            if (menu.visible && !menu.actionsOpen)
                Qt.callLater(menu.focusView);
        }
    }

    Timer {
        id: keystroke

        interval: K.Tokens.duration.short
    }

    // The Command menu's motion is one step quicker than a panel's (docs/DESIGN.md, "Motion"):
    // a key opens and closes it many times an hour, and what it runs waits for it to go.
    NumberAnimation {
        id: opening

        target: menu
        property: "openness"
        to: 1
        duration: K.Tokens.duration.short
        easing.type: Easing.BezierSpline
        easing.bezierCurve: K.Tokens.easing.emphasized_decel
    }

    NumberAnimation {
        id: closing

        target: menu
        property: "openness"
        to: 0
        duration: K.Tokens.duration.micro
        easing.type: Easing.BezierSpline
        easing.bezierCurve: K.Tokens.easing.emphasized_accel
        onFinished: menu.visible = false
    }

    // The body arrives readable: it starts part faded, so a pushed list's rows show in the View's
    // first frame.
    ParallelAnimation {
        id: viewIn

        NumberAnimation {
            id: viewMove

            target: menu
            property: "slide"
            to: 0
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.emphasized_decel
        }

        NumberAnimation {
            target: menu
            property: "settle"
            from: 0.4
            to: 1
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.emphasized_decel
        }
    }

    Item {
        id: panel

        anchors.fill: parent
        opacity: menu.openness
        transform: Translate {
            y: menu.openTravel * menu.openness - menu.openTravel
        }

        // The panel: a surface of the frame's glass, which Hyprland blurs behind, in a hairline.
        Rectangle {
            anchors.fill: parent
            radius: K.Tokens.radius.surface
            color: K.Look.alpha(K.Tokens.color.surface, K.Tokens.opacity.frame)
            border.width: K.Look.hairline(menu.dpr)
            border.color: K.Look.alpha(K.Tokens.color.outline_variant, K.Tokens.opacity.outline_soft)
        }

        // The field, and the jobs still running under it.
        K.Stack {
            id: head

            x: menu.gap
            y: menu.gap
            width: panel.width - menu.gap * 2
            fill: true
            gap: menu.gap

            // Back out of a View; the app the search is in, such as Emoji, with the field holding
            // what follows the app's word; the field, a list View's filter, or another View's
            // title.
            K.Row {
                id: header

                fill: true
                fixedHeight: K.Tokens.size.control_md
                gap: K.Tokens.space.s8
                align: "centre"

                K.IconButton {
                    visible: search.depth > 0
                    icon: "arrow_back"
                    label: "Back"
                    kind: "Text"
                    activeFocusOnTab: false
                    onActivated: search.popView()
                }

                K.Chip {
                    objectName: "scope"
                    visible: menu.kind === "" && search.scope !== ""
                    text: search.scope
                    kind: "Selected"
                }

                K.Field {
                    id: input

                    visible: menu.kind === ""
                    icon: "search"
                    placeholder: search.scope !== "" ? `Search ${search.scope}` : menu.placeholder
                    focus: true
                    onTextChanged: {
                        keystroke.restart();
                        menu.paneTyped = true;
                        search.query = text;
                        list.currentIndex = 0;
                        list.positionViewAtBeginning();
                    }
                    // Backspace with nothing typed in an app leaves it: the whole search again,
                    // with the app's word typed.
                    onKeyPressed: event => {
                        if (event.key === Qt.Key_Backspace && input.text === "" && search.leaveScope())
                            event.accepted = true;
                        else
                            event.accepted = menu.key(event);
                    }
                }

                // A list View's filter, with the View's title until something is typed. The
                // Daemon matches what is typed as it does the search, so the rows narrow as they
                // would there. The selection stays where it is until the matches come; see
                // onSelectRow.
                K.Field {
                    id: filter

                    visible: menu.kind === "list"
                    icon: "search"
                    placeholder: search.view.title ?? ""
                    onTextChanged: {
                        keystroke.restart();
                        menu.paneTyped = true;
                        search.filter(text);
                    }
                    onKeyPressed: event => event.accepted = menu.key(event)
                }

                // It takes the rest of the row, and a long one ends in an ellipsis.
                Label {
                    readonly property bool fillWidth: true

                    visible: !menu.listing
                    text: search.view.title ?? ""
                    type: K.Tokens.type.subheading
                }
            }

            // The jobs still running: each one's state, its title and what it last said, and its
            // bar while it knows how far it has got.
            K.Stack {
                visible: menu.kind !== "progress" && search.jobs.length > 0
                fill: true
                gap: K.Tokens.space.s8

                Repeater {
                    model: search.jobs

                    K.Grid {
                        id: running

                        required property var modelData
                        readonly property bool failed: modelData.state === "failed"
                        readonly property bool succeeded: modelData.state === "succeeded"
                        readonly property bool known: modelData.progress >= 0

                        weights: [3, 1]
                        gap: K.Tokens.space.s12
                        centred: true

                        K.Row {
                            gap: K.Tokens.space.s8
                            align: "centre"

                            // Only a job that can't tell how far it has got spins: a running
                            // animation makes every frame wait on the screen's refresh, typing's
                            // included.
                            K.Spinner {
                                visible: !running.failed && !running.succeeded && !running.known
                                running: menu.moves && menu.visible
                                size: K.Tokens.icon.s16
                            }

                            K.Icon {
                                visible: running.failed || running.succeeded || running.known
                                name: running.failed ? "error" : running.succeeded ? "check_circle" : "hourglass_top"
                                size: K.Tokens.icon.s16
                                color: running.failed ? K.Tokens.color.error : K.Tokens.color.primary
                            }

                            K.Text {
                                text: running.modelData.title + (running.modelData.message !== "" ? ` · ${running.modelData.message}` : "")
                                style: K.Tokens.type.small
                                color: running.failed ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
                            }
                        }

                        K.ProgressBar {
                            visible: !running.failed && running.modelData.progress >= 0
                            value: running.modelData.progress
                        }
                    }
                }
            }
        }

        // Takes the keys of a View with nothing to type into.
        Item {
            id: viewKeys

            Keys.onPressed: event => event.accepted = menu.key(event)
        }

        // The panel inside its hairline, which clips the body as it slides: a View pushed or popped
        // travels further than the panel's padding, and it is cut at the panel's edge, never drawn
        // over the hairline or past it.
        Item {
            id: inside

            readonly property real edge: K.Look.hairline(menu.dpr)

            x: inside.edge
            y: inside.edge
            width: panel.width - inside.edge * 2
            height: panel.height - inside.edge * 2
            clip: true
        }

        // Results and list Views, the pane, or the other Views, in the panel's clip. It slides as a
        // View is pushed or popped.
        Item {
            id: body

            parent: inside
            x: menu.gap - inside.edge
            y: head.y + head.height + menu.gap - inside.edge
            width: panel.width - menu.gap * 2
            height: footer.y - menu.gap * 2 - head.y - head.height - menu.gap
            opacity: menu.settle
            transform: Translate {
                x: menu.slide
            }

            Item {
                id: listArea

                width: parent.width - (menu.paneShown ? menu.paneWidth + menu.gap : 0)
                height: parent.height
                visible: menu.listing

                // An app's intro card, before anything is typed in it. It comes with the app's
                // first Results, so the first frame in the app shows it.
                IntroCard {
                    id: intro

                    objectName: "intro"
                    width: parent.width
                    visible: menu.kind === "" && search.introAbout !== ""
                    note: search.introNote
                    about: search.introAbout
                    examples: search.introExamples
                }

                ListView {
                    id: list

                    y: intro.visible ? intro.height + menu.gap : 0
                    width: parent.width
                    height: parent.height - y
                    clip: true
                    model: search
                    spacing: K.Tokens.space.s4
                    // Pooled rows and the remove transition together leave removed rows on screen,
                    // so rows aren't reused; at fourteen rows at most, making them is cheap.
                    reuseItems: false
                    boundsBehavior: Flickable.StopAtBounds
                    highlightFollowsCurrentItem: false
                    onCountChanged: {
                        if (currentIndex < 0 && count > 0)
                            currentIndex = 0;
                    }
                    // A list View's filter keeps the selected Item selected. In the search, moving
                    // off the top row stops Late Results reranking the rows.
                    onCurrentIndexChanged: {
                        search.select(currentIndex);
                        reveal(currentIndex);
                    }

                    // The view scrolls to keep the selection on screen. With the highlight not
                    // following the current row, ListView doesn't do that itself, so it is
                    // done here, on the selection's glide. Rows are all one height.
                    function reveal(index: int): void {
                        if (index < 0)
                            return;
                        const top = originY + index * menu.pitch;
                        const bottom = top + K.Tokens.size.row_two_line;
                        let to = contentY;
                        if (top < contentY)
                            to = top;
                        else if (bottom > contentY + height)
                            to = bottom - height;
                        to = Math.max(originY, Math.min(to, originY + contentHeight - height));
                        follow.stop();
                        if (to === contentY)
                            return;
                        if (menu.moves && !menu.typing && menu.visible) {
                            follow.to = to;
                            follow.start();
                        } else {
                            contentY = to;
                        }
                    }

                    // The selected row is wholly in view, for the probe.
                    readonly property bool currentShown: currentIndex >= 0 && originY + currentIndex * menu.pitch >= contentY - 0.5 && originY + currentIndex * menu.pitch + K.Tokens.size.row_two_line <= contentY + height + 0.5

                    NumberAnimation {
                        id: follow

                        target: list
                        property: "contentY"
                        duration: K.Tokens.duration.short
                        easing.type: Easing.BezierSpline
                        easing.bezierCurve: K.Tokens.easing.standard
                    }

                    // Once the rerank after a keystroke is over, Late Results go below the rows on
                    // screen, so the Daemon is told which those are.
                    readonly property int firstRow: Math.floor(contentY / menu.pitch)
                    onFirstRowChanged: search.viewport(firstRow, menu.rows)

                    // Where the pointer last was in the window, or null before it has been over a
                    // row since the menu opened.
                    property var pointer: null

                    // Only a pointer that moved picks a row. A window that maps under a resting
                    // pointer gets a pointer-enter, and Qt resends hover when rows move under a
                    // still pointer: either would otherwise take the selection off the top row
                    // the user is typing for.
                    function hover(index: int, area: Item, x: real, y: real): void {
                        const at = area.mapToItem(null, x, y);
                        const moved = list.pointer !== null && (at.x !== list.pointer.x || at.y !== list.pointer.y);
                        list.pointer = at;
                        if (moved)
                            list.currentIndex = index;
                    }

                    // The selection slides between rows as focus moves. It never holds rows back:
                    // they are in place at once. Its corners nest in the panel's, 12 in.
                    highlight: Rectangle {
                        width: list.width
                        height: K.Tokens.size.row_two_line
                        y: list.currentItem?.y ?? 0
                        radius: K.Look.nested(K.Tokens.radius.surface, menu.gap)
                        color: K.Tokens.color.secondary_container
                        visible: list.count > 0

                        Behavior on y {
                            enabled: menu.moves && !menu.typing

                            NumberAnimation {
                                duration: K.Tokens.duration.short
                                easing.type: Easing.BezierSpline
                                easing.bezierCurve: K.Tokens.easing.standard
                            }
                        }
                    }

                    // Rows fade in and out as the Results change, but for a keystroke (`typing`).
                    // A row's opacity on its own is cheap; no row gets a layer.
                    add: Transition {
                        enabled: !menu.typing

                        NumberAnimation {
                            property: "opacity"
                            from: 0
                            to: 1
                            duration: K.Tokens.duration.short
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: K.Tokens.easing.standard
                        }
                    }
                    remove: Transition {
                        enabled: !menu.typing

                        NumberAnimation {
                            property: "opacity"
                            to: 0
                            duration: K.Tokens.duration.short
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: K.Tokens.easing.standard_accel
                        }
                    }
                    displaced: Transition {
                        enabled: menu.moves && !menu.typing

                        NumberAnimation {
                            property: "y"
                            duration: K.Tokens.duration.medium
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: K.Tokens.easing.standard_decel
                        }
                        // A row displaced mid-fade comes back whole.
                        NumberAnimation {
                            property: "opacity"
                            to: 1
                            duration: K.Tokens.duration.short
                        }
                    }
                    // A Result that moves mid-fade comes back whole too: with no `move` transition
                    // the view cancels the fade where it is. Moved rows still jump to their place.
                    move: Transition {
                        NumberAnimation {
                            property: "opacity"
                            to: 1
                            duration: K.Tokens.duration.short
                        }
                    }

                    // A run names the Results of the last frame on screen, so the Daemon refuses a
                    // Late Result the user hasn't seen yet. afterAnimating comes on this thread
                    // just before the frame takes the model; frameSwapped comes queued from the
                    // render thread, after Results that arrived since may have changed it. The
                    // pane catches up on the selection once a frame with the Results is out and
                    // typing has paused, so it never holds a keystroke's Results back.
                    Connections {
                        target: list.Window.window

                        function onAfterAnimating(): void {
                            search.frameStarted();
                        }

                        function onFrameSwapped(): void {
                            search.painted();
                            menu.catchUp();
                        }
                    }

                    delegate: ResultRow {
                        id: row

                        onPointed: (x, y) => list.hover(row.index, row, x, y)
                        onPicked: {
                            menu.failure = "";
                            search.run(row.index);
                        }
                    }
                }

                // Laid out once, while the menu is built, and shown by its opacity, so showing it
                // never waits on a layout pass.
                K.EmptyState {
                    readonly property bool typed: (menu.kind === "list" ? filter.text : input.text) !== ""

                    y: list.y + K.Look.snap((list.height - implicitHeight) / 2, menu.dpr)
                    width: list.width
                    opacity: list.count === 0 ? 1 : 0
                    icon: "search_off"
                    title: typed ? "No results" : "Nothing here yet"
                    body: typed ? "Nothing matches that. Fewer letters, or check the spelling." : "This list is empty. Come back when it isn't."
                }
            }

            DetailPane {
                id: pane

                x: parent.width - width
                width: menu.paneWidth
                height: parent.height
                visible: menu.paneShown
            }

            Loader {
                id: form

                anchors.fill: parent
                active: menu.kind === "form"
                sourceComponent: FormView {
                    // Up to the field above and the footer's divider below: fields scrolled past
                    // the body are cut at those edges.
                    roomAbove: menu.gap
                    roomBelow: menu.gap
                    search: menu.engine
                    keys: event => menu.key(event)
                    fields: search.view.fields ?? []
                    submitLabel: search.view.submit ?? "Submit"
                    onSubmitted: {
                        menu.failure = "";
                        search.submit();
                    }
                }
                onLoaded: {
                    if (menu.visible)
                        Qt.callLater(menu.focusView);
                }
            }

            Loader {
                anchors.fill: parent
                active: menu.kind === "detail"
                sourceComponent: DetailView {
                    markdown: search.view.markdown ?? ""
                    metadata: search.view.metadata ?? []
                }
            }

            Loader {
                anchors.fill: parent
                active: menu.kind === "progress"
                sourceComponent: ProgressView {
                    job: search.view.job ?? ({})
                    moves: menu.moves
                }
            }
        }

        // A form scrolled up keeps the gap under the field clear, and fades in under it, as a
        // panel's list does under its header.
        HeaderFade {
            parent: inside
            // Over the body, whichever of the two `inside` took first.
            z: 1
            y: head.y + head.height - inside.edge
            width: inside.width
            gap: menu.gap
            scrolled: (form.item?.contentY ?? 0) > 0
        }

        // What Forget or "Clear history" just did, and the key that undoes it, over the bottom of
        // the list. It never takes the keys or the selection, and goes when the Daemon withdraws
        // it, once the change is undone or too old to be.
        K.Toast {
            id: toast

            // The last Toast's words, kept while it fades out.
            property string said: ""

            x: K.Look.snap((panel.width - width) / 2, menu.dpr)
            y: footerRule.y - menu.gap - height
            room: panel.width - menu.gap * 2
            shown: footer.status === "" && search.toast !== ""
            text: `${toast.said}. ⌃Z undoes it.`

            Connections {
                target: search

                function onToastChanged(): void {
                    if (search.toast !== "")
                        toast.said = search.toast;
                }
            }
        }

        K.Divider {
            id: footerRule

            // A gap under the body, inside the gap over the footer, so the panel stays a whole
            // number of pixels tall.
            y: footer.y - menu.gap
            width: panel.width
        }

        // The footer: what the user should know now (an Action asking first, why one failed,
        // Extensions that are down), else the keys the menu takes; and on the right, while the
        // Daemon knows of any, how many problems there are, which a click lists.
        Item {
            id: footer

            readonly property string status: search.confirming !== "" ? search.confirming : menu.failure !== "" ? menu.failure : ""
            // A confirmation reads `Power Off? ↵ Confirm · Esc Cancel`: the question, then its
            // keys.
            readonly property int asks: search.confirming.lastIndexOf("? ") + 1
            // Where the left side's words end: before the problems, 12 clear.
            readonly property real room: problems.visible ? problems.x - K.Tokens.space.s12 : footer.width

            x: menu.gap
            y: panel.height - menu.gap - height
            width: panel.width - menu.gap * 2
            height: K.Tokens.size.control_sm

            Label {
                width: footer.room
                height: parent.height
                visible: search.confirming === "" && (menu.failure !== "" || search.down.length > 0)
                type: K.Tokens.type.small
                text: menu.failure !== "" ? menu.failure : `Unavailable: ${search.down.join(", ")}`
                color: menu.failure !== "" ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
            }

            Label {
                id: question

                width: Math.min(K.Look.snapUp(implicitWidth, menu.dpr), footer.room)
                height: parent.height
                visible: search.confirming !== ""
                type: K.Tokens.type.label
                text: search.confirming.slice(0, footer.asks)
                color: K.Tokens.color.error
            }

            // The keys, as keycaps: the confirmation's after its question, else the menu's own. A
            // hint the room the problems leave can't hold is left out whole.
            KeyHints {
                x: search.confirming !== "" ? question.width + K.Tokens.space.s12 : 0
                y: K.Look.snap((parent.height - height) / 2, menu.dpr)
                room: footer.room - x
                visible: search.confirming !== "" || footer.status === "" && search.down.length === 0
                text: search.confirming !== "" ? search.confirming.slice(footer.asks + 1) : menu.footerHints
                color: search.confirming !== "" ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
            }

            K.Button {
                id: problems

                x: footer.width - width
                visible: search.problems > 0
                text: search.problems === 1 ? "1 problem" : `${search.problems} problems`
                kind: "Text"
                size: "Sm"
                activeFocusOnTab: false
                onActivated: search.showProblems()
            }
        }

        // The Action panel: Ctrl+K lists the selected Result's Actions over the bottom right of
        // the list. Up and Down choose, Enter runs, Esc or Ctrl+K closes it. It arrives and goes as
        // the window does.
        ActionPanel {
            id: actionPanel

            property int row: 0

            x: panel.width - width - menu.gap
            y: footerRule.y - height - menu.gap
            width: menu.onGrid(panel.width / 2)
            // No taller than the body it sits over, so it never covers the field.
            maxHeight: body.height
            actions: menu.actions
            visible: opacity > 0
            opacity: menu.actionsOpen ? 1 : 0
            transform: Translate {
                y: menu.actionsOpen ? 0 : menu.openTravel

                Behavior on y {
                    id: drop

                    NumberAnimation {
                        duration: drop.targetValue === 0 ? K.Tokens.duration.short : K.Tokens.duration.micro
                        easing.type: Easing.BezierSpline
                        easing.bezierCurve: drop.targetValue === 0 ? K.Tokens.easing.emphasized_decel : K.Tokens.easing.emphasized_accel
                    }
                }
            }
            onPicked: n => menu.runAction(n)

            Behavior on opacity {
                id: fade

                NumberAnimation {
                    duration: fade.targetValue > 0 ? K.Tokens.duration.short : K.Tokens.duration.micro
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: fade.targetValue > 0 ? K.Tokens.easing.emphasized_decel : K.Tokens.easing.emphasized_accel
                }
            }

            Keys.onPressed: event => {
                const ctrl = menu.ctrlOnly(event);
                // Undo may move the Result the panel is for.
                if (menu.undoKey(event)) {
                    menu.closeActions();
                } else if (event.key === Qt.Key_Escape || (ctrl && event.key === Qt.Key_K)) {
                    menu.closeActions();
                } else if (event.key === Qt.Key_Down || (ctrl && event.key === Qt.Key_N)) {
                    actionPanel.current = Math.min(menu.actions.length - 1, actionPanel.current + 1);
                } else if (event.key === Qt.Key_Up || (ctrl && event.key === Qt.Key_P)) {
                    actionPanel.current = Math.max(0, actionPanel.current - 1);
                } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                    menu.runAction(actionPanel.current);
                } else if (search.runShortcut(actionPanel.row, event.key, event.modifiers)) {
                    menu.failure = "";
                    menu.closeActions();
                }
                event.accepted = true;
            }
        }
    }

    // `c_menu`'s probe (test/lib/e2e.sh), with KUBAOS_MENU_PROBE=1 only.
    LazyLoader {
        active: Quickshell.env("KUBAOS_MENU_PROBE") === "1"

        Probe {
            search: menu.engine
            item: panel
            menuWidth: menu.panelWidth
            rows: menu.rows
            pane: menu.paneShown
            actions: menu.actions.length
            listed: list.count
            selected: list.currentIndex
            selectedShown: list.currentShown && !follow.running
            results: list
            keys: menu.keysAt()
        }
    }
}
