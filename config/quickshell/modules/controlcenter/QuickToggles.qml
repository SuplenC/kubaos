import QtQuick
import QtQuick.Layouts
import qs.theme
import qs.services

// Wi-Fi, Bluetooth, do-not-disturb, wallpaper motion, keyboard layout and,
// once a profile is set up, Proxy tiles. The power stops are a slider
// (Sliders.qml).
GridLayout {
    id: root

    // The keyboard tile's arrow: the control center shows its layouts tab.
    signal keyboardExpanded

    // The Proxy tile's arrow opened its detail.
    property bool proxyDetail: false

    columns: 2
    rowSpacing: Theme.spacing.sm
    columnSpacing: Theme.spacing.sm

    function focusFirst(): void {
        wifi.forceActiveFocus();
    }

    QuickTile {
        id: wifi

        Layout.fillWidth: true
        icon: Net.icon
        title: "Wi-Fi"
        subtitle: Net.wired ? "Wired" : !Net.wifiEnabled ? "Off" : Net.active?.name ?? "Not connected"
        checked: Net.wifiEnabled
        onClicked: Net.setWifiEnabled(!Net.wifiEnabled)
    }

    QuickTile {
        Layout.fillWidth: true
        icon: Bt.icon
        title: "Bluetooth"
        subtitle: !Bt.available ? "Unavailable" : !Bt.enabled ? "Off" : Bt.connectedCount > 0 ? `${Bt.connectedCount} connected` : "On"
        checked: Bt.enabled
        onClicked: Bt.toggle()
    }

    QuickTile {
        Layout.fillWidth: true
        icon: Notifs.dnd ? "notifications_off" : "notifications"
        title: "Do not disturb"
        subtitle: Notifs.dnd ? "On" : "Off"
        checked: Notifs.dnd
        onClicked: Notifs.toggleDnd()
    }

    // Always -> On AC -> Never. Remembered across restarts (services/Motion.qml).
    QuickTile {
        Layout.fillWidth: true
        icon: Motion.mode === "never" ? "motion_photos_off" : Motion.mode === "always" ? "motion_photos_on" : "motion_photos_auto"
        title: "Wallpaper motion"
        subtitle: Motion.mode === "never" ? "Never" : Motion.mode === "always" ? "Always" : Motion.onBattery ? "On AC, paused" : "On AC"
        checked: Motion.mode !== "never"
        onClicked: Motion.cycle()
    }

    // The layout in use; a click switches to the next, as the bar's pill does.
    // The arrow lists them, to add, remove, reorder or pick the default. Across
    // both columns while there is no Proxy tile, so the grid has no hole.
    QuickTile {
        Layout.fillWidth: true
        Layout.columnSpan: Proxy.configured ? 1 : 2
        icon: "keyboard"
        title: "Keyboard"
        subtitle: Keyboard.several ? `${Keyboard.code} · ${Keyboard.active + 1} of ${Keyboard.layouts.length}` : Keyboard.code || "Layouts"
        checked: Keyboard.several
        expandable: true
        onClicked: Keyboard.next()
        onExpanded: root.keyboardExpanded()
    }

    // The work network's switch (services/Proxy.qml): every part of the
    // profile up together, or none. The arrow shows the tunnel's endpoint,
    // last handshake and bytes moved below.
    QuickTile {
        Layout.fillWidth: true
        visible: Proxy.configured
        icon: Proxy.on ? "vpn_lock" : "vpn_key"
        title: "Proxy"
        subtitle: Proxy.subtitle
        checked: Proxy.on !== Proxy.busy
        expandable: true
        onClicked: Proxy.toggle()
        onExpanded: root.proxyDetail = !root.proxyDetail
    }

    ProxyDetail {
        Layout.fillWidth: true
        Layout.columnSpan: 2
        visible: Proxy.configured && root.proxyDetail
    }
}
