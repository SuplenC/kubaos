import QtQuick
import KubaOS.Components as K

// A detail View: the Extension's Markdown, and its metadata under it, each label in a column a
// quarter wide beside its value. Long content scrolls.
Flickable {
    id: view

    property string markdown: ""
    // Each `{label, value}`.
    property var metadata: []

    clip: true
    contentHeight: column.implicitHeight
    boundsBehavior: Flickable.StopAtBounds

    K.Stack {
        id: column

        width: view.width
        fill: true
        gap: K.Tokens.space.s24

        // Markdown sets its own runs of bold, italics, headings and lists, and their line spacing,
        // which `K.Text` has no way to; the body style is its base.
        Text {
            readonly property bool fillWidth: true

            wrapMode: Text.Wrap
            textFormat: Text.MarkdownText
            text: view.markdown
            color: K.Tokens.color.on_surface
            linkColor: K.Tokens.color.primary
            font.family: K.Tokens.type.body.family
            font.weight: K.Tokens.type.body.weight
            font.pixelSize: K.Tokens.type.body.size
        }

        K.Grid {
            visible: view.metadata.length > 0
            weights: [1, 3]
            centred: true
            gap: K.Tokens.space.s8

            Repeater {
                // Each line's label, then its value, filling the grid row by row.
                model: {
                    const cells = [];
                    for (const line of view.metadata) {
                        cells.push({
                            text: line.label,
                            label: true
                        });
                        cells.push({
                            text: line.value,
                            label: false
                        });
                    }
                    return cells;
                }

                K.Text {
                    required property var modelData

                    text: modelData.text
                    style: modelData.label ? K.Tokens.type.label : K.Tokens.type.body_strong
                    color: modelData.label ? K.Tokens.color.on_surface_variant : K.Tokens.color.on_surface
                }
            }
        }
    }
}
