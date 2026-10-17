import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services

// Audio outputs. Enter makes the selected one the default.
NavList {
    model: ScriptModel {
        values: Audio.sinks
    }
    highlightShown: activeFocus
    onActivated: index => Audio.setDefault(Audio.sinks[index])

    delegate: ListRow {
        required property var modelData
        required property int index

        width: ListView.view.width
        icon: modelData === Audio.sink ? "speaker" : "speaker_group"
        title: Audio.label(modelData)
        subtitle: modelData === Audio.sink ? "Default output" : ""
        active: modelData === Audio.sink
        current: ListView.isCurrentItem && ListView.view.activeFocus
        onClicked: Audio.setDefault(modelData)
    }
}
