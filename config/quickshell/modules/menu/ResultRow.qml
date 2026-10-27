import QtQuick
import Quickshell
import Quickshell.Widgets
import KubaOS.Components as K
import "words.js" as Words

// One row of the Results or of a list View, as `K.ListRow` draws a two-line row: 56 tall, 12 in,
// a 20 icon, the title in `body_strong` with the matched characters lit, the subtitle in `small`,
// and the Alias the query named it by as the row's mono detail at its end. The list draws the
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

    readonly property bool selected: ListView.isCurrentItem
    readonly property bool runs: unavailable === ""
    readonly property string iconSource: icon === "" ? "" : icon.startsWith("/") ? `file://${icon}` : Quickshell.iconPath(icon, true)
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property color main: row.selected ? K.Tokens.color.on_secondary_container : K.Tokens.color.on_surface
    readonly property color muted: row.selected ? K.Look.alpha(K.Tokens.color.on_secondary_container, K.Tokens.opacity.subdued) : K.Tokens.color.on_surface_variant
    readonly property real dim: row.runs ? 1 : K.Tokens.opacity.disabled_content
    // Where the words start: past the icon and the gap after it.
    readonly property int textStart: K.Tokens.space.s12 * 2 + K.Tokens.icon.s20
    readonly property bool twoLine: subtitleLine.text !== ""

    // The pointer moved over the row at (x, y) in the row, or clicked it.
    signal pointed(real x, real y)
    signal picked

    width: ListView.view.width
    height: K.Tokens.size.row_two_line

    IconImage {
        id: image

        x: K.Tokens.space.s12
        y: (row.height - K.Tokens.icon.s20) / 2
        implicitSize: K.Tokens.icon.s20
        visible: row.iconSource !== ""
        asynchronous: true
        source: row.iconSource
        opacity: row.dim
    }

    // Items without an icon of their own, such as a sum or a bind, show their Extension's symbol.
    K.Icon {
        x: K.Tokens.space.s12
        y: (row.height - K.Tokens.icon.s20) / 2
        visible: !image.visible
        name: Words.glyph(row.extension)
        size: K.Tokens.icon.s20
        color: K.Look.alpha(row.main, row.dim)
    }

    // Title and subtitle, 4 apart, centred down the row; a title alone sits in the middle.
    Label {
        id: titleLine

        x: row.textStart
        y: row.twoLine ? (row.height - K.Tokens.type.body_strong.line - K.Tokens.space.s4 - K.Tokens.type.small.line) / 2 : (row.height - K.Tokens.type.body_strong.line) / 2
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
        y: titleLine.y + K.Tokens.type.body_strong.line + K.Tokens.space.s4
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
