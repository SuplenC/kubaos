import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// Timer, stopwatch and alarms, on tabs. They run in the Time service, so
// they keep going and ring with the popup closed.
//
// The tabs share one height, the tallest any of them gets (the alarms' field
// over three alarms), so switching tabs never moves the popup.
Item {
    id: root

    property int tab: 0
    property string error: ""
    readonly property real now: Time.now.getTime()
    readonly property int pageHeight: K.Tokens.size.control_md + (K.Tokens.size.control_sm + K.Tokens.space.s8) * 3

    implicitWidth: K.Tokens.size.rail
    implicitHeight: page.y + page.height + (root.error !== "" ? K.Tokens.space.s8 + errorText.implicitHeight : 0)

    function show(index: int): void {
        tab = index;
        error = "";
    }

    Tabs {
        id: tabs

        labels: ["Timer", "Stopwatch", "Alarm"]
        currentIndex: root.tab
        onPicked: index => root.show(index)
    }

    Item {
        id: page

        y: tabs.implicitHeight + K.Tokens.space.s12
        width: parent.width
        height: root.pageHeight

        // Timer: what's left and its bar while set, else a duration to start.
        K.Stack {
            width: parent.width
            room: width
            fill: true
            gap: K.Tokens.space.s8
            visible: root.tab === 0

            K.Stack {
                visible: Time.timerSet
                centred: true

                Figure {
                    text: Time.face(Time.timerEnd > 0 ? Time.timerEnd - root.now : Time.timerLeft, false)
                    style: K.Tokens.type.headline
                    color: Time.timerEnd > 0 ? K.Tokens.color.on_surface : K.Tokens.color.on_surface_variant
                }
            }

            K.ProgressBar {
                readonly property real remaining: Time.timerEnd > 0 ? Time.timerEnd - root.now : Time.timerLeft

                visible: Time.timerSet
                value: Time.timerTotal > 0 ? remaining / Time.timerTotal : 0
            }

            K.Stack {
                visible: Time.timerSet
                centred: true

                K.Row {
                    gap: K.Tokens.space.s8

                    K.IconButton {
                        icon: Time.timerEnd > 0 ? "pause" : "play_arrow"
                        label: Time.timerEnd > 0 ? qsTr("Pause") : qsTr("Resume")
                        kind: "Filled"
                        onActivated: Time.pauseTimer()
                    }

                    K.IconButton {
                        icon: "close"
                        label: qsTr("Cancel")
                        kind: "Tonal"
                        onActivated: Time.cancelTimer()
                    }
                }
            }

            K.Field {
                id: timerField

                visible: !Time.timerSet
                icon: "timer"
                placeholder: "25m, 1h30m, 90 seconds"
                Keys.onEscapePressed: timerField.text = ""
                Keys.onReturnPressed: root.startTimer()
                Keys.onEnterPressed: root.startTimer()
            }

            K.Row {
                visible: !Time.timerSet
                gap: K.Tokens.space.s8

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
        K.Stack {
            width: parent.width
            room: width
            fill: true
            gap: K.Tokens.space.s8
            centred: true
            visible: root.tab === 1

            Figure {
                text: Time.face(Time.stopwatchElapsed(root.now), false)
                style: K.Tokens.type.headline
                color: K.Tokens.color.on_surface
            }

            K.Row {
                gap: K.Tokens.space.s8

                K.IconButton {
                    icon: Time.stopwatchRunning ? "pause" : "play_arrow"
                    label: Time.stopwatchRunning ? qsTr("Stop") : qsTr("Start")
                    kind: "Filled"
                    onActivated: Time.toggleStopwatch()
                }

                K.IconButton {
                    icon: "flag"
                    label: qsTr("Lap")
                    kind: "Tonal"
                    onActivated: Time.lap()
                }

                K.IconButton {
                    icon: "restart_alt"
                    label: qsTr("Reset")
                    kind: "Tonal"
                    onActivated: Time.resetStopwatch()
                }
            }

            Repeater {
                model: Time.laps.slice(0, 2)

                Figure {
                    required property real modelData
                    required property int index

                    text: `Lap ${Time.laps.length - index}  ${Time.face(modelData, false)}`
                    style: K.Tokens.type.small
                    color: K.Tokens.color.on_surface_variant
                }
            }
        }

        // Alarms: a time to add, and the ones set.
        K.Stack {
            width: parent.width
            room: width
            fill: true
            gap: K.Tokens.space.s8
            visible: root.tab === 2

            K.Field {
                id: alarmField

                icon: "alarm_add"
                placeholder: "7:30, 6pm, friday 17:00"
                Keys.onEscapePressed: alarmField.text = ""
                Keys.onReturnPressed: root.addAlarm()
                Keys.onEnterPressed: root.addAlarm()
            }

            K.Text {
                visible: Time.alarms.length === 0
                text: "No alarms set"
                style: K.Tokens.type.small
                color: K.Tokens.color.on_surface_variant
            }

            Repeater {
                model: Time.alarms.slice(0, 3)

                K.Row {
                    required property var modelData
                    required property int index

                    fill: true
                    gap: K.Tokens.space.s8
                    align: "centre"

                    K.Icon {
                        name: "alarm"
                        size: K.Tokens.icon.s20
                        color: K.Tokens.color.tertiary
                    }

                    K.Text {
                        text: modelData.label
                        style: K.Tokens.type.body
                    }

                    K.Fill {}

                    K.IconButton {
                        icon: "close"
                        label: qsTr("Remove")
                        kind: "Text"
                        size: "Sm"
                        onActivated: Time.removeAlarm(index)
                    }
                }
            }
        }
    }

    K.Text {
        id: errorText

        visible: root.error !== ""
        y: page.y + page.height + K.Tokens.space.s8
        room: root.width
        text: root.error
        style: K.Tokens.type.small
        color: K.Tokens.color.error
        maximumLines: 2
    }

    function startTimer(): void {
        Time.startTimer(timerField.text, error => {
            root.error = error;
            if (error === "")
                timerField.text = "";
        });
    }

    function addAlarm(): void {
        Time.addAlarm(alarmField.text, error => {
            root.error = error;
            if (error === "")
                alarmField.text = "";
        });
    }
}
