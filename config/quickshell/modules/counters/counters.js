// What the counters' sheet says and checks, apart from how it draws it: the value box's widest
// value, the name a new counter is offered, why a typed name, count or stop can't be taken, what an
// Auto reset box reads, the counter's menu and its history's rows. Counters are the state file's
// (`services/Counters.qml`); the history's rows are `kubaos-counters log --json`'s.
.pragma library

// The name a new counter takes when nothing else is typed: the command's own.
const DEFAULT_NAME = "Counter";

// The rows' value box: as wide as the widest value or stop any of them shows, every figure a 0.
function widest(counters) {
    let most = 1;
    for (const counter of counters)
        most = Math.max(most, String(counter.value).length, counter.stop > 0 ? String(counter.stop).length : 0);
    return "0".repeat(most);
}

// Whether `name` is taken by a counter but `except`, whatever the case, as the command matches it.
function taken(counters, name, except) {
    const wanted = name.trim().toLowerCase();
    return counters.some(counter => counter.name.toLowerCase() === wanted && counter.name !== except);
}

// The name the New counter prompt opens with: "Counter", or the first "Counter N" that is free,
// as the command names one it is given no name for.
function nextName(counters) {
    if (!taken(counters, DEFAULT_NAME, null))
        return DEFAULT_NAME;
    for (let n = 2; ; n++) {
        const name = `${DEFAULT_NAME} ${n}`;
        if (!taken(counters, name, null))
            return name;
    }
}

// Why `typed` can't name a counter, or "": it can't be empty or another counter's. `except` is the
// name it replaces, which it may keep.
function checkName(typed, counters, except) {
    if (typed.trim() === "")
        return "A name can't be empty.";
    if (taken(counters, typed, except))
        return `There's a counter named ${typed.trim()} already.`;
    return "";
}

// Why `typed` can't be a counter's value, or "": a whole number, 0 or more.
function checkCount(typed) {
    return /^\s*\d+\s*$/.test(typed) ? "" : "A whole number, 0 or more.";
}

// Why `typed` can't be where a counter stops, or "": a whole number, 1 or more.
function checkStop(typed) {
    return /^\s*\d+\s*$/.test(typed) && Number(typed) > 0 ? "" : "A whole number, 1 or more.";
}

// The phrases an Auto reset box reads, as the command offers them.
function tryWords(examples) {
    return `Try ${examples.map(example => `“${example}”`).join(", ")}`;
}

// What an Auto reset box's note says of `typed`, from `reading`, the command's answer for the last
// rule it was asked about: the rule and its next reset, or the phrases it reads.
function ruleNote(typed, reading) {
    if (reading.ok && reading.typed === typed) {
        const words = reading.words ?? "";
        const phrase = words.charAt(0).toLowerCase() + words.slice(1);
        return reading.when ? `Resets ${phrase}. ${reading.when}` : `Resets ${phrase}`;
    }
    return tryWords(reading.examples ?? []);
}

// Why an Auto reset box can't take `typed`, or "": a rule the command reads.
function checkRule(typed, reading) {
    if (reading.ok && reading.typed === typed)
        return "";
    const phrases = tryWords(reading.examples ?? []);
    return typed.trim() === "" ? phrases : `Can't read that. ${phrases}`;
}

// What a counter's menu does by the index of its entry.
const MENU = ["rename", "reset", "auto", "history", "", "remove"];

// A counter's menu: Rename, Reset now (not at 0), Auto reset, History, then Hold to remove.
function menuEntries(counter) {
    return [
        {
            text: "Rename",
            icon: "draw"
        },
        {
            text: "Reset now",
            icon: "restart_alt",
            enabled: counter.value > 0
        },
        {
            text: "Auto reset",
            icon: "repeat"
        },
        {
            text: "History",
            icon: "history"
        },
        {
            separator: true
        },
        {
            text: "Hold to remove",
            icon: "delete",
            held: true
        }
    ];
}

// The stop When it reaches offers: the counter's, or 10 for one that has none.
function stopOf(counter) {
    return counter.stop > 0 ? counter.stop : 10;
}

// Auto reset's choices after its type box: Never, and When it reaches N, the counter's ticked.
function autoEntries(counter) {
    return [
        {
            text: "Never",
            checkable: true,
            checked: counter.rule === "" && !(counter.stop > 0)
        },
        {
            text: `When it reaches ${stopOf(counter)}`,
            checkable: true,
            checked: counter.stop > 0
        }
    ];
}

// The history's groupings and sorts, as the command names them, and how their tabs read.
const GROUPS = ["change", "day", "week", "month"];
const GROUP_LABELS = ["Each change", "Day", "Week", "Month"];
const SORTS = ["newest", "oldest", "highest"];

function sortLabels(group) {
    return ["Newest", "Oldest", group === "change" ? "Highest" : "Most added"];
}

// Whether `a` and `b` are the same day where the shell is.
function sameDay(a, b) {
    return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

// A history row's day: "Today", or the date as `formats.date` writes it.
function dayOf(at, now, formats) {
    return sameDay(at, now) ? "Today" : Qt.formatDateTime(at, formats.date);
}

// A history row as it reads: `{ title, what, value, sign }`. A change is its day and time, its kind
// and the value after it; a group is its day, week or month, how many changes it holds and how
// much they added, never below +0. `formats` are the shell's: `{ date, time, week, month }` for
// `Qt.formatDateTime`, as the rest of the shell writes its dates.
function historyLine(row, group, now, formats) {
    const at = new Date(row.at * 1000);
    if (group === "change") {
        return {
            title: `${dayOf(at, now, formats)} · ${Qt.formatDateTime(at, formats.time)}`,
            what: row.label,
            value: row.value,
            sign: ""
        };
    }
    let title = dayOf(at, now, formats);
    if (group === "week")
        title = `Week of ${Qt.formatDateTime(at, formats.week)}`;
    else if (group === "month")
        title = Qt.formatDateTime(at, formats.month);
    return {
        title: title,
        what: row.changes === 1 ? "1 change" : `${row.changes} changes`,
        // A group says only what it added: one that went down added nothing.
        value: Math.max(0, row.added),
        sign: "+"
    };
}

// The value box of a history's rows: as wide as its widest value, every figure a 0.
function historyWidest(lines) {
    let most = 1;
    for (const line of lines)
        most = Math.max(most, String(line.value).length);
    return "0".repeat(most);
}

// How many changes a counter's history holds, as its head says it.
function changesWords(count) {
    return count === 1 ? "1 change" : `${count} changes`;
}
