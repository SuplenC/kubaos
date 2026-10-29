# Using and hacking on KubaOS

Keys, updates, daily commands, how it is built, the repo map and known limitations. The front page is [`README.md`](../README.md).

## Keys that matter

Every bind is in the Command menu, found by what it does: type "move to workspace 3" and the bind shows with its keys beside it. `SUPER + F1` lists them all. These are the ones you'll use every day:

| Keys | What |
|---|---|
| `SUPER + Return` | Terminal (Ghostty). It opens with the クバ logo and a few lines of system facts; "Terminal greeting off" in the Command menu, `dot-greeting off` or `KUBAOS_GREETING=0` turns that off |
| `SUPER + CTRL + Return` | Terminal in tmux session `main` |
| `SUPER + Space` | Command menu (Kotodama): apps, projects, scripts, windows, binds, KubaOS's actions and the system toggles ("wifi", "volume 40", "next wallpaper"). Type a sum to calculate. It opens on the focused screen |
| `SUPER + SHIFT + F` | Files (Nautilus) in your home folder |
| `SUPER + CTRL + F` | yazi in your home folder, in Ghostty |
| `SUPER + Q` | Close window |
| `SUPER + h/j/k/l` | Focus the window left, below, above, right. `+ SHIFT` moves the window, `+ CTRL` resizes it |
| `SUPER + ←/↓/↑/→` | Focus the screen that way |
| `SUPER + 1..0` | Workspace 1-10 of the focused screen, or of the waiting group it shows. `+ SHIFT` takes the window along, `+ SHIFT + ALT` sends it without following |
| `SUPER + SHIFT + CTRL + →` | Move the window to a new workspace after the group's last |
| `SUPER + Tab` / `SUPER + CTRL + →/←` | Next / previous workspace on the focused screen, through the groups waiting there, as the three-finger swipe does (`+ SHIFT` on Tab for previous); `SUPER + CTRL + Tab` its former one |
| `SUPER + CTRL + ALT + →/←` | First workspace of the next / previous group on the focused screen |
| `SUPER + ALT + Tab` | Next window in a group (`+ SHIFT` for previous) |
| `SUPER + CTRL + A` / `B` / `W` | Control center: audio, Bluetooth, Wi-Fi |
| `SUPER + N` | Notifications |
| `SUPER + CTRL + Space` | Wallpaper picker (the colors follow) |
| `SUPER + SHIFT + Space` | Next keyboard layout, with two or more |
| `Print` / `SUPER + SHIFT + S` | Screenshot a region, copied and saved; the screen freezes when you press the key, so menus and hover states stay as they were |
| `SUPER + SHIFT + R` | Record a region, same key stops |
| `SUPER + Escape` | Power menu |
| `SUPER + CTRL + Escape` | Lock |
| `SUPER + ALT + Return` / `SUPER + ALT + Space` | foot and fuzzel: the emergency terminal and app menu, for when the shell is dead |

The layouts are yours, the first the default: the ones you picked in Kōbō to begin with, then whatever you change them to (`layouts` in `~/.config/kubaos/desktop/keyboard/keyboard.toml`; `us` if there is nothing at all). The console and the disk passphrase prompt type with the default, from `/etc/default/keyboard`. With two or more, the bar shows the one in use ("US", "IT"); click it, press `SUPER + SHIFT + Space` or pick "Next keyboard layout" in the Command menu for the next. The control center's Keyboard tile switches too, and its arrow opens the Keyboard tab: make a layout the default, move it up or down, remove it, or "Add a language…" from every layout and variant by name. That search is the Command menu's, so a typo or a missing accent still finds the layout ("czeh" finds Czech): a click picks one and the Add button under the list adds it, Enter adds the selected one at once. "Keyboard layouts" in the Command menu opens that tab. The Command menu has each layout ("Keyboard: Italian") to switch to it, make it the default, move it or remove it, plus "Keyboard: add a layout", "Keyboard: set layouts" and the "Add a keyboard layout" app (type `layout`, then Enter) that searches every layout and variant you don't type with yet and adds the one you pick. `kubaos keyboard show|set|add|remove|default|move` does the same from a terminal and reloads Hyprland. Only a new default asks for anything: it goes to the console and the disk prompt through a root helper that pkexec runs, which needs no password for an admin at the machine and may ask anyone else for one. If that's refused, your layouts change anyway and the tab says the console keeps the old default. Number keys are bound by keycode, so they're the top-row keys whatever the layout says.

Accents work as Option does on a Mac, whatever the layout: Left Alt + e, then a letter, types it with an acute (é), Left Alt + ` a grave (à), Left Alt + u a diaeresis (ö), Left Alt + i a circumflex (ô) and Left Alt + n a tilde (ñ). Shift with the letter gives the capital, and Space the mark alone. Let go of Alt before the letter. Every other Left Alt chord is plain Alt, ALT + Tab included, and Right Alt is untouched. The Shortcuts View (`SUPER + F1`) lists the chords. Turn them off with "Mac accents on Left Alt: off" in the Command menu or the control center's Keyboard tab (`mac_accents = false` in `~/.config/kubaos/desktop/keyboard/keyboard.toml`).

Workspaces follow where a screen stands, not which screen it is. The laptop panel has its group, and the externals are counted from the left: whatever screen is leftmost shows the first external group, the next one the second. Two screens beside the laptop at work and two others at home show the same groups, so unplug at work, plug in at home and the windows are where they were, left and right. Fewer screens than groups, and the extra groups wait on the laptop, tinted in the bar, until a screen takes their place again; a new place on the desk gets a new group. The order comes from where Hyprland places the screens, so put them in the order they stand (`config/hypr/README.md`, "Your screens"). Type "workspace group" in the Command menu to see which group sits where, and on which screen.

Or pin a screen to a group, and it shows that group whatever port it's on, wherever Hyprland places it and whatever order the screens come up in: "Pin <screen> to group N" in the Command menu, for the screen you're looking at, or `~/.config/kubaos/desktop/workspaces/groups.toml` ("Open workspace groups settings"). Pin the BenQ at home and the left Dell at work to group 1, the LG and the centre Dell to group 2, and each desk's left and centre screens get the same groups every time. When two screens of a group are plugged in together, the first listed wins; screens without a pin take the groups left over, left to right. "Name group N…" gives a group a name the menu shows, and the bar too while the group waits on another screen. The laptop panel can be pinned like any screen: open, it shows the group while no screen listed before it is plugged in, and its own 1-10 waits beside it; closed, it holds no group. A mistake in the file is a notification, and until it's fixed no screen is pinned. `config/hypr/README.md`, "Workspace groups", has the rest.

## Files

The file manager is Files (GNOME's Nautilus), with [yazi](https://yazi-rs.github.io/) in Ghostty a key away for when the mouse is too far. Files opens from:

- `SUPER + SHIFT + F`, or "Files" in the Command menu: your home folder.
- "Files: Downloads", "Files: Documents" and the rest in the Command menu: each XDG user folder that exists (the ones in `~/.config/user-dirs.dirs`, or the English names without it). The Item's second Action opens the folder in yazi instead.
- "Recent files" in the Command menu: what you opened lately.
- `xdg-open <folder>`, and every app's "Open folder". Files is KubaOS's default for folders, in `~/.local/share/applications/mimeapps.list`. Set another with `xdg-mime default <app>.desktop inode/directory`, which writes `~/.config/mimeapps.list`, and that one wins.
- An app's "Show in folder" (Chrome's downloads, VS Code, Telegram): Files opens the folder with the file selected.

In Files:

- `Space` previews the selected file (Sushi): photos, videos, PDFs, text, fonts. `Space` again closes it.
- Double-clicking an archive (zip, 7z, RAR, tar) extracts it next to itself. "Open With" → File Roller looks inside first.
- Select several files and press `F2` to rename them in one go: a pattern with a running number, or find and replace in the names.
- Right-click: "Open in Ghostty" on a folder or the background; "Copy Path" puts the paths of the selection on the clipboard; on images, "Convert Image" to JPG, PNG or WebP and "Resize Image" to half, a quarter or at most 1920 px, each a new file next to the original; on a folder, "Open as Administrator" opens it at `admin://` after a password prompt, for editing what belongs to root; on a camera card's or phone's folder with a `DCIM` folder, "Import Photos" (below).
- `Ctrl + H` shows hidden files, `Ctrl + L` takes a path, `Ctrl + 1` / `Ctrl + 2` switch list and grid.
- Thumbnails cover images (HEIC and AVIF too), videos, PDFs and fonts, on network shares and phones as well. Files picks up the wallpaper's colours when it starts.

Images open in Loupe, videos in Showtime, PDFs in Papers.

yazi opens from `SUPER + CTRL + F` or "Yazi" in the Command menu (your home folder), "Open folder in yazi…" (`open-folder` then a path, any folder, or a file, which opens in its folder with the cursor on it), "Open With" → Yazi in Files, or `y` in a shell: yazi in that terminal, and when you quit it, the shell `cd`s to where you ended up. Plain `yazi` leaves the shell where it was. `kubaos-files open [PATH|URI]...` does the same from a terminal: paths, `~`, and `file://` URIs, several at once as tabs.

### Drives and photos

A USB stick or SD card mounts when you plug it in, and a notification offers Open and Eject. Internal disks and drives that are already mounted are left alone; an encrypted drive waits in the Files sidebar, which asks for its passphrase when you click it. "Eject STICK" in the Command menu (type "eject") unmounts a drive and powers it off, so it can be pulled; if a file on it is still open, the notification says so and the drive stays. "Don't mount drives when plugged in" in the Command menu (or `kubaos-automount off`) leaves new drives in the Files sidebar until you click them; "Mount drives when plugged in" turns it back on.

When the drive has a `DCIM` folder (a camera's card, a phone), the notification also offers Import photos: the photos and videos are copied into `~/Pictures/<year>/<month>/` by the date they were taken (from EXIF, else the file's date), and the notification says how many. A file already imported, the same name and size in that month, is skipped, so importing the same card twice copies only what's new. Nothing on the card is ever deleted or changed. A phone over USB shows up in the Files sidebar instead of mounting; right-click its `DCIM` folder for "Import Photos". `kubaos-automount import <folder>` does it from a terminal.

The service is the `kubaos-automount` user unit: `systemctl --user status kubaos-automount`, logs in `journalctl --user -u kubaos-automount`.

### Tidy Downloads

"Tidy Downloads" in the Command menu sorts `~/Downloads` into `Images`, `Documents`, `Archives`, `Installers`, `Video` and `Audio`, by file extension. Files that arrived in the last day stay where they are, as do folders, hidden files and anything it doesn't recognise; a name already taken gets ` (1)`. "Tidy Downloads daily" does it once a day (a user timer, `kubaos-tidy-downloads.timer`); it is off until you turn it on, and "Stop tidying Downloads daily" turns it off again. From a terminal: `kubaos-files tidy`, `kubaos-files tidy on|off`.

| Key in yazi | What |
|---|---|
| `Enter` / `l`, `h` | Open, go up |
| `Space` | Select; `v` for a visual selection |
| `y`, `x`, `p` | Copy, cut, paste |
| `d` / `D` | Trash / delete for good |
| `a`, `r` | Create (a trailing `/` makes a folder), rename |
| `/`, `s` / `S` | Find in the folder, search names (fd) / contents (ripgrep) |
| `z` / `Z` | Jump with fzf / zoxide |
| `Ctrl + N` | Drag the selected files, or the hovered one, out of yazi (ripdrag): into Chrome, an upload form, anything that takes a drop |
| `~` or `F1` | Every key |

Image, video and PDF previews show in the right pane: yazi draws them with the kitty graphics protocol, which Ghostty speaks. Inside tmux they pass through as well (`allow-passthrough` in `config/tmux/tmux.conf`).

The drag key lives in your own `~/.config/kubaos/tools/yazi/keymap.toml`, which KubaOS seeds with it: change it or delete it there. yazi reads a single keymap file, so KubaOS can't ship keys of its own beside yours. If your file had its own keys before the drag key existed, the update left it alone; add this yourself:

```toml
[[mgr.prepend_keymap]]
on = "<C-n>"
run = 'shell --orphan -- ripdrag --and-exit --all --no-click "$@"'
desc = "Drag the selected files out"
```

## Notifications

A new notification pops up in the top right corner for a few seconds. While the pointer is on the popups, they all stay, their clocks stopped and a new one waiting; they go on with the time they had left when it leaves. Everything lands in the notifications panel too, out of the right edge (`SUPER + N`, the bell in the bar, or "Notifications" in the Command menu): New on top, what you've seen under Earlier, several from one app stacked into one card that a click or → opens. A card shows two lines of its text; when there is more, a chevron sits by its ×, and a click, Enter or Space unfolds it to the whole text (another click, Esc, or unfolding another card folds it again). Clicking a card never runs anything: its main action is the filled button under the text, its other actions sit beside it, then Mark read while it is unread. Delete dismisses it, Clear all empties the panel. Popups and the peek unfold the same way, by a click. The bell shows only while there is something in there ("Always show the bell" in the Command menu keeps it), and resting the pointer on it shows the three most recent unread ones in the corner, with Mark read and Clear.

Do not disturb holds the popups back, all but critical ones (a battery about to die); notifications still land in the panel, and the bell stays in the bar, crossed out, even with none. Turn it on with the moon in the panel, the control center's tile, `SUPER + CTRL + ,` or "Do not disturb: on" in the Command menu, and it stays on until you turn it off. It also turns on by itself while you share your screen (Meet, Discord, OBS) or a window is fullscreen (a video, a game; a maximised window doesn't count), and off again when that ends. Turn it off during one and it stays off for that one only.

## Wi-Fi

"Wi-Fi networks" in the Command menu, or `SUPER + CTRL + W`, opens the control center's Wi-Fi tab. Saved networks in range are under Known, the one in use first and checked ("Connected · 5 GHz"); the others in range are under Nearby, strongest first, a lock on the secured ones. While the tab shows it scans about every 30 seconds, and a network out of range drops off the list.

A press on a known network connects to it; the × that shows under the pointer, or Delete, forgets one that isn't in use. A nearby network turns the list into its form: type the password and press Enter (or Connect), or Esc (or Cancel) to go back. A wrong password keeps the form open with the error; once connected, the network moves up into Known. A saved password that stops working opens the form too.

## Bluetooth

"Pair a Bluetooth device" in the Command menu, or `SUPER + CTRL + B`, opens the control center's Bluetooth tab. Paired devices are under Known, the connected ones first and checked, with their battery; devices in range are under Nearby, nearest first ("Very close", "Nearby", "Far"). Devices that send no name are left out of Nearby: a line under it says how many ("12 unnamed devices"), and its Show button opens a drawer under the line with them, by their address, until Hide. The choice is remembered, and "Show unnamed Bluetooth devices" and "Hide unnamed Bluetooth devices" in the Command menu change it too; paired devices are under Known whether they have a name or not. Neither this list nor Wi-Fi's scrolls by itself while a search finds devices: it stays where you left it, and opens at its top, Known first. While the tab shows it searches for devices for 20 seconds every 45, often enough that a device in range never drops off the list between searches. A press on a known device connects or disconnects it; the × under the pointer, or Delete, forgets one that isn't connected.

Put a new device in pairing mode, then pick it under Nearby: the list turns into its form, and Pair (or Enter) starts pairing. Then:

- A keyboard shows a six-digit code: type it on the keyboard, then Enter. The form counts the digits as you type.
- A phone shows a code to compare: check it matches, then Pair, or Cancel.
- A device that wants a PIN or passkey from you gets a field to type it in.

Cancel or Esc stops a pairing. A pairing gives up after a minute and a half, and a prompt nobody answers after 50 seconds. A failed one keeps its reason in the form ("Wrong code, or the device said no", "Couldn't reach it. Is it in pairing mode?"), and Pair tries again. Once paired, the device moves up into Known; it is trusted, so it reconnects by itself. A pairing the device starts opens its form by itself.

The codes come from KubaOS's pairing agent, the `kubaos-bluetooth` user service: `systemctl --user status kubaos-bluetooth`, logs in `journalctl --user -u kubaos-bluetooth`. Without it the tab says so and can't pair. `bluetoothctl` works beside it; its own `agent on` takes over the prompts until it quits.

When the adapter is off, the tab has a Turn on button. When it is blocked (airplane mode), it says to run `rfkill unblock bluetooth`. When bluetoothd stops answering (it can after a pairing that hung), the tab says "Bluetooth isn't responding": run `sudo systemctl restart bluetooth`.

## Your phone

The phone link shares the clipboard and files between the laptop and your phone (iPhone, iPad or Android) through KDE Connect. It is off until you set it up: until then nothing on the laptop listens for phones or announces itself on the network.

"Set up a phone" in the Command menu opens Kōbō's "Your phone" card (the first-boot setup has it too):

1. Get KDE Connect on the phone: the App Store on iPhone and iPad, Google Play or F-Droid on Android. Same Wi-Fi as the laptop.
2. "Set up a phone" on the card. KDE Connect starts, and the card says what name the laptop shows up as.
3. In the phone's app, tap the laptop, then Request pairing. The card shows the request within a few seconds, with a key: check the phone shows the same one, then "Accept the pairing". Other KDE Connect devices on the network can show up and ask too; accept only the phone in your hand.
4. "Send a test copy", then paste on the phone. Then copy something on the phone, send it (below), and "Check what the phone sent".

What works each way:

- **Laptop to phone.** Copy on the laptop and it is on the phone's clipboard, on iPhone and Android alike. "Send a file to the phone…" in the Command menu sends a file by its path; "Send the clipboard to the phone" sends it again by hand.
- **Phone to laptop, iPhone.** Always by hand: open KDE Connect and tap Send Clipboard, or share to KDE Connect from any app (a photo, a link, text). iOS doesn't let an app read the clipboard in the background, so it never syncs on its own.
- **Phone to laptop, Android.** From the app, its quick tile or its notification. Android 10 and later keep apps from reading the clipboard in the background too, so it is by hand there as well.
- "Ring the phone" rings it even on silent. "Unpair the phone" forgets it. "Phone link status" (or `kubaos-phone status`) shows the name, the addresses and the paired phone with its battery. "Turn the phone link off" stops KDE Connect; setting a phone up again turns it back on, and the pairing is kept.

A paired phone can share the clipboard, send files and text both ways, ping, ring the laptop, be rung, and report its battery and signal; nothing else. It can't move the pointer, type, run commands, control media, lock the screen or keep it awake, or browse the laptop's files: KubaOS turns every other KDE Connect plugin off for every device, at setup, at pairing and each time KDE Connect starts, new plugins from later versions included. To allow one anyway, put its name (`kdeconnect_mousepad`, say; `busctl --user get-property org.kde.kdeconnect /modules/kdeconnect/devices/<id> org.kde.kdeconnect.device supportedPlugins` lists them) on a line of its own in `~/.config/kubaos/desktop/phone/plugins`, then turn the phone link off and on again and switch the plugin on in KDE Connect's settings for the device (or with `busctl --user call org.kde.kdeconnect /modules/kdeconnect/devices/<id> org.kde.kdeconnect.device setPluginEnabled sb kdeconnect_mousepad true`).

On a company or hotel network the phone may not see the laptop: some networks block discovery, and the app sometimes needs a restart before it looks again. After 30 s without a phone the card shows the laptop's address: in the phone's app, add the laptop by IP with it. KubaOS turns no firewall on; with one, KDE Connect needs TCP and UDP 1714-1764 and mDNS open on the local network.

The daemon is the `kubaos-phone` user unit: `systemctl --user status kubaos-phone`, logs in `journalctl --user -u kubaos-phone`. The switch is the file `~/.config/kubaos/desktop/phone/phone-link`.

## Idle, Keep awake and the lid

Out of the box the screen locks after 5 minutes idle, the screens go off after 10, and the laptop suspends after 30 on battery and never plugged in. Closing the lid suspends on battery and does nothing plugged in or docked (a dock, or a second screen connected).

Change any of it from the Command menu: "Lock after…", "Screens off after…", "Suspend after…" and "When the lid closes…". Enter lists the usual choices with the ones in use marked; Enter on a choice sets it on battery and plugged in, and its other Actions (Ctrl+K) set one of them. Or type a time after the word: `lock-after 2m`, `screens-off-after 1h30m`, `suspend-after never ac`, `lid lock battery`. A time is what the clock's timer takes (`90s`, `2m`, `1h30m`, `5h`, a number alone in minutes) or `never`; a trailing `battery` or `ac` sets that power source alone, both otherwise. The lock and screens off take 30 s at the least, suspend a minute, and a day at most; a toast says what changed ("Lock after 15 min idle on battery, 30 min plugged in") or why it was refused. The file is `~/.config/kubaos/desktop/idle/idle.toml`; after a hand edit run `kubaos-idle apply`. "Idle: KubaOS defaults" puts it all back. `kubaos-idle status` prints what is set.

If you change the power source while idle, the other source's time takes over within a minute: unplugging a laptop that has been idle longer than its battery time locks or suspends it then.

The lid setting works without root: while it differs from what logind does, your session holds the lid away from logind and does it itself, and logind's own rule comes back when you log out. Docked, closing the lid never does anything.

**Keep awake** (`SUPER + CTRL + I`, or "Keep awake for 1 hour", "for 2 hours", "for…", "until turned off" in the Command menu) holds the auto-lock, screens off and suspend off. It ends by itself when its time is up, says so, and shows a cup in the bar while on; click the cup or run "Stop keeping awake" to end it sooner. Closing the lid and locking by hand still work.

**Lock, suspend, shut down or reboot in… / at…** do it once, whether you're working or not: `lock-in 10m`, `lock-at 20:00`, `suspend-at 23:00`, `shutdown-at friday 17:00`, `reboot-in 5m` (an "at" is the next time it comes round). Shut down and reboot ask you to confirm first, and close the session the way the power menu does; a suspend locks first. Toasts warn a minute and 10 s before, each with a Cancel button, the bar counts down with the action's icon, and Keep awake doesn't stop any of them: you asked for it. One at a time, whichever it is; a new one replaces the old, and the toast says what it replaced. "Cancel scheduled action", Cancel on a warning, or a click on the countdown cancels it. They are systemd user timers: `systemctl --user list-timers 'kubaos-scheduled*'`.

## Power

The control center's power slider has five stops: **Super Saver · Power Saver · Balanced · Performance · Max Performance**. Drag it (it moves freely and settles on the nearest stop when you let go), click it, scroll it a notch at a time, focus it and press Left and Right, or press its icon, which steps one stop from Max Performance down to Super Saver and back up. The bar's power icon shows the stop: a moon, a leaf, scales, a bolt, a rocket. The Command menu has an Item for each stop too ("Super Saver", "Max Performance", …), and `kubaos power set balanced` does it from a terminal.

Each stop sets every layer of the machine at once:

| | Super Saver | Power Saver | Balanced | Performance | Max Performance |
|---|---|---|---|---|---|
| power-profiles-daemon | power-saver | power-saver | balanced | performance | performance |
| CPU energy preference, boost | power, off | power, off | balance, on | performance, on | performance, on |
| CPU clock cap | 1.6 GHz | none | none | none | none |
| Firmware profile (power limit, fans) | low-power | low-power | balanced | performance | performance |
| GPU (amdgpu DPM) | low | low | auto | auto | high |
| Laptop panel | 60 Hz | native rate with VRR | native rate with VRR | native rate | native rate |
| Panel power saving | strong | medium | off | off | off |
| PCIe ASPM | powersupersave | powersave | default | performance | performance |
| USB autosuspend | on | on | on | on | off |
| Wi-Fi power saving | on | on | on | off | off |
| Background work | indexer paused, monitor every 10 s | the same | normal | normal | normal |
| CPU scheduler, when LAVD is on | LAVD powersave on the fewest cores | LAVD powersave | LAVD autopilot | LAVD performance | LAVD performance |

Input devices (keyboards, mice, security keys), headsets, printers and network adapters (a dock's Ethernet) are never autosuspended, in any stop; a device plugged in takes the stop's settings within a second. External screens are never touched, and switching never moves or rescales anything. "Power: what this stop does" in the Command menu, or `kubaos power status`, lists every row as the machine has it right now, marks what disagrees, and says which rows this machine has no knob for. Other tools that switch power-profiles-daemon (`powerprofilesctl`, GNOME's settings) move the slider to the stop named like their profile. A program that holds a profile for a while (GameMode) moves it too, without changing the stop your power source remembers, and its end puts back your stop.

**Battery and plugged in each remember their stop.** Pick Super Saver on battery and Performance at the desk, and unplugging goes back to Super Saver, plugging in back to Performance, with a quiet toast naming the stop. Until you pick, battery is Power Saver and plugged in is Balanced. A dock that drops power for a moment doesn't swing the stop: the source has to hold for 3 seconds.

**Low battery.** At 20 % on battery the stop drops to Power Saver, with a toast whose Undo puts back the stop you had; the drop isn't remembered as your battery stop, and plugging in ends it. "Low battery: drop to Super Saver", "Low battery: don't drop" and "Low battery drop at…" (`low-battery 15`) change it.

**Charge limit.** "Charge limit: 80 %", "90 %" and "full" in the Command menu, or `kubaos power charge-limit 80`, stop the battery charging there: kinder to it on a laptop that lives on a desk. TUXEDO laptops do it through their charging profile, others through the kernel's `charge_control_end_threshold` when the battery has one.

**Colour-accurate work.** The saver stops turn on AMD's panel power saving, which dims the backlight and lifts the contrast to match. "Panel power saving off" keeps true colours in every stop; "Panel power saving on" brings it back.

**CPU scheduler.** Linux schedules with its own EEVDF until you run "CPU scheduler: LAVD on" (`kubaos power scheduler on`). Then scx_lavd takes over through the kernel's sched_ext, with no reboot: it puts what you are waiting on first (input, the compositor, audio), and its mode follows the stop, as the table shows. Super Saver's packs the work onto the efficient Zen 5c cores, each core's two threads before the next core wakes. A new stop restarts LAVD in its mode, and Linux's own scheduler runs for that moment. LAVD stays on across reboots, and "CPU scheduler: LAVD off" goes back to EEVDF at once.

If LAVD crashes or stalls, the kernel goes back to its own scheduler by itself, a toast says so, and LAVD starts again 2 seconds later. A third crash within 5 minutes leaves it off, with a critical toast, until you run "CPU scheduler: LAVD on" again, or reboot. Alt+SysRq+S also hands the machine back to EEVDF; the next stop starts LAVD again. "CPU scheduler: status" in the Command menu, or `kubaos power scheduler`, says which scheduler runs, why, its mode in this stop, the service's state and LAVD's last error. `kubaos power status` has it as a row. Rolling back to a release from before the CPU scheduler leaves LAVD running with nothing to turn it off: turn it off first, or afterwards run `sudo systemctl disable --now kubaos-scx.service`.

**Measuring.** Unplug, pick a stop, leave the machine as it is, and run "Measure power draw" (`kubaos power measure`, 2 minutes; `--seconds 300` for longer). It prints the battery's average draw and what woke the machine most (interrupts and processes, per second), and saves the report in `~/.local/state/kubaos/power/measure/`.

The settings are `~/.config/kubaos/desktop/power/power.toml`: the panel per stop (`60hz`, `vrr` or `native`), Super Saver's clock cap, the USB devices to keep awake, the charge limit, the low-battery drop and the CPU scheduler (`scheduler = "kernel"` or `"lavd"`). After a hand edit run `kubaos power apply`. The stop each power source remembers is in `~/.local/state/kubaos/power/stops.json`. If your own Hyprland rule for `eDP-1` sets its mode, the stop's rate still applies on top of it, keeping your position and scale. The panel's VRR is Hyprland's `misc:vrr`, which the stops own; an external screen's VRR is its own rule's `vrr` (`vrr = 1` to have it), off otherwise. Each stop's PCIe ASPM policy is `aspm` there too: Super Saver's `powersupersave` sleeps the SSD and the Wi-Fi card hardest, and some drop off the bus there (`journalctl -k` shows nvme or Wi-Fi resets); if yours do, set Super Saver's to `powersave`.

Super Saver's 60 Hz needs amdgpu's FreeSync video modes, which the update turns on for the next boot; until you reboot, Super Saver keeps the panel at its native rate with VRR, so the switch never blanks the screen, and `kubaos power status` says so. The panel's VRR starts at the next login after the update.

## The work network (Proxy)

The Proxy is one switch for the work network: IT's WireGuard tunnel, the work DNS, the company's HTTP proxy and its root certificate, all up together or none. It replaces the WireGuard app. Each part is optional, and nothing switches on by itself: a login starts with it off.

**Set it up.** Get the WireGuard config from IT (a `.conf` file), then run "Import WireGuard config…" in the Command menu, or in a terminal:

```sh
kubaos proxy import ~/Downloads/work.conf
```

bw asks for the master password (unless `BW_SESSION` is exported). The private key and the preshared key go to your Bitwarden vault, in a secure note named `kubaos-proxy`, and to the GNOME keyring, which the switch reads them from; the rest of the config is written, without its keys, beside your profile as `wireguard.conf`. The import says whether it is a split tunnel (only IT's ranges go through it) or a full one (`AllowedIPs = 0.0.0.0/0` or `::/0`: everything does). You can delete the `.conf` IT sent afterwards. On a new machine, `kubaos proxy pull` copies the keys from the vault to the keyring again.

The rest is in your profile, `~/.config/kubaos/desktop/network/proxy.toml` ("Open proxy settings" in the Command menu): the work DNS servers and the domains they answer for (the config's `DNS =` line counts too), the proxy (`http://proxy.corp.example.com:3128`, no user or password) or a PAC file, what goes around it (`no_proxy`), and the company's root certificate (a PEM file beside it). A change takes effect at the next "Proxy on".

**Switch it.** The Proxy tile in the control center (it shows once the profile has a part), "Proxy on" and "Proxy off" in the Command menu, or `kubaos proxy on` and `off`. The first "Proxy on" asks for your password once, to trust the proxy and the company CA (a program running as you can't make the machine trust another or send its traffic elsewhere without it); it asks again only when you change them in the profile. A part that fails takes the others down with it, and a notification says which and why. While it is on, the bar shows a locked globe; the tile's arrow shows the endpoint, when the tunnel last received, the last handshake and the bytes moved, and "Proxy status" in the Command menu (`kubaos proxy status --live`) shows them too.

What each part reaches:

| Part | Where it goes |
|---|---|
| Tunnel | a NetworkManager connection, `wg-kubaos`, made at each "Proxy on" and gone at "Proxy off"; its keys are never written to disk |
| Kill switch | a full tunnel only (`kill_switch = false` in the profile turns it off): while it is up, nothing leaves but through the tunnel, DNS lookups included, so a LAN printer is out of reach too |
| Work DNS | systemd-resolved, on the tunnel's link: only the work domains go to the work servers, every other name to your network's own; a full tunnel sends every name to the work servers |
| HTTP proxy | new shells and services (`http_proxy` and the like), GNOME's proxy setting and Chrome's policy (so browsers follow), apt, Docker (the daemon and `docker build`/`run`), Nix's daemon, and git; "Proxy off" puts back the GNOME, Docker and session proxy you had before |
| PAC file | GNOME and Chrome only; the rest need the proxy itself |
| Company CA | the system's trust store, which curl, git, Nix, Docker's daemon and most tools read, and Chrome's policy |

A terminal or an app already open keeps what it had; open a new one. Docker's and Nix's daemons restart when the proxy changes, and Docker's when the CA does, so running containers stop. While the Proxy is on, apt, Docker and Nix reach the internet through the work proxy: turn it off away from work, or they won't reach anything.

After the laptop sleeps, or when the tunnel drops while it is awake, the tunnel comes back by itself once the network is up. If it can't, everything goes off but the kill switch, which keeps blocking what would leave outside the tunnel: a notification says so, the next change of network tries again, and "Proxy off" lifts it. Without a kill switch, everything goes off. The service is the `kubaos-proxy` user unit: `journalctl --user -u kubaos-proxy`.

## Updating and rolling back

Nothing updates by surprise. No popups, no "restart to finish installing", no "Later" button. Hyprland and its libraries are held in apt. Your Nix packages are pinned in your flake's lock (`~/.config/kubaos/home/flake.lock`). KubaOS's own (Quickshell, home-manager, the icons, the lock screen's PAM modules, the recorder, the GPU drivers) are pinned in KubaOS's lock and move only with a KubaOS release. Ubuntu's own updaters (unattended-upgrades and the apt-daily timers) are off: nothing runs apt but `kubaos update`, security updates included. It waits until you run it, or pick Update KubaOS in the menu (`SUPER + Space`), which opens a floating terminal running it. A daily timer runs `kubaos update --check`: it only looks, as you, never as root. When there is something new, a release or apt security updates, it sends one notification and puts a marker in the bar (a click runs `kubaos update`), and the menu's Update KubaOS says what is waiting.

```sh
kubaos update                    # the whole update, after showing what it brings and asking
kubaos update -y                 # without asking
kubaos update --packages-only    # apt and your flake; KubaOS stays at its release
kubaos update --check            # only look; writes ~/.local/state/kubaos/update-available
kubaos rollback                  # undo the last update: KubaOS's release and your flake.lock
kubaos channel                   # which channel this machine follows
kubaos channel stable            # or edge
kubaos upgrade                   # stable only: move to the next series (Upgrade to <codename> in the menu)
kubaos packages status           # KubaOS's apt packages against the release: installed, missing, held, extra
kubaos packages disable docker   # leave an optional group out for good (enable brings it back)
```

An update, in order:

1. Refuses when another update runs, when `/` or `/nix` has less than 3 GiB free, or when `origin` can't be reached.
2. Picks the release. On **edge** (a machine installed from a build that wasn't a verified release) that's `origin/main`'s head; with `/usr/share/kubaos-keys/allowed_signers` on the machine the head must be signed by one of its keys, without it the update warns once and goes on. On **stable** it's the newest verified `v*` tag of the installed series; with no such tag it says so and only updates packages.
3. Shows the release's commits, the apt packages, a Hyprland pin change and the system migrations it will run, and asks.
4. Switches your home back to the release when it is a live preview ("Hacking on KubaOS", below). Takes a snapper snapshot, copies your `flake.lock` to `~/.local/state/kubaos/locks/`, and adds a line to `/var/lib/kubaos/update-history`.
5. `apt-get upgrade`, then moves `/usr/share/kubaos`'s `release` branch.
6. When the release's `system/files/apt/preferences.d/kubaos-hyprland` differs from the installed one (amd64): unholds the Hyprland stack, installs the pinned versions, holds it again.
7. Brings the apt packages in line with the release's package manifest, `system/packages.toml` (below): installs what its groups list and the machine lacks, holds the held ones, and removes a package no group lists any more, but only one KubaOS installed and nothing else needs.
8. Runs the release's new system migrations (`migrations/system/`, as root), each once.
9. `nix flake update --flake ~/.config/kubaos/home`, `system/relink.sh` (the wallpaper, PAM and recorder links), and a switch.
10. Runs the new user migrations (`migrations/user/`, as you), each once, and writes `/var/lib/kubaos/release`.

In a terminal each of these phases gets one line: a spinner and the program's latest line while it runs, then a tick, a cross or "skipped", its time and what it did. A warning is one dimmed line under its phase. sudo asks for your password only when its cached one has run out, on a clean line. The programs' own output doesn't scroll past: it goes to a log, `~/.local/state/kubaos/update/<date>-update.log` (the last 10 runs are kept). When a step fails you see its last 20 lines and the log's path. At the end a summary says what moved where, the changes, the packages, the migrations that ran, what the session reloaded, whether you need to log in again or reboot, and how to undo it:

```text
  ✓ Home               1m05s  reloaded Hyprland, restarting Quickshell

  Updated     Shinjuku 0.1.0 (1a2b3c4) → Shinjuku 0.1.1 (9f8e7d6), edge
  Packages    12 apt packages, Nix inputs nixpkgs
  Apt         files: installed loupe, showtime
  Migrations  1792765570-backlight-rule.sh
  Next        nothing, it's all running
  Undo        kubaos rollback
```

Piped or in a log, the output is plain lines with no colour; `NO_COLOR=1` drops the colour in a terminal too. `--json` (on `update`, `upgrade` and `rollback`) prints one JSON event per line instead, for a program to show; the events are in [`docs/CONTRACTS.md`](CONTRACTS.md), "Updates and migrations". apt runs without questions and keeps your changed files under `/etc`, leaving the package's new version beside each as `.dpkg-dist`.

### KubaOS's apt packages

Every apt package KubaOS installs is listed in one file, `system/packages.toml` in the release, in groups: the base system, the file manager, Hyprland (held), Docker, Chrome, the security key and so on. The install reads it and so does every `kubaos update`, so a package a release adds reaches your machine with that update, as a Nix package does. `/var/lib/kubaos/apt-installed` records what KubaOS installed; a package you installed yourself is never removed, even when a release drops it from the list.

```sh
kubaos packages status           # each group: installed, missing (the next update installs it), held, or extra
kubaos packages status --json    # the same for a program
kubaos packages enable piv       # turn an optional group on: install it, and keep it in every update
kubaos packages disable docker   # turn it off: remove what KubaOS installed for it, and leave it out
```

The optional groups are `docker`, `chrome` and `u2f` (on unless you say otherwise), `piv` and `tuxedo` (off until you turn them on; first-boot setup turns `tuxedo` on). What you chose at install (`system/40-apps.sh --no-docker`, `system/75-yubikey.sh --piv`, `system/70-bitwarden.sh --no-u2f`) is kept in `/etc/kubaos/apt-groups`, so no update brings back what you left out. `disable u2f` takes the security key out of sudo, polkit and the lock screen before its packages go. On a machine installed before the record, `disable` names the packages it leaves because KubaOS didn't install them, with the `sudo apt remove` for them. The Command menu has all of it: "KubaOS apt packages", and "Install Docker", "Remove Docker" and the like for each group.

The running desktop follows every switch (`kubaos update`, `kubaos rollback`, a `home-manager switch` of your own) without a new login, and only where the defaults changed: Hyprland reloads, Quickshell restarts (after you unlock, if the screen is locked), hypridle restarts if it was running, Ghostty and tmux reload their config. With no desktop running, a switch from a tty or over ssh leaves that for the next login. New Neovim, lazygit and other terminal app settings apply the next time you start them. The details are in [`docs/CONTRACTS.md`](CONTRACTS.md), "The running session after a switch".

Your Nix packages move with your flake's nixpkgs; KubaOS's keep their exact store paths until its release moves. Ways back, from smallest to largest:

- **Shell broken.** `SUPER + ALT + Return` opens foot and `SUPER + ALT + Space` opens fuzzel. Both come from apt and work without Nix or Quickshell.
- **The last update.** `kubaos rollback` (Roll back last update in the menu) moves `release` back to where that update found it, puts back the `flake.lock` it saved and switches. Migrations only go forward: what they changed stays.
- **User environment broken.** `home-manager generations`, then run the previous generation's `activate`.
- **System broken.** Boot the "before kubaos update" snapshot from GRUB's "KubaOS snapshots" menu, then swap `@` by hand ([`system/00-btrfs-subvolumes.md`](../system/00-btrfs-subvolumes.md), "Rolling back the system later"). `snapper rollback` does not work with Ubuntu's GRUB layout. apt runs take their own pre/post snapshots too. Your home isn't in the snapshot: `kubaos rollback` covers your flake.
- **Hyprland won't start.** `Ctrl+Alt+F2`, log in, `touch ~/.config/hypr/no-autostart`.

## Make it yours

Don't edit `/usr/share/kubaos` or the files in `~/.config/<app>`. Those are KubaOS's defaults: read-only, and replaced whole by every update. Your changes go in `~/.config/kubaos`, which every app loads after ours, so what you set there wins. Updates never write to it. When we change a default you haven't touched, you get the new one; when you've set your own, you keep yours.

The folder is grouped by area, and its `README.md` says where to go for what:

```
~/.config/kubaos/
  README.md   want to change X? go here
  desktop/    hypr/ quickshell/ cursor/ clock/ keyboard/ sysmon/ idle/ power/ files/ wallpapers/
  terminal/   ghostty/ foot/ tmux/ zsh/ fastfetch/
  editor/     nvim/
  tools/      git/ lazygit/ yazi/ bat/ bacon/ fuzzel/
  menu/       kotodama/
  home/       your flake
```

In the Command menu, "Settings folder" opens it in Files, and "Settings: Desktop", "Settings: Terminal", "Settings: Editor", "Settings: Tools" and "Settings: Menu" open one group; `dot-config open [group or app]` and `dot-config path [group or app]` do the same from a terminal. Each file starts with the settings people change most, commented out with their defaults, and a link to the app's full docs:

| To change | Edit | Applies |
|---|---|---|
| Hyprland: binds, gaps, monitors, rules | `~/.config/kubaos/desktop/hypr/hyprland.lua` (same Lua API as `~/.config/hypr/`) | `hyprctl reload` (`SUPER + SHIFT + CTRL + R`) |
| Idle times and the lid | `~/.config/kubaos/desktop/idle/idle.toml`, or the Command menu ("Idle, Keep awake and the lid" above) | at once from the menu; `kubaos-idle apply` after a hand edit |
| The power stops: the panel per stop, USB devices kept awake, the charge limit, the low-battery drop | `~/.config/kubaos/desktop/power/power.toml`, or the Command menu ("Power" above) | at once from the menu; `kubaos power apply` after a hand edit |
| Idle listeners of your own, lock screen | `~/.config/kubaos/desktop/hypr/{hypridle,hyprlock}.conf` | `kubaos-idle apply` / next lock |
| Shell aliases, exports | `~/.config/kubaos/terminal/zsh/zshrc` | new shell |
| tmux | `~/.config/kubaos/terminal/tmux/tmux.conf` | `tmux source-file ~/.config/tmux/tmux.conf` |
| Ghostty, foot, fuzzel | `~/.config/kubaos/terminal/{ghostty/config,foot/foot.ini}`, `~/.config/kubaos/tools/fuzzel/fuzzel.ini` | reload or new window |
| git | `~/.config/kubaos/tools/git/config`, `~/.config/kubaos/tools/git/ignore` | at once (`ignore` after `dot-config apply`) |
| Neovim | `~/.config/kubaos/editor/nvim/{options,keymaps,autocmds}.lua`, `plugins/*.lua`; extras with `:LazyExtras` | restart Neovim |
| lazygit, bacon, yazi | `~/.config/kubaos/tools/{lazygit/config.yml,bacon/prefs.toml,yazi/yazi.toml,yazi/keymap.toml}` | next start (lazygit and bacon: in a new shell). The yazi keymap starts with KubaOS's drag key ("Files") |
| bat, fastfetch | `~/.config/kubaos/tools/bat/config`, `~/.config/kubaos/terminal/fastfetch/config.jsonc` | after `dot-config apply` |
| btop, tms | `~/.config/btop/btop.conf`, `~/.config/tms/config.toml`: they rewrite their own file, so it's yours as is | next start |
| Command menu: your own actions, hiding KubaOS's | `~/.config/kubaos/menu/kotodama/user.toml` (below) | on save |
| Nix packages and home-manager options | `~/.config/kubaos/home/home.nix`, e.g. `home.packages = [ pkgs.hello ];` | `home-manager switch --flake ~/.config/kubaos/home` |
| A package, without writing Nix | `kubaos package add ripgrep` (`remove`, `list`); `kubaos try cowsay` opens a shell with it and installs nothing | the switch `add` and `remove` queue |
| Unfree packages: allowed (the default) or free ones only | `kubaos package unfree off` (`on`; without either it prints which). In the Command menu: "Free packages only", "Allow unfree packages" | the switch it queues |
| A whole language setup (a Suite) | `kubaos suite list`, `kubaos suite add NAME [--version V] [--extra E]`, `configure`, `remove [--purge-caches]`; `kubaos suite new NAME PROJECT` starts a project in `~/Projects` (another folder: `home.sessionVariables.KUBAOS_PROJECTS` in `home/home.nix`) | the switch `add`, `configure` and `remove` queue |
| A Suite version for one folder (a Project pin) | `kubaos pin node 18 ~/Projects/acme` writes `use suite node 18` into `~/Projects/acme/.envrc`; `kubaos unpin node ~/Projects/acme`; `kubaos pins` shows what applies where you are, `kubaos pins --all` every pin under `~/Projects`. In the Command menu: "Pin a Suite version…", "Unpin a Suite version…", "Project pins" | the next `cd` into the folder |

- `~/.zshrc` and the other files home-manager links are read-only, so an installer that offers to "update your profile" fails there. Say no, or pass its flag for that (proto: `bash <(curl -fsSL https://moonrepo.dev/install/proto.sh) --no-profile`), and put any lines it wanted in `~/.config/kubaos/terminal/zsh/zshrc`. rustup's, cargo's and proto's PATH entries are already set (`PROTO_HOME` is `~/.local/share/proto`).
- To see what KubaOS ships for an app, read `~/.local/share/kubaos/defaults/<app>/` (or `config/<app>/` in the repo).
- Delete a file in `~/.config/kubaos` to get the example back on the next `dot-config apply` or update.
- Coming from a release before the groups, the update moves each `~/.config/kubaos/<app>` into its group for you and loses nothing. Where you had edited both the old file and a new one in the group, it keeps the group's and puts yours beside it as `<file>.old`, and says so; merge the two by hand. A user layer under git sees the moves as renames, for you to commit.
- Quickshell's settings, the colors (matugen) and the prompt (oh-my-posh) can't be overridden yet.
- Your own wallpapers go in `~/Pictures/wallpapers/`, the picker's last tab; KubaOS's sets are in `/usr/share/kubaos/wallpapers/`.
- `~/.config/kubaos/home/` is your flake: KubaOS wrote it once, and an update never does again. Put it under git if you like (then `git add` new files of yours before a switch; the `kubaos` command does that for what it writes into `kubaos.d/`).
- The Command menu's KubaOS actions and toggles are listed in `~/.local/share/kubaos/defaults/kotodama/kubaos.toml`, each under `[extensions.kubaos.actions.<id>]`. Add your own in `user.toml` the same way, and hide any of KubaOS's, actions or toggles (`wifi`, `bluetooth`, `volume`, `brightness`, `power-saver`, `power-status` and so on), by id:

  ```toml
  [extensions.kubaos]
  hide = ["screenshot-window", "bluetooth"]

  [extensions.kubaos.actions.notes]
  title = "Open my notes"
  keywords = ["journal"]
  run = ["ghostty", "-e", "nvim"]

  [extensions.kubaos.actions.timer]
  title = "Timer"
  keyword = "timer"   # type "timer 5m"
  arguments = [{ name = "duration", placeholder = "5m" }]   # fills {duration}
  run = ["sh", "-c", "sleep \"$1\" && notify-send Done", "sh", "{duration}"]
  ```

  Keys set under one of KubaOS's ids change only those keys of its action. `terminal = true` runs it in a floating terminal, centred over your windows instead of tiled between them, which stays open until you close it; and `delay_ms` waits that long first, so the menu is gone before a screenshot. `shown_when = { state = "packages.chrome", is = true }` lists an action only in that state (`is_not` for every other), which is how only one half of each on/off pair shows, "Remove Google Chrome" while it is installed and "Install Google Chrome" while it isn't; the states are listed in `docs/CONTRACTS.md`, "The Command menu's KubaOS Extension", and an action whose state can't be read shows. Leave a word of an app's name out of `keywords` on an action about that app: a keyword equal to what you typed ranks the action above the app. The Extension logs what it can't use, such as an id it doesn't know (`journalctl --user -u kotodamad`).
- `kubaos package add` records the package in `~/.config/kubaos/home/kubaos.d/packages.toml`, a file the command owns, and switches. Every switch (`kubaos switch`, or one an `add` queued) runs in a systemd user unit, one at a time: closing the terminal doesn't stop it, and `kubaos job list` and `kubaos job follow <id>` show it. The first `add` on an older install adds `kubaos.stateDir = ./kubaos.d;` to your `home.nix`, after backing it up to `~/.local/state/kubaos/backups/home.nix.bak`. If your flake is under git, the command records each file it writes with `git add --intent-to-add` (and drops the ones it deletes from the index), so Nix sees them; it never commits.
- Unfree packages install like any other: `kubaos package add discord` (or spotify, slack, …) just works, and says `added discord (unfree)`; `kubaos try discord` works too. `kubaos package list` marks each unfree package `unfree`, and `--json` gives every package its `from` files and `unfree` (`true`, `false`, or `null` when it can't tell, such as for a name your nixpkgs doesn't have). Both ask your nixpkgs once per command (`nix eval` of `meta.unfree`): under a second usually, longer the first time after your nixpkgs moves. If that fails they say so and go on unmarked, unless unfree packages are off. For a free-only system, `kubaos package unfree off` writes `allow_unfree = false` to `kubaos.d/nixpkgs.toml` and switches; `kubaos package unfree` prints `on` or `off`. From then on your nixpkgs refuses unfree packages, `home.nix`'s too, and `kubaos package add` refuses one before it records or switches anything, naming the setting and `kubaos package unfree on`, which turns them back on; it refuses too when it can't check. `off` itself refuses while `kubaos.d/` lists an unfree package, and says which and how to remove it, since the switch would fail on it; an unfree package in your `home.nix` fails that switch instead, and `kubaos package unfree on` undoes it. A Suite's packages aren't checked when you add it. Your own `nixpkgs.config` in `home.nix` merges with KubaOS's, so with the setting off an `allowUnfreePredicate` of yours still lets in the packages it names.
- `kubaos suite add` installs a Suite: its toolchain, environment and editor settings, which load before yours in `~/.config/kubaos/editor/nvim/`, so your settings win. It writes `kubaos.d/suites/<name>.toml` and creates any file of yours it has an example for only if that file is missing. `kubaos suite remove` takes it all away again, shows the size of its caches and keeps them unless you pass `--purge-caches`, lists your files that mention the Suite, and never touches your projects.
- A Project pin covers its folder and every folder below it: with `use suite node 18` in `~/Projects/acme/.envrc`, `node --version` is 18 anywhere under `acme/`, and your installed version everywhere else. The version doesn't need to be installed, nor the Suite: the first `cd` fetches it into the Nix store (direnv says so), and a GC root in the folder's `.direnv/` keeps it there, so add `.direnv` to your global ignore file (`~/.config/kubaos/tools/git/ignore`). direnv loads only the nearest `.envrc`, so a project inside `acme/` with an `.envrc` of its own needs `source_up` in it to keep the pin; a `use suite` line after `source_up` pins that project differently. `kubaos pins` tells you when a pin above doesn't reach you. For a project shared with people who don't run KubaOS, pin the toolchain in its own `flake.nix` and use `use flake` instead.

## Hacking on KubaOS

KubaOS itself is the root-owned checkout `/usr/share/kubaos`. To change it, clone it into `~/Projects/kubaos` and try a branch live on your own home with a **live preview**:

```sh
git clone https://github.com/SuplenC/kubaos.git ~/Projects/kubaos
kubaos switch --from ~/Projects/kubaos          # a checkout, by its folder
kubaos switch --from feature/issue-12-waybar    # or a branch: the worktree of ~/Projects/kubaos that has it
kubaos switch --release                         # back to the release this machine is on
```

`--from` first checks that your home evaluates with the checkout as KubaOS and refuses it, switching nothing, when it doesn't. Then it switches your home with your flake's `kubaos` input overridden to the checkout, as a queued switch job like any other. Your `flake.lock` stays as it is. The build sees the files git knows about, uncommitted changes to them included: `git add` new ones first.

While a preview is on:

- The bar shows a pill in the cold accent naming the branch, beside the workspaces. Its hint gives the commit and since when; a click opens the Command menu at "Back to the release", which runs `kubaos switch --release` in a terminal.
- `kubaos switch` alone is refused and names both ways on: `--from` the same checkout builds it again, with its latest changes, and `--release` goes back. A switch that `kubaos package` or `kubaos suite` queues keeps the preview.
- A second `--from` replaces the first, and says so. There is one preview at a time.
- `kubaos update`, `upgrade` and `rollback` switch back to the release first, once you've said yes and before they change anything, so no update runs on top of a preview.

A preview only changes what your home builds: the shell, Hyprland's config, the Command menu's KubaOS Extension, the configs and the packages of your home. The system stays the release's: apt packages, `system/`, migrations, the boot and the login screen. Try those through a branch's own install or update. `~/.local/state/kubaos/preview.json` records the preview on (`docs/CONTRACTS.md`, "Live previews").

## Daily commands

| Command | What |
|---|---|
| `home-manager switch --flake ~/.config/kubaos/home` | Apply your `home.nix` |
| `dot-theme set <image>` | Set a wallpaper (an image or a loop) and regenerate colors everywhere |
| `dot-theme next`, `prev`, `toggle-folder` | Cycle wallpapers (stills and loops), cycle the folders: the sets you picked, your own `~/Pictures/wallpapers` |
| `~/.config/kubaos/desktop/wallpapers/sets` | What the picker shows, one per line: a set's name for all of it, `<set>/<file>` for one wallpaper (every set in `/usr/share/kubaos/wallpapers` without the file). A set a new release brings shows up once you add it. The Command menu has one Item a set ("wallpaper sets", "Wallpaper set: Shinjuku") to show it whole or hide it, through `dot-theme show-set\|hide-set <set>`; the last set in the picker stays. `dot-theme all-sets` (Command menu: Show every wallpaper set) lists every set again |
| `dot-theme apply [--force]` | Regenerate from the current wallpaper; without `--force` it does nothing when no input changed (login runs it) |
| `kubaos cursor style dark\|accent\|classic`, `kubaos cursor size <16-96>` | The pointer: dark with a light outline, the wallpaper's accent, or plain Bibata black and white, and its size, in steps of 4. Applies at once and is kept in `~/.config/kubaos/desktop/cursor/cursor.toml`; the Command menu has both ("cursor") |
| `kubaos keyboard show`, `kubaos keyboard set us,it(intl)`, `kubaos keyboard add\|remove\|default <layout>`, `kubaos keyboard move <layout> <place>`, `kubaos keyboard list`, `kubaos keyboard accents [on\|off\|apply]` | Your keyboard layouts (`~/.config/kubaos/desktop/keyboard/keyboard.toml`), the first the default; Hyprland reloads after a change. A new default also goes to `/etc/default/keyboard` for the console and the disk prompt, through a root helper and pkexec, which may ask for your password. At most 4, and the last one stays; the Command menu has them all ("keyboard"). `list` prints every layout and variant by name; `accents` turns the Mac-style accents on Left Alt on or off |
| `dot-config apply` | Create missing files in `~/.config/kubaos` and rewrite the merged configs (bat, git ignore, fastfetch) from the defaults plus yours. Every `home-manager switch` runs it |
| `qs ipc show` | List the Quickshell IPC targets Hyprland binds call |
| `dot-hypr-record region\|window\|output [--audio] [--mic]` | Start or stop a recording (gpu-screen-recorder, `~/Videos/Recordings`); `replay` keeps the last 30 s, `replay save` writes them. Run `system/65-recorder.sh` once so monitor capture needs no password |
| `kubaos bitwarden setup`, `refresh`, `check` | Log in to Bitwarden and write SSH keys (public halves), Host blocks, git signing, gh token and project env from the vault; after a vault change, pull again; or just look. The Command menu has "Set up Bitwarden", Kōbō's Bitwarden card (server, login, folders, the SSH keys the vault gave, log out), and "Refresh from Bitwarden". `dot-bw-setup` is the old name, for one release |
| `kubaos bitwarden env pull <project>` | Refresh one project's env secrets in the keyring; `use bwenv <project>` in `.envrc` loads them on `cd`. `dot-bwenv` is the old name, for one release |
| `dot-u2f-enroll` | Enroll a FIDO2 key for sudo, polkit and the lock screen (`--list`, `--remove N`, `--test`) |

Secrets, Bitwarden and the YubiKey have their own page: [`docs/SECRETS.md`](SECRETS.md).

## Rust

rustup (outside Nix) owns the toolchains. `home/dev.nix` writes `~/.cargo/config.toml`: sccache as the rustc wrapper, and clang (apt) as the linker driver for x86_64 and aarch64 Linux with mold (`-C link-arg=-fuse-ld=mold`). wild 0.8.0 from nixpkgs is on PATH as the opt-in alternative; `docs/ALTERNATIVES.md` measured 1.15x to 1.26x faster incremental rebuilds than mold.

Try wild in one project without switching, for one build:

```sh
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=--ld-path=wild" cargo build
```

or for every build of that project, in its `.cargo/config.toml` (keep it out of git; teammates may not have wild):

```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=--ld-path=wild"]
```

Cargo appends these flags to the global `-fuse-ld=mold`, and clang's `--ld-path` takes precedence over `-fuse-ld`, so wild links. Check with `readelf -p .comment target/debug/<bin> | grep -i wild`. Different rustflags mean a full rebuild the first time (cargo and sccache key on them).

Switch for good: set `rustLinker = "wild"` at the top of `home/dev.nix` and `home-manager switch` (`"mold"` to go back). With wild as the default, one build with mold is `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=--ld-path=mold" cargo build` (the last `--ld-path` wins).

Benchmark on a project of yours, with a separate target dir per linker so they don't invalidate each other, and without sccache so clean builds aren't served from its cache:

```sh
cd ~/Projects/myapp
export RUSTC_WRAPPER=   # empty: cargo skips sccache
wild='CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=--ld-path=wild"'

# Incremental: touch the binary's crate, so the run is mostly linking.
hyperfine --warmup 2 --runs 10 --prepare 'touch src/main.rs' \
  -n mold 'cargo build -q -p myapp --target-dir target/bench-mold' \
  -n wild "$wild cargo build -q -p myapp --target-dir target/bench-wild"

# Clean: everything rebuilt; the linker is a small share of this.
hyperfine --runs 3 \
  --prepare 'cargo clean -q --target-dir target/bench-mold' \
  --prepare 'cargo clean -q --target-dir target/bench-wild' \
  -n mold 'cargo build -q -p myapp --target-dir target/bench-mold' \
  -n wild "$wild cargo build -q -p myapp --target-dir target/bench-wild"

rm -rf target/bench-mold target/bench-wild
```

`cargo build --timings` on the same two target dirs shows where the time goes.

## Under the hood

For the people who care. The reasoning is in [`docs/PLAN.md`](PLAN.md), the interfaces between the parts in [`docs/CONTRACTS.md`](CONTRACTS.md).

- **Base:** Ubuntu Server 26.04 LTS ("resolute"), minimal, with snapd, cloud-init and the Ubuntu Pro adverts ripped out. apt owns the system: kernel, firmware, drivers, Hyprland, PAM, Docker, Chrome. `/etc/os-release` says KubaOS where a person reads it and stays Ubuntu where apt and install scripts look.
- **Disk:** LUKS2, then btrfs with flat subvolumes (`@ @home @snapshots @log @cache @nix @docker @swap`), snapper with timeline and apt pre/post snapshots, grub-btrfs so snapshots are bootable from GRUB.
- **Session:** autologin on tty1 after the LUKS unlock, zsh, `start-hyprland`. No display manager.
- **Compositor:** Hyprland 0.56 from `ppa:cppiber/hyprland`, held in apt, with a Lua config split into files (`config/hypr/`). Dwindle layout, Omarchy 4 keybinds moved to hjkl. hypridle locks after 5 min, turns screens off after 10, and suspends after 30 on battery only, unless you change them (`kubaos-idle`).
- **Shell:** KubaOS's own, written in QML on Quickshell v0.3.2 (pinned), in the structure of Caelestia: bar, notifications, OSD, control center, power menu, wallpaper picker, lock screen with password, fingerprint and security key in parallel. The Command menu is [Kotodama](https://github.com/SuplenC/kotodama), drawn by the shell in its colors.
- **User environment:** standalone home-manager (nixos-26.05) owns everything in `$HOME`. KubaOS's configs are read-only links from the store into `~/.config`, and yours live in `~/.config/kubaos` (above, "Make it yours"). Nix apps get the GPU through home-manager's `targets.genericLinux.gpu`.
- **Theming:** matugen (`scheme-fidelity`) turns the wallpaper into Material 3 colors and renders 22 files in `~/.local/state/theme/` for every consumer, the pointer included: Bibata, recoloured. Geist, M PLUS 1 for the Japanese bits, KubaOS Symbols (Tabler's outline icons, drawn as SVGs), Colloid icons (orange).
- **Terminal and dev:** Ghostty (foot as the fallback), tmux + tms, Neovim 0.12 (LazyVim), rustup, proto + moon, sccache, clang + mold (wild optional), Docker Engine + lazydocker, Chrome.
- **Secrets:** Bitwarden desktop's SSH agent (private keys never touch the disk), `bw` for SSH Host blocks, git signing, the gh token and per-project env through direnv. A FIDO2 security key with a fingerprint sensor (the YubiKey C Bio is the tested one) does sudo, polkit and the lock screen; the password stays as the fallback.
- **Hardware:** `tuxedo-drivers` and power-profiles-daemon (no TUXEDO Control Center), fprintd, bolt for the dock.
- **Install:** Kōbō, an iced app, on a casper live ISO, with the Rust engine (`kubaos-core`, also behind the `kubaos` CLI) running inside it. It unpacks a prebuilt image with `/nix` and the home-manager generation already inside, which is why it needs no network.

### Repo map

| Path | What |
|---|---|
| `flake.nix`, `flake.lock` | Inputs (nixpkgs 26.05, home-manager, Quickshell v0.3.2) and the `kuba@tuxedo` / `kuba@vm` outputs |
| `home/` | home-manager modules: packages, zsh, tmux plugins, GPU wiring, GTK/Qt theme, defaults and the user layer (`links.nix`) |
| `config/` | The defaults, one dir per app, linked read-only into `~/.config`: `hypr`, `quickshell`, `nvim`, `tmux`, `tms`, `ghostty`, `foot`, `fuzzel`, `matugen`, `ohmyposh`, `lazygit`, `yazi`, `btop`, `bat`, `bacon`, `git` |
| `scripts/` | `dot-*` commands, linked to `~/.local/bin` (`dot-theme`, `dot-config`, `dot-hypr-*` including screenshots and `dot-hypr-record`, `dot-u2f-enroll`, `dot-ssh-sk-new`, and `dot-bw-setup` and `dot-bwenv`, aliases of `kubaos bitwarden` for one release) |
| `user-layer/` | The starting files of `~/.config/kubaos`, grouped by area: each app's commonly changed settings, commented out with their defaults, and the maps (`README.md`) |
| `system/` | Ubuntu bootstrap scripts (`10-strip.sh` to `75-yubikey.sh`), the package manifest `packages.toml` and its reader `manifest.sh`, `apt-repo.sh` and `relink.sh`, which `kubaos update` runs, files for `/etc` |
| `crates/` | The Rust workspace (root `Cargo.toml`): `kubaos-core`, the engine, `kubaos-cli`, the `kubaos` command, `kubaos-kobo`, Kōbō, `kubaos-cursor`, the pointer recoloured from the wallpaper, `kotodama-kubaos`, the Command menu's KubaOS actions and system toggles, `kubaos-files`, yazi in a terminal window and the Downloads tidy, `kubaos-bluetooth`, the Bluetooth pairing agent, `kubaos-automount`, the automount for removable drives and the photo import, and `kubaos-phone`, the phone link through KDE Connect |
| `install/` | Putting KubaOS on a machine: the payload build, the live ISO, the first-boot files and the engine's end-to-end test |
| `test/` | The Docker test harness (`test/run`) |
| `docs/` | `PLAN.md` (why), `INSTALL.md` (how), `CONTRACTS.md` and `INSTALL-INTERNALS.md` (interfaces), `SECRETS.md` (Bitwarden, security key), `TESTING.md`, `BRAND.md` (name and voice), `screenshots/` |

## Testing

Two benches, both on the x86_64 development machine.

- **`test/run`** (`just e2e`) builds an x86_64 Ubuntu 26.04 image with the repo's own `system/*.sh`, keeps the Nix store in a volume, and on every run does a real `home-manager switch` and the checks below. `test/run shots` starts a headless session with Quickshell for screenshots. Commands, caches and timings: [`docs/TESTING.md`](TESTING.md).
- **`test/vm/run`** drives QEMU on KVM: boots the live ISO, runs a headless install, boots the result, takes screenshots and snapshots. See [`docs/INSTALL-INTERNALS.md`](INSTALL-INTERNALS.md), "Test bench".

Verified in the harness's `ubuntu:26.04` container (user `kuba`, hostname `vm`, Nix installed with the official `--daemon` installer, daemon started by hand because there is no systemd):

- `system/10-strip.sh`, `20-base.sh`, `50-hyprland.sh` (the PPA's Hyprland) and `60-nix.sh` run clean; systemd, netplan and snapper steps skip themselves.
- `home-manager switch --flake .#kuba@vm` builds and activates; every file in `~/.config/<dir>` and `~/.local/bin/dot-*` is a link into the store.
- zsh login shell loads without errors (zoxide doctor quiet), PATH order, atuin Ctrl-R, direnv hook, oh-my-posh prompt, `.zprofile` sources the autostart snippet.
- `dot-theme apply` and `dot-theme set` write every file in `~/.local/state/theme/`; tmux, Neovim (LazyVim from the lockfile), Ghostty config validation and tms work with the real configs.
- PAM: through Nix's libpam (the same build Quickshell links), `qs-lock` accepts the right password and rejects a wrong one as a non-root user, using Ubuntu's `unix_chkpwd` linked to `/run/wrappers/bin`. Ubuntu's own modules don't load under Nix's libpam; details in `docs/INSTALL.md`.
- Secrets, against Vaultwarden 1.37 with the official `bw` 2026.10.0 (the standalone binary, sha256-checked by `70-bitwarden.sh`): `kubaos bitwarden setup` from logged-out to done (server prompt, password login, unlock), writes the `.pub` files, Host blocks (`ssh -G` picks the right `IdentityFile`), git signing (a commit verifies with "Good git signature" through an agent on `~/.bitwarden-ssh-agent.sock`) and keyring entries; rejects a vault field that tries to inject an ssh_config line; leaves no secret in plaintext under `$HOME`. direnv loads 5 variables (multi-line PEM and shell metacharacters intact) on `cd` in zsh and unloads them on leave; `direnv export` takes 22 ms. The agent was OpenSSH's on Bitwarden's socket path (Bitwarden desktop needs a desktop session and its own unlock); gh login was tested only up to GitHub rejecting a fake token.
- Security key PAM without a key: `sudo`, `sudo-i` and `polkit-1` with the pam_u2f line accept the right password in about 50 ms, reject a wrong one, also with no mapping or no mapping file, and through sudo-rs. Under Nix's libpam, Ubuntu's `pam_u2f.so` fails ("Module is unknown": `libfido2.so.1` not found) and nixpkgs' through the root-owned link loads and fails in 17 ms with no key. The lock screen runs `qs-lock-u2f` every 4 s while locked, stops when the mapping goes away, and shows the hint.
- Hyprland 0.56.2 (nixos-unstable) runs the Lua config unchanged in a NixOS VM (QEMU, virtio-gpu, llvmpipe): `--verify-config` passes, windows round and blur, panels open over windows without resizing them. Slow without KVM, so only a few stills.
- Quickshell v0.3.2 from Nix loads the full config in headless sway (Mesa llvmpipe OpenGL) with no QML errors, and `qs ipc show` lists every contract target. Every panel, the lock screen and the terminal stack were screenshotted there (`docs/screenshots/README.md`).
- Screen recording: `dot-hypr-record` start/stop, the state file and the bar indicator (elapsed time, gone on stop), stopped through `qs ipc call recorder stop`, with wf-recorder under headless sway; gsr's failure path without a GPU ("Recording failed" notification, state back to idle); satty opens from `dot-hypr-screenshot edit`. `system/65-recorder.sh` installs the setcap'd helper and is idempotent, but Docker refuses to exec a file capability, so gsr's KMS capture, VA-API encoding, region crop at scale 1.5 and replay need the laptop.

Needs the real machine: LUKS + Btrfs subvolumes and the rollback drill, Hyprland 0.56 from the PPA (amd64 only), the Lua config under a real compositor, GPU wiring (`non-nixos-gpu-setup`), fingerprint reader, tuxedo-drivers and power profiles, a dock with external monitors, lid and idle behaviour.

## Known limitations

- Smoothness at a high refresh rate, Hyprland's blur behind the shell and the workspace animations can only be judged on the machine.
- Fingerprint unlock in the Quickshell lock screen is untested: no reader in a container. nixpkgs' `pam_fprintd.so` loads and answers like Ubuntu's does without a reader. It's off now (`Settings.fingerprint`); hyprlock is the fallback.
- Untested without hardware: a real FIDO2 touch (sudo, polkit, lock screen), Bitwarden desktop's SSH agent and its polkit unlock (amd64 only), and `bw` with real two-step login. `bw` can't do FIDO2 two-step login; `kubaos bitwarden setup --login apikey` is the way in for a key-only account (`docs/SECRETS.md`).
- The in-shell polkit agent is off; hyprpolkitagent is the agent until the Quickshell one is tested on the machine.
- On ARM machines the session runs nixpkgs' Hyprland: the PPA is amd64 only, and Ubuntu's own build, 0.53, can't read the Lua config.
- grub-btrfs is built from source (not packaged for 26.04) and isn't tracked by dpkg.
- The official Nix installer has no uninstaller; the manual steps are in `docs/INSTALL.md`.

## Credits

- [Omarchy](https://omarchy.org) for the keybinds, per-screen workspaces (`config/hypr/workspaces.lua` adapts its `monitor-workspaces.lua`, MIT) and the idea that installing a desktop should take one stick and a few minutes.
- [Caelestia](https://github.com/caelestia-dots/shell) for the shell's structure: the frame, panels that grow out of it, the Material 3 motion. No code was taken; the shell here is written from scratch.
- [Quickshell](https://quickshell.org), [Hyprland](https://hyprland.org), [matugen](https://github.com/InioX/matugen), [home-manager](https://github.com/nix-community/home-manager) and Ubuntu, which do the actual work.
- The wallpapers are by Pema G. Lama, Alex Knight, Jezael Melgoza, Marek Okon, Manuel Cosentino, LoboStudio Hamburg, ayumi kubo, Letian Zhang and Lucius Hunter on Unsplash, and Bobby Brown on Pexels (each set's `CREDITS.md` in [SuplenC/kubaos-wallpapers](https://github.com/SuplenC/kubaos-wallpapers)).
- Built with Claude. Kuba made the calls; the robots typed.
