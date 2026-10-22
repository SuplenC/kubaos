import QtQuick
import qs.theme
import qs.components

// An app's intro card, before anything is typed in it: what the Extension
// says now (why it can't do anything, say), what the app does, and a few
// queries with what they give. Drawn as one of Kōbō's wells.
Rectangle {
    id: card

    property MenuLook look
    property string note: ""
    property string about: ""
    // Each `{query, result}`.
    property var examples: []

    implicitHeight: column.implicitHeight + 2 * look.inset
    radius: look.cardRadius
    color: Theme.alpha(Theme.m3surfaceContainerLow, 0.75)
    border.width: 1
    border.color: Theme.alpha(Theme.m3outlineVariant, 0.55)

    Column {
        id: column

        x: card.look.wide
        y: card.look.inset
        width: card.width - 2 * card.look.wide
        spacing: card.look.tight

        StyledText {
            width: parent.width
            visible: card.note !== ""
            text: card.note
            wrapMode: Text.Wrap
            maximumLineCount: 2
            color: Theme.m3primary
            font.pixelSize: card.look.font.body
        }

        StyledText {
            width: parent.width
            text: card.about
            wrapMode: Text.Wrap
            maximumLineCount: 3
            font.pixelSize: card.look.font.body
        }

        Repeater {
            model: card.examples

            Row {
                id: example

                required property var modelData

                spacing: card.look.gap

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: card.look.snap(query.implicitWidth + 2 * card.look.gap)
                    height: card.look.badgeHeight
                    radius: card.look.keyRadius
                    color: Theme.m3surfaceContainerHigh

                    StyledText {
                        id: query

                        anchors.centerIn: parent
                        text: example.modelData.query
                        font.family: Theme.font.mono
                        font.pixelSize: card.look.font.small
                    }
                }

                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "arrow_forward"
                    size: card.look.exampleIcon
                    color: Theme.m3onSurfaceVariant
                }

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: example.modelData.result
                    color: Theme.m3onSurfaceVariant
                    font.pixelSize: card.look.font.small
                }
            }
        }
    }
}
