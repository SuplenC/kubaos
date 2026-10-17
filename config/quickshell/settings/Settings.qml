pragma Singleton

import QtQuick
import Quickshell

// User-facing knobs. Edit these instead of hunting through modules.
Singleton {
    readonly property string home: Quickshell.env("HOME")

    // Wallpaper picker: one tab per folder under wallpaperRoot.
    readonly property string wallpaperRoot: `${home}/Pictures/wallpapers`
    readonly property list<string> wallpaperFolders: ["shinjuku", "japanese", "cyberpunk"]
    // ukiyo's live scenes (docs/UKIYO.md) are listed first in this folder,
    // the same one dot-theme steps through them in (SCENE_FOLDER).
    readonly property string sceneFolder: "shinjuku"
    // Moving wallpapers (mp4, webm, mkv, mov next to a <name>.poster.jpg):
    // "always", "ac" or "never". On AC by default, because a looping 4K city
    // on battery is how a laptop dies by 3pm. Off, you get the poster frame.
    // This is the first-run default; the control center tile (or
    // `qs ipc call wallpaper animate <mode>`) changes it and remembers it in
    // ~/.local/state/quickshell/kubaos.json.
    readonly property string wallpaperAnimate: "ac"
    // Lock screen loop when the wallpaper itself is a still, relative to
    // wallpaperRoot. "" = the still, blurred, no motion. An animated
    // wallpaper always plays itself on the lock screen.
    readonly property string lockLoop: ""
    // Seconds of no input before the screens go dark (hypridle's DPMS
    // listener, config/hypr/hypridle.conf). Videos stop decoding then.
    readonly property int screenOffAfter: 600

    // Hyprland workspaces are grouped per monitor: 1-3, 4-6, 7-9.
    readonly property int workspacesPerMonitor: 3
    // Label workspaces with kanji numerals (一 二 三) instead of digits.
    readonly property bool workspaceKanji: false

    // Launcher: desktop entry ids (file name without .desktop) to leave out, on
    // top of the entries that set NoDisplay or Hidden themselves.
    readonly property list<string> launcherHidden: ["qt5ct", "qt6ct", "org.codeberg.dnkl.footclient", "org.codeberg.dnkl.foot-server"]
    // Launcher: display names by desktop entry id, for entries with clumsy names.
    readonly property var launcherNames: ({
            nvim: "Neovim"
        })

    // Terminal used for desktop entries with Terminal=true.
    readonly property list<string> terminal: ["ghostty", "-e"]

    // Lock screen: a FIDO2 security key (qs-lock-u2f, pam_u2f) next to the password.
    // Only runs when /etc/u2f_mappings has a key for you (dot-u2f-enroll). Wants
    // user verification: an enrolled finger on the YubiKey C Bio, no PIN.
    readonly property bool u2f: true
    // Lock screen: an fprintd fingerprint reader (qs-lock-fingerprint). Off since the
    // security key replaced the USB reader; turn on if you use a reader again.
    readonly property bool fingerprint: false

    // In-shell polkit agent. Off by default: hyprpolkitagent stays the agent
    // until the Quickshell one is verified on the machine (see README, PAM and polkit).
    // Flip together with POLKIT_AGENT in config/hypr/autostart.lua ("quickshell").
    readonly property bool polkitAgent: false

    // Popup lifetime when the sender does not set one, in milliseconds.
    readonly property int notificationTimeout: 6000
}
