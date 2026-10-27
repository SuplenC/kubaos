import QtQuick
import KubaOS.Components as K
import Kotodama
import qs.components
import qs.services

// "Add a language…" in the Keyboard tab: a search over every layout and
// variant not in use, with Kotodama's engine. Its own `KotodamaSearch` enters
// the "Add a keyboard layout" app (crates/kotodama-kubaos/layouts/) and sends
// what is typed, and this list draws the app's Results. A click picks a
// layout and the Add button under the list adds it; Enter adds the selected
// one at once. KeyboardList.qml makes it anew each time the view opens.
//
// It is a second Front end of the Daemon beside the Command menu, so it never
// says it was shown, answers a close at once, and hands `kotodama toggle` and
// `open` on to the menu (services/MenuRequests.qml) in case the Daemon took it
// for the menu.
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
    // The layout a click picked, { spelling, name }, waiting for Add.
    property var picked: null
    readonly property Item button: addButton
    readonly property Item list: matchList

    // Add `spelling` for a click on Add, which may be a stray one.
    signal add(string spelling)
    // Add `spelling` for Enter, which can't be.
    signal addAtOnce(string spelling)
    signal cancelled
    signal failed(string message)

    function spellingAt(row: int): string {
        return engine.data(engine.index(row, 0), subtitleRole) ?? "";
    }

    // The Daemon takes a query as an app's only once the app was entered from
    // its entry Item, which the global search for the word lists; running it
    // enters the app. The rows are on screen as far as the Daemon is told, so
    // the run is the one they show. Once is enough: the Daemon keeps the app
    // for as long as the query starts with its word, and a second run, while
    // the first one's Results are still on their way, would take the search
    // out of the app and back in under the user's typing.
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

    // In the app the engine puts the word in front itself. Out of it (before
    // it is entered, or while the Daemon hands back the query on entering it,
    // or after a `kotodama toggle` took the search out) the word goes first,
    // with the space after it that keeps the query in the app. It reads the
    // scope itself: `entered` may not have followed it yet while the scope
    // changes.
    function send(): void {
        engine.query = engine.scope !== "" ? search.text : `${word} ${search.text}`;
    }

    Component.onCompleted: {
        send();
        search.forceActiveFocus();
    }

    onPickedChanged: if (picked)
        matchList.positionViewAtIndex(matchList.currentIndex, ListView.Contain)

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

    K.Field {
        id: search

        width: view.width
        icon: "search"
        placeholder: "Search languages and layouts"
        // The list sees each key first and takes the ones it moves by, Ctrl+K among them, before
        // the text would delete to the line's end; Esc goes back to the layouts.
        forwardTo: [matchList]
        Keys.onEscapePressed: view.cancelled()
        onTextChanged: {
            view.picked = null;
            view.send();
        }
    }

    Well {
        y: search.height + K.Tokens.space.s12
        width: view.width
        height: Math.max(0, (confirm.visible ? confirm.y - K.Tokens.space.s12 : view.height) - y)
        focused: matchList.activeFocus

        NavList {
            id: matchList

            anchors.fill: parent
            visible: view.entered && count > 0
            vimKeys: false
            fieldKeys: true
            model: view.entered ? engine : null
            onActivated: index => view.addAtOnce(view.spellingAt(index))
            // The Add bar takes its room from the list as it shows; the pick stays in view.
            onHeightChanged: if (view.picked)
                positionViewAtIndex(currentIndex, ListView.Contain)
            // New Results select their top row, as the Command menu does.
            onCountChanged: currentIndex = 0

            // The roles by `model`, as K.ListRow has a title and a subtitle of its own.
            delegate: K.ListRow {
                id: row

                required property var model
                required property int index
                readonly property string spelling: model.subtitle ?? ""

                width: ListView.view.width
                title: model.title ?? ""
                subtitle: spelling
                selected: ListView.isCurrentItem
                checkable: true
                checked: view.picked?.spelling === spelling
                onActivated: {
                    ListView.view.currentIndex = index;
                    view.picked = {
                        spelling: row.spelling,
                        name: row.title
                    };
                }
            }
        }

        // While there is nothing to list: why.
        K.EmptyState {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width
            room: parent.width
            visible: !matchList.visible
            icon: !engine.connected ? "error" : !view.entered ? "search" : "search_off"
            title: !engine.connected ? "Nothing to search" : !view.entered ? "Searching…" : "No match"
            body: !engine.connected ? "The Command menu's search (kotodamad) isn't running." : !view.entered ? "The layouts show up here." : `No layout matches “${search.text.trim()}”.`
        }
    }

    // The picked layout and the button that adds it.
    K.Row {
        id: confirm

        y: view.height - height
        width: view.width
        room: view.width
        visible: view.picked !== null
        fill: true
        gap: K.Tokens.space.s8
        align: "centre"

        // Takes the room the buttons leave.
        K.Stack {
            fill: true

            K.Text {
                text: `Add ${view.picked?.name ?? ""}`
                style: K.Tokens.type.body_strong
                maximumLines: 1
            }
        }

        K.Button {
            text: "Cancel"
            kind: "Text"
            size: "Sm"
            onActivated: view.picked = null
        }

        K.Button {
            id: addButton

            function press(): void {
                if (view.picked)
                    view.add(view.picked.spelling);
            }

            text: "Add"
            kind: "Filled"
            size: "Sm"
            onActivated: press()
        }
    }
}
