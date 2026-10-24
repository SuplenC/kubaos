import QtQuick
import Quickshell
import Quickshell.Services.Notifications
import qs.theme
import qs.components
import qs.services

// Critical notifications while an open panel covers the popups' corner: a
// stack out of the frame's right edge, just under that panel, so an urgent
// one is seen without drawing over the panel. With no room under the panel
// (the control center and the wallpaper picker reach the bottom edge), they
// wait with the rest for the corner.
Drawer {
    id: root

    readonly property int padding: Theme.spacing.sm
    // A card with its icon, and the padding around it.
    readonly property int oneCard: 40 + Theme.spacing.md * 2 + padding * 2
    // The open panel covering the corner, or null.
    property Drawer cover: null
    // Where that panel ends. It is kept once the panel closes, so the stack
    // retracts in place instead of jumping up into the corner.
    property real coverBottom: 0
    readonly property var critical: Notifs.popups.filter(n => n.urgency === NotificationUrgency.Critical)
    readonly property real room: frame.b - Theme.frame.flare - start
    readonly property int drawn: list.drawn

    edge: "right"
    start: coverBottom + Theme.frame.flare + Theme.spacing.md
    shown: cover !== null && critical.length > 0 && room >= oneCard
    takesFocus: false
    contentWidth: 380
    contentHeight: Math.min(Math.max(list.contentHeight, 1) + padding * 2, room)

    Binding {
        target: root
        property: "coverBottom"
        value: root.cover ? root.cover.area.y + root.cover.area.height : 0
        when: root.cover !== null
        restoreMode: Binding.RestoreNone
    }

    ToastList {
        id: list

        x: root.padding
        y: root.padding
        width: root.contentWidth - root.padding * 2
        drawer: root
        notifications: root.shown || root.active ? root.critical.slice(0, 5) : []
        // The corner shows these cards as soon as the panel has gone, so
        // they leave here at once rather than fade beside their copies.
        opacity: root.cover ? 1 : 0
    }
}
