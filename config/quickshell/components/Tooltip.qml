import QtQuick
import Quickshell
import qs.theme

// Small label that appears under (or above) its parent after a short hover.
// It shows in a popup of its own: drawn as a child, it painted under the
// buttons laid out after its parent, and a panel's clip cut it off. As a
// popup it is above everything on its surface, passes the pointer through,
// stays inside the panel it was opened from (the nearest Drawer) and the
// compositor slides it back onto the screen.
Item {
    id: root

    property string text
    property bool shown: false
    property bool above: false
    readonly property bool visibleNow: shown && delay.done
    // The label's left edge in the parent's coordinates. Worked out as it
    // shows, because a position in the window isn't a binding.
    property real popupX: 0

    function place(): void {
        const owner = root.parent;
        if (!owner)
            return;
        const centred = (owner.width - bubble.width) / 2;
        let panel = null;
        for (let item = owner.parent; item; item = item.parent) {
            if (item.keepsTooltips === true) {
                panel = item;
                break;
            }
        }
        if (!panel) {
            popupX = Math.round(centred);
            return;
        }
        const left = panel.mapToItem(owner, 0, 0).x + Theme.spacing.sm;
        const right = left + panel.width - 2 * Theme.spacing.sm - bubble.width;
        popupX = Math.round(Math.max(left, Math.min(centred, right)));
    }

    onShownChanged: {
        delay.done = false;
        if (shown) {
            place();
            delay.restart();
        } else {
            delay.stop();
        }
    }

    Timer {
        id: delay

        property bool done: false

        interval: 450
        onTriggered: done = true
    }

    PopupWindow {
        anchor.item: root.parent
        anchor.rect.x: root.popupX
        anchor.rect.y: root.above ? -bubble.height - Theme.spacing.sm : (root.parent?.height ?? 0) + Theme.spacing.sm
        anchor.adjustment: PopupAdjustment.Slide | PopupAdjustment.FlipY
        implicitWidth: bubble.width
        implicitHeight: bubble.height
        visible: bubble.opacity > 0
        color: "transparent"
        mask: Region {}

        Rectangle {
            id: bubble

            width: label.implicitWidth + Theme.spacing.md * 2
            height: label.implicitHeight + Theme.spacing.xs * 2 + 2
            radius: Theme.rounding.sm
            color: Theme.m3inverseSurface
            opacity: root.visibleNow ? 1 : 0
            scale: root.visibleNow ? 1 : 0.92
            transformOrigin: root.above ? Item.Bottom : Item.Top
            // A label that changes while it shows (the bar's status) stays
            // centred and inside the panel.
            onWidthChanged: if (root.shown)
                root.place()

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

            StyledText {
                id: label

                anchors.centerIn: parent
                text: root.text
                color: Theme.m3inverseOnSurface
                font.pixelSize: Theme.font.sm
                font.weight: Font.Medium
            }
        }
    }
}
