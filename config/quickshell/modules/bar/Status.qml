import QtQuick
import qs.theme
import qs.components
import qs.services

// System indicators. Click opens the control center, scroll changes volume.
Pill {
    id: root

    interactive: true
    spacing: Theme.spacing.sm + 2
    tooltip: [Net.wired ? "Wired" : Net.active?.name ?? (Net.wifiEnabled ? "Wi-Fi not connected" : "Wi-Fi off"), `Volume ${Audio.muted ? "muted" : Math.round(Audio.volume * 100) + "%"}`, Battery.available ? `Battery ${Math.round(Battery.percentage * 100)}%` : ""].filter(Boolean).join("  ·  ")
    onClicked: Panels.toggle("controlcenter")
    onScrolled: steps => Audio.setVolume(Audio.volume + steps * 0.05)

    component StatusIcon: Icon {
        anchors.verticalCenter: parent?.verticalCenter
        size: Theme.bar.icon
        color: Theme.m3onSurfaceVariant
    }

    StatusIcon {
        name: Notifs.dnd ? "notifications_off" : "notifications"
        filled: Notifs.count > 0
        color: Notifs.count > 0 ? Theme.m3tertiary : Theme.m3onSurfaceVariant
        visible: Notifs.dnd || Notifs.count > 0
    }

    StatusIcon {
        name: Profiles.icon
    }

    StatusIcon {
        name: Net.icon
    }

    StatusIcon {
        name: Bt.icon
        visible: Bt.available
    }

    StatusIcon {
        name: Audio.icon
    }

    Row {
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2
        visible: Battery.available

        StatusIcon {
            name: Battery.icon
            rotation: 90
            color: Battery.low ? Theme.m3error : Battery.charging ? Theme.m3primary : Theme.m3onSurfaceVariant
        }

        StyledText {
            anchors.verticalCenter: parent.verticalCenter
            text: `${Math.round(Battery.percentage * 100)}%`
            font.weight: Font.Medium
            font.features: Theme.font.tabular
        }
    }
}
