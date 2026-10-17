import QtQuick
import qs.theme
import qs.components
import qs.services

// Red dot and elapsed time while dot-hypr-record runs. Click stops it.
// In replay mode the dot is hollow and the label says how much is kept.
// No blinking: a looping animation would repaint the frame layer (and its
// shadow) every frame for as long as the recording runs.
Pill {
    id: root

    visible: Recorder.active
    interactive: true
    spacing: Theme.spacing.sm
    color: Theme.m3errorContainer
    tooltip: Recorder.replay ? "Replay buffer on: SUPER+SHIFT+P saves the last 30 s. Click to turn off" : "Recording. Click to stop"
    onClicked: Recorder.stop()

    Rectangle {
        id: dot

        anchors.verticalCenter: parent.verticalCenter
        width: 8
        height: 8
        radius: 4
        color: Recorder.replay ? "transparent" : Theme.m3error
        border.width: Recorder.replay ? 2 : 0
        border.color: Theme.m3error
    }

    StyledText {
        anchors.verticalCenter: parent.verticalCenter
        text: Recorder.replay ? "Replay 30 s" : Recorder.elapsedText
        color: Theme.m3onErrorContainer
        font.weight: Font.Medium
        font.features: Theme.font.tabular
    }
}
