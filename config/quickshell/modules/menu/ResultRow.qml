import QtQuick
import Quickshell
import Quickshell.Widgets
import qs.theme
import qs.components

// One row of the Results or of a list View: the Item's icon, its title with
// the matched characters lit, its subtitle, and the Alias the query named it
// by. The list draws the selection behind it. Cheap on purpose, since every
// keystroke rewrites the rows: no layouts, layers or effects.
Item {
    id: row

    required property int index
    required property string title
    required property string markup
    required property string subtitle
    required property string icon
    required property string extension
    // The Alias the query named this Result by, drawn as a badge; empty for
    // none.
    required property string alias
    // Why the Result can't run, shown in place of its subtitle; the row is
    // greyed out. Empty when it can run.
    required property string unavailable

    property MenuLook look
    readonly property bool current: ListView.isCurrentItem
    readonly property bool runs: unavailable === ""
    readonly property string iconSource: icon === "" ? "" : icon.startsWith("/") ? `file://${icon}` : Quickshell.iconPath(icon, true)

    // The pointer moved over the row at (x, y) in the row, or clicked it.
    signal pointed(real x, real y)
    signal picked

    width: ListView.view.width
    height: look.rowHeight

    IconImage {
        id: image

        x: row.look.rowPad
        y: row.look.snap((row.height - height) / 2)
        implicitSize: row.look.iconSize
        visible: row.iconSource !== ""
        asynchronous: true
        source: row.iconSource
        opacity: row.runs ? 1 : 0.4
    }

    // Items without an icon of their own, such as a sum or a bind, show
    // their Extension's glyph.
    Icon {
        anchors.centerIn: image
        visible: !image.visible
        name: row.look.glyph(row.extension)
        size: Math.round(row.look.iconSize * 0.7)
        color: row.current ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
        opacity: row.runs ? 1 : 0.4
    }

    Rectangle {
        id: badge

        anchors.right: parent.right
        anchors.rightMargin: row.look.rowPad
        y: row.look.snap((row.height - height) / 2)
        visible: row.alias !== ""
        width: visible ? row.look.snap(badgeText.implicitWidth + 2 * row.look.gap) : 0
        height: row.look.snap(badgeText.implicitHeight + row.look.nudge)
        radius: height / 2
        color: "transparent"
        border.width: 1
        border.color: row.current ? Theme.alpha(Theme.m3onSecondaryContainer, 0.4) : Theme.m3outlineVariant

        StyledText {
            id: badgeText

            anchors.centerIn: parent
            text: row.alias
            color: row.current ? Theme.m3onSecondaryContainer : Theme.m3onSurfaceVariant
            font.pixelSize: row.look.font.small
        }
    }

    Column {
        anchors.left: image.right
        anchors.leftMargin: row.look.inset
        anchors.right: badge.left
        anchors.rightMargin: badge.visible ? row.look.gap : 0
        anchors.verticalCenter: parent.verticalCenter

        Text {
            width: parent.width
            text: row.markup
            textFormat: Text.StyledText
            renderType: Text.NativeRendering
            elide: Text.ElideRight
            maximumLineCount: 1
            color: !row.runs ? Theme.m3onSurfaceVariant : row.current ? Theme.m3onSecondaryContainer : Theme.m3onSurface
            opacity: row.runs ? 1 : 0.6
            font.family: Theme.font.sans
            font.pixelSize: row.look.font.title
            font.weight: row.current ? Font.Medium : Font.Normal
        }

        StyledText {
            width: parent.width
            visible: text !== ""
            text: row.runs ? row.subtitle : row.unavailable
            color: row.current ? Theme.alpha(Theme.m3onSecondaryContainer, 0.75) : Theme.m3onSurfaceVariant
            font.pixelSize: row.look.font.small
        }
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
