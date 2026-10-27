import QtQuick
import QtQuick.Layouts
import Kotodama
import qs.theme
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
ColumnLayout {
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

    spacing: Theme.spacing.sm

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
        search.focusInput();
    }

    onPickedChanged: if (picked)
        matchList.positionViewAtIndex(matchList.currentIndex, ListView.Contain)

    KotodamaSearch {
        id: engine

        highlightColor: Theme.m3primary
        onRowsInserted: view.enterApp()
        onModelReset: view.enterApp()
        onScopeChanged: view.send()
        onToggleRequested: MenuRequests.toggle()
        onOpenRequested: query => MenuRequests.open(query)
        onCloseRequested: engine.closed()
        onActionFailed: message => view.failed(message)
    }

    SearchField {
        id: search

        Layout.fillWidth: true
        placeholder: "Search languages and layouts"
        catchEscape: true
        forwardTo: [matchList]
        onEscaped: view.cancelled()
        onTextChanged: {
            view.picked = null;
            view.send();
        }
    }

    NavList {
        id: matchList

        Layout.fillWidth: true
        Layout.fillHeight: true
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

        // The roles by `model`, as ListRow has a title and a subtitle of its own.
        delegate: ListRow {
            id: row

            required property var model
            required property int index
            readonly property string spelling: model.subtitle ?? ""

            width: ListView.view.width
            title: model.title ?? ""
            subtitle: spelling
            current: ListView.isCurrentItem
            active: view.picked?.spelling === spelling
            onClicked: {
                ListView.view.currentIndex = index;
                view.picked = {
                    spelling: row.spelling,
                    name: row.title
                };
            }
        }
    }

    // While there is nothing to list: why.
    StyledText {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: !matchList.visible
        verticalAlignment: Text.AlignTop
        wrapMode: Text.Wrap
        color: Theme.m3onSurfaceVariant
        text: !engine.connected ? "The Command menu's search (kotodamad) isn't running, so there is nothing to search." : !view.entered ? "Searching…" : `No layout matches “${search.text.trim()}”.`
    }

    // The picked layout and the button that adds it.
    Rectangle {
        Layout.fillWidth: true
        visible: view.picked !== null
        implicitHeight: confirm.implicitHeight + Theme.spacing.sm * 2
        radius: Theme.rounding.md
        color: Theme.m3secondaryContainer

        RowLayout {
            id: confirm

            anchors.fill: parent
            anchors.leftMargin: Theme.spacing.md
            anchors.rightMargin: Theme.spacing.sm
            spacing: Theme.spacing.sm

            StyledText {
                Layout.fillWidth: true
                text: `Add ${view.picked?.name ?? ""}`
                color: Theme.m3onSecondaryContainer
                font.weight: Font.Medium
                elide: Text.ElideRight
            }

            IconButton {
                size: 30
                icon: "close"
                tooltip: "Cancel"
                tint: Theme.m3onSecondaryContainer
                onClicked: view.picked = null
            }

            Rectangle {
                id: addButton

                readonly property real radius: Theme.rounding.full

                function press(): void {
                    if (view.picked)
                        view.add(view.picked.spelling);
                }

                implicitWidth: addLabel.implicitWidth + Theme.spacing.lg * 2
                implicitHeight: 32
                color: Theme.m3primary
                activeFocusOnTab: true
                Keys.onReturnPressed: press()
                Keys.onEnterPressed: press()
                Keys.onSpacePressed: press()

                StyledText {
                    id: addLabel

                    anchors.centerIn: parent
                    text: "Add"
                    color: Theme.m3onPrimary
                    font.weight: Font.Medium
                }

                StateLayer {
                    color: Theme.m3onPrimary
                    onClicked: addButton.press()
                }

                FocusRing {}
            }
        }
    }
}
