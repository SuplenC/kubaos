import QtQuick
import KubaOS.Components as K

// Stand-in for Kotodama's KotodamaSearch: a fixed set of Results matched by substring, and every
// property and call the shell reads, which the drivers set to put a view in a state. `@ICONS@` is
// the harness's icon directory, filled in when a tree is staged.
ListModel {
    id: search

    property string query: ""
    property color highlightColor: K.Tokens.color.primary
    property bool connected: true
    property var down: []
    property int problems: 0
    property int latency: 0
    property var view: ({})
    property int depth: 0
    property var jobs: []
    property string toast: ""
    property string scope: ""
    property string introAbout: ""
    property string introNote: ""
    property string confirming: ""
    property var introExamples: []
    // A list View's Items, while one is on top.
    property var listItems: []
    property int submits: 0
    property var lastChecked: null
    property string lastText: ""
    property int selected: 0
    // The paths offered to complete a file field, and its place in the form, which a driver sets;
    // `complete` keeps what it was last asked.
    property var completions: []
    property int completingField: -1
    property string lastCompleted: ""

    signal closeRequested
    signal openRequested(string query, string view)
    signal toggleRequested
    signal selectRow(int row)
    signal actionFailed(string message)
    signal queryReplaced(string text)

    // What the "Add a keyboard layout" app lists once entered.
    readonly property var layouts: [
        {
            title: "German",
            subtitle: "de",
            icon: "",
            extension: "kubaos"
        },
        {
            title: "German (Switzerland)",
            subtitle: "ch",
            icon: "",
            extension: "kubaos"
        },
        {
            title: "German (Austria)",
            subtitle: "at",
            icon: "",
            extension: "kubaos"
        },
        {
            title: "German (no dead keys)",
            subtitle: "de(nodeadkeys)",
            icon: "",
            extension: "kubaos"
        },
        {
            title: "French",
            subtitle: "fr",
            icon: "",
            extension: "kubaos"
        },
        {
            title: "Polish",
            subtitle: "pl",
            icon: "",
            extension: "kubaos"
        }
    ]

    readonly property var results: [
        {
            title: "Files",
            subtitle: "Access and organize files",
            icon: "@ICONS@/folder.svg",
            extension: "apps"
        },
        {
            title: "Firefox",
            subtitle: "Web browser",
            icon: "@ICONS@/text-html.svg",
            extension: "apps"
        },
        {
            title: "Document Viewer",
            subtitle: "View, search or annotate documents in many different formats",
            icon: "@ICONS@/x-office-document.svg",
            extension: "apps"
        },
        {
            title: "Image Viewer",
            subtitle: "View images",
            icon: "@ICONS@/image-x-generic.svg",
            extension: "apps"
        },
        {
            title: "Settings: Desktop",
            subtitle: "Shell, wallpaper and the bar",
            icon: "",
            extension: "kubaos"
        },
        {
            title: "Timer",
            subtitle: "Count down and ring",
            icon: "",
            extension: "system"
        },
        {
            title: "kubaos",
            subtitle: "~/Projects/kubaos",
            icon: "",
            extension: "folders"
        },
        {
            title: "Downloads",
            subtitle: "~/Downloads",
            icon: "@ICONS@/folder-download.svg",
            extension: "folders"
        },
        {
            title: "Emoji",
            subtitle: "Pick an emoji and paste it",
            icon: "",
            extension: "emoji"
        },
        {
            title: "Clipboard history",
            subtitle: "What you copied, newest first",
            icon: "",
            extension: "clipboard"
        },
        {
            title: "Power off",
            subtitle: "Shut the machine down",
            icon: "",
            extension: "system"
        },
        {
            title: "Suspend",
            subtitle: "Sleep until a key is pressed",
            icon: "",
            extension: "system",
            unavailable: "Inhibited by Firefox: a video is playing"
        },
        {
            title: "Workspace 3",
            subtitle: "Move to workspace 3",
            icon: "",
            extension: "hyprland"
        },
        {
            title: "Design specimen",
            subtitle: "Every token and component, live",
            icon: "",
            extension: "kubaos"
        }
    ]

    function markup(title: string, typed: string): string {
        if (typed === "")
            return title;
        const at = title.toLowerCase().indexOf(typed.toLowerCase());
        if (at < 0)
            return title;
        return `${title.slice(0, at)}<font color="${search.highlightColor}">${title.slice(at, at + typed.length)}</font>${title.slice(at + typed.length)}`;
    }

    function fill(): void {
        const inApp = search.scope !== "";
        const typed = search.depth > 0 ? (search.view.filter ?? "") : search.query;
        const source = search.depth > 0 ? search.listItems : inApp ? search.layouts : search.results;
        // The app's word enters it, as running its entry Item does with the Daemon: the view's
        // own entering reads the title by `KotodamaSearch`'s role number, which a ListModel's
        // roles don't have.
        if (!inApp && search.query.startsWith("layout ")) {
            search.clear();
            Qt.callLater(() => search.scope = "layout");
            return;
        }
        const rows = source.filter(r => typed === "" || r.title.toLowerCase().includes(typed.toLowerCase()));
        search.clear();
        for (const r of rows)
            search.append({
                title: r.title,
                markup: search.markup(r.title, typed),
                subtitle: r.subtitle,
                icon: r.icon,
                extension: r.extension,
                alias: r.alias ?? "",
                unavailable: r.unavailable ?? ""
            });
    }

    onQueryChanged: fill()
    onScopeChanged: fill()
    Component.onCompleted: fill()

    function run(row: int): bool {
        return true;
    }

    function confirm(): bool {
        search.confirming = "";
        return true;
    }

    function cancelConfirm(): bool {
        const had = search.confirming !== "";
        search.confirming = "";
        return had;
    }

    function actions(row: int): var {
        if (row < 0 || row >= search.count)
            return [];
        return [
            {
                title: "Open",
                group: "",
                shortcut: ""
            },
            {
                title: "Open in new window",
                group: "",
                shortcut: "ctrl+enter"
            },
            {
                title: "Copy path",
                group: "",
                shortcut: "ctrl+shift+c"
            },
            {
                title: "Forget",
                group: "",
                shortcut: "ctrl+shift+backspace"
            },
            {
                title: "Pin to the top",
                group: "kubaos",
                shortcut: "ctrl+p"
            },
            {
                title: "Search the web for it",
                group: "web-search",
                shortcut: ""
            }
        ];
    }

    function hints(row: int): string {
        if (row < 0 || row >= search.count)
            return "";
        return row % 2 === 1 ? "↵ Run · ⌃K Actions" : "↵ Open · Tab Enter app · ⌃K Actions";
    }

    function runAction(row: int, n: int): bool {
        return true;
    }

    function runShortcut(row: int, key: int, modifiers: int): bool {
        return false;
    }

    function filter(text: string): bool {
        const v = Object.assign({}, search.view);
        v.filter = text;
        search.view = v;
        search.fill();
        return true;
    }

    function popView(): bool {
        if (search.depth === 0)
            return false;
        search.view = ({});
        search.depth = 0;
        search.fill();
        return true;
    }

    function setText(field: int, text: string): bool {
        search.lastText = `${field}:${text}`;
        return true;
    }

    function complete(field: int, text: string): bool {
        search.lastCompleted = `${field}:${text}`;
        search.completingField = field;
        return true;
    }

    function setChecked(field: int, checked: bool): bool {
        search.lastChecked = `${field}:${checked}`;
        return true;
    }

    function submit(): bool {
        search.submits += 1;
        return true;
    }

    function select(row: int): void {
        search.selected = row;
    }

    function undo(): bool {
        search.toast = "";
        return true;
    }

    function isUndoKey(key: int, modifiers: int): bool {
        return key === Qt.Key_Z && modifiers === Qt.ControlModifier;
    }

    function showProblems(): bool {
        return true;
    }

    function viewport(top: int, rows: int): void {
    }

    function frameStarted(): void {
    }

    function painted(): void {
    }

    function closed(): void {
    }

    function shown(): void {
    }

    function leaveScope(): bool {
        return false;
    }

    function leaveApp(): bool {
        return false;
    }

    function enter(row: int): bool {
        return false;
    }

    function rowCount(): int {
        return search.count;
    }

    // Pushes a View: `kind` with its fields, as the View's map holds them.
    function push(view: var, items: var): void {
        search.listItems = items ?? [];
        search.view = view;
        search.depth = 1;
        search.fill();
    }
}
