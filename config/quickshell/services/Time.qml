pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.settings

// The clock popup's state, and its timer, stopwatch and alarms.
//
// Dates come from `kubaos-clock serve`, one process for the shell's life: a
// request is a line of JSON on its stdin, the reply a line on its stdout with
// the request's id. A keystroke in the date search costs a line each way, not
// a process. The settings file (~/.config/kubaos/desktop/clock/clock.toml) is watched,
// so a change from the Command menu shows at once.
//
// Timers and alarms live here, in memory: they ring with a notification and
// a sound, whether the popup is open or not, and are gone after a shell
// restart.
Singleton {
    id: root

    readonly property string configDir: UserLayer.dir("clock")
    readonly property string sound: (Quickshell.env("XDG_DATA_HOME") || `${Quickshell.env("HOME")}/.local/share`) + "/kubaos/clock/alarm.oga"

    // What `{"op":"state"}` answers: settings, today, clocks, countdowns.
    property var state: ({})
    readonly property bool ready: state.settings !== undefined
    readonly property var settings: state.settings ?? ({})
    // The engine's reason the settings were not read, or "".
    readonly property string problem: state.problem ?? ""
    // The engine is running; false where kubaos-clock isn't installed.
    readonly property bool available: engine.running

    // How the shell writes times and dates: the clock settings' `time_format`
    // and `date_order`, else the locale's, which Qt reads from LC_TIME. Until
    // the engine answers, the locale's.
    readonly property bool twelveHourByLocale: root.localeTwelveHour(Qt.locale().timeFormat(Locale.ShortFormat))
    readonly property bool dayFirstByLocale: root.localeDayFirst(Qt.locale().dateFormat(Locale.LongFormat))
    readonly property bool twelveHour: settings.timeFormat ? settings.timeFormat === "12h" : twelveHourByLocale
    readonly property bool dayFirst: settings.dateOrder ? settings.dateOrder === "day-month" : dayFirstByLocale
    // Hours and minutes alone, for a face that puts the seconds or AM/PM apart.
    readonly property string clockFormat: twelveHour ? "h:mm" : "HH:mm"
    readonly property string timeFormat: twelveHour ? "h:mm AP" : "HH:mm"
    readonly property string shortDateFormat: dayFirst ? "ddd d MMM" : "ddd MMM d"
    readonly property string dateFormat: dayFirst ? "dddd, d MMMM" : "dddd, MMMM d"
    readonly property string fullDateFormat: dayFirst ? "dddd, d MMMM yyyy" : "dddd, MMMM d, yyyy"

    // The bar's countdown, if the settings name one that exists.
    readonly property var barCountdown: (state.countdowns ?? []).find(c => c.name === settings.barCountdown) ?? null

    // Timer: when it rings (ms since the epoch) and how long it was set for.
    property real timerEnd: 0
    property real timerTotal: 0
    // Paused: what was left; 0 when running or unset.
    property real timerLeft: 0
    readonly property bool timerSet: timerEnd > 0 || timerLeft > 0

    // Stopwatch: ms counted before the current run, and when the run started.
    property real stopwatchBase: 0
    property real stopwatchStart: 0
    readonly property bool stopwatchRunning: stopwatchStart > 0
    property var laps: []

    // Alarms: { at (ms), label }, soonest first.
    property var alarms: []
    // Countdowns that have rung, as "name@end".
    property var rung: []

    // Now, to the second, while anything shown counts seconds; to the minute
    // otherwise.
    property bool seconds: false
    readonly property date now: clock.date

    // What `qs ipc call clock ...` asked the popup for, until it shows.
    property string pendingPage: ""
    property string pendingSearch: ""

    signal ringing(string title, string body)
    signal showRequested

    // Opens the popup on `page` ("settings", "timer", "stopwatch", "alarm"
    // or "") with `search` typed.
    function show(page: string, search: string): void {
        pendingPage = page;
        pendingSearch = search;
        Panels.open("clock");
        showRequested();
    }

    function notify(title: string, body: string): void {
        Quickshell.execDetached(["notify-send", "-a", "Clock", "-i", "alarm-symbolic", title, body]);
    }

    property int nextId: 1
    property var waiting: ({})

    // Sends `op` with `fields`, and calls `done(reply)` with the reply.
    function request(op: string, fields: var, done: var): void {
        if (!engine.running) {
            done?.({ error: "kubaos-clock isn't running" });
            return;
        }
        const id = root.nextId++;
        if (done)
            root.waiting[id] = done;
        engine.write(JSON.stringify(Object.assign({ id: id, op: op }, fields ?? {})) + "\n");
    }

    function refresh(): void {
        request("state", {}, reply => {
            if (!reply.error)
                root.state = reply;
        });
    }

    // A change to the settings: the reply is the new state, or the reason it
    // was refused, which `done` gets.
    function change(op: string, fields: var, done: var): void {
        request(op, fields, reply => {
            if (!reply.error)
                root.state = reply;
            done?.(reply.error ?? "");
        });
    }

    function copy(text: string): void {
        if (text !== "")
            Quickshell.execDetached(["wl-copy", "--", text]);
    }

    // "12 d 4 h", "3 h 20 min", "4 min 10 s": what's left of `ms`, two units.
    // A Qt date or time format without its quoted literals, whose letters
    // would read as fields: es writes "d 'de' MMMM".
    function fields(format: string): string {
        return format.replace(/'[^']*'/g, "");
    }

    // The locale's short time format has an AM/PM field.
    function localeTwelveHour(format: string): bool {
        return /a/i.test(root.fields(format));
    }

    // The locale's long date format puts the day of the month before the
    // month; the weekday's name (ddd, dddd) doesn't count.
    function localeDayFirst(format: string): bool {
        const plain = root.fields(format).replace(/d{3,}/g, "");
        const day = plain.indexOf("d"), month = plain.indexOf("M");
        return day >= 0 && (month < 0 || day < month);
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

    // "1:05:09" or "05:09": a timer's or stopwatch's face, with tenths if asked.
    function face(ms: real, tenths: bool): string {
        const t = Math.max(0, ms);
        const s = Math.floor(t / 1000);
        const pad = n => String(n).padStart(2, "0");
        const h = Math.floor(s / 3600);
        let text = h > 0 ? `${h}:${pad(Math.floor(s % 3600 / 60))}:${pad(s % 60)}` : `${pad(Math.floor(s / 60))}:${pad(s % 60)}`;
        if (tenths)
            text += `.${Math.floor(t % 1000 / 100)}`;
        return text;
    }

    // Starts a timer for `text` (`25m`, `1h30m`, `90 seconds`); `done(error)`.
    function startTimer(text: string, done: var): void {
        request("duration", { text: text }, reply => {
            if (!reply.ms) {
                done?.(reply.error ?? `"${text}" isn't a duration: try 25m, 1h30m or 90 seconds`);
                return;
            }
            root.timerTotal = reply.ms;
            root.timerLeft = 0;
            root.timerEnd = Date.now() + reply.ms;
            done?.("");
        });
    }

    function pauseTimer(): void {
        if (timerEnd > 0) {
            timerLeft = Math.max(0, timerEnd - Date.now());
            timerEnd = 0;
        } else if (timerLeft > 0) {
            timerEnd = Date.now() + timerLeft;
            timerLeft = 0;
        }
    }

    function cancelTimer(): void {
        timerEnd = 0;
        timerLeft = 0;
        timerTotal = 0;
    }

    function stopwatchElapsed(at: real): real {
        return stopwatchBase + (stopwatchStart > 0 ? at - stopwatchStart : 0);
    }

    function toggleStopwatch(): void {
        if (stopwatchStart > 0) {
            stopwatchBase += Date.now() - stopwatchStart;
            stopwatchStart = 0;
        } else {
            stopwatchStart = Date.now();
        }
    }

    function lap(): void {
        if (stopwatchRunning)
            laps = [stopwatchElapsed(Date.now()), ...laps].slice(0, 20);
    }

    function resetStopwatch(): void {
        stopwatchBase = 0;
        stopwatchStart = 0;
        laps = [];
    }

    // Sets an alarm at `text` (`7:30`, `6pm`, `friday 17:00`); `done(error)`.
    function addAlarm(text: string, done: var): void {
        request("alarm", { text: text }, reply => {
            if (!reply.at) {
                done?.(reply.error ?? `"${text}" isn't a time to come: try 7:30, 6pm or friday 17:00`);
                return;
            }
            root.alarms = [...root.alarms, { at: reply.at, label: reply.label }].sort((a, b) => a.at - b.at);
            done?.("");
        });
    }

    function removeAlarm(index: int): void {
        alarms = alarms.filter((_, i) => i !== index);
    }

    function ring(title: string, body: string): void {
        Quickshell.execDetached(["notify-send", "-a", "Clock", "-i", "alarm-symbolic", "-u", "critical", title, body]);
        Quickshell.execDetached(["pw-play", root.sound]);
        root.ringing(title, body);
    }

    function check(): void {
        const t = Date.now();
        if (timerEnd > 0 && t >= timerEnd) {
            const total = timerTotal;
            cancelTimer();
            ring("Time's up", `The ${left(total)} timer is done`);
        }
        while (alarms.length > 0 && t >= alarms[0].at) {
            const alarm = alarms[0];
            alarms = alarms.slice(1);
            ring("Alarm", alarm.label);
        }
        // Both clocks check inside the window, so a countdown rings once by
        // its name and end.
        for (const countdown of state.countdowns ?? []) {
            const key = `${countdown.name}@${countdown.end}`;
            if (countdown.end && countdown.end <= t && countdown.end > t - 1500 && !rung.includes(key)) {
                rung = [key, ...rung].slice(0, 20);
                ring(countdown.name, "The countdown has reached zero");
            }
        }
    }

    // What `qs ipc call clock status` says.
    function status(): var {
        return {
            available: root.available,
            ready: root.ready,
            problem: root.problem,
            today: state.today ?? null,
            zones: (state.clocks ?? []).map(c => c.name),
            countdowns: (state.countdowns ?? []).map(c => c.name),
            timer: timerSet ? { left: timerEnd > 0 ? timerEnd - Date.now() : timerLeft, running: timerEnd > 0 } : null,
            stopwatch: { elapsed: stopwatchElapsed(Date.now()), running: stopwatchRunning },
            alarms: alarms.map(a => a.label)
        };
    }

    SystemClock {
        id: clock

        precision: root.seconds || root.timerEnd > 0 || root.stopwatchRunning ? SystemClock.Seconds : SystemClock.Minutes
        // The day, the week and the countdowns move on: the state is read
        // again each minute, which costs a line.
        onDateChanged: {
            root.check();
            if (date.getSeconds() === 0 || clock.precision === SystemClock.Minutes)
                root.refresh();
        }
    }

    // A timer or an alarm rings to the second even while the clock counts
    // minutes.
    Timer {
        interval: 1000
        repeat: true
        running: root.timerEnd > 0 || root.alarms.length > 0 || (root.state.countdowns ?? []).length > 0
        onTriggered: root.check()
    }

    Process {
        id: engine

        command: ["kubaos-clock", "serve"]
        running: true
        stdinEnabled: true
        stdout: SplitParser {
            onRead: line => {
                let reply;
                try {
                    reply = JSON.parse(line);
                } catch (e) {
                    console.warn(`Time: unreadable reply from kubaos-clock: ${line}`);
                    return;
                }
                restart.failures = 0;
                const done = root.waiting[reply.id];
                delete root.waiting[reply.id];
                done?.(reply);
            }
        }
        onStarted: root.refresh()
        // A crash or an update of the binary: start again, a little later,
        // and later each time it fails before answering, up to a minute.
        onExited: (code, status) => {
            root.waiting = {};
            restart.interval = Math.min(60000, 2000 * Math.pow(2, restart.failures++));
            restart.restart();
        }
    }

    Timer {
        id: restart

        property int failures: 0

        onTriggered: engine.running = true
    }

    FileView {
        path: `${root.configDir}/clock.toml`
        watchChanges: true
        printErrors: false
        onFileChanged: root.refresh()
    }
}
