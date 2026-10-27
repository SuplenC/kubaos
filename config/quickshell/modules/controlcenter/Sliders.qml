import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// Volume (with mute), screen brightness and the power stops. Each slider ends where the power
// stop's name begins, so the three tracks are one length.
K.Stack {
    id: root

    // The room after each track: the power stop's widest name, or the mute button if wider.
    readonly property real end: Math.max(power.widest, mute.implicitWidth)

    fill: true
    gap: K.Tokens.space.s8

    K.Row {
        fill: true
        gap: K.Tokens.space.s12
        align: "centre"

        K.Icon {
            name: Audio.icon
            size: K.Tokens.icon.s20
            color: K.Tokens.color.on_surface_variant
        }

        K.Slider {
            value: Audio.muted ? 0 : Audio.volume
            Accessible.name: "Volume"
            onMoved: v => Audio.setVolume(v)
        }

        Item {
            implicitWidth: root.end
            implicitHeight: mute.implicitHeight

            K.IconButton {
                id: mute

                icon: Audio.muted ? "volume_off" : "volume_up"
                label: Audio.muted ? "Unmute" : "Mute"
                kind: Audio.muted ? "Filled" : "Tonal"
                size: "Sm"
                onActivated: Audio.toggleMute()
            }
        }
    }

    K.Row {
        visible: Brightness.available
        fill: true
        gap: K.Tokens.space.s12
        align: "centre"
        rightPadding: root.end + K.Tokens.space.s12

        K.Icon {
            name: Brightness.icon
            size: K.Tokens.icon.s20
            color: K.Tokens.color.on_surface_variant
        }

        K.Slider {
            value: Brightness.value
            Accessible.name: "Brightness"
            onMoved: v => Brightness.set(v)
        }
    }

    PowerSlider {
        id: power

        fill: true
    }
}
