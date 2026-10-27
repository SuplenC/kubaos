import QtQuick
import qs.theme

// Horizontal slider for values in 0..1, a filled pill with the icon inside.
// It only reports user input through moved(value), so `value` can stay bound
// to a live service property. Keys: Left/Right or h/l step 5%.
//
// With `stops` (two or more), it snaps: the fill ends at one of the stops,
// a dot marks each stop ahead of it, and `labels` names the stop in the
// track. A drag shows the stop under the pointer and picks it on release;
// a click, a key (Left/Right, h/l) or a wheel notch picks at once. Each pick
// is reported through stepped(index), and `index` stays bound to the
// service's.
Item {
    id: root

    property real value: 0
    property string icon
    property int stops: 0
    property int index: 0
    property var labels: []

    readonly property bool stepping: stops > 1
    // The last stop's index.
    readonly property int last: stops - 1
    // The stop the fill shows: the one under the pointer while dragging.
    readonly property int shownIndex: drag.pressed && drag.dragIndex >= 0 ? drag.dragIndex : index
    // The fill's share of the track, from the circle around the icon (0) to
    // the whole track (1).
    readonly property real shown: stepping ? shownIndex / last : Math.min(1, value)
    // The fill's width once it settles, which the label's place follows from
    // the start of a glide, so the label never jumps halfway.
    readonly property real fillTarget: stepping ? Math.round(height + (width - height) * shown) : Math.max(height, width * shown)
    // A stop's dot.
    readonly property int tickSize: Theme.spacing.xs

    signal moved(real value)
    signal stepped(int index)

    implicitHeight: 36
    activeFocusOnTab: true

    function request(v: real): void {
        moved(Math.max(0, Math.min(1, v)));
    }

    function step(to: int): void {
        const clamped = Math.max(0, Math.min(last, to));
        if (clamped !== index)
            stepped(clamped);
    }

    // The stop nearest the pointer at `x`, along the travel of the fill's end.
    function stopAt(x: real): int {
        const travel = width - height;
        return travel > 0 ? Math.round((x - height / 2) / travel * last) : index;
    }

    Keys.onPressed: event => {
        const back = event.key === Qt.Key_Left || event.key === Qt.Key_H;
        const forward = event.key === Qt.Key_Right || event.key === Qt.Key_L;
        if (!back && !forward)
            return;
        if (stepping)
            step(index + (forward ? 1 : -1));
        else
            request(value + (forward ? 0.05 : -0.05));
        event.accepted = true;
    }

    Rectangle {
        id: track

        anchors.fill: parent
        radius: height / 2
        color: Theme.m3surfaceContainerHighest

        // A dot at each stop the fill hasn't reached, centred where the handle
        // stands at that stop.
        Repeater {
            model: root.stepping ? root.stops : 0

            Rectangle {
                required property int index

                x: Math.round(track.height + (track.width - track.height) * index / root.last - Theme.spacing.sm - (handle.width + root.tickSize) / 2)
                anchors.verticalCenter: parent.verticalCenter
                width: root.tickSize
                height: root.tickSize
                radius: root.tickSize / 2
                color: Theme.m3onSurfaceVariant
                visible: index > root.shownIndex
            }
        }

        Rectangle {
            id: fill

            height: parent.height
            width: root.fillTarget
            radius: height / 2
            color: Theme.m3primary

            Behavior on width {
                // A continuous drag follows the pointer; stops always glide.
                enabled: root.stepping || !drag.pressed

                Anim {
                    duration: Theme.anim.small
                    easing.bezierCurve: Theme.anim.emphasizedDecel
                }
            }

            // Handle: a short bar near the end of the fill.
            Rectangle {
                id: handle

                anchors.right: parent.right
                anchors.rightMargin: Theme.spacing.sm
                anchors.verticalCenter: parent.verticalCenter
                width: 3
                height: parent.height * (drag.pressed ? 0.6 : 0.4)
                radius: 2
                color: Theme.m3onPrimary
                opacity: fill.width > fill.height * 1.6 ? 0.8 : 0

                Behavior on height {
                    Anim {
                        duration: Theme.anim.fast
                    }
                }
            }
        }

        Icon {
            anchors.left: parent.left
            anchors.leftMargin: (parent.height - width) / 2
            anchors.verticalCenter: parent.verticalCenter
            name: root.icon
            size: 20
            color: Theme.m3onPrimary
            filled: true
        }

        // The stop's name: in the empty track after the fill while it fits
        // there, else inside the fill before the handle, so it never crosses
        // the fill's edge.
        StyledText {
            id: label

            readonly property bool inside: track.width - root.fillTarget < implicitWidth + Theme.spacing.lg * 2

            visible: root.stepping && text !== ""
            // The track's height, centred by the text's own alignment.
            height: parent.height
            // Whole pixels, so the text stays crisp while it glides with the fill.
            x: Math.round(inside ? fill.width - Theme.spacing.sm - handle.width - Theme.spacing.md - width : track.width - Theme.spacing.lg - width)
            text: root.labels[root.shownIndex] ?? ""
            color: inside ? Theme.m3onPrimary : Theme.m3onSurfaceVariant
        }

        FocusRing {
            shown: root.activeFocus
        }
    }

    MouseArea {
        id: drag

        // The stop under the pointer during a press, -1 when none.
        property int dragIndex: -1
        // The wheel's turn not yet a whole notch: a touchpad sends it in pieces.
        property real wheelTurn: 0

        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onPressed: mouse => {
            if (root.stepping)
                dragIndex = Math.max(0, Math.min(root.last, root.stopAt(mouse.x)));
            else
                root.request(mouse.x / width);
        }
        onPositionChanged: mouse => {
            if (root.stepping)
                dragIndex = Math.max(0, Math.min(root.last, root.stopAt(mouse.x)));
            else
                root.request(mouse.x / width);
        }
        onReleased: {
            if (root.stepping && dragIndex >= 0)
                root.step(dragIndex);
            dragIndex = -1;
        }
        onCanceled: dragIndex = -1
        onWheel: wheel => {
            if (!root.stepping) {
                root.request(root.value + (wheel.angleDelta.y > 0 ? 0.05 : -0.05));
                return;
            }
            // 120 is one notch of a wheel (Qt's QWheelEvent::angleDelta).
            wheelTurn += wheel.angleDelta.y;
            const notches = Math.trunc(wheelTurn / 120);
            if (notches !== 0) {
                wheelTurn -= notches * 120;
                root.step(root.index + notches);
            }
        }
    }
}
