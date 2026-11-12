import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.components
import qs.services

// The Audio tab: the outputs under an Output label, then the inputs under an Input label. Enter
// makes the selected device the default of its kind, which carries the check; the keys skip the
// labels.
Well {
    id: root

    // A section's label: an object like the devices, as a model takes one kind of value, and made
    // once, so the model keeps its row as the devices change.
    component Section: QtObject {
        property string label
    }

    readonly property Section outputLabel: Section {
        label: "Output"
    }
    readonly property Section inputLabel: Section {
        label: "Input"
    }
    readonly property var entries: [root.outputLabel, ...Audio.sinks, root.inputLabel, ...Audio.sources]

    function pick(entry): void {
        if (entry.label !== undefined)
            return;
        if (entry.isSink)
            Audio.setDefault(entry);
        else
            Audio.setDefaultSource(entry);
    }

    focused: list.activeFocus

    NavList {
        id: list

        anchors.fill: parent
        // The well's padding above the first row and below the last, scrolled with them.
        topMargin: K.Tokens.space.s12
        bottomMargin: K.Tokens.space.s12
        model: ScriptModel {
            values: root.entries
        }
        highlightShown: activeFocus
        selectable: index => index >= 0 && index < root.entries.length && root.entries[index].label === undefined
        // Onto a device, never a label, as the keyboard arrives.
        onActiveFocusChanged: if (activeFocus && !selectable(currentIndex))
            jump(currentIndex)
        onActivated: index => root.pick(root.entries[index])

        delegate: Loader {
            id: entry

            required property var modelData
            required property int index

            readonly property bool output: entry.modelData.isSink === true
            readonly property bool isDefault: entry.modelData === Audio.sink || entry.modelData === Audio.source
            // A label after rows stands 12 clear of them, with the 4 the list puts between rows.
            readonly property int labelRoom: entry.index > 0 ? K.Tokens.space.s12 : 0

            width: ListView.view.width
            sourceComponent: entry.modelData.label === undefined ? deviceRow : sectionLabel

            Component {
                id: deviceRow

                K.ListRow {
                    icon: entry.output ? (entry.isDefault ? "speaker" : "speaker_group") : (entry.isDefault ? "mic" : "mic_external_on")
                    title: Audio.label(entry.modelData)
                    subtitle: entry.isDefault ? (entry.output ? "Default output" : "Default input") : ""
                    checkable: true
                    checked: entry.isDefault
                    selected: entry.ListView.isCurrentItem && entry.ListView.view.highlightShown
                    onActivated: root.pick(entry.modelData)
                }
            }

            Component {
                id: sectionLabel

                Item {
                    implicitHeight: entry.labelRoom + K.Tokens.type.label.line

                    K.Text {
                        x: K.Tokens.space.s12
                        y: entry.labelRoom
                        text: entry.modelData.label
                        style: K.Tokens.type.label
                        color: K.Tokens.color.on_surface_variant
                    }
                }
            }
        }
    }
}
