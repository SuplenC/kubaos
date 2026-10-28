import QtQuick
import KubaOS.Components as K
import qs.services

// The five busiest processes, by CPU or by memory. A click shows one's
// details, with End (it may save and quit) and Kill (it stops at once), each
// asking first.
K.Stack {
    id: root

    property string order: "cpu"
    // The process shown, from the engine's `process` reply, or null.
    property var details: null
    // "term" or "kill" while its confirmation shows, else "".
    property string confirming: ""
    property string error: ""
    readonly property var list: Sysmon.sample?.top?.[order] ?? []

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

    // `text` padded at the start to `width` characters with no-break spaces, so the numbers of
    // every row end in one column in mono.
    function padded(text: string, width: int): string {
        return " ".repeat(Math.max(0, width - text.length)) + text;
    }

    K.Row {
        fill: true
        gap: K.Tokens.space.s8
        align: "centre"

        K.Icon {
            name: "list_alt"
            size: K.Tokens.icon.s20
            color: K.Tokens.color.primary
        }

        K.Text {
            text: "Processes"
            style: K.Tokens.type.body_strong
        }

        K.Fill {
        }

        // The order is picked with the pointer only, as before: it is no stop for Tab.
        K.Tabs {
            labels: ["CPU", "Memory"]
            currentIndex: root.order === "cpu" ? 0 : 1
            activeFocusOnTab: false
            onPicked: index => root.order = index === 0 ? "cpu" : "memory"
        }
    }

    // The rows sit on the popup's own surface, 4 apart as in a list's well: a well round them
    // would take the room of a sixth row from a popup that already reaches far down the screen.
    K.Stack {
        visible: root.list.length > 0
        fill: true
        gap: K.Tokens.space.s4

        Repeater {
            model: root.list

            K.ListRow {
                required property var modelData

                title: modelData.name
                // The column it is ordered by first.
                detail: {
                    const cpu = root.padded(`${modelData.cpu.toFixed(1)}%`, 6);
                    const memory = root.padded(Sysmon.bytes(modelData.memory), 8);
                    return root.order === "cpu" ? `${cpu}  ${memory}` : `${memory}  ${cpu}`;
                }
                selected: root.details?.pid === modelData.pid
                onActivated: selected ? root.clear() : root.show(modelData.pid)
            }
        }
    }

    K.EmptyList {
        visible: root.list.length === 0
        text: Sysmon.sample?.detail ? "No processes to show" : "Counting…"
    }

    // The picked process: what it runs, whose it is, and End or Kill.
    K.Card {
        visible: root.details !== null

        K.Stack {
            fill: true
            gap: K.Tokens.space.s4

            K.Text {
                text: root.details ? `${root.details.name} · pid ${root.details.pid}` : ""
                style: K.Tokens.type.body_strong
            }

            K.Text {
                text: root.details?.command ?? ""
                style: K.Tokens.type.mono
                color: K.Tokens.color.on_surface_variant
                maximumLines: 2
            }

            K.Text {
                text: root.details ? [root.details.user, root.details.state, `${root.details.threads} threads`, Sysmon.bytes(root.details.memory), root.details.started ? `since ${Qt.formatDateTime(new Date(root.details.started * 1000), "ddd HH:mm")}` : "", `parent ${root.details.parent}`].filter(Boolean).join(" · ") : ""
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }

            K.Gap {
                space: K.Tokens.space.s8
            }

            K.Row {
                visible: root.confirming === ""
                gap: K.Tokens.space.s8
                align: "centre"

                K.Button {
                    text: "End"
                    kind: "Tonal"
                    size: "Sm"
                    activeFocusOnTab: false
                    onActivated: root.confirming = "term"

                    K.Hint {
                        text: "Ask it to quit; it may save its work first"
                        shown: parent.pointerOver
                    }
                }

                K.Button {
                    text: "Kill"
                    kind: "Text"
                    size: "Sm"
                    activeFocusOnTab: false
                    onActivated: root.confirming = "kill"

                    K.Hint {
                        text: "Stop it at once; unsaved work is lost"
                        shown: parent.pointerOver
                    }
                }
            }

            K.Row {
                visible: root.confirming !== ""
                gap: K.Tokens.space.s8
                align: "centre"

                K.Text {
                    text: root.details ? `${root.confirming === "kill" ? "Kill" : "End"} ${root.details.name}?` : ""
                    style: K.Tokens.type.body_strong
                    color: root.confirming === "kill" ? K.Tokens.color.error : K.Tokens.color.on_surface
                }

                K.Button {
                    text: "Cancel"
                    kind: "Text"
                    size: "Sm"
                    activeFocusOnTab: false
                    onActivated: root.confirming = ""
                }

                K.Button {
                    text: root.confirming === "kill" ? "Kill" : "End"
                    kind: root.confirming === "kill" ? "Danger" : "Filled"
                    size: "Sm"
                    activeFocusOnTab: false
                    onActivated: root.stop()
                }
            }
        }
    }

    K.Text {
        visible: root.error !== ""
        text: root.error
        style: K.Tokens.type.small
        color: K.Tokens.color.error
    }
}
