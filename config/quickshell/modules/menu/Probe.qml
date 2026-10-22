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
    property real menuWidth: -1
    property int rows: -1
    property bool pane: false
    property int actions: 0
    // How many rows a pushed View lists, to see its filter work.
    property int listed: -1

    property real typedAt: 0
    // The Results for the query typed are in the model, and then in a frame
    // begun since.
    property bool arrived: false
    property bool drawn: false

    function log(line: string): void {
        console.info(`menu probe: ${line}`);
    }

    function layout(): void {
        if (menuWidth >= 0)
            log(`layout width=${Math.round(menuWidth)} rows=${rows} pane=${pane}`);
    }

    onActionsChanged: log(`actions ${actions}`)
    onPaneChanged: layout()
    onListedChanged: {
        if (listed >= 0 && search.depth > 0)
            log(`listed ${listed}`);
    }

    readonly property Connections searchEvents: Connections {
        target: probe.search

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
