import QtQuick
import KubaOS.Components as K
import qs.components
import qs.settings
import qs.services
import "../notifications/feed.js" as Feed

// The bell: a pill with the bell alone, no count, while there is a notification (always, with the
// "Always show the bell" setting), crossed out while Do not disturb is on. A click opens or closes
// the notifications panel. The pointer resting on it a moment asks for the peek (`peeking`), so
// passing over it shows nothing; with nothing unread there is no peek, and a hint says so.
Pill {
    id: root

    // The pointer has rested on the bell long enough for the peek.
    readonly property bool peeking: root.hovered && root.rested
    property bool rested: false

    visible: Feed.bellShown(Notifs.count, Settings.alwaysShowBell)
    interactive: true
    padding: K.Tokens.space.s8
    tooltip: Notifs.unreadCount === 0 ? "No new notifications" : ""
    // The panel takes the peek's place, and closing it doesn't bring the peek back until the
    // pointer rests on the bell again.
    onClicked: {
        root.rested = false;
        rest.stop();
        Panels.toggle("notifications");
    }
    onHoveredChanged: {
        root.rested = false;
        if (root.hovered)
            rest.restart();
        else
            rest.stop();
    }

    // About as long as a glance: the pointer passing over the bell on its way elsewhere is gone
    // before it ends.
    Timer {
        id: rest

        interval: K.Tokens.duration.medium
        onTriggered: root.rested = true
    }

    K.Icon {
        name: Notifs.dnd ? "notifications_off" : "notifications"
        size: K.Tokens.icon.s16
        color: K.Tokens.color.on_surface_variant
    }
}
