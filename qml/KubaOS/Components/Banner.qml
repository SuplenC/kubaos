import QtQuick
import KubaOS.Components as K

// A message that matters on the page itself: `kubaos-ui`'s `banner::banner`. A symbol beside a
// `subheading` title and a `body` line, which wrap to the room there is, 16 inside at the surface
// radius, in one of three tones: `Info` (a container with an accent hairline), `Warning` (one
// with a warning hairline) or `Danger` (really red: the error container and an error hairline). Its
// actions sit at its end beside the words, 12 apart; its foot (a held button's bar) 16 under the
// rest.
Item {
    id: root

    // `Info`, `Warning` or `Danger`.
    property string tone: "Info"
    property string icon: ""
    property string title
    property string body
    // More of the body, a paragraph each, 4 apart: the steps of what to do.
    property list<string> paragraphs
    // What to do about it, beside the words: the action the banner is about last.
    default property alias actions: actionsRow.data
    // A line across the foot: how far a held button is.
    property alias foot: footSlot.data

    readonly property bool fillWidth: true
    readonly property var colours: {
        const c = Tokens.color;
        switch (root.tone) {
        case "Danger":
            return {
                fill: c.error_container,
                edge: c.error,
                glyph: c.error,
                title: c.on_error_container,
                body: K.Look.alpha(c.on_error_container, Tokens.opacity.subdued_text)
            };
        case "Warning":
            return {
                fill: c.surface_container_high,
                edge: c.warning,
                glyph: c.warning,
                title: c.on_surface,
                body: c.on_surface_variant
            };
        default:
            return {
                fill: c.surface_container_high,
                edge: c.primary,
                glyph: c.primary,
                title: c.on_surface,
                body: c.on_surface_variant
            };
        }
    }
    readonly property var probe: ({
            role: "component",
            name: "Banner",
            variant: root.tone,
            fixed: {
                width: false,
                height: false
            }
        })

    implicitHeight: column.implicitHeight + Tokens.space.s16 * 2

    function relayout(): void {
        column.relayout();
    }

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.surface
        color: root.colours.fill
        border.width: K.Look.hairline(column.dpr)
        border.color: root.colours.edge
    }

    K.Stack {
        id: column

        x: Tokens.space.s16
        y: Tokens.space.s16
        width: root.width - Tokens.space.s16 * 2
        fill: true
        gap: Tokens.space.s16

        K.Row {
            fill: true
            gap: Tokens.space.s16
            align: "centre"

            K.Icon {
                name: root.icon
                size: Tokens.type.subheading.icon
                color: root.colours.glyph
            }

            K.Stack {
                fill: true
                gap: Tokens.space.s4

                K.Text {
                    text: root.title
                    style: Tokens.type.subheading
                    color: root.colours.title
                }

                K.Text {
                    text: root.body
                    style: Tokens.type.body
                    color: root.colours.body
                }

                Repeater {
                    model: root.paragraphs

                    K.Text {
                        required property string modelData

                        text: modelData
                        style: Tokens.type.body
                        color: root.colours.body
                    }
                }
            }

            K.Row {
                id: actionsRow

                visible: actionsRow.children.length > 0
                gap: Tokens.space.s12
                align: "centre"
            }
        }

        K.Slot {
            id: footSlot

            visible: footSlot.children.length > 0
            fill: true
        }
    }
}
