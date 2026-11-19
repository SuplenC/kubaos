pragma Singleton

import Quickshell

// Which overlay panel is open (controlcenter, notifications, powermenu, wallpaper, clock,
// traymenu, sysmon, counters, or "") and on which screen. Only one panel is open at a time.
Singleton {
    id: root

    property string current: ""
    property ShellScreen screen: Hypr.focusedScreen

    // A screen unplugged takes its open panel with it: Quickshell deletes the screen's object,
    // which leaves `screen` null and the panel drawn nowhere, and the next toggle would close it
    // rather than open it.
    onScreenChanged: {
        if (!root.screen)
            root.close();
    }

    function open(name: string): void {
        if (Session.locked)
            return;
        root.screen = Hypr.focusedScreen;
        root.current = name;
    }

    function close(): void {
        root.current = "";
    }

    function toggle(name: string): void {
        if (root.current === name)
            close();
        else
            open(name);
    }
}
