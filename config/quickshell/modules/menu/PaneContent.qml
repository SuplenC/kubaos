import QtQuick
import Quickshell
import Quickshell.Widgets
import KubaOS.Components as K
import "words.js" as Words

// What the detail pane shows of one row: its icon, title, subtitle and kind, its Action panel with
// each Action's keys, and the keys the row takes. `entry` is what DetailPane hands it; see
// `Menu.qml`'s `paneEntry`.
//
// It changes with every move of the selection, so it is set in `Label`s on Qt's positioners, as
// the rows are: arrowing down the list at key repeat redraws it thirty times a second.
Item {
    id: content

    property var entry: null
    readonly property bool runs: (entry?.unavailable ?? "") === ""
    readonly property string iconSource: {
        const icon = entry?.icon ?? "";
        return icon === "" ? "" : icon.startsWith("/") ? `file://${icon}` : Quickshell.iconPath(icon, true);
    }
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property real dim: content.runs ? 1 : K.Tokens.opacity.disabled_content

    Column {
        id: top

        width: content.width

        // A page's one big symbol: the app's icon, or its Extension's.
        Item {
            width: K.Tokens.icon.s48
            height: K.Tokens.icon.s48

            IconImage {
                id: image

                anchors.fill: parent
                implicitSize: K.Tokens.icon.s48
                visible: content.iconSource !== ""
                asynchronous: true
                source: content.iconSource
                opacity: content.dim
            }

            K.Icon {
                visible: !image.visible
                name: Words.glyph(content.entry?.extension ?? "")
                size: K.Tokens.icon.s48
                color: K.Look.alpha(K.Tokens.color.primary, content.dim)
            }
        }

        Item {
            height: K.Tokens.space.s16
        }

        Label {
            width: parent.width
            text: content.entry?.title ?? ""
            type: K.Tokens.type.heading
            wrapMode: Text.Wrap
            maximumLineCount: 3
        }

        Item {
            height: K.Tokens.space.s4
        }

        Label {
            width: parent.width
            visible: text !== ""
            text: content.entry?.subtitle ?? ""
            type: K.Tokens.type.body
            wrapMode: Text.Wrap
            maximumLineCount: 4
            color: K.Tokens.color.on_surface_variant
        }

        Label {
            width: parent.width
            visible: !content.runs
            text: content.entry?.unavailable ?? ""
            type: K.Tokens.type.body
            wrapMode: Text.Wrap
            maximumLineCount: 3
            color: K.Tokens.color.error
        }

        Item {
            height: K.Tokens.space.s8
        }

        // The kind, and the Alias the query named it by.
        Label {
            width: parent.width
            text: [Words.kindLabel(content.entry?.extension ?? ""), content.entry?.alias ?? ""].filter(word => word !== "").join(" · ")
            type: K.Tokens.type.label
            color: K.Tokens.color.on_surface_variant
        }
    }

    // The Action panel as Ctrl+K lists it: the Item's own Actions, then the Contributed ones
    // under the Extension offering each.
    Column {
        id: actions

        anchors.top: top.bottom
        anchors.topMargin: K.Tokens.space.s24
        anchors.bottom: keys.top
        anchors.bottomMargin: K.Tokens.space.s16
        width: content.width
        clip: true
        visible: (content.entry?.actions ?? []).length > 0
        spacing: K.Tokens.space.s8

        Label {
            width: parent.width
            text: "Actions"
            type: K.Tokens.type.label
            color: K.Tokens.color.on_surface_variant
        }

        Repeater {
            model: content.entry?.actions ?? []

            Column {
                id: action

                required property int index
                required property var modelData
                readonly property bool heading: modelData.group !== "" && (index === 0 || content.entry.actions[index - 1].group !== modelData.group)

                width: actions.width
                spacing: K.Tokens.space.s8

                Label {
                    width: parent.width
                    height: K.Tokens.space.s8 + K.Tokens.type.label.line
                    visible: action.heading
                    verticalAlignment: Text.AlignBottom
                    text: Words.kindLabel(action.modelData.group)
                    type: K.Tokens.type.label
                    color: K.Tokens.color.on_surface_variant
                }

                Item {
                    width: parent.width
                    height: K.Tokens.type.body.line

                    Label {
                        width: shortcut.x - K.Tokens.space.s8
                        height: parent.height
                        text: action.modelData.title
                        color: action.index === 0 ? K.Tokens.color.on_surface : K.Tokens.color.on_surface_variant
                    }

                    // The Action's keys as keycaps, as tall as the title's line, at the row's end.
                    Row {
                        id: shortcut

                        readonly property var keys: action.index === 0 ? ["↵"] : Words.shortcutKeys(action.modelData.shortcut)

                        x: parent.width - width
                        spacing: K.Tokens.space.s4

                        Repeater {
                            model: shortcut.keys.length

                            Keycap {
                                required property int index

                                key: shortcut.keys[index] ?? ""
                            }
                        }
                    }
                }
            }
        }
    }

    KeyHints {
        id: keys

        anchors.bottom: parent.bottom
        room: content.width
        text: content.entry?.hints ?? ""
    }
}
