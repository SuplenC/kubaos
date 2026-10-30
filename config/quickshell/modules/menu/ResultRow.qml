import QtQuick
import Quickshell
import KubaOS.Components as K
import "words.js" as Words

// One row of the Results or of a list View: 56 tall, 12 in, the Item's icon on a regular icon
// tile (40, its icon spanning both lines), 16 before the words so they stand clear of it, the
// title in `body_strong` with the matched characters lit right over the subtitle in `small`, and
// the Alias the query named it by as the row's mono detail at its end. The list draws the
// selection behind it, so it can slide.
//
// HAZARD: no layout primitives and no `K.Text` in here. Every keystroke makes up to fourteen of
// these, and fourteen `K.ListRow`s took 15 ms to make against 0.9 ms for these, most of a frame at
// 60 Hz. Every box is a token or a sum of them instead, so the row lands on the same pixels.
Item {
    id: row

    required property int index
    required property string title
    required property string markup
    required property string subtitle
    required property string icon
    required property string extension
    // The Alias the query named this Result by; empty for none.
    required property string alias
    // Why the Result can't run, shown in place of its subtitle, with the rest of the row dimmed.
    // Empty when it can run.
    required property string unavailable
    // Whether a verb in the query chose the Action Enter would run, in place of the Item's own
    // first; the menu keeps to the Item's own first (enter.js).
    required property bool primaryChosen

    readonly property bool selected: ListView.isCurrentItem
    readonly property bool runs: unavailable === ""
    readonly property string iconSource: icon === "" ? "" : icon.startsWith("/") ? `file://${icon}` : Quickshell.iconPath(icon, true)
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property color main: row.selected ? K.Tokens.color.on_secondary_container : K.Tokens.color.on_surface
    readonly property color muted: row.selected ? K.Look.alpha(K.Tokens.color.on_secondary_container, K.Tokens.opacity.subdued_text) : K.Tokens.color.on_surface_variant
    readonly property real dim: row.runs ? 1 : K.Tokens.opacity.disabled_content
    // Where the words start: past the tile and the gap after it. Every row is 56 tall, a title
    // alone included, so every tile is the regular one and the words start in one column.
    readonly property int textStart: K.Tokens.space.s12 + K.Tokens.size.control_md + K.Tokens.space.s16
    readonly property bool twoLine: subtitleLine.text !== ""

    // The pointer moved over the row at (x, y) in the row, or clicked it.
    signal pointed(real x, real y)
    signal picked

    width: ListView.view.width
    height: K.Tokens.size.row_two_line

    // Items without an icon of their own, such as a sum or a bind, show their Extension's symbol.
    K.IconTile {
        x: K.Tokens.space.s12
        y: (row.height - K.Tokens.size.control_md) / 2
        icon: Words.glyph(row.extension)
        source: row.iconSource
        opacity: row.dim
    }

    // Title and subtitle line on line, centred down the row; a title alone sits in the middle.
    Label {
        id: titleLine

        x: row.textStart
        y: row.twoLine ? (row.height - K.Tokens.type.body_strong.line - K.Tokens.type.small.line) / 2 : (row.height - K.Tokens.type.body_strong.line) / 2
        width: (detail.visible ? detail.x - K.Tokens.space.s12 : row.width - K.Tokens.space.s12) - row.textStart
        height: K.Tokens.type.body_strong.line
        type: K.Tokens.type.body_strong
        text: row.markup
        textFormat: Text.StyledText
        color: K.Look.alpha(row.main, row.dim)
    }

    Label {
        id: subtitleLine

        x: row.textStart
        y: titleLine.y + K.Tokens.type.body_strong.line
        width: titleLine.width
        height: K.Tokens.type.small.line
        visible: text !== ""
        type: K.Tokens.type.small
        text: row.runs ? row.subtitle : row.unavailable
        color: row.muted
    }

    Label {
        id: detail

        x: row.width - K.Tokens.space.s12 - K.Look.snapUp(implicitWidth, row.dpr)
        y: (row.height - K.Tokens.type.mono.line) / 2
        height: K.Tokens.type.mono.line
        visible: row.alias !== ""
        type: K.Tokens.type.mono
        text: row.alias
        color: row.muted
    }

    MouseArea {
        id: area

        anchors.fill: parent
        hoverEnabled: true
        onEntered: row.pointed(area.mouseX, area.mouseY)
        onPositionChanged: mouse => row.pointed(mouse.x, mouse.y)
        onClicked: row.picked()
    }
}
