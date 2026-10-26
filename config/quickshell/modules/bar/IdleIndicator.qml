import QtQuick
import qs.theme
import qs.components
import qs.services

// Keep awake and a scheduled action, while either is on: a cup, and the
// action's icon (lock, suspend, power, restart) with the time left. Click
// stops Keep awake, or cancels the action when only that is on. Theme tokens
// only; the bar's look is about to change.
Pill {
    id: root

    visible: Idle.keepAwake || Idle.scheduledPending
    interactive: true
    padding: Theme.spacing.sm
    spacing: Theme.spacing.xs
    tooltip: {
        const parts = [];
        if (Idle.keepAwake)
            parts.push(Idle.keepAwakeUntil === 0 ? "Keeping awake until you turn it off" : `Keeping awake until ${Qt.formatDateTime(new Date(Idle.keepAwakeUntil * 1000), Time.timeFormat)}`);
        if (Idle.scheduledPending)
            parts.push(`${Idle.scheduledNoun} at ${Qt.formatDateTime(new Date(Idle.scheduledAt * 1000), Time.timeFormat)}`);
        parts.push(Idle.keepAwake ? "Click to stop keeping awake" : "Click to cancel it");
        return parts.join(". ");
    }
    onClicked: Idle.keepAwake ? Idle.stopKeepAwake() : Idle.cancelScheduled()

    Icon {
        anchors.verticalCenter: parent.verticalCenter
        visible: Idle.keepAwake
        name: "coffee"
        size: Theme.bar.icon
        color: Theme.m3tertiary
    }

    Icon {
        anchors.verticalCenter: parent.verticalCenter
        visible: Idle.scheduledPending
        name: Idle.scheduledIcon
        size: Theme.bar.icon
        color: Theme.m3tertiary
    }

    StyledText {
        anchors.verticalCenter: parent.verticalCenter
        visible: Idle.scheduledPending
        text: Idle.countdown(Idle.scheduledLeft)
        font.features: Theme.font.tabular
    }
}
