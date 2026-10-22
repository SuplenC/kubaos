import QtQuick
import Quickshell
import Quickshell.Widgets
import qs.theme
import qs.components

// What the detail pane shows of one row: its icon, title, subtitle and kind,
// its Action panel with each Action's shortcut, and the keys it takes.
// `entry` is what DetailPane hands it; see `Menu.qml`'s `paneEntry`.
Item {
    id: content

    property MenuLook look
    property var entry: null
    readonly property bool runs: (entry?.unavailable ?? "") === ""
    readonly property string iconSource: {
        const icon = entry?.icon ?? "";
        return icon === "" ? "" : icon.startsWith("/") ? `file://${icon}` : Quickshell.iconPath(icon, true);
    }

    Column {
        id: top

        x: content.look.pad
        y: content.look.pad
        width: content.width - 2 * content.look.pad
        spacing: content.look.nudge

        Rectangle {
            width: content.look.paneIconBox
            height: width
            radius: content.look.rowRadius
            color: Theme.m3surfaceContainerHigh

            IconImage {
                id: image

                anchors.centerIn: parent
                implicitSize: content.look.paneIcon
                visible: content.iconSource !== ""
                asynchronous: true
                source: content.iconSource
                opacity: content.runs ? 1 : 0.4
            }

            Icon {
                anchors.centerIn: parent
                visible: !image.visible
                name: content.look.glyph(content.entry?.extension ?? "")
                size: content.look.paneGlyph
                color: Theme.m3primary
            }
        }

        Item {
            width: 1
            height: content.look.inset
        }

        StyledText {
            width: parent.width
            text: content.entry?.title ?? ""
            wrapMode: Text.Wrap
            maximumLineCount: 3
            verticalAlignment: Text.AlignTop
            font.pixelSize: content.look.font.heading
            font.weight: Font.Medium
        }

        StyledText {
            width: parent.width
            visible: text !== ""
            text: content.entry?.subtitle ?? ""
            wrapMode: Text.Wrap
            maximumLineCount: 4
            verticalAlignment: Text.AlignTop
            color: Theme.m3onSurfaceVariant
            font.pixelSize: content.look.font.body
        }

        StyledText {
            width: parent.width
            visible: !content.runs
            text: content.entry?.unavailable ?? ""
            wrapMode: Text.Wrap
            maximumLineCount: 3
            verticalAlignment: Text.AlignTop
            color: Theme.m3error
            font.pixelSize: content.look.font.body
        }

        Item {
            width: 1
            height: content.look.gap
        }

        // The kind, and the Alias the query named it by.
        Row {
            spacing: content.look.gap

            Repeater {
                model: [content.look.kindLabel(content.entry?.extension ?? ""), content.entry?.alias ?? ""].filter(label => label !== "")

                Rectangle {
                    required property string modelData

                    width: content.look.snap(label.implicitWidth + 2 * content.look.badgePad)
                    height: content.look.badgeHeight
                    radius: height / 2
                    color: Theme.m3secondaryContainer

                    StyledText {
                        id: label

                        anchors.centerIn: parent
                        text: parent.modelData
                        color: Theme.m3onSecondaryContainer
                        font.pixelSize: content.look.font.small
                        font.weight: Font.Medium
                    }
                }
            }
        }
    }

    // The Action panel as Ctrl+K lists it: the Item's own Actions, then the
    // Contributed ones under the Extension offering each.
    Column {
        id: actions

        x: content.look.pad
        anchors.top: top.bottom
        anchors.topMargin: content.look.pad
        anchors.bottom: keys.top
        anchors.bottomMargin: content.look.inset
        width: top.width
        clip: true
        visible: (content.entry?.actions ?? []).length > 0

        StyledText {
            height: content.look.labelHeight
            text: "Actions"
            color: Theme.m3onSurfaceVariant
            font.pixelSize: content.look.font.small
            font.weight: Font.Medium
        }

        Repeater {
            model: content.entry?.actions ?? []

            Column {
                id: action

                required property int index
                required property var modelData
                readonly property bool heading: modelData.group !== "" && (index === 0 || content.entry.actions[index - 1].group !== modelData.group)

                width: actions.width

                StyledText {
                    visible: action.heading
                    height: visible ? content.look.headingHeight : 0
                    verticalAlignment: Text.AlignBottom
                    bottomPadding: content.look.nudge
                    text: content.look.kindLabel(action.modelData.group)
                    color: Theme.m3onSurfaceVariant
                    font.pixelSize: content.look.font.small
                }

                Item {
                    width: parent.width
                    height: content.look.actionHeight

                    StyledText {
                        anchors.left: parent.left
                        anchors.right: shortcut.left
                        anchors.rightMargin: content.look.gap
                        anchors.verticalCenter: parent.verticalCenter
                        text: action.modelData.title
                        color: action.index === 0 ? Theme.m3onSurface : Theme.m3onSurfaceVariant
                        font.pixelSize: content.look.font.body
                    }

                    Keycap {
                        id: shortcut

                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        look: content.look
                        visible: key !== ""
                        key: action.index === 0 ? "↵" : content.look.shortcutLabel(action.modelData.shortcut)
                    }
                }
            }
        }
    }

    KeyHints {
        id: keys

        x: content.look.pad
        anchors.bottom: parent.bottom
        anchors.bottomMargin: content.look.pad
        width: top.width
        look: content.look
        text: content.entry?.hints ?? ""
    }
}
