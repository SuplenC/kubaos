import QtQuick
import KubaOS.Components as K
import "words.js" as Words

// The Action panel, Ctrl+K: the selected row's Actions, the Item's own first, then the Contributed
// ones under the Extension offering each, with their keys. A popup surface over the list, its
// entries one-line rows 4 apart and 12 in, so their corners nest in its own. Menu.qml gives it the
// keys; the pointer picks too.
//
// Its rows are drawn as `ResultRow` draws one, for the same reason: Ctrl+K opens it at once.
Rectangle {
    id: panel

    // Each `{title, group, shortcut}`, as `KotodamaSearch.actions` gives them.
    property var actions: []
    // The entry Enter runs.
    property int current: 0

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property int rowRadius: K.Look.nested(K.Tokens.radius.surface, K.Tokens.space.s12)

    // The pointer chose entry `n`.
    signal picked(int n)

    implicitHeight: entries.implicitHeight + K.Tokens.space.s12 * 2
    radius: K.Tokens.radius.surface
    color: K.Tokens.color.surface_container_high
    border.width: K.Look.hairline(panel.dpr)
    border.color: K.Look.alpha(K.Tokens.color.outline_variant, K.Tokens.opacity.outline_soft)

    Column {
        id: entries

        x: K.Tokens.space.s12
        y: K.Tokens.space.s12
        width: panel.width - K.Tokens.space.s12 * 2
        spacing: K.Tokens.space.s4

        Repeater {
            model: panel.actions

            Column {
                id: entry

                required property int index
                required property var modelData
                readonly property bool heading: modelData.group !== "" && (index === 0 || panel.actions[index - 1].group !== modelData.group)
                readonly property bool current: index === panel.current
                readonly property color main: entry.current ? K.Tokens.color.on_secondary_container : K.Tokens.color.on_surface

                width: entries.width
                spacing: K.Tokens.space.s4

                // An Extension's name over its Contributed actions, 8 under the row above.
                Label {
                    x: K.Tokens.space.s12
                    width: parent.width - K.Tokens.space.s12 * 2
                    height: K.Tokens.space.s8 + K.Tokens.type.label.line
                    visible: entry.heading
                    verticalAlignment: Text.AlignBottom
                    type: K.Tokens.type.label
                    text: Words.kindLabel(entry.modelData.group)
                    color: K.Tokens.color.on_surface_variant
                }

                Rectangle {
                    width: parent.width
                    height: K.Tokens.size.row
                    radius: panel.rowRadius
                    color: entry.current ? K.Tokens.color.secondary_container : "transparent"

                    Label {
                        x: K.Tokens.space.s12
                        y: (parent.height - height) / 2
                        width: shortcut.x - K.Tokens.space.s12 * 2
                        height: K.Tokens.type.body_strong.line
                        type: K.Tokens.type.body_strong
                        text: entry.modelData.title
                        color: entry.main
                    }

                    // The Action's key as a keycap, centred down the row at its end.
                    K.Keycap {
                        id: shortcut

                        x: parent.width - K.Tokens.space.s12 - width
                        y: K.Look.snap((parent.height - height) / 2, panel.dpr)
                        width: implicitWidth
                        height: implicitHeight
                        visible: key !== ""
                        key: entry.index === 0 ? "↵" : Words.shortcutLabel(entry.modelData.shortcut)
                    }

                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        onPositionChanged: panel.current = entry.index
                        onClicked: panel.picked(entry.index)
                    }
                }
            }
        }
    }
}
