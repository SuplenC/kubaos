import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services

// Side sheet on the right edge, from the bar down to the bottom border:
// header, quick toggles, capture (screenshot, annotate, record, replay),
// sliders, the notifications, and tabs for Wi-Fi, Bluetooth, audio output
// and keyboard layouts, whose list takes the room left at the bottom.
// Keys: Tab/Shift+Tab move between controls, h/l switch tabs on the tab bar,
// 1-4 jump to a tab, j/k move in lists, Esc closes.
K.Sheet {
    id: root

    property int tab: 0
    // The Keyboard tab, which the keyboard tile's arrow and
    // `qs ipc call controlcenter keyboard` open.
    readonly property int keyboardTab: 3
    readonly property var kanji: ["日", "月", "火", "水", "木", "金", "土"]

    edge: "right"
    flushStart: true
    flushEnd: true
    contentWidth: K.Tokens.size.panel
    onOpened: toggles.focusFirst()

    Keys.onPressed: event => {
        const n = event.key - Qt.Key_1;
        if (n >= 0 && n < tabBar.labels.length && !(event.modifiers & Qt.ControlModifier)) {
            root.tab = n;
            event.accepted = true;
        }
    }

    // A pairing prompt, or "Pair a Bluetooth device" from the Command menu.
    Connections {
        target: Bt

        function onReveal() {
            root.tab = 1;
        }
    }

    // "Wi-Fi networks" from the Command menu.
    Connections {
        target: Net

        function onReveal() {
            root.tab = 0;
        }
    }

    // Each radio scans only while its tab shows, not while its connection form does: Wi-Fi about
    // every 30 s, Bluetooth a short discovery about once a minute, which also refreshes the
    // batteries. NetworkManager's scan takes a few seconds; a discovery 10 s finds what is in
    // pairing mode nearby.
    ScanPulse {
        active: root.shown && root.tab === 0 && Net.wifiEnabled && !wifi.formOpen
        every: 30000
        window: 5000
        onScanningChanged: Net.setScanning(scanning)
    }

    ScanPulse {
        active: root.shown && root.tab === 1 && Bt.enabled && !bluetooth.formOpen
        every: 60000
        window: 10000
        onScanningChanged: Bt.setScanning(scanning)
    }

    Connections {
        target: Keyboard

        function onReveal() {
            root.tab = root.keyboardTab;
        }
    }

    SystemClock {
        id: clock

        precision: SystemClock.Minutes
    }

    // The panel's body, the container padding in at the sides and 16 above and below: the
    // sections, then the tabs in what is left.
    K.Container {
        id: body

        width: root.contentWidth

        K.Stack {
            id: top

            fill: true
            gap: K.Tokens.space.s16
            topPadding: K.Tokens.space.s16
            bottomPadding: K.Tokens.space.s16

            K.Row {
                fill: true
                gap: K.Tokens.space.s8
                align: "centre"

                // The weekday's kanji above the time, as a page's accent sits above its title.
                K.Header {
                    accent: `${root.kanji[clock.date.getDay()]}曜日`
                    title: Qt.formatDateTime(clock.date, Time.timeFormat)
                    lede: Qt.formatDateTime(clock.date, Time.dateFormat) + (Battery.available ? ` · ${Math.round(Battery.percentage * 100)}% ${Battery.timeLeft}` : "")
                }

                K.Fill {
                }

                K.IconButton {
                    icon: "wallpaper"
                    label: "Wallpaper"
                    kind: "Tonal"
                    onActivated: Panels.open("wallpaper")
                }

                K.IconButton {
                    icon: "power_settings_new"
                    label: "Power"
                    kind: "Tonal"
                    onActivated: Panels.open("powermenu")
                }
            }

            QuickToggles {
                id: toggles

                onKeyboardExpanded: root.tab = root.keyboardTab
            }

            CaptureRow {
            }

            Sliders {
            }

            NotificationHistory {
            }
        }
    }

    // The tabs and their list, in the room the sections above leave, the container padding in
    // from the panel's edges.
    Item {
        x: body.padding
        y: body.height
        width: root.contentWidth - body.padding * 2
        height: Math.max(0, root.height - y - K.Tokens.space.s16)

        K.Tabs {
            id: tabBar

            room: parent.width
            labels: ["Wi-Fi", "Bluetooth", "Output", "Keyboard"]
            currentIndex: root.tab
            onPicked: index => root.tab = index
        }

        // Clipped `focus.offset` outside the tab's room: a list taller than the room the sections
        // leave on a short screen is cut there, rather than drawn over them, a focused control's
        // glow with it.
        Item {
            x: -K.Tokens.focus.offset
            y: tabBar.height + K.Tokens.space.s12 - K.Tokens.focus.offset
            width: parent.width + K.Tokens.focus.offset * 2
            height: Math.max(0, parent.height - y + K.Tokens.focus.offset)
            clip: true

            WifiList {
                id: wifi

                anchors.fill: parent
                anchors.margins: K.Tokens.focus.offset
                visible: root.tab === 0
            }

            BluetoothList {
                id: bluetooth

                anchors.fill: parent
                anchors.margins: K.Tokens.focus.offset
                visible: root.tab === 1
            }

            OutputList {
                anchors.fill: parent
                anchors.margins: K.Tokens.focus.offset
                visible: root.tab === 2
            }

            KeyboardList {
                anchors.fill: parent
                anchors.margins: K.Tokens.focus.offset
                visible: root.tab === 3
            }
        }
    }
}
