import QtQuick
import Quickshell.Widgets
import KubaOS.Components as K
import "words.js" as Words

// The Action panel, Ctrl+K: the selected row's Actions, the Item's own first, then the Contributed
// ones under the Extension offering each, with their keys. A popup surface over the list, its
// entries one-line rows 4 apart and 12 in, so their corners nest in its own. Menu.qml gives it the
// keys; the pointer picks too.
//
// It is never taller than `maxHeight`, the room Menu.qml gives it: more Actions than that scroll
// right up to its rounded edge, which cuts them, its hairline drawn over them. The keys keep the
// entry Enter runs in view, with its Extension's name when it is the first under it; the pointer
// never scrolls it, so the entry it is on stays under it.
//
// Its rows are placed with token arithmetic, as `K.ListRow` places the Results', for the same
// reason: Ctrl+K opens it at once.
Item {
    id: panel

    // Each `{title, group, shortcut}`, as `KotodamaSearch.actions` gives them.
    property var actions: []
    // The entry Enter runs.
    property int current: 0
    // The tallest it grows.
    property real maxHeight: Infinity
    // How far its entries are scrolled.
    property alias contentY: view.contentY
    // The entries with the padding round them, which may be taller than the panel.
    readonly property real fullHeight: entries.implicitHeight + K.Tokens.space.s12 * 2
    // The pointer is moving `current`, which then scrolls nothing.
    property bool pointing: false

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property int rowRadius: K.Look.nested(K.Tokens.radius.surface, K.Tokens.space.s12)

    // The pointer chose entry `n`.
    signal picked(int n)

    // Scrolls the entry Enter runs into view with the padding round it, and its heading if it has
    // one.
    function reveal(): void {
        const entry = rows.itemAt(panel.current);
        if (!entry)
            return;
        const top = entry.y;
        const bottom = entries.y + entry.y + entry.height + K.Tokens.space.s12;
        if (top < view.contentY)
            view.contentY = top;
        else if (bottom > view.contentY + view.height)
            view.contentY = bottom - view.height;
    }

    function point(n: int): void {
        panel.pointing = true;
        panel.current = n;
        panel.pointing = false;
    }

    implicitHeight: Math.min(panel.fullHeight, panel.maxHeight)
    onCurrentChanged: {
        if (!panel.pointing)
            panel.reveal();
    }
    onActionsChanged: view.contentY = 0

    Rectangle {
        anchors.fill: parent
        radius: K.Tokens.radius.surface
        color: K.Tokens.color.surface_container_high
    }

    ClippingRectangle {
        anchors.fill: parent
        radius: K.Tokens.radius.surface
        color: "transparent"

        Flickable {
            id: view

            anchors.fill: parent
            contentWidth: width
            contentHeight: panel.fullHeight
            boundsBehavior: Flickable.StopAtBounds

            Column {
                id: entries

                x: K.Tokens.space.s12
                y: K.Tokens.space.s12
                width: panel.width - K.Tokens.space.s12 * 2
                spacing: K.Tokens.space.s4

                Repeater {
                    id: rows

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
                        Item {
                            width: parent.width
                            height: K.Tokens.space.s8 + K.Tokens.type.label.line
                            visible: entry.heading

                            K.Text {
                                x: K.Tokens.space.s12
                                y: K.Tokens.space.s8
                                width: parent.width - K.Tokens.space.s12 * 2
                                maximumLines: 1
                                style: K.Tokens.type.label
                                text: Words.kindLabel(entry.modelData.group)
                                color: K.Tokens.color.on_surface_variant
                            }
                        }

                        Rectangle {
                            width: parent.width
                            height: K.Tokens.size.row
                            radius: panel.rowRadius
                            color: entry.current ? K.Tokens.color.secondary_container : "transparent"

                            K.Text {
                                x: K.Tokens.space.s12
                                y: (parent.height - height) / 2
                                width: shortcut.x - K.Tokens.space.s12 * 2
                                maximumLines: 1
                                style: K.Tokens.type.body_strong
                                text: entry.modelData.title
                                color: entry.main
                            }

                            // The Action's keys as keycaps, centred down the row at its end.
                            Row {
                                id: shortcut

                                readonly property var keys: entry.index === 0 ? ["↵"] : Words.shortcutKeys(entry.modelData.shortcut)

                                x: parent.width - K.Tokens.space.s12 - width
                                y: K.Look.snap((parent.height - height) / 2, panel.dpr)
                                spacing: K.Tokens.space.s4

                                Repeater {
                                    model: shortcut.keys.length

                                    K.Keycap {
                                        required property int index

                                        key: shortcut.keys[index] ?? ""
                                    }
                                }
                            }

                            MouseArea {
                                anchors.fill: parent
                                hoverEnabled: true
                                onPositionChanged: panel.point(entry.index)
                                onClicked: panel.picked(entry.index)
                            }
                        }
                    }
                }
            }
        }
    }

    // The hairline over the entries, so one scrolled to the edge never covers it.
    Rectangle {
        anchors.fill: parent
        radius: K.Tokens.radius.surface
        color: "transparent"
        border.width: K.Look.hairline(panel.dpr)
        border.color: K.Look.alpha(K.Tokens.color.outline_variant, K.Tokens.opacity.outline_soft)
    }
}
