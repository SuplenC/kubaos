pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// User-facing knobs. Edit these instead of hunting through modules. The ones
// read from `user` can be set in the user layer too:
// ~/.config/kubaos/quickshell/settings.json wins over the defaults here.
Singleton {
    id: root

    readonly property string home: Quickshell.env("HOME")

    // The wallpaper picker's tabs are `dot-theme folders`: the sets picked in
    // ~/.config/kubaos/wallpapers/sets (from this directory), and
    // ~/Pictures/wallpapers.
    readonly property string wallpaperSets: "/usr/share/kubaos/wallpapers"
    // Moving wallpapers (mp4, webm, mkv, mov next to a <name>.poster.jpg):
    // "always", "ac" or "never". On AC by default, because a looping 4K city
    // on battery is how a laptop dies by 3pm. Off, you get the poster frame.
    // This is the first-run default; the control center tile (or
    // `qs ipc call wallpaper animate <mode>`) changes it and remembers it in
    // ~/.local/state/quickshell/kubaos.json.
    readonly property string wallpaperAnimate: "ac"
    // Lock screen loop when the wallpaper itself is a still: a path, or
    // relative to wallpaperSets. "" = the still, blurred, no motion. An animated
    // wallpaper always plays itself on the lock screen.
    readonly property string lockLoop: ""
    // Seconds of no input before the screens go dark (hypridle's DPMS
    // listener, config/hypr/hypridle.conf). Videos stop decoding then.
    readonly property int screenOffAfter: 600

    // Label workspaces with kanji numerals (一 二 三) instead of digits.
    readonly property bool workspaceKanji: false

    // The Command menu's motion: "full", or "fades" for fades alone, with
    // nothing sliding or moving. The user layer's `menuMotion` wins.
    readonly property string menuMotion: ["full", "fades"].includes(user.menuMotion) ? user.menuMotion : "full"

    // The Command menu's hidden and renamed apps, and the terminal for apps
    // with Terminal=true, are Kotodama's settings: config/kotodama/kubaos.toml,
    // and yours in ~/.config/kubaos/kotodama/user.toml.

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

    // The user layer's settings.json: JSON, with whole-line // comments.
    property var user: ({})

    function readUser(text: string): void {
        try {
            const json = text.split("\n").filter(line => !line.trim().startsWith("//")).join("\n");
            root.user = json.trim() === "" ? {} : JSON.parse(json);
        } catch (e) {
            console.warn(`Settings: ignoring unreadable ${userFile.path}: ${e}`);
        }
    }

    FileView {
        id: userFile

        path: `${Quickshell.env("XDG_CONFIG_HOME") || `${root.home}/.config`}/kubaos/quickshell/settings.json`
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.readUser(text())
        onLoadFailed: root.user = {}
    }
}
