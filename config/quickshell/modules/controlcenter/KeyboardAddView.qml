import QtQuick
import KubaOS.Components as K
import Kotodama
import qs.components
import qs.services

// "Add a language" in the Keyboard tab's well, in place of the layouts: a header with the way
// back, then a search over every layout and variant not in use, with Kotodama's engine, and its
// matches. Its own `KotodamaSearch` enters the "Add a keyboard layout" app
// (crates/kotodama-kubaos/layouts/) and sends what is typed, and this list draws the app's
// Results. A click or Enter adds the match; the focused one shows a + where a row's check would
// be. ← or Esc goes back. The header, the field and the matches are a list's rows apart.
// KeyboardList.qml makes it anew each time the view opens, so nothing from the last time (a
// query, the focused row) shows over the new one.
//
// It is a second Front end of the Daemon beside the Command menu, so it never says it was shown,
// answers a close at once, and hands `kotodama toggle` and `open` on to the menu
// (services/MenuRequests.qml) in case the Daemon took it for the menu.
Item {
    id: view

    // The app's title and word, as its manifest names them.
    readonly property string app: "Add a keyboard layout"
    readonly property string word: "layout"
    // The title and subtitle roles of `KotodamaSearch`'s rows.
    readonly property int titleRole: 0x100
    readonly property int subtitleRole: 0x101

    readonly property bool entered: engine.scope !== ""
    // The app's entry Item was run.
    property bool entering: false
    readonly property Item list: matchList
    // Whether the matches have the keyboard, which lights the well's edge.
    readonly property bool listFocused: matchList.activeFocus

    // Add `spelling` for a click, which may be the second of a double-click.
    signal add(string spelling)
    // Add `spelling` for Enter, which can't be.
    signal addAtOnce(string spelling)
    signal cancelled
    signal failed(string message)

    function spellingAt(row: int): string {
        return engine.data(engine.index(row, 0), subtitleRole) ?? "";
    }

    // The Daemon takes a query as an app's only once the app was entered from its entry Item,
    // which the global search for the word lists; running it enters the app. The rows are on
    // screen as far as the Daemon is told, so the run is the one they show. Once is enough: the
    // Daemon keeps the app for as long as the query starts with its word, and a second run, while
    // the first one's Results are still on their way, would take the search out of the app and
    // back in under the user's typing.
    function enterApp(): void {
        if (entering)
            return;
        for (let row = 0; row < engine.rowCount(); row++) {
            if (engine.data(engine.index(row, 0), titleRole) === app) {
                engine.frameStarted();
                engine.painted();
                entering = engine.enter(row);
                return;
            }
        }
    }

    // In the app the engine puts the word in front itself. Out of it (before it is entered, or
    // while the Daemon hands back the query on entering it, or after a `kotodama toggle` took the
    // search out) the word goes first, with the space after it that keeps the query in the app.
    // It reads the scope itself: `entered` may not have followed it yet while the scope changes.
    function send(): void {
        engine.query = engine.scope !== "" ? search.text : `${word} ${search.text}`;
    }

    Component.onCompleted: {
        send();
        search.forceActiveFocus();
    }

    KotodamaSearch {
        id: engine

        highlightColor: K.Tokens.color.primary
        onRowsInserted: view.enterApp()
        onModelReset: view.enterApp()
        onScopeChanged: view.send()
        onToggleRequested: MenuRequests.toggle()
        onOpenRequested: query => MenuRequests.open(query)
        onCloseRequested: engine.closed()
        onActionFailed: message => view.failed(message)
    }

    // The well's padding above it, as above the layouts' first row.
    K.Row {
        id: header

        y: K.Tokens.space.s12
        width: view.width
        room: view.width
        gap: K.Tokens.space.s8
        align: "centre"

        K.IconButton {
            icon: "arrow_back"
            label: "Back"
            kind: "Text"
            size: "Sm"
            onActivated: view.cancelled()
        }

        K.Text {
            text: "Add a language"
            style: K.Tokens.type.body_strong
        }
    }

    K.Field {
        id: search

        y: header.y + header.height + K.Tokens.space.s4
        width: view.width
        icon: "search"
        placeholder: "Search languages and layouts"
        // The list sees each key first and takes the ones it moves by, Ctrl+K among them, before
        // the text would delete to the line's end; Esc goes back to the layouts.
        forwardTo: [matchList]
        Keys.onEscapePressed: view.cancelled()
        onTextChanged: view.send()
    }

    NavList {
        id: matchList

        y: search.y + search.height + K.Tokens.space.s4
        width: view.width
        height: Math.max(0, view.height - y)
        // The well's padding below the last row, scrolled with them.
        bottomMargin: K.Tokens.space.s12
        visible: view.entered && count > 0
        vimKeys: false
        fieldKeys: true
        model: view.entered ? engine : null
        onActivated: index => view.addAtOnce(view.spellingAt(index))
        Keys.onEscapePressed: view.cancelled()
        // New Results select their top row, as the Command menu does.
        onCountChanged: currentIndex = 0

        // The roles by `model`, as K.ListRow has a title and a subtitle of its own.
        delegate: K.ListRow {
            id: row

            required property var model
            required property int index
            readonly property string spelling: model.subtitle ?? ""

            width: ListView.view.width
            icon: "keyboard"
            tile: true
            title: model.title ?? ""
            subtitle: row.spelling
            selected: ListView.isCurrentItem
            onActivated: {
                ListView.view.currentIndex = row.index;
                view.add(row.spelling);
            }

            // What Enter or a click does to the focused match, where a row's check would be.
            K.Icon {
                x: row.width - K.Tokens.space.s12 - width
                y: K.Look.snap((row.height - height) / 2, dpr)
                visible: row.selected
                name: "add"
                size: K.Tokens.type.body_strong.icon
                color: row.main
            }
        }
    }

    // While there is nothing to list: why.
    K.EmptyState {
        y: matchList.y + Math.max(0, (matchList.height - height) / 2)
        width: view.width
        room: view.width
        visible: !matchList.visible
        icon: !engine.connected ? "error" : !view.entered ? "search" : "search_off"
        title: !engine.connected ? "Nothing to search" : !view.entered ? "Searching…" : "No match"
        body: !engine.connected ? "The Command menu's search (kotodamad) isn't running." : !view.entered ? "The layouts show up here." : `No layout matches “${search.text.trim()}”.`
    }
}
