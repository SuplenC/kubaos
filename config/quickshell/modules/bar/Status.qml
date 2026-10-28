import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// System indicators. Click opens the control center, scroll changes volume.
Pill {
    id: root

    interactive: true
    tooltip: [Net.wired ? "Wired" : Net.active?.name ?? (Net.wifiEnabled ? "Wi-Fi not connected" : "Wi-Fi off"), `Volume ${Audio.muted ? "muted" : Math.round(Audio.volume * 100) + "%"}`, Battery.available ? `Battery ${Math.round(Battery.percentage * 100)}%` : ""].filter(Boolean).join("  ·  ")
    onClicked: Panels.toggle("controlcenter")
    onScrolled: steps => Audio.setVolume(Audio.volume + steps * 0.05)

    component StatusIcon: K.Icon {
        size: K.Tokens.icon.s16
        color: K.Tokens.color.on_surface_variant
    }

    // The unread count on the bell, unless do not disturb has it quiet.
    BadgedIcon {
        name: Notifs.dnd ? "notifications_off" : "notifications"
        visible: Notifs.dnd || Notifs.count > 0
        badged: !Notifs.dnd && Notifs.count > 0
        count: Notifs.count
    }

    StatusIcon {
        name: Power.icon
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

    K.Row {
        visible: Battery.available
        gap: K.Tokens.space.s4
        align: "centre"

        StatusIcon {
            name: Battery.icon
            color: Battery.low ? K.Tokens.color.error : Battery.charging ? K.Tokens.color.primary : K.Tokens.color.on_surface_variant
        }

        K.Text {
            text: `${Math.round(Battery.percentage * 100)}%`
            style: K.Tokens.type.mono
            color: K.Tokens.color.on_surface
        }
    }
}
