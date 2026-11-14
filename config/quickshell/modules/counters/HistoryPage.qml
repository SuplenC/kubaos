import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services
import "counters.js" as Logic

// A counter's history, inside the counters' sheet: Back, its name and how many changes it has, the
// grouping (Each change, Day, Week, Month) and the order (Newest, Oldest, Highest, or Most added
// when grouped), then its changes in a well, each with its time, what it was and the value after
// it, or a group with how many changes it holds and how much they added. Past `mostRows` rows the
// well scrolls. A counter never changed shows that it has no history yet. It opens with the
// keyboard on Back, so Enter or Esc goes back; Back lights only once a key has been used in the
// sheet, so nothing lights for a pointer.
FocusScope {
    id: root

    // The counter it shows, or null.
    property var counter: null
    // The rows `kubaos-counters log` gave for the grouping and order picked, and as they read.
    property var rows: []
    // Each line carries the value box's width, so rows on their way out never see a new one.
    property var lines: []
    property int group: 0
    property int sort: 0
    // The rows the well shows before it scrolls.
    readonly property int mostRows: 8
    // A row's day as the shell's short date writes it ("Wed 11 Nov"), and its time with no
    // leading zero ("9:14"), as the counters' rules write theirs.
    readonly property var formats: ({
            date: Time.shortDateFormat,
            time: Time.twelveHour ? "h:mm AP" : "H:mm",
            week: Time.dayFirst ? "d MMM" : "MMM d",
            month: "MMMM yyyy"
        })
    // Today, as the rows' days are told from it: set as the page opens.
    property date now: Time.now
    readonly property int changes: root.rows.reduce((sum, row) => sum + row.changes, 0)
    // A key has been used in the sheet since it opened.
    property bool keyed: false

    signal back
    // A key was pressed on the page.
    signal keyPressed

    // Reads the history again, for the grouping and order picked.
    function read(): void {
        if (!root.counter)
            return;
        const asked = root.counter.id;
        const group = Logic.GROUPS[root.group];
        Counters.history(root.counter.name, group, Logic.SORTS[root.sort], rows => {
            if (root.counter?.id !== asked)
                return;
            const lines = rows.map(row => Logic.historyLine(row, group, root.now, root.formats));
            const widest = Logic.historyWidest(lines);
            root.rows = rows;
            root.lines = lines.map(line => Object.assign(line, {
                    widest: widest
                }));
        });
    }

    // Opens on `counter`'s changes, each, newest first, the keyboard on Back.
    function showFor(counter: var): void {
        root.counter = counter;
        root.now = Time.now;
        root.group = 0;
        root.sort = 0;
        root.rows = [];
        root.lines = [];
        root.read();
        backButton.forceActiveFocus();
    }

    implicitHeight: column.implicitHeight
    // Every key reaches this first, Tab included, as Back and the tabs leave it unaccepted.
    Keys.onPressed: event => {
        root.keyPressed();
        if (event.key === Qt.Key_Escape) {
            root.back();
            event.accepted = true;
        }
    }

    // A change made while it shows (the key, the menu, an auto reset) is in the history at once.
    Connections {
        target: Counters

        function onListChanged(): void {
            if (root.visible)
                root.read();
        }
    }

    K.Stack {
        id: column

        width: root.width
        fill: true
        gap: K.Tokens.space.s8

        K.Row {
            fill: true
            fixedHeight: K.Tokens.size.control_md
            leftPadding: K.Tokens.space.s4
            rightPadding: K.Tokens.space.s4
            gap: K.Tokens.space.s8
            align: "centre"

            K.IconButton {
                id: backButton

                icon: "arrow_back"
                label: "Back"
                kind: "Text"
                size: "Sm"
                focused: backButton.activeFocus && root.keyed
                onActivated: root.back()
            }

            K.Stack {
                fill: true

                K.Text {
                    text: root.counter?.name ?? ""
                    style: K.Tokens.type.body_strong
                    maximumLines: 1
                }
            }

            K.Text {
                text: Logic.changesWords(root.changes)
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }
        }

        K.Tabs {
            labels: Logic.GROUP_LABELS
            currentIndex: root.group
            onPicked: index => {
                root.group = index;
                root.read();
            }
        }

        K.Tabs {
            labels: Logic.sortLabels(Logic.GROUPS[root.group])
            currentIndex: root.sort
            onPicked: index => {
                root.sort = index;
                root.read();
            }
        }

        K.EmptyState {
            visible: root.rows.length === 0
            icon: "history"
            title: "No changes yet"
            body: "Every +1, −1, set and reset shows here, with its time."
        }

        Well {
            visible: root.rows.length > 0
            implicitHeight: Math.min(root.lines.length, root.mostRows) * (K.Tokens.size.row + K.Tokens.space.s4) - K.Tokens.space.s4 + inset * 2
            focused: list.activeFocus

            NavList {
                id: list

                width: parent.width
                height: parent.height
                topMargin: K.Tokens.space.s12
                bottomMargin: K.Tokens.space.s12
                highlightShown: list.activeFocus
                model: root.lines

                delegate: K.ListRow {
                    id: line

                    required property var modelData
                    required property int index

                    width: ListView.view.width
                    title: line.modelData.title
                    selected: ListView.isCurrentItem && ListView.view.highlightShown
                    onActivated: ListView.view.currentIndex = line.index

                    K.Text {
                        text: line.modelData.what
                        style: K.Tokens.type.body
                        color: line.muted
                    }

                    K.Row {
                        align: "centre"

                        K.Text {
                            visible: line.modelData.sign !== ""
                            text: line.modelData.sign
                            style: K.Tokens.type.mono_large
                            color: line.main
                        }

                        K.RollingNumber {
                            value: line.modelData.value
                            widest: line.modelData.widest
                            color: line.main
                        }
                    }
                }
            }
        }
    }
}
