import QtQuick
import qs.theme
import qs.components

// Key hints as Kotodama writes them, `↵ Open · Tab Enter app · ⌃K Actions`,
// drawn as keycaps with their labels: each hint's first word is its key.
// Hints that don't fit the width wrap.
Flow {
    id: hints

    property MenuLook look
    property string text
    property color tint: Theme.m3onSurfaceVariant

    spacing: look.apart

    Repeater {
        model: hints.text === "" ? [] : hints.text.split(" · ")

        Row {
            id: hint

            required property string modelData
            readonly property int split: modelData.indexOf(" ")

            spacing: hints.look.tight

            Keycap {
                anchors.verticalCenter: parent.verticalCenter
                look: hints.look
                tint: hints.tint
                key: hint.split < 0 ? hint.modelData : hint.modelData.slice(0, hint.split)
            }

            StyledText {
                anchors.verticalCenter: parent.verticalCenter
                visible: hint.split >= 0
                text: hint.modelData.slice(hint.split + 1)
                color: hints.tint
                font.pixelSize: hints.look.font.small
            }
        }
    }
}
