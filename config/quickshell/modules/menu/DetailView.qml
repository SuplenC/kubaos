import QtQuick
import qs.theme
import qs.components

// A detail View: the Extension's Markdown, and its metadata as labelled
// lines under it. Long content scrolls.
Flickable {
    id: view

    property MenuLook look
    property string markdown: ""
    // Each `{label, value}`.
    property var metadata: []

    clip: true
    contentHeight: column.implicitHeight + 2 * look.pad
    boundsBehavior: Flickable.StopAtBounds

    Column {
        id: column

        x: view.look.pad
        y: view.look.pad
        width: view.width - 2 * view.look.pad
        spacing: view.look.tight

        Text {
            width: parent.width
            wrapMode: Text.Wrap
            textFormat: Text.MarkdownText
            renderType: Text.NativeRendering
            text: view.markdown
            color: Theme.m3onSurface
            linkColor: Theme.m3primary
            font.family: Theme.font.sans
            font.pixelSize: view.look.font.title
        }

        Item {
            width: 1
            height: view.look.gap
            visible: view.metadata.length > 0
        }

        Repeater {
            model: view.metadata

            Row {
                id: line

                required property var modelData

                spacing: view.look.inset

                StyledText {
                    width: view.look.labelWidth
                    text: line.modelData.label
                    color: Theme.m3onSurfaceVariant
                    font.pixelSize: view.look.font.small
                }

                StyledText {
                    width: column.width - view.look.labelWidth - view.look.inset
                    text: line.modelData.value
                    font.pixelSize: view.look.font.small
                }
            }
        }
    }
}
