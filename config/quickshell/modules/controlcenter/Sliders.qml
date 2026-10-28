import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// Volume, screen brightness and the power stops, compact pills the panel's width. Volume's icon
// mutes and unmutes, keeping the level to come back to; the power slider's icon steps from high to
// low and back.
K.Stack {
    id: root

    fill: true
    gap: K.Tokens.space.s8

    K.Slider {
        size: "Compact"
        icon: Audio.icon
        value: Audio.muted ? 0 : Audio.volume
        words: Audio.muted ? "Muted" : ""
        iconButton: true
        iconLabel: Audio.muted ? "Unmute" : "Mute"
        Accessible.name: "Volume"
        onMoved: v => Audio.setVolume(v)
        onIconActivated: Audio.toggleMute()
    }

    K.Slider {
        visible: Brightness.available
        size: "Compact"
        icon: Brightness.icon
        value: Brightness.value
        Accessible.name: "Brightness"
        onMoved: v => Brightness.set(v)
    }

    PowerSlider {
        size: "Compact"
    }
}
