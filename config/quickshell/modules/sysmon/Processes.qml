import QtQuick
import qs.theme
import qs.components
import qs.services
import qs.modules.clock

// The five busiest processes, by CPU or by memory. A click shows one's
// details, with End (it may save and quit) and Kill (it stops at once), each
// asking first.
Column {
    id: root

    property string order: "cpu"
    // The process shown, from the engine's `process` reply, or null.
    property var details: null
    // "term" or "kill" while its confirmation shows, else "".
    property string confirming: ""
    property string error: ""
    readonly property var list: Sysmon.sample?.top?.[order] ?? []
    readonly property int rowHeight: 30

    spacing: Theme.spacing.sm

    // Shows process `pid`; one that has exited says so.
    function show(pid: int): void {
        confirming = "";
        error = "";
        Sysmon.request("process", { pid: pid }, reply => {
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

    Item {
        width: parent.width
        height: orderChips.height

        Row {
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.spacing.sm

            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: "list_alt"
                size: Theme.font.xl
                color: Theme.m3primary
            }

            StyledText {
                anchors.verticalCenter: parent.verticalCenter
                text: "Processes"
                font.weight: Font.DemiBold
            }
        }

        Row {
            id: orderChips

            anchors.right: parent.right
            spacing: Theme.spacing.xs

            Chip {
                label: "CPU"
                selected: root.order === "cpu"
                onClicked: root.order = "cpu"
            }

            Chip {
                label: "Memory"
                selected: root.order === "memory"
                onClicked: root.order = "memory"
            }
        }
    }

    Repeater {
        model: root.list

        Rectangle {
            id: row

            required property var modelData
            readonly property bool picked: root.details?.pid === modelData.pid

            width: root.width
            height: root.rowHeight
            radius: Theme.rounding.sm
            color: picked ? Theme.m3secondaryContainer : "transparent"

            StyledText {
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacing.sm
                anchors.right: cpuText.left
                anchors.rightMargin: Theme.spacing.md
                anchors.verticalCenter: parent.verticalCenter
                text: row.modelData.name
                font.weight: Font.Medium
                color: row.picked ? Theme.m3onSecondaryContainer : Theme.m3onSurface
            }

            StyledText {
                id: cpuText

                anchors.right: memoryText.left
                anchors.verticalCenter: parent.verticalCenter
                width: Theme.spacing.xxxl + Theme.spacing.lg
                horizontalAlignment: Text.AlignRight
                text: `${row.modelData.cpu.toFixed(1)}%`
                font.features: Theme.font.tabular
                color: root.order === "cpu" ? Theme.m3onSurface : Theme.m3onSurfaceVariant
            }

            StyledText {
                id: memoryText

                anchors.right: parent.right
                anchors.rightMargin: Theme.spacing.sm
                anchors.verticalCenter: parent.verticalCenter
                width: Theme.spacing.xxxl * 2
                horizontalAlignment: Text.AlignRight
                text: Sysmon.bytes(row.modelData.memory)
                font.features: Theme.font.tabular
                color: root.order === "memory" ? Theme.m3onSurface : Theme.m3onSurfaceVariant
            }

            StateLayer {
                onClicked: row.picked ? root.clear() : root.show(row.modelData.pid)
            }
        }
    }

    StyledText {
        visible: root.list.length === 0
        width: parent.width
        text: Sysmon.sample?.detail ? "No processes to show" : "Counting…"
        color: Theme.m3outline
        font.pixelSize: Theme.font.sm
    }

    // The picked process: what it runs, whose it is, and End or Kill.
    Rectangle {
        visible: root.details !== null
        width: parent.width
        height: detailsColumn.implicitHeight + Theme.spacing.md * 2
        radius: Theme.rounding.md
        color: Theme.m3surfaceContainer

        Column {
            id: detailsColumn

            x: Theme.spacing.md
            y: Theme.spacing.md
            width: parent.width - Theme.spacing.md * 2
            spacing: Theme.spacing.xs

            StyledText {
                width: parent.width
                text: root.details ? `${root.details.name} · pid ${root.details.pid}` : ""
                font.weight: Font.DemiBold
            }

            StyledText {
                width: parent.width
                text: root.details?.command ?? ""
                font.family: Theme.font.mono
                font.pixelSize: Theme.font.sm
                color: Theme.m3onSurfaceVariant
                wrapMode: Text.WrapAnywhere
                maximumLineCount: 2
            }

            StyledText {
                width: parent.width
                text: root.details ? [root.details.user, root.details.state, `${root.details.threads} threads`, Sysmon.bytes(root.details.memory), root.details.started ? `since ${Qt.formatDateTime(new Date(root.details.started * 1000), "ddd HH:mm")}` : "", `parent ${root.details.parent}`].filter(Boolean).join(" · ") : ""
                font.pixelSize: Theme.font.sm
                font.features: Theme.font.tabular
                color: Theme.m3onSurfaceVariant
            }

            Item {
                width: parent.width
                height: Theme.spacing.xs
            }

            Row {
                visible: root.confirming === ""
                spacing: Theme.spacing.sm

                Chip {
                    label: "End"
                    tooltip: "Ask it to quit; it may save its work first"
                    onClicked: root.confirming = "term"
                }

                Chip {
                    label: "Kill"
                    tooltip: "Stop it at once; unsaved work is lost"
                    onClicked: root.confirming = "kill"
                }
            }

            Row {
                visible: root.confirming !== ""
                spacing: Theme.spacing.sm

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.details ? `${root.confirming === "kill" ? "Kill" : "End"} ${root.details.name}?` : ""
                    font.weight: Font.Medium
                    color: root.confirming === "kill" ? Theme.m3error : Theme.m3onSurface
                }

                Chip {
                    label: "Cancel"
                    onClicked: root.confirming = ""
                }

                Chip {
                    label: root.confirming === "kill" ? "Kill" : "End"
                    selected: true
                    onClicked: root.stop()
                }
            }
        }
    }

    StyledText {
        visible: root.error !== ""
        width: parent.width
        text: root.error
        color: Theme.m3error
        font.pixelSize: Theme.font.sm
        wrapMode: Text.Wrap
    }
}
