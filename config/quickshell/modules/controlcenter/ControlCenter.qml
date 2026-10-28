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
    // Onto the tab's connection form when one is open (a pairing the device started), else the
    // first quick toggle.
    onOpened: {
        const radio = [wifi, bluetooth][root.tab];
        if (radio?.formOpen)
            radio.form.focusFirst();
        else
            toggles.focusFirst();
    }

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

    // Each radio scans only while its tab shows. Wi-Fi scans about every 30 s, for a few seconds,
    // as long as NetworkManager's scan takes, and not while its form connects. Bluetooth discovers
    // for 20 s every 45 s, and goes on while its form shows, so the device being paired stays.
    // HAZARD: BlueZ forgets a device it hasn't heard for 30 s (`TemporaryTimeout`); a gap between
    // discoveries longer than that empties Nearby before the next one and fills it again, every
    // time.
    ScanPulse {
        active: root.shown && root.tab === 0 && Net.wifiEnabled && !wifi.formOpen
        every: 30000
        window: 5000
        onScanningChanged: Net.setScanning(scanning)
    }

    ScanPulse {
        active: root.shown && root.tab === 1 && Bt.enabled
        every: 45000
        window: 20000
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
        id: tabs

        // The gap between the tabs and the tab, and the room between the tab and the panel's edge.
        readonly property int gap: K.Tokens.space.s12
        readonly property int foot: K.Tokens.space.s16

        x: body.padding
        y: body.height
        width: root.contentWidth - body.padding * 2
        height: Math.max(0, root.height - y - tabs.foot)

        K.Tabs {
            id: tabBar

            room: parent.width
            labels: ["Wi-Fi", "Bluetooth", "Output", "Keyboard"]
            currentIndex: root.tab
            onPicked: index => root.tab = index
        }

        // The tab's room has no edge of its own, so it doesn't clip: a focused control's glow
        // reaches past it to the panel's own edge. A tab that scrolls on a short screen reaches
        // its scroll to the tabs and the panel's edge.
        Item {
            y: tabBar.height + tabs.gap
            width: parent.width
            height: Math.max(0, parent.height - y)

            WifiList {
                id: wifi

                anchors.fill: parent
                visible: root.tab === 0
            }

            BluetoothList {
                id: bluetooth

                anchors.fill: parent
                visible: root.tab === 1
            }

            OutputList {
                anchors.fill: parent
                visible: root.tab === 2
            }

            KeyboardList {
                anchors.fill: parent
                roomAbove: tabs.gap
                roomBelow: tabs.foot
                visible: root.tab === 3
            }
        }
    }
}
