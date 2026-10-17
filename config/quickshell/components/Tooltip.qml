import QtQuick
import qs.theme

// Small label that appears under (or above) its parent after a short hover.
// Children are drawn outside their parent's bounds, so it needs no window of
// its own as long as nothing clips it.
Rectangle {
    id: root

    property string text
    property bool shown: false
    property bool above: false
    readonly property bool visibleNow: shown && delay.done

    x: Math.round((parent.width - width) / 2)
    y: above ? -height - Theme.spacing.sm : parent.height + Theme.spacing.sm
    z: 100
    implicitWidth: label.implicitWidth + Theme.spacing.md * 2
    implicitHeight: label.implicitHeight + Theme.spacing.xs * 2 + 2
    radius: Theme.rounding.sm
    color: Theme.m3inverseSurface
    opacity: visibleNow ? 1 : 0
    visible: opacity > 0
    scale: visibleNow ? 1 : 0.92
    transformOrigin: above ? Item.Bottom : Item.Top

    onShownChanged: {
        delay.done = false;
        if (shown)
            delay.restart();
        else
            delay.stop();
    }

    Behavior on opacity {
        Anim {
            duration: Theme.anim.fast
            easing.bezierCurve: Theme.anim.effects
        }
    }

    Behavior on scale {
        Anim {
            duration: Theme.anim.small
            easing.bezierCurve: Theme.anim.spatialFast
        }
    }

    Timer {
        id: delay

        property bool done: false

        interval: 450
        onTriggered: done = true
    }

    StyledText {
        id: label

        anchors.centerIn: parent
        text: root.text
        color: Theme.m3inverseOnSurface
        font.pixelSize: Theme.font.sm
        font.weight: Font.Medium
    }
}
