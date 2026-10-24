import QtQuick
import qs.theme
import qs.components
import qs.services

// Timer, stopwatch and alarms, on tabs. They run in the Time service, so
// they keep going and ring with the popup closed.
Item {
    id: root

    property int tab: 0
    property string error: ""
    readonly property real now: Time.now.getTime()

    implicitWidth: 280
    implicitHeight: tabs.height + Theme.spacing.md + 150

    function show(index: int): void {
        tab = index;
        error = "";
    }

    TabBar {
        id: tabs

        width: parent.width
        labels: ["Timer", "Stopwatch", "Alarm"]
        currentIndex: root.tab
        onPicked: index => root.show(index)
    }

    Item {
        y: tabs.height + Theme.spacing.md
        width: parent.width
        height: 150

        // Timer: what's left and its bar while set, else a duration to start.
        Column {
            anchors.fill: parent
            spacing: Theme.spacing.sm
            visible: root.tab === 0

            StyledText {
                visible: Time.timerSet
                width: parent.width
                horizontalAlignment: Text.AlignHCenter
                text: Time.face(Time.timerEnd > 0 ? Time.timerEnd - root.now : Time.timerLeft, false)
                font.pixelSize: 40
                font.weight: Font.Light
                font.features: Theme.font.tabular
                color: Time.timerEnd > 0 ? Theme.m3onSurface : Theme.m3onSurfaceVariant
            }

            Rectangle {
                visible: Time.timerSet
                width: parent.width
                height: 6
                radius: 3
                color: Theme.m3surfaceContainerHighest

                Rectangle {
                    readonly property real remaining: Time.timerEnd > 0 ? Time.timerEnd - root.now : Time.timerLeft

                    width: parent.width * Math.max(0, Math.min(1, Time.timerTotal > 0 ? remaining / Time.timerTotal : 0))
                    height: parent.height
                    radius: 3
                    color: Theme.m3primary

                    Behavior on width {
                        Anim {
                            duration: 1000
                            easing.type: Easing.Linear
                        }
                    }
                }
            }

            Row {
                visible: Time.timerSet
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.spacing.sm

                IconButton {
                    icon: Time.timerEnd > 0 ? "pause" : "play_arrow"
                    toggled: true
                    tooltip: Time.timerEnd > 0 ? "Pause" : "Resume"
                    onClicked: Time.pauseTimer()
                }

                IconButton {
                    icon: "close"
                    baseColor: Theme.m3surfaceContainerHigh
                    tooltip: "Cancel"
                    onClicked: Time.cancelTimer()
                }
            }

            SearchField {
                id: timerField

                visible: !Time.timerSet
                width: parent.width
                icon: "timer"
                placeholder: "25m, 1h30m, 90 seconds"
                catchEscape: true
                onEscaped: text = ""
                onAccepted: Time.startTimer(text, error => {
                    root.error = error;
                    if (error === "")
                        text = "";
                })
            }

            Row {
                visible: !Time.timerSet
                spacing: Theme.spacing.sm

                Repeater {
                    model: ["5m", "10m", "25m", "1h"]

                    Chip {
                        required property string modelData

                        label: modelData
                        onClicked: Time.startTimer(modelData, error => root.error = error)
                    }
                }
            }
        }

        // Stopwatch: the time, start/stop, lap and reset, the last laps.
        Column {
            anchors.fill: parent
            spacing: Theme.spacing.sm
            visible: root.tab === 1

            StyledText {
                width: parent.width
                horizontalAlignment: Text.AlignHCenter
                text: Time.face(Time.stopwatchElapsed(root.now), false)
                font.pixelSize: 40
                font.weight: Font.Light
                font.features: Theme.font.tabular
            }

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: Theme.spacing.sm

                IconButton {
                    icon: Time.stopwatchRunning ? "pause" : "play_arrow"
                    toggled: true
                    tooltip: Time.stopwatchRunning ? "Stop" : "Start"
                    onClicked: Time.toggleStopwatch()
                }

                IconButton {
                    icon: "flag"
                    baseColor: Theme.m3surfaceContainerHigh
                    tooltip: "Lap"
                    onClicked: Time.lap()
                }

                IconButton {
                    icon: "restart_alt"
                    baseColor: Theme.m3surfaceContainerHigh
                    tooltip: "Reset"
                    onClicked: Time.resetStopwatch()
                }
            }

            Repeater {
                model: Time.laps.slice(0, 2)

                StyledText {
                    required property real modelData
                    required property int index

                    width: parent.width
                    horizontalAlignment: Text.AlignHCenter
                    text: `Lap ${Time.laps.length - index}  ${Time.face(modelData, false)}`
                    font.pixelSize: Theme.font.sm
                    font.features: Theme.font.tabular
                    color: Theme.m3onSurfaceVariant
                }
            }
        }

        // Alarms: a time to add, and the ones set.
        Column {
            anchors.fill: parent
            spacing: Theme.spacing.sm
            visible: root.tab === 2

            SearchField {
                width: parent.width
                icon: "alarm_add"
                placeholder: "7:30, 6pm, friday 17:00"
                catchEscape: true
                onEscaped: text = ""
                onAccepted: Time.addAlarm(text, error => {
                    root.error = error;
                    if (error === "")
                        text = "";
                })
            }

            StyledText {
                visible: Time.alarms.length === 0
                width: parent.width
                text: "No alarms set"
                color: Theme.m3outline
                font.pixelSize: Theme.font.sm
            }

            Repeater {
                model: Time.alarms.slice(0, 3)

                Item {
                    required property var modelData
                    required property int index

                    width: parent.width
                    height: 28

                    Icon {
                        id: bell

                        anchors.verticalCenter: parent.verticalCenter
                        name: "alarm"
                        size: 18
                        color: Theme.m3tertiary
                    }

                    StyledText {
                        anchors.left: bell.right
                        anchors.leftMargin: Theme.spacing.sm
                        anchors.right: remove.left
                        anchors.verticalCenter: parent.verticalCenter
                        text: modelData.label
                    }

                    IconButton {
                        id: remove

                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        icon: "close"
                        size: 26
                        tooltip: "Remove"
                        onClicked: Time.removeAlarm(index)
                    }
                }
            }
        }
    }

    StyledText {
        visible: root.error !== ""
        anchors.bottom: parent.bottom
        width: parent.width
        text: root.error
        color: Theme.m3error
        font.pixelSize: Theme.font.sm
        wrapMode: Text.Wrap
        maximumLineCount: 2
    }
}
