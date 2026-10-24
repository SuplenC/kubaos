pragma Singleton

import Quickshell

// Which overlay panel is open (controlcenter, powermenu, wallpaper, clock,
// or "") and on which screen. Only one panel is open at a time.
Singleton {
    id: root

    property string current: ""
    property ShellScreen screen: Hypr.focusedScreen

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
