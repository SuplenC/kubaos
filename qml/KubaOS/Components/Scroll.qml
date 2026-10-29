import QtQuick

// A page's or a panel's content taller than its room: `kubaos-ui`'s `layout::scroll`. It has no
// edge of its own, so it reaches past its box into the room around it, as far as a glow reaches
// (`focus.far`) or, above and below, up to the part of the panel there (`roomAbove`,
// `roomBelow`), with its content that far in: the content lines up with what is around it, a
// resting control at its edge keeps its whole glow, and the panel's own edge cuts what passes it.
// The thumb is `scrollbar` wide on the box's right edge, a pill in the outline variant (the
// outline while the pointer is on it), floating over the content; it shows only while the content
// is taller than the room. A move the owner makes to `contentY` glides over `medium`.
Item {
    id: root

    default property alias content: body.data
    // The room between it and what the panel draws above and below it, all of which it reaches
    // into: content scrolled past its box shows there and stops at that part, never over it.
    property real roomAbove: Tokens.focus.far
    property real roomBelow: Tokens.focus.far
    property alias contentY: flick.contentY
    // How far it can scroll: the content's height less the room it shows.
    readonly property real travel: Math.max(0, flick.contentHeight - flick.height)
    readonly property real reachSides: Tokens.focus.far
    // In a layout it takes all the room it is given, as iced's scroll does.
    readonly property bool fillWidth: true

    // A scroll never rests past its end, as iced's never does: content that shrinks under it, a
    // glide still heading for where its end was, or an offset set while the content was still
    // being laid out, ends at the last of the content. Without it, a form whose field's list of
    // paths closed as it scrolled to that field rests scrolled past everything it holds.
    function keepInside(): void {
        if (flick.contentY > root.travel || glide.running && gliding.targetValue > root.travel)
            flick.contentY = root.travel;
    }

    onTravelChanged: root.keepInside()
    // On its right it reaches as iced's does not, past content that runs to its edge, so the
    // geometry check is told the other three sides.
    readonly property var probe: ({
            role: "scroll",
            reach: {
                above: root.roomAbove,
                below: root.roomBelow,
                left: root.reachSides
            }
        })

    Flickable {
        id: flick

        x: -root.reachSides
        y: -root.roomAbove
        width: root.width + root.reachSides * 2
        height: root.height + root.roomAbove + root.roomBelow
        contentWidth: width
        contentHeight: body.implicitHeight + root.roomAbove + root.roomBelow
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Behavior on contentY {
            id: gliding

            enabled: !flick.moving && !drag.pressed

            NumberAnimation {
                id: glide

                duration: Tokens.duration.medium
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
                onRunningChanged: {
                    if (!running)
                        root.keepInside();
                }
            }
        }

        Item {
            id: body

            x: root.reachSides
            y: root.roomAbove
            width: root.width
            // Down to the foot of the lowest child shown: `childrenRect` counts hidden ones too,
            // and a hidden one would give the content room to scroll that shows nothing.
            implicitHeight: {
                let foot = 0;
                for (const kid of body.children) {
                    if (kid.visible)
                        foot = Math.max(foot, kid.y + kid.height);
                }
                return foot;
            }
            height: implicitHeight
        }
    }

    // The thumb, along the whole room the scroll shows.
    Rectangle {
        x: root.width - width
        y: flick.y + flick.visibleArea.yPosition * flick.height
        width: Tokens.size.scrollbar
        height: flick.visibleArea.heightRatio * flick.height
        radius: Tokens.radius.full
        visible: root.travel > 0
        color: drag.containsMouse || drag.pressed ? Tokens.color.outline : Tokens.color.outline_variant

        MouseArea {
            id: drag

            property real grabbedAt: 0
            property real fromY: 0

            anchors.fill: parent
            hoverEnabled: true
            onPressed: mouse => {
                grabbedAt = mapToItem(root, mouse.x, mouse.y).y;
                fromY = flick.contentY;
            }
            onPositionChanged: mouse => {
                if (!pressed)
                    return;
                const moved = mapToItem(root, mouse.x, mouse.y).y - grabbedAt;
                flick.contentY = Math.max(0, Math.min(root.travel, fromY + moved / flick.visibleArea.heightRatio));
            }
        }
    }
}
