import QtQuick
import KubaOS.Components as K
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
// It arrives and leaves as every panel does. Its columns are the month, a
// regular control a day, and the rail's width for countdowns and the tools.
Drawer {
    id: root

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
    // The search's text before its last change: a `t` typed into an empty search also goes to
    // today.
    property string lastQuery: ""

    readonly property date now: Time.now
    readonly property var today: Time.state.today ?? null
    readonly property bool analog: !!Time.settings.analog
    readonly property int columnGap: K.Tokens.space.s20
    // Above and below the body; at its sides, the container padding round the month and the
    // rail beside it.
    readonly property int pad: K.Tokens.space.s20
    readonly property int side: K.Tokens.containerPaddingAround(grid.implicitWidth + K.Tokens.size.rail + columnGap)
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    edge: "top"
    align: 0.5
    contentWidth: grid.implicitWidth + K.Tokens.size.rail + columnGap + side * 2
    contentHeight: body.implicitHeight + pad * 2

    onShownChanged: Time.seconds = shown
    // Esc the search let through: out of the settings or the year first.
    Keys.onEscapePressed: event => {
        if (page === "settings") {
            page = "main";
            search.forceActiveFocus();
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
        search.forceActiveFocus();
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

    // The body, the container padding in at the sides and `pad` above and below.
    K.Container {
        y: root.pad
        width: root.contentWidth

        K.Stack {
            id: body

            fill: true
            gap: K.Tokens.space.s16

            // The time, the date, the week and the day of the year.
            K.Row {
                fill: true

                K.Row {
                    gap: K.Tokens.space.s20
                    align: "centre"

                    AnalogFace {
                        visible: root.analog
                        time: root.now
                    }

                    K.Stack {
                        gap: K.Tokens.space.s4

                        K.Row {
                            gap: K.Tokens.space.s12

                            // It changes once a minute, so it keeps its own width; the seconds hold
                            // still beside it in between.
                            K.Text {
                                id: hhmm

                                text: Qt.formatDateTime(root.now, Time.clockFormat)
                                style: K.Tokens.type.hero_number
                            }

                            // The seconds on the time's baseline.
                            Item {
                                implicitWidth: ss.implicitWidth
                                implicitHeight: hhmm.implicitHeight

                                Figure {
                                    id: ss

                                    y: K.Look.snap(root.baselineOf(heroMetrics, K.Tokens.type.hero_number) - root.baselineOf(secondsMetrics, K.Tokens.type.headline), root.dpr)
                                    text: Qt.formatDateTime(root.now, Time.twelveHour ? "ss AP" : "ss")
                                    style: K.Tokens.type.headline
                                    color: K.Tokens.color.on_surface_variant
                                }
                            }
                        }

                        K.Row {
                            gap: K.Tokens.space.s8
                            align: "centre"

                            K.Text {
                                text: root.kanji[root.now.getDay()]
                                style: K.Tokens.type.accent
                                color: K.Tokens.color.tertiary
                            }

                            K.Text {
                                text: Qt.formatDateTime(root.now, Time.fullDateFormat)
                                style: K.Tokens.type.body_large
                            }
                        }

                        K.Text {
                            text: root.today ? `Day ${root.today.dayOfYear} · Week ${root.today.week}` : ""
                            style: K.Tokens.type.small
                            color: K.Tokens.color.on_surface_variant
                        }
                    }
                }

                K.Fill {}

                K.IconButton {
                    icon: root.page === "settings" ? "close" : "settings"
                    label: root.page === "settings" ? qsTr("Back") : qsTr("Calendar settings")
                    kind: root.page === "settings" ? "Filled" : "Text"
                    onActivated: {
                        root.page = root.page === "settings" ? "main" : "settings";
                        if (root.page === "main")
                            search.forceActiveFocus();
                    }
                }
            }

            // The extra clocks: the time there, how far off, day or night.
            Flow {
                readonly property bool fillWidth: true

                visible: (Time.state.clocks ?? []).length > 0 && root.page === "main"
                spacing: K.Tokens.space.s8

                Repeater {
                    model: Time.state.clocks ?? []

                    Item {
                        id: zone

                        required property var modelData
                        readonly property date there: new Date(root.now.getTime() + (modelData.offset - (Time.state.here?.offset ?? 0)) * 1000)

                        implicitWidth: zoneRow.implicitWidth + K.Tokens.space.s12 * 2
                        implicitHeight: K.Tokens.size.control_md
                        width: implicitWidth
                        height: implicitHeight

                        Rectangle {
                            anchors.fill: parent
                            radius: K.Tokens.radius.control
                            color: K.Tokens.color.surface_container
                        }

                        K.Row {
                            id: zoneRow

                            x: K.Tokens.space.s12
                            y: K.Look.snap((K.Tokens.size.control_md - implicitHeight) / 2, root.dpr)
                            gap: K.Tokens.space.s8
                            align: "centre"

                            K.Icon {
                                name: root.daylight(zone.there.getHours())
                                size: K.Tokens.icon.s20
                                color: name === "light_mode" ? K.Tokens.color.primary : K.Tokens.color.tertiary
                            }

                            K.Stack {
                                K.Text {
                                    text: zone.modelData.label
                                    style: K.Tokens.type.caption
                                    color: K.Tokens.color.on_surface_variant
                                }

                                Figure {
                                    text: Qt.formatDateTime(zone.there, Time.timeFormat)
                                    style: K.Tokens.type.body_strong
                                }
                            }

                            K.Text {
                                text: zone.modelData.relative
                                style: K.Tokens.type.small
                                color: K.Tokens.color.on_surface_variant
                            }
                        }
                    }
                }
            }

            // The date search, and what it comes to.
            K.Stack {
                visible: root.page === "main"
                fill: true
                gap: K.Tokens.space.s8

                // Keys it lets through: months and today while it's empty, the copies always, and
                // Esc clearing what was typed before it closes anything.
                K.Field {
                    id: search

                    icon: "event"
                    placeholder: "next friday, days until christmas, 12 mar to 4 jul, 15:00 tokyo"
                    onTextChanged: {
                        if (root.lastQuery === "" && text.toLowerCase() === "t") {
                            grid.yearView = false;
                            root.goToday(true);
                        }
                        root.lastQuery = text;
                        root.ask(text);
                    }
                    Keys.onEscapePressed: event => {
                        event.accepted = search.text !== "";
                        if (event.accepted)
                            search.text = "";
                    }
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
                        }
                    }
                }

                // What the search comes to, opening as a log does.
                Item {
                    readonly property bool fillWidth: true

                    implicitHeight: root.answer ? answerCard.implicitHeight : 0
                    // Shown from the answer, not from its height: a hidden layout never measures,
                    // so a height read while hidden stays 0. It stays shown while it closes.
                    visible: !!root.answer || implicitHeight > 0
                    clip: true

                    Behavior on implicitHeight {
                        NumberAnimation {
                            duration: K.Tokens.duration.long
                            easing.type: Easing.BezierSpline
                            easing.bezierCurve: K.Tokens.easing.emphasized_decel
                        }
                    }

                    K.Card {
                        id: answerCard

                        width: parent.width

                        K.Stack {
                            fill: true
                            gap: K.Tokens.space.s4

                            K.Text {
                                text: root.answer?.title ?? ""
                                style: K.Tokens.type.heading
                                color: K.Tokens.color.primary
                            }

                            K.Text {
                                text: root.answer?.detail ?? ""
                                style: K.Tokens.type.body
                                color: K.Tokens.color.on_surface_variant
                            }

                            K.Text {
                                visible: root.copied !== ""
                                text: `Copied ${root.copied}`
                                style: K.Tokens.type.caption
                                color: K.Tokens.color.tertiary
                            }

                            K.KeyHints {
                                visible: root.copied === ""
                                hints: [["Enter", "copies"], ["Ctrl+Enter", "copies ISO"]]
                            }
                        }
                    }
                }
            }

            // The month and the picked day; countdowns and the tools beside.
            K.Row {
                visible: root.page === "main"
                gap: root.columnGap

                K.Stack {
                    gap: K.Tokens.space.s12

                    MonthGrid {
                        id: grid

                        selected: root.selected
                        onPicked: date => root.land(date)
                        onMoved: months => root.move(months)
                        onPickedMonth: month => root.showMonth(root.viewYear, month, Math.sign(month - root.viewMonth))
                    }

                    // The picked day: how far, its weekday and week, its holiday,
                    // and the extra clocks' times at noon here. As wide as the month above
                    // it, which the stack around it would otherwise widen to the words.
                    Item {
                        visible: root.day !== null
                        implicitWidth: grid.implicitWidth
                        implicitHeight: dayDetails.implicitHeight

                        K.Stack {
                            id: dayDetails

                            width: parent.width
                            room: grid.implicitWidth
                            gap: K.Tokens.space.s4

                            K.Text {
                                text: root.day?.title ?? ""
                                style: K.Tokens.type.body_strong
                            }

                            K.Text {
                                text: root.day?.detail ?? ""
                                style: K.Tokens.type.small
                                color: K.Tokens.color.on_surface_variant
                            }

                            K.Text {
                                visible: !!root.day?.holiday
                                text: root.day?.holiday ?? ""
                                style: K.Tokens.type.small
                                color: K.Tokens.color.tertiary
                            }

                            K.Text {
                                visible: (root.day?.zones ?? []).length > 0
                                text: (root.day?.zones ?? []).map(z => `${z.label} ${z.time}`).join("  ·  ")
                                style: K.Tokens.type.small
                                color: K.Tokens.color.on_surface_variant
                            }
                        }
                    }
                }

                K.Stack {
                    id: side

                    room: K.Tokens.size.rail
                    gap: K.Tokens.space.s16

                    // Countdowns: what's left of each, and one to add.
                    K.Stack {
                        room: K.Tokens.size.rail
                        gap: K.Tokens.space.s8

                        K.Text {
                            text: "Countdowns"
                            style: K.Tokens.type.body_strong
                        }

                        Repeater {
                            model: Time.state.countdowns ?? []

                            K.Row {
                                id: countdown

                                required property var modelData
                                readonly property real remaining: modelData.end - root.now.getTime()

                                fill: true
                                gap: K.Tokens.space.s8
                                align: "centre"

                                K.Icon {
                                    name: Time.settings.barCountdown === countdown.modelData.name ? "push_pin" : "hourglass_top"
                                    size: K.Tokens.icon.s20
                                    color: K.Tokens.color.tertiary
                                }

                                Item {
                                    readonly property bool fillWidth: true

                                    implicitHeight: countdownWords.implicitHeight

                                    K.Stack {
                                        id: countdownWords

                                        width: parent.width
                                        room: parent.width
                                        fill: true

                                        K.Text {
                                            text: countdown.modelData.name
                                            style: K.Tokens.type.body_strong
                                            maximumLines: 1
                                        }

                                        K.Text {
                                            text: countdown.remaining > 0 ? `in ${Time.left(countdown.remaining)} · ${countdown.modelData.at}` : `${countdown.modelData.at} · passed`
                                            style: K.Tokens.type.caption
                                            color: K.Tokens.color.on_surface_variant
                                            maximumLines: 1
                                        }
                                    }
                                }

                                K.IconButton {
                                    icon: "close"
                                    label: qsTr("Remove")
                                    kind: "Text"
                                    size: "Sm"
                                    onActivated: Time.change("countdown-remove", {
                                        name: countdown.modelData.name
                                    }, error => root.countdownError = error)
                                }
                            }
                        }

                        K.Field {
                            id: countdownField

                            icon: "more_time"
                            placeholder: "Vacation: 19 dec"
                            Keys.onEscapePressed: countdownField.text = ""
                            Keys.onReturnPressed: root.addCountdown(countdownField.text)
                            Keys.onEnterPressed: root.addCountdown(countdownField.text)
                        }

                        K.Text {
                            visible: root.countdownError !== ""
                            text: root.countdownError
                            style: K.Tokens.type.small
                            color: K.Tokens.color.error
                        }
                    }

                    Tools {
                        id: tools
                    }
                }
            }

            ClockSettings {
                visible: root.page === "settings"
            }

            K.Text {
                visible: !Time.available
                text: "kubaos-clock isn't running: the calendar and the search need it"
                style: K.Tokens.type.small
                color: K.Tokens.color.error
            }
        }
    }

    // Where a style's baseline sits in its line box: the box centres the font's height.
    function baselineOf(metrics: FontMetrics, style: QtObject): real {
        return (style.line - metrics.height) / 2 + metrics.ascent;
    }

    FontMetrics {
        id: heroMetrics

        font.family: K.Tokens.type.hero_number.family
        font.weight: K.Tokens.type.hero_number.weight
        font.pixelSize: K.Tokens.type.hero_number.size
    }

    FontMetrics {
        id: secondsMetrics

        font.family: K.Tokens.type.headline.family
        font.weight: K.Tokens.type.headline.weight
        font.pixelSize: K.Tokens.type.headline.size
    }
}
