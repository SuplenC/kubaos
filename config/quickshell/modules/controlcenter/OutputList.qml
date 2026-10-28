import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services

// Audio outputs. Enter makes the selected one the default, which carries the check.
Well {
    id: root

    focused: list.activeFocus

    NavList {
        id: list

        anchors.fill: parent
        // The well's padding above the first row and below the last, scrolled with them.
        topMargin: K.Tokens.space.s12
        bottomMargin: K.Tokens.space.s12
        model: ScriptModel {
            values: Audio.sinks
        }
        highlightShown: activeFocus
        onActivated: index => Audio.setDefault(Audio.sinks[index])

        delegate: K.ListRow {
            required property var modelData
            required property int index

            width: ListView.view.width
            icon: modelData === Audio.sink ? "speaker" : "speaker_group"
            title: Audio.label(modelData)
            subtitle: modelData === Audio.sink ? "Default output" : ""
            checkable: true
            checked: modelData === Audio.sink
            selected: ListView.isCurrentItem && ListView.view.highlightShown
            onActivated: Audio.setDefault(modelData)
        }
    }
}
