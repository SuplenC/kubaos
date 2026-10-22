import QtQuick
import Quickshell
import Kotodama
import qs.theme
import qs.services

// The Command menu: Kotodama's menu window in the shell's matugen colors.
// `kotodama toggle` (SUPER + Space, config/hypr/fallback.lua) opens it through
// the Daemon, so it has no IPC target of its own. Hyprland blurs what shows
// through its translucent panel (the `kotodama` layer, config/hypr/rules.lua).
KotodamaMenu {
    id: root

    // It opens on the focused screen, and stays there while it is open, even
    // when the pointer crosses to another screen and takes the focus with it.
    Binding on screen {
        value: Hypr.focusedScreen
        when: !root.visible
        restoreMode: Binding.RestoreNone
    }

    background: Theme.alpha(Theme.m3surface, Theme.frame.opacity)
    foreground: Theme.m3onSurface
    dimmed: Theme.m3onSurfaceVariant
    accent: Theme.m3primary
    selection: Theme.m3secondaryContainer
    border: Theme.m3outlineVariant
    error: Theme.m3error
    fontFamily: Theme.font.sans
    fontSize: Theme.font.lg
    radius: Theme.rounding.xl
    menuWidth: 560
    rowHeight: 46
    maxRows: 7
    placeholder: "Search apps, projects, windows, or type a sum"
}
