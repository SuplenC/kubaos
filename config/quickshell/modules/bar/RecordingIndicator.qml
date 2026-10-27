import QtQuick
import KubaOS.Components as K
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
    color: K.Tokens.color.error_container
    ink: K.Tokens.color.on_error_container
    tooltip: Recorder.replay ? "Replay buffer on: SUPER+SHIFT+P saves the last 30 s. Click to turn off" : "Recording. Click to stop"
    onClicked: Recorder.stop()

    // The radio button's dot: the same mark of something on.
    Rectangle {
        implicitWidth: K.Tokens.size.radio_dot
        implicitHeight: K.Tokens.size.radio_dot
        radius: K.Tokens.radius.full
        color: Recorder.replay ? "transparent" : K.Tokens.color.error
        border.width: Recorder.replay ? K.Look.thin(root.dpr) : 0
        border.color: K.Tokens.color.error
    }

    K.Text {
        text: Recorder.replay ? "Replay 30 s" : Recorder.elapsedText
        style: K.Tokens.type.mono
        color: K.Tokens.color.on_error_container
    }
}
