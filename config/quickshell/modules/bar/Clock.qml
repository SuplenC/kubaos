import QtQuick
import Quickshell
import qs.theme
import qs.components
import qs.services

// Date and time. The weekday's kanji (月 火 水 木 金 土 日) leads as a small
// accent. Click opens the control center.
Pill {
    id: root

    readonly property var kanji: ["日", "月", "火", "水", "木", "金", "土"]

    interactive: true
    spacing: Theme.spacing.sm
    tooltip: Qt.formatDateTime(clock.date, "dddd, d MMMM yyyy")
    onClicked: Panels.toggle("controlcenter")

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
        text: Qt.formatDateTime(clock.date, "ddd d MMM")
        color: Theme.m3onSurfaceVariant
    }

    StyledText {
        anchors.verticalCenter: parent.verticalCenter
        text: Qt.formatDateTime(clock.date, "HH:mm")
        font.weight: Font.DemiBold
        font.features: Theme.font.tabular
    }
}
