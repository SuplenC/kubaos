import QtQuick
import Quickshell
import Quickshell.Wayland
import Kotodama
import qs.theme
import qs.components
import qs.services

// The Command menu: KubaOS's own front end on Kotodama's `KotodamaSearch`, in
// Kōbō's look. A search box over the Daemon's Results, a detail pane beside
// them when the screen has room, the Views Extensions push, the jobs they
// run, and each Result's Action panel. Kotodama's `KotodamaMenu.qml` handles
// the same calls; this file follows it call for call.
//
// `kotodama toggle` (SUPER + Space, config/hypr/fallback.lua) opens it
// through the Daemon, so it has no IPC target of its own. Hyprland blurs what
// shows through its translucent panel (the `kotodama` layer,
// config/hypr/rules.lua), and leaves its motion to it.
//
// Speed comes first: a keystroke's Results reach the screen in the frame they
// arrive. Nothing waits on an animation, the rows are cheap (ResultRow.qml),
// and the detail pane updates only once a frame with the Results is on
// screen and typing has paused.
PanelWindow {
    id: menu

    // It opens on the focused screen, and stays there while it is open, even
    // when the pointer crosses to another screen and takes the focus with it.
    Binding on screen {
        value: Hypr.focusedScreen
        when: !menu.visible
        restoreMode: Binding.RestoreNone
    }

    readonly property MenuLook look: MenuLook {
        screen: menu.screen
    }
    // The search under a name its children's own `search` properties don't
    // shadow.
    readonly property KotodamaSearch engine: search
    readonly property string placeholder: "Search apps, projects and windows. Sums work too"

    // Why the last Action failed, until the next Action or opening.
    property string failure: ""
    // The top View's kind (`list`, `form`, `detail` or `progress`), or empty
    // at the search.
    readonly property string kind: search.view.kind ?? ""
    // Whether the body is a list: the Results, or a list View's Items.
    readonly property bool listing: kind === "" || kind === "list"
    readonly property bool paneShown: look.paneFits && listing
    // The Action panel of the selected row while Ctrl+K shows it: each entry
    // `{title, group, shortcut}`, as `KotodamaSearch.actions` gives them.
    property var actions: []
    readonly property bool actionsOpen: actions.length > 0
    // The keys the selected row takes, for the footer when there is no pane
    // to show them. Reading the row's markup and unavailable makes them
    // follow a new Result in the same place, not only a move of the
    // selection.
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

    // The selected row, by what it is: a new key is a new selection, which
    // the pane crossfades to.
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
    // How far the body has yet to slide, and how faded it still is, after a
    // View was pushed or popped.
    property real slide: 0
    property real settle: 1
    property int lastDepth: 0
    // Results a keystroke changed show at once. While any animation runs,
    // each frame waits on the screen's next refresh, so the rows and the
    // highlight move only for changes nobody typed, and the pane catches up
    // when typing pauses: a glide after the last key.
    readonly property bool typing: keystroke.running

    // The Daemon types or pastes once the window is gone and focus is back
    // where the user came from, so it is told only after the window unmaps.
    property bool closeAsked: false

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

    // Closing pops no View: Extensions aren't told the user left one, so their
    // jobs keep running, and the menu opens again where it was. Only Esc pops.
    function hide(): void {
        menu.actions = [];
        search.cancelConfirm();
        if (!visible || closing.running)
            return;
        opening.stop();
        closing.restart();
    }

    // Gives keys to the top View: the search box, a list's filter, the first
    // field of a form, or the menu itself for a detail or a progress View.
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

    // Ctrl and no other modifier: the menu's own Ctrl keys leave Ctrl+Shift+K
    // and the like to the Actions' key bindings.
    function ctrlOnly(event: KeyEvent): bool {
        return (event.modifiers & (Qt.ControlModifier | Qt.ShiftModifier | Qt.AltModifier | Qt.MetaModifier)) === Qt.ControlModifier;
    }

    // Whether `event` is a modifier pressed on its own, as on the way to a
    // key binding.
    function modifierAlone(event: KeyEvent): bool {
        return [Qt.Key_Shift, Qt.Key_Control, Qt.Key_Alt, Qt.Key_Meta, Qt.Key_Super_L, Qt.Key_Super_R].includes(event.key);
    }

    // While the Toast shows, its key undoes what it offers to, whatever has
    // the keys; otherwise the key is the text box's own undo. True if
    // `event` was it.
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
        // An Action that asks first waits in the footer: Enter runs it, Esc
        // drops it, and any other key drops it and does what it does.
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
            // One rule everywhere, so Esc never throws away more than the
            // user can see: it clears the text if there is any, else pops
            // the View, else leaves the app for the whole search, else
            // closes the menu.
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
            list.currentIndex = Math.min(list.count - 1, list.currentIndex + menu.look.rows);
        } else if (event.key === Qt.Key_PageUp) {
            list.currentIndex = Math.max(0, list.currentIndex - menu.look.rows);
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
    margins.top: look.top
    // The surface keeps one size for the screen, so typing never waits on a
    // resize, and nothing jumps as Results, the pane or a status come and go.
    implicitWidth: look.width
    implicitHeight: look.height
    exclusionMode: ExclusionMode.Ignore
    color: "transparent"
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.namespace: "kotodama"
    WlrLayershell.keyboardFocus: visible && !closing.running ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

    // However the window opened, the Daemon is told, so the Empty query it
    // shows is ranked for this time of day.
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

        highlightColor: Theme.m3primary
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
        // The box shows the rest of the query once it is in an app,
        // or what the Daemon typed. The cursor keeps its place from the end.
        onQueryReplaced: text => {
            const fromEnd = input.text.length - input.cursorPosition;
            input.text = text;
            input.cursorPosition = Math.max(0, text.length - fromEnd);
        }
        // A list View's rows changed: its selected Item is at `row`, or the
        // top row is selected, as in the search, if it went away.
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
            viewMove.from = search.depth > menu.lastDepth ? menu.look.motion.viewTravel : -menu.look.motion.viewTravel;
            menu.lastDepth = search.depth;
            if (menu.visible)
                viewIn.restart();
        }
        // A View that is pushed, popped or changed takes the keys; one whose
        // job progressed keeps them where they are. A list's filter shows
        // what was typed into it, which is nothing for a new one.
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

        interval: menu.look.motion.glide
    }

    NumberAnimation {
        id: opening

        target: menu
        property: "openness"
        to: 1
        duration: menu.look.motion.open
        easing.type: Easing.BezierSpline
        easing.bezierCurve: menu.look.motion.arrive
    }

    NumberAnimation {
        id: closing

        target: menu
        property: "openness"
        to: 0
        duration: menu.look.motion.close
        easing.type: Easing.BezierSpline
        easing.bezierCurve: menu.look.motion.leave
        onFinished: menu.visible = false
    }

    // The body arrives readable: it starts part faded, so a pushed list's
    // rows show in the View's first frame.
    ParallelAnimation {
        id: viewIn

        NumberAnimation {
            id: viewMove

            target: menu
            property: "slide"
            to: 0
            duration: menu.look.motion.glide
            easing.type: Easing.BezierSpline
            easing.bezierCurve: menu.look.motion.arrive
        }

        NumberAnimation {
            target: menu
            property: "settle"
            from: 0.4
            to: 1
            duration: menu.look.motion.glide
            easing.type: Easing.BezierSpline
            easing.bezierCurve: menu.look.motion.effects
        }
    }

    Item {
        id: panel

        anchors.fill: parent
        opacity: menu.openness
        transform: Translate {
            y: (menu.openness - 1) * menu.look.motion.openTravel
        }

        Rectangle {
            anchors.fill: parent
            radius: menu.look.radius
            color: Theme.alpha(Theme.m3surface, Theme.frame.opacity)
            border.width: 1
            border.color: Theme.alpha(Theme.m3outlineVariant, 0.55)
        }

        Item {
            id: header

            width: parent.width
            height: menu.look.headerHeight

            // A magnifier at the search, an arrow back in a View.
            Icon {
                id: lead

                x: menu.look.pad
                anchors.verticalCenter: parent.verticalCenter
                name: search.depth > 0 ? "arrow_back" : "search"
                size: menu.look.leadIcon
                color: search.depth > 0 ? Theme.m3onSurface : Theme.m3primary

                MouseArea {
                    anchors.fill: parent
                    anchors.margins: -menu.look.gap
                    enabled: search.depth > 0
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: search.popView()
                }
            }

            // The app the search is in, such as Emoji: the box holds what
            // follows the app's word.
            Rectangle {
                id: chip

                objectName: "scope"
                x: lead.x + lead.width + menu.look.inset
                anchors.verticalCenter: parent.verticalCenter
                visible: menu.kind === "" && search.scope !== ""
                width: visible ? menu.look.snap(chipLabel.implicitWidth + 2 * menu.look.inset) : 0
                height: menu.look.chipHeight
                radius: height / 2
                color: Theme.m3secondaryContainer

                StyledText {
                    id: chipLabel

                    anchors.centerIn: parent
                    text: search.scope
                    color: Theme.m3onSecondaryContainer
                    font.pixelSize: menu.look.font.title
                    font.weight: Font.Medium
                }
            }

            TextInput {
                id: input

                anchors.fill: parent
                anchors.leftMargin: (chip.visible ? chip.x + chip.width : lead.x + lead.width) + menu.look.inset
                anchors.rightMargin: menu.look.pad
                visible: menu.kind === ""
                verticalAlignment: TextInput.AlignVCenter
                clip: true
                color: Theme.m3onSurface
                selectionColor: Theme.m3primary
                selectedTextColor: Theme.m3onPrimary
                renderType: Text.NativeRendering
                font.family: Theme.font.sans
                font.pixelSize: menu.look.font.query
                focus: true
                onTextChanged: {
                    keystroke.restart();
                    menu.paneTyped = true;
                    search.query = text;
                    list.currentIndex = 0;
                    list.positionViewAtBeginning();
                }

                StyledText {
                    anchors.fill: parent
                    visible: input.text === "" && !input.preeditText
                    text: search.scope !== "" ? `Search ${search.scope}` : menu.placeholder
                    color: Theme.alpha(Theme.m3onSurfaceVariant, 0.8)
                    font.pixelSize: menu.look.font.query
                }

                // Backspace with nothing typed in an app leaves it: the whole
                // search again, with the app's word typed.
                Keys.onPressed: event => {
                    if (event.key === Qt.Key_Backspace && input.text === "" && search.leaveScope())
                        event.accepted = true;
                    else
                        event.accepted = menu.key(event);
                }
            }

            // A list View's filter, with the View's title until something is
            // typed. The Daemon matches what is typed as it does the search,
            // so the rows narrow as they would there.
            TextInput {
                id: filter

                anchors.fill: parent
                anchors.leftMargin: lead.x + lead.width + menu.look.inset
                anchors.rightMargin: menu.look.pad
                visible: menu.kind === "list"
                verticalAlignment: TextInput.AlignVCenter
                clip: true
                color: Theme.m3onSurface
                selectionColor: Theme.m3primary
                selectedTextColor: Theme.m3onPrimary
                renderType: Text.NativeRendering
                font.family: Theme.font.sans
                font.pixelSize: menu.look.font.query
                // The selection stays where it is until the matches come;
                // see onSelectRow.
                onTextChanged: {
                    keystroke.restart();
                    menu.paneTyped = true;
                    search.filter(text);
                }

                StyledText {
                    anchors.fill: parent
                    visible: filter.text === "" && !filter.preeditText
                    text: search.view.title ?? ""
                    color: Theme.alpha(Theme.m3onSurfaceVariant, 0.8)
                    font.pixelSize: menu.look.font.query
                }

                Keys.onPressed: event => event.accepted = menu.key(event)
            }

            // The title of any other View stands where the search box was.
            StyledText {
                anchors.fill: parent
                anchors.leftMargin: lead.x + lead.width + menu.look.inset
                anchors.rightMargin: menu.look.pad
                visible: !menu.listing
                text: search.view.title ?? ""
                font.pixelSize: menu.look.font.query
                font.weight: Font.Medium
            }

            // Takes the keys of a View with nothing to type into.
            Item {
                id: viewKeys

                Keys.onPressed: event => event.accepted = menu.key(event)
            }
        }

        Rectangle {
            id: headerRule

            y: header.height
            width: parent.width
            height: 1
            color: Theme.alpha(Theme.m3outlineVariant, 0.55)
        }

        // The jobs still running, under the search box.
        Column {
            id: jobs

            y: headerRule.y + 1
            width: parent.width
            visible: menu.kind !== "progress" && search.jobs.length > 0
            height: visible ? implicitHeight : 0
            topPadding: menu.look.nudge
            bottomPadding: menu.look.nudge

            Repeater {
                model: search.jobs

                Item {
                    id: running

                    required property var modelData

                    width: jobs.width
                    height: menu.look.jobHeight

                    Icon {
                        id: jobIcon

                        x: menu.look.pad
                        anchors.verticalCenter: parent.verticalCenter
                        name: running.modelData.state === "failed" ? "error" : running.modelData.state === "succeeded" ? "check_circle" : "progress_activity"
                        size: menu.look.jobIcon
                        color: running.modelData.state === "failed" ? Theme.m3error : Theme.m3primary
                    }

                    StyledText {
                        anchors.left: jobIcon.right
                        anchors.leftMargin: menu.look.gap
                        anchors.right: jobBar.left
                        anchors.rightMargin: menu.look.inset
                        anchors.verticalCenter: parent.verticalCenter
                        text: running.modelData.title + (running.modelData.message !== "" ? ` · ${running.modelData.message}` : "")
                        color: Theme.m3onSurfaceVariant
                        font.pixelSize: menu.look.font.small
                    }

                    JobBar {
                        id: jobBar

                        anchors.right: parent.right
                        anchors.rightMargin: menu.look.pad
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width / 4
                        look: menu.look
                        progress: running.modelData.progress
                        failed: running.modelData.state === "failed"
                    }
                }
            }
        }

        // Results and list Views, the pane, or the other Views. It slides as
        // a View is pushed or popped.
        Item {
            id: body

            y: jobs.y + jobs.height
            width: parent.width
            height: menu.look.bodyHeight - jobs.height
            opacity: menu.settle
            transform: Translate {
                x: menu.slide
            }

            Item {
                id: listArea

                width: parent.width - (menu.paneShown ? menu.look.paneWidth : 0)
                height: parent.height
                visible: menu.listing

                // An app's intro card, before anything is typed in it. It
                // comes with the app's first Results, so the first frame in
                // the app shows it.
                IntroCard {
                    id: intro

                    objectName: "intro"
                    x: menu.look.inset
                    y: menu.look.inset
                    width: parent.width - 2 * menu.look.inset
                    look: menu.look
                    visible: menu.kind === "" && search.introAbout !== ""
                    note: search.introNote
                    about: search.introAbout
                    examples: search.introExamples
                }

                ListView {
                    id: list

                    x: menu.look.inset
                    y: intro.visible ? intro.y + intro.height + menu.look.gap : menu.look.inset
                    width: parent.width - 2 * menu.look.inset
                    height: parent.height - y - menu.look.inset
                    clip: true
                    model: search
                    // Pooled rows and the remove transition together leave
                    // removed rows on screen, so rows aren't reused; at
                    // fourteen rows at most, making them is cheap.
                    reuseItems: false
                    boundsBehavior: Flickable.StopAtBounds
                    highlightFollowsCurrentItem: false
                    onCountChanged: {
                        if (currentIndex < 0 && count > 0)
                            currentIndex = 0;
                    }
                    // A list View's filter keeps the selected Item selected. In
                    // the search, moving off the top row stops Late Results
                    // reranking the rows.
                    onCurrentIndexChanged: {
                        search.select(currentIndex);
                        reveal(currentIndex);
                    }

                    // The view scrolls to keep the selection on screen. With
                    // the highlight not following the current row, ListView
                    // no longer does that itself, so it is done here, on the
                    // selection's glide. Rows are all one height.
                    function reveal(index: int): void {
                        if (index < 0)
                            return;
                        const row = menu.look.rowHeight;
                        const top = originY + index * row;
                        let to = contentY;
                        if (top < contentY)
                            to = top;
                        else if (top + row > contentY + height)
                            to = top + row - height;
                        to = Math.max(originY, Math.min(to, originY + contentHeight - height));
                        follow.stop();
                        if (to === contentY)
                            return;
                        if (menu.look.motion.moves && !menu.typing && menu.visible) {
                            follow.to = to;
                            follow.start();
                        } else {
                            contentY = to;
                        }
                    }

                    // The selected row is wholly in view, for the probe.
                    readonly property bool currentShown: currentIndex >= 0 && originY + currentIndex * menu.look.rowHeight >= contentY - 0.5 && originY + (currentIndex + 1) * menu.look.rowHeight <= contentY + height + 0.5

                    NumberAnimation {
                        id: follow

                        target: list
                        property: "contentY"
                        duration: menu.look.motion.glide
                        easing.type: Easing.BezierSpline
                        easing.bezierCurve: menu.look.motion.arrive
                    }

                    // Once the rerank after a keystroke is over, Late Results
                    // go below the rows on screen, so the Daemon is told which
                    // those are.
                    readonly property int firstRow: Math.floor(contentY / menu.look.rowHeight)
                    onFirstRowChanged: search.viewport(firstRow, menu.look.rows)

                    // Where the pointer last was in the window, or null before
                    // it has been over a row since the menu opened.
                    property var pointer: null

                    // Only a pointer that moved picks a row. A window that maps
                    // under a resting pointer gets a pointer-enter, and Qt
                    // resends hover when rows move under a still pointer:
                    // either would otherwise take the selection off the top
                    // row the user is typing for.
                    function hover(index: int, area: Item, x: real, y: real): void {
                        const at = area.mapToItem(null, x, y);
                        const moved = list.pointer !== null && (at.x !== list.pointer.x || at.y !== list.pointer.y);
                        list.pointer = at;
                        if (moved)
                            list.currentIndex = index;
                    }

                    // The selection slides between rows on Kōbō's glide. It
                    // never holds rows back: they are in place at once.
                    highlight: Rectangle {
                        width: list.width
                        height: menu.look.rowHeight
                        y: list.currentItem?.y ?? 0
                        radius: menu.look.rowRadius
                        color: Theme.m3secondaryContainer
                        visible: list.count > 0

                        Behavior on y {
                            enabled: menu.look.motion.moves && !menu.typing
                            NumberAnimation {
                                duration: menu.look.motion.glide
                                easing.type: Easing.BezierSpline
                                easing.bezierCurve: menu.look.motion.arrive
                            }
                        }
                    }

                    // Rows ease in and out as the Results change, but for a
                    // keystroke (`typing`). A row's opacity on its own is
                    // cheap; no row gets a layer.
                    add: Transition {
                        enabled: !menu.typing
                        NumberAnimation {
                            property: "opacity"
                            from: 0
                            to: 1
                            duration: menu.look.motion.fade
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: menu.look.motion.effects
                        }
                    }
                    remove: Transition {
                        enabled: !menu.typing
                        NumberAnimation {
                            property: "opacity"
                            to: 0
                            duration: menu.look.motion.fade
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: menu.look.motion.effects
                        }
                    }
                    displaced: Transition {
                        enabled: menu.look.motion.moves && !menu.typing
                        NumberAnimation {
                            property: "y"
                            duration: menu.look.motion.glide
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: menu.look.motion.arrive
                        }
                        // A row displaced mid-fade comes back whole.
                        NumberAnimation {
                            property: "opacity"
                            to: 1
                            duration: menu.look.motion.fade
                        }
                    }
                    // A Result that moves mid-fade comes back whole too: with
                    // no `move` transition the view cancels the fade where it
                    // is. Moved rows still jump to their place.
                    move: Transition {
                        NumberAnimation {
                            property: "opacity"
                            to: 1
                            duration: menu.look.motion.fade
                        }
                    }

                    // A run names the Results of the last frame on screen, so
                    // the Daemon refuses a Late Result the user hasn't seen
                    // yet. afterAnimating comes on this thread just before the
                    // frame takes the model; frameSwapped comes queued from the
                    // render thread, after Results that arrived since may have
                    // changed it. The pane catches up on the selection once a
                    // frame with the Results is out and typing has paused, so
                    // it never holds a keystroke's Results back.
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

                        look: menu.look
                        onPointed: (x, y) => list.hover(row.index, row, x, y)
                        onPicked: {
                            menu.failure = "";
                            search.run(row.index);
                        }
                    }
                }

                EmptyState {
                    anchors.centerIn: list
                    visible: list.count === 0
                    icon: "search_off"
                    text: (menu.kind === "list" ? filter.text : input.text) !== "" ? "No results" : "Nothing here yet"
                }
            }

            DetailPane {
                id: pane

                x: parent.width - width
                width: menu.look.paneWidth
                height: parent.height
                look: menu.look
                visible: menu.paneShown
            }

            Loader {
                id: form

                anchors.fill: parent
                active: menu.kind === "form"
                sourceComponent: FormView {
                    look: menu.look
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
                    look: menu.look
                    markdown: search.view.markdown ?? ""
                    metadata: search.view.metadata ?? []
                }
            }

            Loader {
                anchors.fill: parent
                active: menu.kind === "progress"
                sourceComponent: ProgressView {
                    look: menu.look
                    job: search.view.job ?? ({})
                }
            }
        }

        Rectangle {
            id: footerRule

            y: footer.y - 1
            width: parent.width
            height: 1
            color: Theme.alpha(Theme.m3outlineVariant, 0.55)
        }

        // The footer: what the user should know now (an Action asking first,
        // why one failed, the Toast, Extensions that are down), else the keys
        // the menu takes; and on the right, while the Daemon knows of any,
        // how many problems there are, which a click lists.
        Item {
            id: footer

            y: parent.height - height
            width: parent.width
            height: menu.look.footerHeight

            readonly property string status: search.confirming !== "" ? search.confirming : menu.failure !== "" ? menu.failure : ""
            // A confirmation reads `Power Off? ↵ Confirm · Esc Cancel`: the
            // question, then its keys.
            readonly property int asks: search.confirming.lastIndexOf("? ") + 1

            StyledText {
                anchors.left: parent.left
                anchors.leftMargin: menu.look.pad
                anchors.right: problems.left
                anchors.rightMargin: menu.look.inset
                anchors.verticalCenter: parent.verticalCenter
                visible: search.confirming === "" && (menu.failure !== "" || search.toast === "" && search.down.length > 0)
                text: menu.failure !== "" ? menu.failure : `Unavailable: ${search.down.join(", ")}`
                color: menu.failure !== "" ? Theme.m3error : Theme.m3onSurfaceVariant
                font.pixelSize: menu.look.font.small
            }

            Row {
                anchors.left: parent.left
                anchors.leftMargin: menu.look.pad
                anchors.verticalCenter: parent.verticalCenter
                visible: search.confirming !== ""
                spacing: menu.look.inset

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: search.confirming.slice(0, footer.asks)
                    color: Theme.m3error
                    font.pixelSize: menu.look.font.small
                    font.weight: Font.Medium
                }

                KeyHints {
                    anchors.verticalCenter: parent.verticalCenter
                    look: menu.look
                    tint: Theme.m3error
                    text: search.confirming.slice(footer.asks + 1)
                }
            }

            // The Toast: what Forget or "Clear history" just did, and the key
            // that undoes it. It never takes the keys or the selection, and
            // goes when the Daemon withdraws it, once the change is undone or
            // too old to be.
            Row {
                anchors.left: parent.left
                anchors.leftMargin: menu.look.pad
                anchors.verticalCenter: parent.verticalCenter
                visible: footer.status === "" && search.toast !== ""
                spacing: menu.look.inset

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.min(implicitWidth, footer.width / 2)
                    text: search.toast
                    font.pixelSize: menu.look.font.small
                }

                KeyHints {
                    anchors.verticalCenter: parent.verticalCenter
                    look: menu.look
                    tint: Theme.m3primary
                    text: "⌃Z Undo"
                }
            }

            KeyHints {
                anchors.left: parent.left
                anchors.leftMargin: menu.look.pad
                anchors.right: problems.left
                anchors.rightMargin: menu.look.inset
                anchors.verticalCenter: parent.verticalCenter
                visible: footer.status === "" && search.toast === "" && search.down.length === 0
                look: menu.look
                text: menu.footerHints
            }

            StyledText {
                id: problems

                anchors.right: parent.right
                anchors.rightMargin: visible ? menu.look.pad : 0
                anchors.verticalCenter: parent.verticalCenter
                width: visible ? implicitWidth : 0
                visible: search.problems > 0
                text: search.problems === 1 ? "1 problem" : `${search.problems} problems`
                color: Theme.m3error
                font.pixelSize: menu.look.font.small
                font.weight: Font.Medium

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: search.showProblems()
                }
            }
        }

        // The Action panel: Ctrl+K lists the selected Result's Actions over
        // the bottom right of the list. Up and Down choose, Enter runs, Esc
        // or Ctrl+K closes it.
        ActionPanel {
            id: actionPanel

            property int row: 0

            x: parent.width - width - menu.look.inset
            y: footerRule.y - height - menu.look.inset
            width: Math.min(menu.look.actionPanelWidth, parent.width / 2)
            look: menu.look
            actions: menu.actions
            visible: opacity > 0
            opacity: menu.actionsOpen ? 1 : 0
            transform: Translate {
                y: menu.actionsOpen ? 0 : menu.look.motion.openTravel
                Behavior on y {
                    NumberAnimation {
                        duration: menu.look.motion.fade
                        easing.type: Easing.BezierSpline
                        easing.bezierCurve: menu.look.motion.arrive
                    }
                }
            }
            onPicked: n => menu.runAction(n)

            Behavior on opacity {
                NumberAnimation {
                    duration: menu.look.motion.fade
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: menu.look.motion.effects
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
            menuWidth: menu.look.width
            rows: menu.look.rows
            pane: menu.paneShown
            actions: menu.actions.length
            listed: list.count
            selected: list.currentIndex
            selectedShown: list.currentShown && !follow.running
            results: list
        }
    }
}
