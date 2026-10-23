import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services

// Side sheet on the right edge, from the bar down to the bottom border:
// header, quick toggles, capture (screenshot, annotate, record, replay),
// sliders, and tabs for Wi-Fi, Bluetooth, audio
// output and notification history.
// Keys: Tab/Shift+Tab move between controls, h/l switch tabs on the tab bar,
// 1-4 jump to a tab, j/k move in lists, Esc closes.
Drawer {
    id: root

    property int tab: 0
    readonly property bool scanWifi: shown && tab === 0
    readonly property var kanji: ["日", "月", "火", "水", "木", "金", "土"]

    edge: "right"
    flushStart: true
    flushEnd: true
    contentWidth: 384
    onOpened: toggles.focusFirst()
    onScanWifiChanged: Net.setScanning(scanWifi)
    onShownChanged: if (!shown)
        Bt.setScanning(false)

    Keys.onPressed: event => {
        const n = event.key - Qt.Key_1;
        if (n >= 0 && n < tabBar.labels.length && !(event.modifiers & Qt.ControlModifier)) {
            root.tab = n;
            event.accepted = true;
        }
    }

    SystemClock {
        id: clock

        precision: SystemClock.Minutes
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacing.xl
        anchors.topMargin: Theme.spacing.lg
        spacing: Theme.spacing.lg

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacing.sm

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                RowLayout {
                    spacing: Theme.spacing.sm

                    StyledText {
                        text: Qt.formatDateTime(clock.date, "HH:mm")
                        font.pixelSize: Theme.font.xxl + 6
                        font.weight: Font.DemiBold
                        font.features: Theme.font.tabular
                    }

                    Text {
                        Layout.alignment: Qt.AlignBaseline
                        text: `${root.kanji[clock.date.getDay()]}曜日`
                        color: Theme.m3tertiary
                        font.family: Theme.font.jp
                        font.pixelSize: Theme.font.lg
                        font.weight: Font.Medium
                        renderType: Text.NativeRendering
                    }
                }

                StyledText {
                    Layout.fillWidth: true
                    text: Qt.formatDateTime(clock.date, "dddd, d MMMM") + (Battery.available ? `  ·  ${Math.round(Battery.percentage * 100)}% ${Battery.timeLeft}` : "")
                    color: Theme.m3onSurfaceVariant
                }
            }

            IconButton {
                icon: "wallpaper"
                tooltip: "Wallpaper"
                baseColor: Theme.m3surfaceContainerHigh
                onClicked: Panels.open("wallpaper")
            }

            IconButton {
                icon: "power_settings_new"
                tooltip: "Power"
                baseColor: Theme.m3surfaceContainerHigh
                tint: Theme.m3error
                onClicked: Panels.open("powermenu")
            }
        }

        QuickToggles {
            id: toggles

            Layout.fillWidth: true
        }

        CaptureRow {
            Layout.fillWidth: true
        }

        Sliders {
            Layout.fillWidth: true
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: Theme.rounding.xl
            color: Theme.m3surfaceContainer

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacing.sm
                spacing: Theme.spacing.sm

                TabBar {
                    id: tabBar

                    Layout.fillWidth: true
                    labels: ["Wi-Fi", "Bluetooth", "Output", "Notifications"]
                    currentIndex: root.tab
                    onPicked: index => root.tab = index
                }

                StackLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.margins: Theme.spacing.xs
                    currentIndex: root.tab

                    WifiList {}

                    BluetoothList {}

                    OutputList {}

                    NotificationHistory {}
                }
            }
        }
    }
}
