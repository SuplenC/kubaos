pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// Lock state and power actions.
Singleton {
    id: root

    // Survives config hot-reloads so editing the shell never drops the lock.
    readonly property bool locked: persisted.locked

    onLockedChanged: lockFile.setText(locked ? "locked\n" : "")

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

    // The lock outlives the shell. A shell that died with the screen locked
    // leaves Hyprland locked with no lock screen; the next one finds this file
    // saying so and locks again as it starts, taking the session lock over
    // (`allow_session_lock_restore`, `config/hypr/looks.lua`). In
    // $XDG_RUNTIME_DIR, so a new login starts unlocked.
    FileView {
        id: lockFile

        path: `${Quickshell.env("XDG_RUNTIME_DIR")}/kubaos-session-lock`
        // Read and written at once: the shell locks before its first frame,
        // and a crash right after locking still finds the file saying so.
        blockLoading: true
        blockWrites: true
        printErrors: false
    }

    // HAZARD: read `text()` here, not in the FileView's `onLoaded`. A blocking
    // load finishes while the FileView is built, before its handlers are
    // connected, so `loaded` reaches nobody.
    Component.onCompleted: {
        if (lockFile.text().trim() === "locked")
            persisted.locked = true;
    }
}
