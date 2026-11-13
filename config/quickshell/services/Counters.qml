pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The counters, from the state file `kubaos-counters` writes whole after every change
// ($XDG_RUNTIME_DIR/kubaos-counters.json, `docs/CONTRACTS.md`, "Counters"): watched, never polled.
// Every change goes through the command, which writes the file again, so what the bar shows is
// always what the counters are. What a rule reads as and a counter's history are asked of it too.
Singleton {
    id: root

    readonly property string runtime: Quickshell.env("XDG_RUNTIME_DIR")
    // `{ id, name, value, stop, rule, next, detail }` each, in their order.
    property var list: []
    // The name the key adds to, or "".
    property string last: ""

    // What the last rule typed into an Auto reset box reads as, as `kubaos-counters check --json`
    // says it: `{ typed, ok, rule, words, when }` or `{ typed, ok: false, error, examples }`.
    property var reading: ({
            typed: "",
            ok: false,
            examples: []
        })
    // The rule asked about last, which `reading` answers once its turn comes.
    property string pendingCheck: ""

    // `kubaos-counters ARGS`: the change is in the state file once it has run.
    function run(args: var): void {
        Quickshell.execDetached(["kubaos-counters", ...args]);
    }

    function create(name: string): void {
        root.run(["new", name]);
    }

    function add(name: string): void {
        root.run(["add", name]);
    }

    function sub(name: string): void {
        root.run(["sub", name]);
    }

    function set(name: string, value: int): void {
        root.run(["set", name, String(value)]);
    }

    function reset(name: string): void {
        root.run(["reset", name]);
    }

    function rename(name: string, to: string): void {
        root.run(["rename", name, to]);
    }

    // An auto reset rule as typed, or "never" to take the rule or the stop away.
    function setRule(name: string, rule: string): void {
        root.run(["rule", name, rule]);
    }

    function setStop(name: string, stop: int): void {
        root.run(["stop", name, String(stop)]);
    }

    function remove(name: string): void {
        root.run(["remove", name]);
    }

    // Asks what `typed` reads as; `reading` holds the answer once it comes. Only the last asked is
    // kept: one typed while another is read waits for it, so the answers never arrive out of order.
    function check(typed: string): void {
        root.pendingCheck = typed;
        if (!checker.running)
            root.nextCheck();
    }

    function nextCheck(): void {
        const typed = root.pendingCheck;
        if (typed.trim() === "") {
            root.reading = {
                typed: typed,
                ok: false,
                examples: root.reading.examples ?? []
            };
            return;
        }
        checker.typed = typed;
        checker.command = ["kubaos-counters", "check", typed, "--json"];
        checker.running = true;
    }

    // A counter's history, `group` "change", "day", "week" or "month", `sort` "newest", "oldest" or
    // "highest": the rows `kubaos-counters log --json` gives, to `done(rows)`.
    function history(name: string, group: string, sort: string, done: var): void {
        const reader = historyReader.createObject(root, {
            done: done
        });
        reader.command = ["kubaos-counters", "log", name, "--group", group, "--sort", sort, "--json"];
        reader.running = true;
    }

    function parse(text: string): var {
        try {
            return JSON.parse(text) ?? {};
        } catch (e) {
            return {};
        }
    }

    function load(text: string): void {
        const state = root.parse(text);
        root.list = state.counters ?? [];
        root.last = state.last ?? "";
    }

    FileView {
        id: file

        path: `${root.runtime}/kubaos-counters.json`
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.load(text())
        // Not written yet this login (the login's `tick` writes it): `tick` writes it now, from the
        // counters there are, so the watch has a file to follow.
        onLoadFailed: {
            root.load("");
            writer.running = true;
        }
    }

    Process {
        id: writer

        command: ["kubaos-counters", "tick"]
        onExited: file.reload()
    }

    Process {
        id: checker

        property string typed

        stdout: StdioCollector {
            onStreamFinished: {
                const answer = root.parse(text);
                answer.typed = checker.typed;
                root.reading = answer;
            }
        }
        onExited: {
            if (root.pendingCheck !== checker.typed)
                root.nextCheck();
        }
    }

    Component {
        id: historyReader

        Process {
            id: reader

            property var done

            stdout: StdioCollector {
                onStreamFinished: {
                    const rows = root.parse(text);
                    reader.done(Array.isArray(rows) ? rows : []);
                }
            }
            onExited: reader.destroy()
        }
    }
}
