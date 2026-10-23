import QtQuick
import QtQuick.Layouts
import qs.theme
import qs.components
import qs.services

// Volume (with mute) and screen brightness.
ColumnLayout {
    spacing: Theme.spacing.sm

    RowLayout {
        spacing: Theme.spacing.sm

        StyledSlider {
            Layout.fillWidth: true
            icon: Audio.icon
            value: Audio.muted ? 0 : Audio.volume
            onMoved: v => Audio.setVolume(v)
        }

        IconButton {
            id: mute

            icon: Audio.muted ? "volume_off" : "volume_up"
            tooltip: Audio.muted ? "Unmute" : "Mute"
            baseColor: Theme.m3surfaceContainerHigh
            toggled: Audio.muted
            onClicked: Audio.toggleMute()
        }
    }

    StyledSlider {
        Layout.fillWidth: true
        // Ends where the volume slider does, beside the mute button.
        Layout.rightMargin: mute.width + Theme.spacing.sm
        visible: Brightness.available
        icon: Brightness.icon
        value: Brightness.value
        onMoved: v => Brightness.set(v)
    }
}
