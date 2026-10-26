import QtQuick
import Quickshell
import qs.theme
import qs.components
import qs.services

// Date and time. The weekday's kanji (月 火 水 木 金 土 日) leads as a small
// accent; the countdown the clock settings pin follows the time. Click opens
// the clock popup under it.
Pill {
    id: root

    readonly property var kanji: ["日", "月", "火", "水", "木", "金", "土"]
    readonly property var countdown: Time.barCountdown

    interactive: true
    spacing: Theme.spacing.sm
    tooltip: Qt.formatDateTime(clock.date, Time.fullDateFormat)
    onClicked: Panels.toggle("clock")

    SystemClock {
        id: clock

        precision: SystemClock.Minutes
    }

    Text {
        anchors.verticalCenter: parent.verticalCenter
        text: root.kanji[clock.date.getDay()]
        color: Theme.m3tertiary
        font.family: Theme.font.jp
        font.pixelSize: Theme.font.md
        font.weight: Font.Bold
        renderType: Text.NativeRendering
    }

    StyledText {
        anchors.verticalCenter: parent.verticalCenter
        text: Qt.formatDateTime(clock.date, Time.shortDateFormat)
        color: Theme.m3onSurfaceVariant
    }

    StyledText {
        anchors.verticalCenter: parent.verticalCenter
        text: Qt.formatDateTime(clock.date, Time.timeFormat)
        font.weight: Font.DemiBold
        font.features: Theme.font.tabular
    }

    // "Vacation 12 d 4 h": to the minute, as the clock beside it.
    StyledText {
        visible: root.countdown !== null && root.countdown.end > clock.date.getTime()
        anchors.verticalCenter: parent.verticalCenter
        text: root.countdown ? `${root.countdown.name} ${Time.left(root.countdown.end - clock.date.getTime()).replace(/ \d+ s$/, "")}` : ""
        color: Theme.m3tertiary
        font.features: Theme.font.tabular
    }
}
