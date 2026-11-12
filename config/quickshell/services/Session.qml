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

    // Only the lock screen calls this, after PAM succeeded. A security key's
    // touch is no keyboard or mouse input, so neither hypridle's resume nor
    // Hyprland lights screens its idle timer turned off: light them here,
    // the way hypridle's resume does, lid sync after so a failing script
    // leaves them on.
    function unlock(): void {
        persisted.locked = false;
        Quickshell.execDetached(["sh", "-c", `hyprctl dispatch 'hl.dsp.dpms({ action = "enable" })'; exec dot-hypr-lid sync`]);
    }

    function suspend(): void {
        Quickshell.execDetached(["systemctl", "suspend"]);
    }

    // `dot-session leave` stops the session's services first, while Hyprland
    // still runs, so they don't hold the shutdown.
    function logout(): void {
        Quickshell.execDetached(["dot-session", "leave", "logout"]);
    }

    function reboot(): void {
        Quickshell.execDetached(["dot-session", "leave", "reboot"]);
    }

    function poweroff(): void {
        Quickshell.execDetached(["dot-session", "leave", "poweroff"]);
    }

    PersistentProperties {
        id: persisted

        reloadableId: "session"

        property bool locked: false
    }
}
