import QtQuick
import KubaOS.Components as K
import qs.services
import "panel.js" as Panel

// The busiest processes, by CPU or by memory, in compact rows on the panel's own surface, as many
// as fit in `tallest` and never fewer than five. The heading and the column heads show at once; a
// row not counted yet is a placeholder, and the process fades in over it, so nothing moves as the
// list arrives. A click on a row opens its details under it, with End (it may save and quit) and
// Kill (it stops at once), each asking first; a process shown that isn't in the list (one a
// Command menu Item asked for) sits above the rest with its details.
K.Stack {
    id: root

    // The tallest the processes may grow to fill the panel, the heading included. Not `room`,
    // which is the width a layout is offered.
    property real tallest: 0
    // The sample the panel shows.
    property var sample: null
    // Whether processes fade in over their placeholders (`Line`, `fades`).
    property bool fades: true
    property string order: "cpu"
    // The process shown, from the engine's `process` reply, or null.
    property var details: null
    // "term" or "kill" while its confirmation shows, else "".
    property string confirming: ""
    property string error: ""

    // Whether the processes are counted: the panel's samples are detailed, and only those scan them.
    readonly property bool counted: root.sample?.detail === true
    readonly property var list: root.counted ? (root.sample?.top?.[root.order] ?? []) : []
    readonly property int least: 5
    // The most rows there are: as many as a detailed sample lists (`TOP_DETAILED` in
    // `crates/kubaos-sysmon/src/sampler.rs`). They are all made once and those past the room
    // hidden, so a room that changes as the panel lays out makes and drops none.
    readonly property int most: 40
    readonly property int slots: Panel.slots(root.tallest - rows.y - heads.height, K.Tokens.size.row_compact, root.least, root.most)
    // Where the shown process is in the rows, or -1 where it isn't among them.
    readonly property int pickedAt: {
        const at = root.details ? root.list.findIndex(process => process.pid === root.details.pid) : -1;
        return at < root.slots ? at : -1;
    }
    // The numbers' columns are wide enough for "1023 MB" and "100.0%" in `mono_large` in a
    // panel's width; the name takes the rest.
    readonly property var columns: [
        {
            title: "Name",
            weight: 2
        },
        {
            title: "CPU",
            weight: 1,
            numeric: true
        },
        {
            title: "Memory",
            weight: 1,
            numeric: true
        }
    ]

    fill: true
    gap: K.Tokens.space.s8

    // Shows process `pid`; one that has exited says so.
    function show(pid: int): void {
        confirming = "";
        error = "";
        Sysmon.request("process", {
            pid: pid
        }, reply => {
            root.details = reply.process ?? null;
            root.error = reply.error ?? "";
        });
    }

    function clear(): void {
        details = null;
        confirming = "";
        error = "";
    }

    function stop(): void {
        const how = confirming;
        const pid = details.pid;
        confirming = "";
        Sysmon.stop(pid, how, error => {
            root.error = error;
            if (error === "")
                root.details = null;
        });
    }

    function toggle(pid: int): void {
        root.details?.pid === pid ? root.clear() : root.show(pid);
    }

    K.Row {
        fill: true
        gap: K.Tokens.space.s8
        align: "centre"

        K.Icon {
            name: "list_alt"
            size: K.Tokens.type.body_strong.icon
            color: K.Tokens.color.primary
        }

        K.Text {
            text: "Processes"
            style: K.Tokens.type.body_strong
        }

        K.Fill {
        }

        // The order is picked with the pointer only: it is no stop for Tab.
        K.Tabs {
            labels: ["CPU", "Memory"]
            currentIndex: root.order === "cpu" ? 0 : 1
            activeFocusOnTab: false
            onPicked: index => root.order = index === 0 ? "cpu" : "memory"
        }
    }

    K.Stack {
        id: rows

        fill: true

        // The column heads alone: the rows under them are `ProcessRow`s, so a row's details can
        // open under it.
        K.Table {
            id: heads

            columns: root.columns
            rows: []
        }

        ProcessRow {
            visible: root.details !== null && root.pickedAt < 0
            owner: root
            process: root.details
        }

        Repeater {
            model: root.most

            ProcessRow {
                id: slot

                required property int index

                visible: slot.index < root.slots
                owner: root
                slotIndex: slot.index
                kind: Panel.slot(slot.index, root.counted, root.list.length)
                process: slot.kind === "process" ? root.list[slot.index] : null
            }
        }
    }

    // The engine's reply when the shown process couldn't be read, with no row to open under.
    K.Text {
        visible: root.error !== "" && root.details === null
        text: root.error
        style: K.Tokens.type.small
        color: K.Tokens.color.error
    }

    K.Text {
        visible: root.counted && root.list.length === 0
        text: "No processes to show"
        style: K.Tokens.type.small
        color: K.Tokens.color.on_surface_variant
    }
}
