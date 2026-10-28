pragma Singleton
import QtQuick

// Stand-in for the clock engine: a frozen evening, Tuesday 27 October 2026 21:47:32, a holiday on
// 1 November, an extra clock, two countdowns, a running timer and two alarms. A question typed
// into the search gets one answer.
QtObject {
    id: root

    readonly property date now: new Date(2026, 9, 27, 21, 47, 32)
    property var state: ({
            today: {
                date: "2026-10-27",
                dayOfYear: 300,
                week: 44
            },
            settings: {
                holidays: "pl",
                firstWeekday: 1,
                analog: false,
                barCountdown: "Vacation"
            },
            country: "Poland",
            countries: [
                {
                    code: "pl",
                    name: "Poland"
                },
                {
                    code: "jp",
                    name: "Japan"
                },
                {
                    code: "de",
                    name: "Germany"
                }
            ],
            here: {
                // design-allow(size): Warsaw's UTC offset in seconds, not a length.
                offset: 3600
            },
            clocks: [
                {
                    label: "Tokyo",
                    name: "Asia/Tokyo",
                    // design-allow(size): Tokyo's UTC offset in seconds, not a length.
                    offset: 32400,
                    relative: "+8 h"
                }
            ],
            countdowns: [
                {
                    name: "Vacation",
                    at: "19 Dec",
                    end: new Date(2026, 11, 19).getTime()
                },
                {
                    name: "Release",
                    at: "3 Nov",
                    end: new Date(2026, 10, 3).getTime()
                }
            ]
        })
    readonly property bool ready: true
    readonly property var settings: state.settings
    readonly property string problem: ""
    readonly property bool available: true
    readonly property string sound: ""
    readonly property bool twelveHourByLocale: false
    readonly property bool dayFirstByLocale: true
    readonly property bool twelveHour: false
    readonly property bool dayFirst: true
    readonly property string clockFormat: "HH:mm"
    readonly property string timeFormat: "HH:mm"
    readonly property string shortDateFormat: "ddd d MMM"
    readonly property string dateFormat: "dddd, d MMMM"
    readonly property string fullDateFormat: "dddd, d MMMM yyyy"
    readonly property var barCountdown: state.countdowns[0]
    property real timerEnd: now.getTime() + 12 * 60000 + 8000
    property real timerTotal: 25 * 60000
    property real timerLeft: 0
    readonly property bool timerSet: timerEnd > 0 || timerLeft > 0
    property real stopwatchBase: 0
    property real stopwatchStart: 0
    readonly property bool stopwatchRunning: stopwatchStart > 0
    property var laps: []
    property var alarms: [
        {
            label: "Tomorrow 07:30"
        },
        {
            label: "Friday 17:00"
        }
    ]
    property var rung: []
    property bool seconds: false
    property string pendingPage: ""
    property string pendingSearch: ""

    signal ringing(string title, string body)
    signal showRequested

    function pad(n: int): string {
        return String(n).padStart(2, "0");
    }

    function iso(d: date): string {
        return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
    }

    function month(year: int, month: int): var {
        const names = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
        const first = new Date(year, month - 1, 1);
        const start = new Date(year, month - 1, 1 - (first.getDay() + 6) % 7);
        const weeks = [];
        for (let w = 0; w < 6; w++) {
            const days = [];
            let number = 0;
            for (let i = 0; i < 7; i++) {
                const d = new Date(start.getFullYear(), start.getMonth(), start.getDate() + w * 7 + i);
                const date = iso(d);
                if (i === 3)
                    number = Math.ceil(((d - new Date(d.getFullYear(), 0, 1)) / 86400000 + 1) / 7);
                days.push({
                    date: date,
                    day: d.getDate(),
                    today: date === "2026-10-27",
                    holiday: date === "2026-11-01" ? "All Saints' Day" : date === "2026-11-11" ? "Independence Day" : "",
                    weekend: i >= 5,
                    inMonth: d.getMonth() === month - 1
                });
            }
            weeks.push({
                number: number,
                days: days
            });
        }
        return {
            year: year,
            month: month,
            title: `${names[month - 1]} ${year}`,
            weekdays: ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"],
            weeks: weeks
        };
    }

    function show(page: string, search: string): void {
        root.pendingPage = page;
        root.pendingSearch = search;
        root.showRequested();
    }

    function notify(title: string, body: string): void {
    }

    function request(op: string, fields: var, done: var): void {
        if (op === "month")
            done({
                month: root.month(fields.year, fields.month)
            });
        else if (op === "day")
            done({
                day: {
                    title: "Today, Tuesday 27 October",
                    detail: "Week 44 · day 300 · 3 working days to Friday",
                    holiday: "",
                    zones: [
                        {
                            label: "Tokyo",
                            time: "05:47"
                        }
                    ]
                }
            });
        else if (op === "query")
            done({
                answer: fields.text === "" ? null : {
                    title: "53 days",
                    detail: "From today to Saturday 19 December 2026",
                    iso: "P53D",
                    date: ""
                }
            });
        else
            done({});
    }

    function refresh(): void {
    }

    function change(op: string, fields: var, done: var): void {
        done?.("");
    }

    function copy(text: string): void {
    }

    function fields(format: string): string {
        return format;
    }

    function localeTwelveHour(format: string): bool {
        return false;
    }

    function localeDayFirst(format: string): bool {
        return true;
    }

    function left(ms: real): string {
        const s = Math.max(0, Math.round(ms / 1000));
        const d = Math.floor(s / 86400), h = Math.floor(s % 86400 / 3600), m = Math.floor(s % 3600 / 60);
        if (d > 0)
            return `${d} d ${h} h`;
        if (h > 0)
            return `${h} h ${m} min`;
        return m > 0 ? `${m} min ${s % 60} s` : `${s % 60} s`;
    }

    function face(ms: real, tenths: bool): string {
        const s = Math.floor(Math.max(0, ms) / 1000);
        const h = Math.floor(s / 3600);
        return h > 0 ? `${h}:${pad(Math.floor(s % 3600 / 60))}:${pad(s % 60)}` : `${pad(Math.floor(s / 60))}:${pad(s % 60)}`;
    }

    function startTimer(text: string, done: var): void {
        done?.("");
    }

    function pauseTimer(): void {
    }

    function cancelTimer(): void {
    }

    function stopwatchElapsed(at: real): real {
        return 754000;
    }

    function toggleStopwatch(): void {
    }

    function lap(): void {
    }

    function resetStopwatch(): void {
    }

    function addAlarm(text: string, done: var): void {
        done?.("");
    }

    function removeAlarm(index: int): void {
    }

    function ring(title: string, body: string): void {
    }

    function check(): void {
    }

    function status(): var {
        return ({});
    }
}
