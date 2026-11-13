pragma Singleton
import QtQuick

// Stand-in for Counters: four counters, one counting every day and one stopped short of its stop,
// a history of a few days for each but the last, and an Auto reset box that reads "every day at 9".
// Changes do nothing; a driver sets `list` for a state.
QtObject {
    id: root

    readonly property string runtime: "/nonexistent/run"
    property var list: [
        {
            id: 1,
            name: "Coffee",
            value: 3,
            stop: 0,
            rule: "every day at 9:00",
            next: 1793170800,
            detail: "Resets every day at 9:00"
        },
        {
            id: 2,
            name: "Glasses of water",
            value: 7,
            stop: 8,
            rule: "",
            next: 0,
            detail: "Stops at 8"
        },
        {
            id: 3,
            name: "Push-ups",
            value: 120,
            stop: 0,
            rule: "every monday at 0:00",
            next: 1793660400,
            detail: "Resets every Monday at 0:00"
        },
        {
            id: 4,
            name: "Counter",
            value: 0,
            stop: 0,
            rule: "",
            next: 0,
            detail: ""
        }
    ]
    property string last: "Coffee"
    property var reading: ({
            typed: "",
            ok: false,
            examples: root.examples
        })
    readonly property var examples: ["every day at 9", "mondays 8am", "every 2 hours", "1st of the month"]

    function run(args: var): void {
    }

    function create(name: string): void {
    }

    function add(name: string): void {
    }

    function sub(name: string): void {
    }

    function set(name: string, value: int): void {
    }

    function reset(name: string): void {
    }

    function rename(name: string, to: string): void {
    }

    function setRule(name: string, rule: string): void {
    }

    function setStop(name: string, stop: int): void {
    }

    function remove(name: string): void {
    }

    // Reads "every day at 9" and nothing else.
    function check(typed: string): void {
        if (/^every day at 9$/.test(typed.trim()))
            root.reading = {
                typed: typed,
                ok: true,
                rule: "every day at 9:00",
                words: "Every day at 9:00",
                next: 1793170800,
                when: "Next tomorrow at 9:00"
            };
        else
            root.reading = {
                typed: typed,
                ok: false,
                error: "Can't read that",
                examples: root.examples
            };
    }

    // A counter's changes over the last few days, a few hours apart, the same on every run: +1
    // mostly, now and then −1, a set or a reset. Grouped and sorted as the command does.
    function history(name: string, group: string, sort: string, done: var): void {
        const counter = root.list.find(c => c.name === name);
        const count = !counter || counter.value === 0 ? 0 : counter.id === 3 ? 80 : 40;
        const kinds = ["add", "add", "add", "sub", "add", "set", "add", "reset"];
        const labels = {
            add: "+1",
            sub: "−1",
            set: "Set",
            reset: "Reset"
        };
        // Tuesday 27 October 2026, 21:37, then back a few hours each.
        let at = 1793137020;
        let value = counter?.value ?? 0;
        const changes = [];
        for (let i = 0; i < count; i++) {
            const kind = i === 0 ? "set" : kinds[(i * 5) % kinds.length];
            const added = kind === "add" ? 1 : kind === "sub" ? -1 : 0;
            changes.push({
                at: at,
                kind: kind,
                label: labels[kind],
                value: value,
                added: added,
                changes: 1
            });
            value = Math.max(0, value - added) + (kind === "set" || kind === "reset" ? 4 : 0);
            at -= 3600 * (2 + (i * 7) % 5) + 60 * ((i * 13) % 50);
        }
        let rows = changes;
        if (group !== "change") {
            const groups = new Map();
            for (const change of changes) {
                const day = new Date(change.at * 1000);
                day.setHours(0, 0, 0, 0);
                if (group === "week")
                    day.setDate(day.getDate() - (day.getDay() + 6) % 7);
                if (group === "month")
                    day.setDate(1);
                const key = day.getTime() / 1000;
                const sum = groups.get(key) ?? {
                    at: key,
                    kind: "",
                    label: "",
                    value: change.value,
                    added: 0,
                    changes: 0
                };
                sum.added += change.added;
                sum.changes += 1;
                groups.set(key, sum);
            }
            rows = [...groups.values()];
        }
        if (sort === "oldest")
            rows.reverse();
        else if (sort === "highest")
            rows.sort((a, b) => group === "change" ? b.value - a.value : b.added - a.added);
        done(rows);
    }
}
