import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services

// Date and time. The weekday's kanji (月 火 水 木 金 土 日) leads as a small accent; the countdown
// the clock settings pin follows the time. The time is in mono, whose figures are all one width,
// so the pill keeps its width as the minutes turn. Click opens the clock popup under it.
Pill {
    id: root

    readonly property var kanji: ["日", "月", "火", "水", "木", "金", "土"]
    readonly property var countdown: Time.barCountdown

    interactive: true
    tooltip: Qt.formatDateTime(clock.date, Time.fullDateFormat)
    onClicked: Panels.toggle("clock")

    SystemClock {
        id: clock

        precision: SystemClock.Minutes
    }

    K.Text {
        text: root.kanji[clock.date.getDay()]
        style: K.Tokens.type.accent
        color: K.Tokens.color.tertiary
    }

    K.Text {
        text: Qt.formatDateTime(clock.date, Time.shortDateFormat)
        style: K.Tokens.type.body
        color: K.Tokens.color.on_surface_variant
    }

    K.Text {
        text: Qt.formatDateTime(clock.date, Time.timeFormat)
        style: K.Tokens.type.mono_large
        color: K.Tokens.color.on_surface
    }

    // "Vacation 12 d 4 h": to the minute, as the clock beside it.
    K.Text {
        visible: root.countdown !== null && root.countdown.end > clock.date.getTime()
        text: root.countdown ? `${root.countdown.name} ${Time.left(root.countdown.end - clock.date.getTime()).replace(/ \d+ s$/, "")}` : ""
        style: K.Tokens.type.body
        color: K.Tokens.color.tertiary
    }
}
