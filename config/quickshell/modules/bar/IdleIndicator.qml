import QtQuick
import qs.theme
import qs.components
import qs.services

// Keep awake and a scheduled lock, while either is on: a cup, and a lock with
// the time left. Click stops Keep awake, or cancels the lock when only that
// is on. Theme tokens only; the bar's look is about to change.
Pill {
    id: root

    visible: Idle.keepAwake || Idle.lockPending
    interactive: true
    padding: Theme.spacing.sm
    spacing: Theme.spacing.xs
    tooltip: {
        const parts = [];
        if (Idle.keepAwake)
            parts.push(Idle.keepAwakeUntil === 0 ? "Keeping awake until you turn it off" : `Keeping awake until ${Qt.formatDateTime(new Date(Idle.keepAwakeUntil * 1000), Time.timeFormat)}`);
        if (Idle.lockPending)
            parts.push(`Locking at ${Qt.formatDateTime(new Date(Idle.lockAt * 1000), Time.timeFormat)}`);
        parts.push(Idle.keepAwake ? "Click to stop keeping awake" : "Click to cancel the lock");
        return parts.join(". ");
    }
    onClicked: Idle.keepAwake ? Idle.stopKeepAwake() : Idle.cancelLock()

    Icon {
        anchors.verticalCenter: parent.verticalCenter
        visible: Idle.keepAwake
        name: "coffee"
        size: Theme.bar.icon
        color: Theme.m3tertiary
    }

    Icon {
        anchors.verticalCenter: parent.verticalCenter
        visible: Idle.lockPending
        name: "lock_clock"
        size: Theme.bar.icon
        color: Theme.m3tertiary
    }

    StyledText {
        anchors.verticalCenter: parent.verticalCenter
        visible: Idle.lockPending
        text: Idle.countdown(Idle.lockLeft)
        font.features: Theme.font.tabular
    }
}
