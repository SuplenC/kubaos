import QtQuick
import KubaOS.Components as K
import qs.services

// Wi-Fi, Bluetooth, do-not-disturb, wallpaper motion, keyboard layout and,
// once a profile is set up, Proxy tiles, two to a row. The power stops are a
// slider (Sliders.qml).
K.Stack {
    id: root

    // The keyboard tile's arrow: the control center shows its layouts tab.
    signal keyboardExpanded

    // The Proxy tile's arrow opened its detail.
    property bool proxyDetail: false

    gap: K.Tokens.space.s8
    fill: true

    function focusFirst(): void {
        wifi.forceActiveFocus();
    }

    // The layout in use; a click switches to the next, as the bar's pill does.
    // The arrow lists them, to add, remove, reorder or pick the default.
    component KeyboardTile: QuickTile {
        icon: "keyboard"
        title: "Keyboard"
        subtitle: Keyboard.several ? `${Keyboard.code} · ${Keyboard.active + 1} of ${Keyboard.layouts.length}` : Keyboard.code || "Layouts"
        checked: Keyboard.several
        expandable: true
        onActivated: Keyboard.next()
        onExpanded: root.keyboardExpanded()
    }

    K.Grid {
        columns: 2
        gap: K.Tokens.space.s8

        QuickTile {
            id: wifi

            icon: Net.icon
            title: "Wi-Fi"
            subtitle: Net.wired ? "Wired" : !Net.wifiEnabled ? "Off" : Net.active?.name ?? "Not connected"
            checked: Net.wifiEnabled
            onActivated: Net.setWifiEnabled(!Net.wifiEnabled)
        }

        QuickTile {
            icon: Bt.icon
            title: "Bluetooth"
            subtitle: !Bt.available ? "Unavailable" : !Bt.enabled ? "Off" : Bt.connectedCount > 0 ? `${Bt.connectedCount} connected` : "On"
            checked: Bt.enabled
            onActivated: Bt.toggle()
        }

        QuickTile {
            icon: Notifs.dnd ? "notifications_off" : "notifications"
            title: "Do not disturb"
            subtitle: Notifs.dnd ? "On" : "Off"
            checked: Notifs.dnd
            onActivated: Notifs.toggleDnd()
        }

        // Always -> On AC -> Never. Remembered across restarts (services/Motion.qml).
        QuickTile {
            icon: Motion.mode === "never" ? "motion_photos_off" : Motion.mode === "always" ? "motion_photos_on" : "motion_photos_auto"
            title: "Wallpaper motion"
            subtitle: Motion.mode === "never" ? "Never" : Motion.mode === "always" ? "Always" : Motion.onBattery ? "On AC, paused" : "On AC"
            checked: Motion.mode !== "never"
            onActivated: Motion.cycle()
        }

        // Beside the Proxy tile while there is one.
        KeyboardTile {
            visible: Proxy.configured
        }

        // The work network's switch (services/Proxy.qml): every part of the
        // profile up together, or none. The arrow shows the tunnel's endpoint,
        // last handshake and bytes moved below.
        QuickTile {
            visible: Proxy.configured
            icon: Proxy.on ? "vpn_lock" : "key"
            title: "Proxy"
            subtitle: Proxy.subtitle
            checked: Proxy.on !== Proxy.busy
            expandable: true
            onActivated: Proxy.toggle()
            onExpanded: root.proxyDetail = !root.proxyDetail
        }
    }

    // Across the whole row while there is no Proxy tile, so the grid has no hole.
    KeyboardTile {
        visible: !Proxy.configured
    }

    ProxyDetail {
        visible: Proxy.configured && root.proxyDetail
    }
}
