pragma Singleton
import QtQuick

// Stand-in for Session: locked, and every action only logged.
QtObject {
    readonly property bool locked: true

    function lock(): void {
        console.log("session: lock");
    }

    function unlock(): void {
        console.log("session: unlock");
    }

    function suspend(): void {
        console.log("session: suspend");
    }

    function logout(): void {
        console.log("session: logout");
    }

    function reboot(): void {
        console.log("session: reboot");
    }

    function poweroff(): void {
        console.log("session: poweroff");
    }
}
