import QtQuick
import qs.theme
import qs.components
import qs.services

// The clock popup, under the bar's clock: the time to the second, the date,
// its ISO week and day of the year, an optional analog face, the extra
// clocks, a date search, the month, the picked day, countdowns, and the
// timer, stopwatch and alarms. A gear swaps in the settings.
//
// Keys: typing goes to the search; with it empty, Left/Right move a month and
// T comes back to today (and types the t, so `today + 90d` still reads).
// Enter copies the answer, or the picked day; Ctrl+Enter its ISO form. Esc
// clears the search, leaves the year or the settings, then closes.
//
// It glides in as the Command menu does (Kōbō's 220 ms on emphasized
// decelerate), and so do the months and the year view inside it.
Drawer {
    id: root

    readonly property int glide: 220
    readonly property list<real> arrive: Theme.anim.emphasizedDecel
    readonly property var kanji: ["日", "月", "火", "水", "木", "金", "土"]

    property string page: "main"
    property int viewYear: 0
    property int viewMonth: 0
    property string selected: ""
    property var day: null
    property var answer: null
    property int querySeq: 0
    property string countdownError: ""
    property string copied: ""

    readonly property date now: Time.now
    readonly property var today: Time.state.today ?? null
    readonly property bool analog: !!Time.settings.analog
    readonly property int columnGap: Theme.spacing.xl
    readonly property int pad: Theme.spacing.xl

    edge: "top"
    align: 0.5
    enterDuration: glide
    enterCurve: arrive
    exitDuration: 120
    contentWidth: grid.implicitWidth + side.width + columnGap + pad * 2
    contentHeight: body.implicitHeight + pad * 2

    onShownChanged: Time.seconds = shown
    // Esc the search let through: out of the settings or the year first.
    Keys.onEscapePressed: event => {
        if (page === "settings") {
            page = "main";
            search.focusInput();
        } else if (grid.yearView) {
            grid.yearView = false;
        } else {
            event.accepted = false;
        }
    }
    onOpened: {
        page = "main";
        search.text = "";
        answer = null;
        grid.yearView = false;
        goToday(false);
        apply();
        search.focusInput();
    }

    // What `qs ipc call clock ...` asked for: a page, a tool, a search.
    function apply(): void {
        const page = Time.pendingPage;
        const query = Time.pendingSearch;
        Time.pendingPage = "";
        Time.pendingSearch = "";
        if (page === "settings")
            root.page = "settings";
        else if (page !== "")
            tools.show(["timer", "stopwatch", "alarm"].indexOf(page));
        if (query !== "")
            search.text = query;
    }

    function goToday(slide: bool): void {
        if (!today)
            return;
        const [y, m] = today.date.split("-").map(Number);
        showMonth(y, m, slide ? Math.sign((y - viewYear) * 12 + m - viewMonth) : 0);
        select(today.date);
    }

    function showMonth(year: int, month: int, direction: int): void {
        viewYear = year;
        viewMonth = month;
        Time.request("month", { year: year, month: month }, reply => {
            if (reply.month && reply.month.year === root.viewYear && reply.month.month === root.viewMonth) {
                grid.direction = direction;
                grid.month = reply.month;
            }
        });
    }

    function move(months: int): void {
        const index = viewYear * 12 + viewMonth - 1 + months;
        showMonth(Math.floor(index / 12), index % 12 + 1, Math.sign(months));
    }

    function select(date: string): void {
        selected = date;
        Time.request("day", { date: date }, reply => {
            if (reply.day && date === root.selected)
                root.day = reply.day;
        });
    }

    // A date the search or a click lands on: its month, and the day picked.
    function land(date: string): void {
        const [y, m] = date.split("-").map(Number);
        if (y !== viewYear || m !== viewMonth)
            showMonth(y, m, Math.sign((y - viewYear) * 12 + m - viewMonth));
        select(date);
    }

    function ask(text: string): void {
        const seq = ++querySeq;
        if (text.trim() === "") {
            answer = null;
            return;
        }
        Time.request("query", { text: text }, reply => {
            if (seq !== root.querySeq)
                return;
            root.answer = reply.answer ?? null;
            if (root.answer?.date)
                root.land(root.answer.date);
        });
    }

    function copy(iso: bool): void {
        let text = "";
        if (answer)
            text = iso ? answer.iso : answer.title;
        else if (day)
            text = iso ? selected : day.title;
        if (text === "")
            return;
        Time.copy(text);
        copied = text;
        copiedTimer.restart();
    }

    function addCountdown(text: string): void {
        const at = text.indexOf(":");
        if (at < 1) {
            countdownError = "Name it, then the date: Vacation: 19 dec";
            return;
        }
        Time.change("countdown-add", { name: text.slice(0, at).trim(), when: text.slice(at + 1).trim() }, error => {
            root.countdownError = error;
            if (error === "")
                countdownField.text = "";
        });
    }

    // "dark_mode" and the like for a clock at `hour` there.
    function daylight(hour: int): string {
        if (hour >= 7 && hour < 18)
            return "light_mode";
        if (hour >= 5 && hour < 7 || hour >= 18 && hour < 21)
            return "wb_twilight";
        return "dark_mode";
    }

    Connections {
        target: Time

        function onShowRequested(): void {
            if (root.shown)
                root.apply();
        }

        // A setting changed (holidays, first weekday) or the day turned.
        function onStateChanged(): void {
            if (!root.shown || root.viewYear === 0)
                return;
            root.showMonth(root.viewYear, root.viewMonth, 0);
            if (root.selected !== "")
                root.select(root.selected);
        }
    }

    Timer {
        id: copiedTimer

        interval: 1600
        onTriggered: root.copied = ""
    }

    Column {
        id: body

        x: root.pad
        y: root.pad
        width: root.contentWidth - root.pad * 2
        spacing: Theme.spacing.lg

        // The time, the date, the week and the day of the year.
        Item {
            width: parent.width
            height: Math.max(time.height, root.analog ? face.height : 0)

            AnalogFace {
                id: face

                visible: root.analog
                time: root.now
                size: 96
                anchors.verticalCenter: parent.verticalCenter
            }

            Column {
                id: time

                anchors.left: root.analog ? face.right : parent.left
                anchors.leftMargin: root.analog ? Theme.spacing.xl : 0
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacing.xxs

                Row {
                    spacing: Theme.spacing.md

                    StyledText {
                        id: hhmm

                        text: Qt.formatDateTime(root.now, Time.clockFormat)
                        font.pixelSize: 56
                        font.weight: Font.Light
                        font.features: Theme.font.tabular
                    }

                    StyledText {
                        anchors.baseline: hhmm.baseline
                        text: Qt.formatDateTime(root.now, Time.twelveHour ? "ss AP" : "ss")
                        font.pixelSize: Theme.font.xxl
                        font.weight: Font.Light
                        font.features: Theme.font.tabular
                        color: Theme.m3onSurfaceVariant
                    }
                }

                Row {
                    spacing: Theme.spacing.sm

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.kanji[root.now.getDay()]
                        color: Theme.m3tertiary
                        font.family: Theme.font.jp
                        font.pixelSize: Theme.font.xl
                        font.weight: Font.Bold
                        renderType: Text.NativeRendering
                    }

                    StyledText {
                        anchors.verticalCenter: parent.verticalCenter
                        text: Qt.formatDateTime(root.now, Time.fullDateFormat)
                        font.pixelSize: Theme.font.lg
                    }
                }

                StyledText {
                    text: root.today ? `Day ${root.today.dayOfYear} · Week ${root.today.week}` : ""
                    font.pixelSize: Theme.font.sm
                    font.features: Theme.font.tabular
                    color: Theme.m3onSurfaceVariant
                }
            }

            IconButton {
                anchors.right: parent.right
                anchors.top: parent.top
                icon: root.page === "settings" ? "close" : "settings"
                toggled: root.page === "settings"
                tooltip: root.page === "settings" ? "Back" : "Calendar settings"
                onClicked: {
                    root.page = root.page === "settings" ? "main" : "settings";
                    if (root.page === "main")
                        search.focusInput();
                }
            }
        }

        // The extra clocks: the time there, how far off, day or night.
        Flow {
            visible: (Time.state.clocks ?? []).length > 0 && root.page === "main"
            width: parent.width
            spacing: Theme.spacing.sm

            Repeater {
                model: Time.state.clocks ?? []

                Rectangle {
                    id: zone

                    required property var modelData
                    readonly property date there: new Date(root.now.getTime() + (modelData.offset - (Time.state.here?.offset ?? 0)) * 1000)

                    width: zoneRow.implicitWidth + Theme.spacing.md * 2
                    height: 40
                    radius: Theme.rounding.md
                    color: Theme.m3surfaceContainer

                    Row {
                        id: zoneRow

                        anchors.centerIn: parent
                        spacing: Theme.spacing.sm

                        Icon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: root.daylight(zone.there.getHours())
                            size: 18
                            color: name === "light_mode" ? Theme.m3primary : Theme.m3tertiary
                        }

                        Column {
                            anchors.verticalCenter: parent.verticalCenter

                            StyledText {
                                text: zone.modelData.label
                                font.pixelSize: Theme.font.xs
                                color: Theme.m3onSurfaceVariant
                            }

                            StyledText {
                                text: Qt.formatDateTime(zone.there, Time.timeFormat)
                                font.weight: Font.DemiBold
                                font.features: Theme.font.tabular
                            }
                        }

                        StyledText {
                            anchors.verticalCenter: parent.verticalCenter
                            text: zone.modelData.relative
                            font.pixelSize: Theme.font.sm
                            font.features: Theme.font.tabular
                            color: Theme.m3outline
                        }
                    }
                }
            }
        }

        // The date search, and what it comes to.
        Column {
            visible: root.page === "main"
            width: parent.width
            spacing: Theme.spacing.sm

            SearchField {
                id: search

                width: parent.width
                icon: "event"
                placeholder: "next friday, days until christmas, 12 mar to 4 jul, 15:00 tokyo"
                catchEscape: text !== ""
                forwardTo: [keys]
                onTextChanged: root.ask(text)
                onEscaped: text = ""
            }

            Rectangle {
                width: parent.width
                height: root.answer ? answerColumn.implicitHeight + Theme.spacing.md * 2 : 0
                radius: Theme.rounding.lg
                color: Theme.m3surfaceContainer
                clip: true
                visible: height > 0

                Behavior on height {
                    Anim {
                        duration: root.glide
                        easing.bezierCurve: root.arrive
                    }
                }

                Column {
                    id: answerColumn

                    x: Theme.spacing.lg
                    y: Theme.spacing.md
                    width: parent.width - Theme.spacing.lg * 2
                    spacing: Theme.spacing.xxs

                    StyledText {
                        width: parent.width
                        text: root.answer?.title ?? ""
                        font.pixelSize: Theme.font.xl
                        font.weight: Font.DemiBold
                        color: Theme.m3primary
                    }

                    StyledText {
                        width: parent.width
                        text: root.answer?.detail ?? ""
                        wrapMode: Text.Wrap
                        color: Theme.m3onSurfaceVariant
                    }

                    StyledText {
                        width: parent.width
                        text: root.copied !== "" ? `Copied ${root.copied}` : "Enter copies · Ctrl+Enter copies ISO"
                        font.pixelSize: Theme.font.xs
                        color: root.copied !== "" ? Theme.m3tertiary : Theme.m3outline
                    }
                }
            }
        }

        // The month and the picked day; countdowns and the tools beside.
        Row {
            visible: root.page === "main"
            spacing: root.columnGap

            Column {
                spacing: Theme.spacing.md

                MonthGrid {
                    id: grid

                    glide: root.glide
                    arrive: root.arrive
                    selected: root.selected
                    onPicked: date => root.land(date)
                    onMoved: months => root.move(months)
                    onPickedMonth: month => root.showMonth(root.viewYear, month, Math.sign(month - root.viewMonth))
                }

                // The picked day: how far, its weekday and week, its holiday,
                // and the extra clocks' times at noon here.
                Column {
                    width: grid.width
                    spacing: Theme.spacing.xxs
                    visible: root.day !== null

                    StyledText {
                        width: parent.width
                        text: root.day?.title ?? ""
                        font.weight: Font.DemiBold
                    }

                    StyledText {
                        width: parent.width
                        text: root.day?.detail ?? ""
                        font.pixelSize: Theme.font.sm
                        color: Theme.m3onSurfaceVariant
                    }

                    StyledText {
                        visible: !!root.day?.holiday
                        width: parent.width
                        text: root.day?.holiday ?? ""
                        font.pixelSize: Theme.font.sm
                        color: Theme.m3tertiary
                    }

                    StyledText {
                        visible: (root.day?.zones ?? []).length > 0
                        width: parent.width
                        text: (root.day?.zones ?? []).map(z => `${z.label} ${z.time}`).join("  ·  ")
                        font.pixelSize: Theme.font.sm
                        font.features: Theme.font.tabular
                        color: Theme.m3outline
                    }
                }
            }

            Column {
                id: side

                width: 280
                spacing: Theme.spacing.lg

                // Countdowns: what's left of each, and one to add.
                Column {
                    width: parent.width
                    spacing: Theme.spacing.sm

                    StyledText {
                        text: "Countdowns"
                        font.weight: Font.DemiBold
                    }

                    Repeater {
                        model: Time.state.countdowns ?? []

                        Item {
                            id: countdown

                            required property var modelData
                            readonly property real remaining: modelData.end - root.now.getTime()

                            width: side.width
                            height: 34

                            Icon {
                                id: hourglass

                                anchors.verticalCenter: parent.verticalCenter
                                name: Time.settings.barCountdown === countdown.modelData.name ? "push_pin" : "hourglass_top"
                                size: 18
                                color: Theme.m3tertiary
                            }

                            Column {
                                anchors.left: hourglass.right
                                anchors.leftMargin: Theme.spacing.sm
                                anchors.right: removeCountdown.left
                                anchors.verticalCenter: parent.verticalCenter

                                StyledText {
                                    width: parent.width
                                    text: countdown.modelData.name
                                    font.weight: Font.Medium
                                }

                                StyledText {
                                    width: parent.width
                                    text: countdown.remaining > 0 ? `in ${Time.left(countdown.remaining)} · ${countdown.modelData.at}` : `${countdown.modelData.at} · passed`
                                    font.pixelSize: Theme.font.xs
                                    font.features: Theme.font.tabular
                                    color: Theme.m3onSurfaceVariant
                                }
                            }

                            IconButton {
                                id: removeCountdown

                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                icon: "close"
                                size: 26
                                tooltip: "Remove"
                                onClicked: Time.change("countdown-remove", { name: countdown.modelData.name }, error => root.countdownError = error)
                            }
                        }
                    }

                    SearchField {
                        id: countdownField

                        width: parent.width
                        icon: "more_time"
                        placeholder: "Vacation: 19 dec"
                        catchEscape: true
                        onEscaped: text = ""
                        onAccepted: root.addCountdown(text)
                    }

                    StyledText {
                        visible: root.countdownError !== ""
                        width: parent.width
                        text: root.countdownError
                        color: Theme.m3error
                        font.pixelSize: Theme.font.sm
                        wrapMode: Text.Wrap
                    }
                }

                Tools {
                    id: tools

                    width: parent.width
                }
            }
        }

        ClockSettings {
            visible: root.page === "settings"
            width: parent.width
        }

        StyledText {
            visible: !Time.available
            width: parent.width
            text: "kubaos-clock isn't running: the calendar and the search need it"
            color: Theme.m3error
            font.pixelSize: Theme.font.sm
        }
    }

    // Keys the search passes on: months and today while it's empty, the
    // copies always.
    Item {
        id: keys

        Keys.onPressed: event => {
            const empty = search.text === "";
            const ctrl = (event.modifiers & Qt.ControlModifier) !== 0;
            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                root.copy(ctrl);
                event.accepted = true;
            } else if (empty && event.key === Qt.Key_Left) {
                root.move(grid.yearView ? -12 : -1);
                event.accepted = true;
            } else if (empty && event.key === Qt.Key_Right) {
                root.move(grid.yearView ? 12 : 1);
                event.accepted = true;
            } else if (empty && event.key === Qt.Key_T && !ctrl) {
                grid.yearView = false;
                root.goToday(true);
            }
        }
    }
}
