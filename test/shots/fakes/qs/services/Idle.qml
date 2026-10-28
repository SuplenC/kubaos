pragma Singleton
import QtQuick

// Stand-in for Idle: keeping awake, nothing scheduled.
QtObject {
    readonly property string runtime: "/nonexistent/run"
    readonly property int now: 1793137652
    readonly property int keepAwakeUntil: 0
    readonly property bool keepAwake: true
    readonly property string scheduled: "lock"
    readonly property int scheduledAt: 0
    readonly property bool scheduledPending: false
    readonly property int scheduledLeft: 0
    readonly property string scheduledIcon: "bedtime"
    readonly property string scheduledNoun: "Lock"

    function countdown(seconds: int): string {
        return "12:34";
    }

    function stopKeepAwake(): void {
    }

    function cancelScheduled(): void {
    }

    function parse(text: string): var {
        return null;
    }
}
