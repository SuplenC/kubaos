import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// Keep awake and a scheduled action, while either is on: a cup, and the
// action's icon (suspend, power, restart, or a clock) with the time left. Click
// stops Keep awake, or cancels the action when only that is on.
Pill {
    id: root

    visible: Idle.keepAwake || Idle.scheduledPending
    interactive: true
    padding: Idle.scheduledPending ? K.Tokens.space.s12 : K.Tokens.space.s8
    spacing: K.Tokens.space.s4
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

    K.Icon {
        visible: Idle.keepAwake
        name: "coffee"
        size: K.Tokens.icon.s16
        color: K.Tokens.color.tertiary
    }

    K.Icon {
        visible: Idle.scheduledPending
        name: Idle.scheduledIcon
        size: K.Tokens.icon.s16
        color: K.Tokens.color.tertiary
    }

    K.Text {
        visible: Idle.scheduledPending
        text: Idle.countdown(Idle.scheduledLeft)
        style: K.Tokens.type.mono
        color: K.Tokens.color.on_surface
    }
}
