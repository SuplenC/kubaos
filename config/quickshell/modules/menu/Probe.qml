import QtQuick
import Kotodama

// Logs what a Command menu shows, and how long a keystroke takes to reach the
// screen, as `menu probe: ...` lines in Quickshell's log. Menu.qml loads it
// only when Quickshell runs with KUBAOS_MENU_PROBE=1; `c_menu`
// (test/lib/e2e.sh) reads the lines. It works on Kotodama's ready-made
// `KotodamaMenu` too, given its `search` and an item in its window, so both
// menus are measured the same way.
//
// `latency_us` is Kotodama's own (`KotodamaSearch.latency`): from the query
// to the model showing its Results. `pixels_ms` runs from the query changing,
// which a keystroke does at once, to the swap of the first frame begun after
// those Results arrived, the way Kotodama's `frameStarted` and `painted`
// tell which Results are on screen.
QtObject {
    id: probe

    required property KotodamaSearch search
    // Any item in the menu's window.
    required property Item item
    // What the menu itself knows, for its layout line; the defaults leave
    // them out.
    property real menuWidth: NaN
    property int rows: -1
    property bool pane: false
    property int actions: 0
    // How many rows a pushed View lists, to see its filter work.
    property int listed: -1
    // The selected row, and whether the list has it wholly in view once
    // any scroll to it is over.
    property int selected: -1
    property bool selectedShown: true
    // The menu's list of Results: its top row's title and subtitle are
    // logged as they change, since an answer, such as a date's, lands there.
    // Its delegate is made a frame after the Results arrive, so it's read on
    // every frame, not on the frame line.
    property ListView results: null
    property string top: ""
    // What takes the keys (`search`, `filter`, `form`, `actions` or `view`) once the compositor
    // has given the menu's window the keyboard; empty until then. A key typed before it lands
    // nowhere, so the test types only once it names the place.
    property string keys: ""

    // The form View on screen, and the line last logged for it: each field's value, how many
    // fields a frame shows (each drawn opaque, sized, and inside the window) and how far its
    // scroll rests past the end of its content, which must be 0.
    property Item formView: null
    property string formLine: ""
    // The View the menu shows, which can stay behind the Daemon's while an Action settles: each
    // change is logged with whether the window was on screen for it.
    property var shownView: ({})

    onShownViewChanged: log(`screen kind=${shownView.kind ?? ""} title=${JSON.stringify(shownView.title ?? "")} visible=${item.Window.window?.visible ?? false}`)

    property real typedAt: 0
    // The Results for the query typed are in the model, and then in a frame
    // begun since.
    property bool arrived: false
    property bool drawn: false

    function log(line: string): void {
        console.info(`menu probe: ${line}`);
    }

    function logTop(): void {
        const first = results?.itemAtIndex(0) ?? null;
        const line = first ? `${first.title} · ${first.subtitle}` : "";
        if (line === top)
            return;
        top = line;
        log(`top ${JSON.stringify(line)}`);
    }

    // Whether `item` is drawn: visible, opaque and sized all the way up, and inside its window.
    function onScreen(item: Item): bool {
        const window = item.Window.window;
        if (!window || !window.visible || item.width <= 0 || item.height <= 0)
            return false;
        for (let it = item; it; it = it.parent) {
            if (!it.visible || it.opacity <= 0)
                return false;
        }
        const at = item.mapToItem(window.contentItem, 0, 0);
        return at.x + item.width > 0 && at.y + item.height > 0 && at.x < window.width && at.y < window.height;
    }

    function logForm(): void {
        const fields = probe.formView ? probe.formView.shown() : [];
        const past = probe.formView ? Math.round(Math.max(0, probe.formView.contentY - probe.formView.travel)) : 0;
        const line = fields.length === 0 ? "" : `fields=${fields.length} shown=${fields.filter(field => probe.onScreen(field.item)).length} past=${past} values=${JSON.stringify(fields.map(field => field.value))}`;
        if (line === probe.formLine)
            return;
        probe.formLine = line;
        if (line !== "")
            log(`form ${line}`);
    }

    function layout(): void {
        if (!isNaN(menuWidth))
            log(`layout width=${Math.round(menuWidth)} rows=${rows} pane=${pane}`);
    }

    onActionsChanged: log(`actions ${actions}`)
    onKeysChanged: log(`keys ${keys}`)
    // The menu's search reached the Daemon, so `kotodama toggle` has a menu to answer it.
    Component.onCompleted: log(`connected ${search.connected}`)
    onPaneChanged: layout()
    onSelectedShownChanged: log(`selected row=${selected} shown=${selectedShown}`)
    onSelectedChanged: log(`selected row=${selected} shown=${selectedShown}`)
    onListedChanged: {
        if (listed >= 0 && search.depth > 0)
            log(`listed ${listed}`);
    }

    readonly property Connections searchEvents: Connections {
        target: probe.search

        function onConnectedChanged(): void {
            probe.log(`connected ${probe.search.connected}`);
        }

        function onQueryChanged(): void {
            probe.typedAt = Date.now();
            probe.arrived = false;
            probe.drawn = false;
        }

        function onLatencyChanged(): void {
            probe.arrived = probe.typedAt > 0;
        }

        function onViewChanged(): void {
            const view = probe.search.view;
            const job = view.job ? ` job=${view.job.state}` : "";
            probe.log(`view kind=${view.kind ?? ""} title=${JSON.stringify(view.title ?? "")} depth=${probe.search.depth}${job}`);
        }

        function onJobsChanged(): void {
            for (const job of probe.search.jobs)
                probe.log(`job ${JSON.stringify(job.title)} state=${job.state} progress=${job.progress}`);
        }

        function onScopeChanged(): void {
            probe.log(`scope ${JSON.stringify(probe.search.scope)}`);
        }

        function onIntroAboutChanged(): void {
            probe.log(`intro ${JSON.stringify(probe.search.introAbout)}`);
        }

        function onConfirmingChanged(): void {
            probe.log(`confirming ${JSON.stringify(probe.search.confirming)}`);
        }

        function onToastChanged(): void {
            probe.log(`toast ${JSON.stringify(probe.search.toast)}`);
        }

        function onActionFailed(message: string): void {
            probe.log(`failed ${JSON.stringify(message)}`);
        }
    }

    readonly property Connections windowEvents: Connections {
        target: probe.item.Window.window

        function onAfterAnimating(): void {
            if (!probe.arrived)
                return;
            probe.arrived = false;
            probe.drawn = true;
        }

        function onFrameSwapped(): void {
            probe.logTop();
            probe.logForm();
            if (!probe.drawn)
                return;
            probe.drawn = false;
            probe.log(`frame query=${JSON.stringify(probe.search.query)} rows=${probe.search.rowCount()} latency_us=${probe.search.latency} pixels_ms=${Date.now() - probe.typedAt}`);
        }

        function onVisibleChanged(): void {
            probe.log(`visible ${probe.item.Window.window.visible}`);
            if (probe.item.Window.window.visible)
                probe.layout();
        }
    }
}
