import QtQuick
import Quickshell
import Kotodama
import KubaOS.Components as K
import qs.services

// A form's file or folder field: a `Field` for the path, which the Daemon completes as it is
// typed, and under it, while the field has the keys, a `List` of the paths that complete it, each
// with its folder and when it changed. With nothing typed the list holds the newest files the
// field takes. ↓ and ↑ (or Ctrl+N and Ctrl+P) pick a path, Tab completes it, going on into a
// folder, Enter takes it, Esc closes the list; Ctrl+O opens the system's picker (`browse`). A file
// dropped on the field fills it. Every other key goes to `keys`, the form's and the menu's.
K.Stack {
    id: root

    required property KotodamaSearch search
    // The field's place in its form, which `complete` and the Daemon's answers name it by.
    required property int place
    // A folder, not a file: Enter takes a folder, which in a file field Tab and Enter go into.
    property bool folder: false
    property string placeholder
    property alias text: typed.text
    // The menu's keys, after the field's own: true if they took the event.
    property var keys: event => false
    // Whether the form's field is a path at all: a form makes one of these for every field, and
    // only a path's makes its rows.
    property bool active: true

    // The row Tab or Enter takes, or -1 while none is picked.
    property int picked: -1
    // The list stays closed after a path is taken, or Esc, until something is typed.
    property bool dismissed: false
    // A path is being filled in, so the text it changes asks the Daemon for nothing.
    property bool taking: false
    // The most rows the list shows: the Daemon answers with more, best first.
    readonly property int rows: 5
    readonly property var paths: root.search.completingField === root.place ? root.search.completions : []
    readonly property int count: Math.min(root.paths.length, root.rows)
    // Whether the path's field has the keys.
    readonly property bool hasKeys: typed.activeFocus
    readonly property bool listOpen: root.hasKeys && !root.dismissed && root.count > 0
    readonly property string home: Quickshell.env("HOME")

    // Ctrl+O: the system's file picker, for the path.
    signal browse

    // `path` written `~/…` under the home folder, as it would be typed.
    function homeRelative(path: string): string {
        if (root.home === "")
            return path;
        if (path === root.home)
            return "~";
        return path.startsWith(root.home + "/") ? "~" + path.slice(root.home.length) : path;
    }

    // The folder `path` is in, as it would be typed.
    function folderOf(path: string): string {
        return root.homeRelative(path.slice(0, Math.max(1, path.lastIndexOf("/"))));
    }

    // When `modified`, in seconds since the Unix epoch, was: the time today, else the day. Empty
    // when the file system can't tell (-1).
    function when(modified: real): string {
        if (modified < 0)
            return "";
        const at = new Date(modified * 1000);
        const now = Time.now;
        if (at.toDateString() === now.toDateString())
            return Qt.formatDateTime(at, Time.timeFormat);
        if (at.getFullYear() === now.getFullYear())
            return Qt.formatDateTime(at, Time.dayFirst ? "d MMM" : "MMM d");
        return Qt.formatDateTime(at, Time.dayFirst ? "d MMM yyyy" : "MMM d, yyyy");
    }

    // Puts `path` in the field as the value, closing the list.
    function take(path: string): void {
        root.taking = true;
        typed.text = path;
        root.taking = false;
        root.dismissed = true;
        typed.cursorPosition = path.length;
    }

    // Fills in the `n`th path offered. A folder goes on completing, its `/` typed, when Tab took it
    // or the field wants a file; anything else is the value.
    function fill(n: int, byTab: bool): void {
        const offered = root.paths[n];
        const slash = typed.text.lastIndexOf("/");
        // What was typed of the folder stays as it was, such as `~/`; with nothing typed, the
        // path is whole.
        const filled = slash >= 0 ? typed.text.slice(0, slash + 1) + offered.title : root.homeRelative(offered.path);
        if (offered.folder && (byTab || !root.folder)) {
            typed.text = filled + "/";
            typed.cursorPosition = typed.text.length;
        } else {
            root.take(filled);
        }
    }

    // The field's own keys while the list is open, then Ctrl+O; true if it took the event.
    function key(event: var): bool {
        const ctrl = (event.modifiers & Qt.ControlModifier) !== 0;
        const plain = (event.modifiers & (Qt.ControlModifier | Qt.ShiftModifier | Qt.AltModifier | Qt.MetaModifier)) === 0;
        if (ctrl && event.key === Qt.Key_O) {
            root.browse();
            return true;
        }
        if (!root.listOpen)
            return false;
        const down = (plain && event.key === Qt.Key_Down) || (ctrl && (event.key === Qt.Key_N || event.key === Qt.Key_J));
        const up = (plain && event.key === Qt.Key_Up) || (ctrl && (event.key === Qt.Key_P || event.key === Qt.Key_K));
        if (down) {
            root.picked = Math.min(root.count - 1, root.picked + 1);
            return true;
        }
        if (up) {
            root.picked = Math.max(-1, root.picked - 1);
            return true;
        }
        // Tab with nothing picked completes the best path for what is typed, as a shell does;
        // with nothing typed it moves on, as it does everywhere else.
        if (plain && event.key === Qt.Key_Tab && (root.picked >= 0 || typed.text !== "")) {
            root.fill(Math.max(0, root.picked), true);
            return true;
        }
        if (plain && (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) && root.picked >= 0) {
            root.fill(root.picked, false);
            return true;
        }
        if (event.key === Qt.Key_Escape) {
            root.dismissed = true;
            return true;
        }
        return false;
    }

    // Gives the field the keys.
    function focusField(): void {
        typed.forceActiveFocus();
    }

    fill: true
    gap: K.Tokens.space.s8

    K.Field {
        id: typed

        placeholder: root.placeholder
        onTextChanged: {
            root.search.setText(root.place, text);
            root.picked = -1;
            if (typed.activeFocus && !root.taking) {
                root.dismissed = false;
                root.search.complete(root.place, text);
            }
        }
        // With nothing typed, the Daemon offers the newest files the field takes.
        onActiveFocusChanged: {
            if (typed.activeFocus) {
                root.picked = -1;
                root.search.complete(root.place, text);
            }
        }
        onKeyPressed: event => event.accepted = root.key(event) || root.keys(event)

        DropArea {
            id: drop

            anchors.fill: parent
            keys: ["text/uri-list"]
            onDropped: dropped => {
                if (!dropped.hasUrls)
                    return;
                const url = String(dropped.urls[0]);
                root.take(root.homeRelative(url.startsWith("file://") ? decodeURIComponent(url.slice("file://".length)) : url));
                typed.forceActiveFocus();
                dropped.acceptProposedAction();
            }
        }
    }

    K.List {
        visible: root.listOpen

        // A fixed set of rows, each showing the answer's path at its place: an answer per
        // keystroke then rebinds five rows' text rather than making five rows.
        Repeater {
            model: root.active ? root.rows : 0

            K.ListRow {
                required property int index
                readonly property var entry: root.paths[index] ?? ({})

                visible: index < root.count
                title: entry.title ?? ""
                subtitle: root.folderOf(entry.path ?? "")
                detail: entry.folder ? "" : root.when(entry.modified ?? -1)
                icon: entry.folder ? "folder" : "draft"
                selected: index === root.picked
                activeFocusOnTab: false
                onActivated: {
                    typed.forceActiveFocus();
                    root.fill(index, false);
                }
            }
        }
    }
}
