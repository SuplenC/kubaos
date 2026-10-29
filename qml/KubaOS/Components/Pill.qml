import QtQuick
import KubaOS.Components as K

// A group in the bar: a pill as tall as a small control, on the surface container, its children
// side by side and centred on device pixels. With `interactive` it lays the state layer over
// itself under the pointer and sends clicked(), rightClicked() and scrolled(steps); it is never a
// stop for Tab, so a panel's keyboard focus never wanders into the bar. `tooltip` shows in a hint
// under it, kept inside the window.
Item {
    id: root

    default property alias content: row.data
    // Room at each end, between the pill's curve and its content.
    property int padding: K.Tokens.space.s12
    property alias spacing: row.gap
    property bool interactive: false
    property color color: K.Tokens.color.surface_container
    // The content's colour, which the state layer lays over the fill.
    property color ink: K.Tokens.color.on_surface
    property string tooltip
    readonly property bool hovered: pointer.containsMouse
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // How far the hint moves sideways from under the pill to stay in the window, worked out as it
    // shows, since a place in the window isn't a binding.
    property real hintShift: 0
    // The words the hint shows: `tooltip` while the pointer is over the pill, kept as it fades
    // out. Bound to `tooltip` itself, a hint that changes every sample (the system monitor's)
    // would lay its words out every sample unseen.
    property string hintText

    signal clicked
    signal rightClicked
    signal scrolled(int steps)

    // Places the hint under the pill, or as near as keeps it whole inside the window, `space.s8`
    // from its edge.
    function placeHint(): void {
        root.hintText = root.tooltip;
        const window = root.Window.window;
        if (!window) {
            root.hintShift = 0;
            return;
        }
        const half = K.Look.snapUp(hintWidth.implicitWidth + K.Tokens.space.s8 * 2, root.dpr) / 2;
        const centre = root.mapToItem(null, root.width / 2, 0).x;
        const lowest = K.Tokens.space.s8 + half;
        const highest = window.width - K.Tokens.space.s8 - half;
        root.hintShift = K.Look.snap(Math.max(lowest, Math.min(highest, centre)) - centre, root.dpr);
    }

    implicitWidth: K.Look.snapUp(row.implicitWidth + root.padding * 2, root.dpr)
    implicitHeight: K.Tokens.size.control_sm
    onHoveredChanged: {
        if (root.hovered)
            root.placeHint();
    }
    // A hint that changes while it shows (the status pill's) stays inside the window.
    onTooltipChanged: {
        if (root.hovered)
            root.placeHint();
    }

    // New content settles into the pill's new width, as a list reordering does.
    Behavior on implicitWidth {
        NumberAnimation {
            duration: K.Tokens.duration.medium
            easing.type: Easing.BezierSpline
            easing.bezierCurve: K.Tokens.easing.standard_decel
        }
    }

    Rectangle {
        anchors.fill: parent
        radius: K.Tokens.radius.full
        color: root.color

        K.StateLayer {
            radius: parent.radius
            content: root.ink
            active: root.interactive
            hovered: root.hovered
            pressed: pointer.pressed
        }
    }

    // While the pill's width moves to fit new content, the content is cut at the pill's edge
    // rather than drawn over its neighbours.
    Item {
        anchors.fill: parent
        clip: true

        K.Row {
            id: row

            x: K.Look.snap((root.width - row.implicitWidth) / 2, root.dpr)
            y: K.Look.snap((root.height - row.implicitHeight) / 2, root.dpr)
            gap: K.Tokens.space.s8
            align: "centre"
        }
    }

    MouseArea {
        id: pointer

        anchors.fill: parent
        enabled: root.interactive
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: mouse => mouse.button === Qt.RightButton ? root.rightClicked() : root.clicked()
        onWheel: wheel => root.scrolled(wheel.angleDelta.y > 0 ? 1 : -1)
    }

    // The hint's words, measured and never drawn, so the hint can be kept inside the window.
    K.Text {
        id: hintWidth

        visible: false
        text: root.hintText
        style: K.Tokens.type.label
    }

    Item {
        x: root.hintShift
        width: root.width
        height: root.height

        K.Hint {
            text: root.hintText
            shown: root.tooltip !== "" && root.hovered
        }
    }
}
