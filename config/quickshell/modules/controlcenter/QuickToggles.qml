import QtQuick
import QtQuick.Layouts
import Quickshell.Services.UPower
import qs.theme
import qs.services

// Wi-Fi, Bluetooth, do-not-disturb, power profile and wallpaper motion tiles.
GridLayout {
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

    QuickTile {
        Layout.fillWidth: true
        icon: Profiles.icon
        title: "Power profile"
        subtitle: Profiles.nameFor(Profiles.current)
        checked: Profiles.current !== PowerProfile.Balanced
        onClicked: Profiles.cycle()
    }

    // Always -> On AC -> Never. Remembered across restarts (services/Motion.qml).
    QuickTile {
        Layout.fillWidth: true
        Layout.columnSpan: 2
        icon: Motion.mode === "never" ? "motion_photos_off" : Motion.mode === "always" ? "motion_photos_on" : "motion_photos_auto"
        title: "Wallpaper motion"
        subtitle: Motion.mode === "never" ? "Never. Posters only" : Motion.mode === "always" ? "Always. Battery be damned" : Motion.onBattery ? "Only on AC. Paused, you're on battery" : "Only on AC. Plugged in, so it moves"
        checked: Motion.mode !== "never"
        onClicked: Motion.cycle()
    }
}
