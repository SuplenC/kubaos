pragma Singleton

import Quickshell

// Lock state and power actions.
Singleton {
    id: root

    // Survives config hot-reloads so editing the shell never drops the lock.
    readonly property bool locked: persisted.locked

    function lock(): void {
        Panels.close();
        persisted.locked = true;
    }

    // Only the lock screen calls this, after PAM succeeded.
    function unlock(): void {
        persisted.locked = false;
    }

    function suspend(): void {
        Quickshell.execDetached(["systemctl", "suspend"]);
    }

    function logout(): void {
        Quickshell.execDetached(["sh", "-c", 'loginctl terminate-session "$XDG_SESSION_ID"']);
    }

    function reboot(): void {
        Quickshell.execDetached(["systemctl", "reboot"]);
    }

    function poweroff(): void {
        Quickshell.execDetached(["systemctl", "poweroff"]);
    }

    PersistentProperties {
        id: persisted

        reloadableId: "session"

        property bool locked: false
    }
}
