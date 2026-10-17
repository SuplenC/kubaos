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
            icon: Audio.muted ? "volume_off" : "volume_up"
            tooltip: Audio.muted ? "Unmute" : "Mute"
            baseColor: Theme.m3surfaceContainerHigh
            toggled: Audio.muted
            onClicked: Audio.toggleMute()
        }
    }

    StyledSlider {
        Layout.fillWidth: true
        Layout.rightMargin: 36 + Theme.spacing.sm // lines up with the volume slider
        visible: Brightness.available
        icon: Brightness.icon
        value: Brightness.value
        onMoved: v => Brightness.set(v)
    }
}
