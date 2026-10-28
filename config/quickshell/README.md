# Quickshell desktop shell

KubaOS's desktop shell: my own, for Hyprland, written in QML on [Quickshell](https://quickshell.org) **v0.3.2**, the version pinned in `flake.lock`.

## Design

Tokyo at night, in the structure of [Caelestia](https://github.com/caelestia-dots/shell). Every screen gets a frame: the bar along the top and a thin border on the other three edges, with rounded inner corners. Panels are not floating cards. They grow out of the frame's edges and meet it through concave fillets, so the frame and every open panel read as one surface with one soft shadow. The colors come from the wallpaper (matugen, `scheme-fidelity`), which keeps the photo's own neon hue in the accents: amber and cyan on the Shinjuku street, violet and gold at purple hour.

- **Look:** `docs/DESIGN.md`. The bar, the frame, the panels, the control center, the notifications, the popups, the Command menu, the lock screen, the OSD, the power menu and the polkit prompt are built from `KubaOS.Components` and its tokens alone (`import KubaOS.Components as K`); `just design-check` holds them to it.
- **Type:** the type styles of `docs/DESIGN.md`: Geist for the interface, Geist Mono for numbers that tick (Geist's figures are proportional, so a ticking Geist number would jitter), M PLUS 1 for the few Japanese accents: the weekday kanji next to the bar clock, 曜日 in the control center, and 東京 on the lock screen. Icons are KubaOS Symbols, drawn outlines.
- **Motion:** the duration and curve tokens. A panel arrives over `long` (500 ms) on emphasized decelerate and leaves over `medium` (300 ms) on emphasized accelerate. Opening a panel animates its depth, and the content rides on its leading edge, so it slides out from under the frame instead of fading in place. The workspace indicator stretches: its leading edge moves first and the trailing edge follows. The workspace pill changes shape in a glide, never a jump: the group the screen shows ends in a faint `+`, the ghost slot where next creates a new workspace (a click does the same; none once the group has all ten). A new workspace fills it in place and a new ghost grows after it, a workspace that goes away shrinks out, and the cells, the gaps between groups, the pill's width and the window title beside it follow. The Command menu moves on the design tokens, one step quicker than a panel (docs/DESIGN.md, "Motion"): the window arrives over `short` and leaves over `micro`, a View slides in over `medium`, and the highlight glides between rows over `short`. What a keystroke changes shows at once, never animated, so typing never waits on a frame.
- **Placement:**

  | Panel | Position |
  |---|---|
  | Command menu (Kotodama) | Its own window on the focused screen, centred, with two fifths of the free height above it. Its width and rows follow the screen; on screens at least 1472 logical pixels wide it has a detail pane |
  | Control center | Side sheet on the right |
  | Notifications panel | Side sheet on the right, in the control center's place; one or the other |
  | Power menu | Column on the right edge |
  | Wallpaper picker | Rises from the bottom |
  | OSD | Small bar rising from the bottom |
  | Notification popups and the bell's peek | Top-right corner. While a panel reaching into that corner is open (control center, notifications panel, system monitor, power menu, or the clock popup or wallpaper picker on a narrow screen), they wait and come back when it closes; a critical one shows out of the right edge under the panel, or waits too if the panel reaches the bottom |
  | Clock popup | Under the bar's clock |
  | System monitor | Under the bar's right end, by its pill |

- **Density:** sized for Full HD at scale 1 (the laptop runs at 1.5, see `config/hypr/monitors.lua`): a 40 px bar (`size.bar`) of 32 px pills, 14 px body text.
- **Panels over windows:** only the frame reserves space (the bar's 40 px and 8 px on the other edges, `Exclusions.qml`). Panels, notifications and the OSD reserve nothing: they float over the tiled windows, so nothing resizes or reflows when one opens. Modal panels dim the desktop behind them (`opacity.panel_scrim`, 20%; the power menu as a dialog does, `opacity.scrim`, 60%) so a window's text doesn't read as part of the panel; the wallpaper picker stays undimmed because the desktop is its preview. Screenshots: `docs/screenshots/v3/`.
- **Glass:** the frame is drawn at 94% opacity (`opacity.frame`). Hyprland blurs what shows through (`qs-frame` layer rule, `ignore_alpha 0.5`, so the shadow is not blurred).

The design was studied from Caelestia's source and README: its frame and drawer structure, Material 3 roles and motion tokens. No code comes from it (Caelestia is GPL-3.0); everything here was written from scratch, and the panel shapes are built differently (an SVG path per panel, see `KubaOS.Components`' `sheetpath.js`).

Screenshots: `just shots` (see `docs/TESTING.md`).

## Run and reload

```sh
qs                      # starts ~/.config/quickshell (config/quickshell, from the store)
qs log                  # follow the log of the running instance
qs ipc show             # list IPC targets
```

Quickshell hot-reloads on save. `~/.config/quickshell` is a read-only copy (`home-manager switch` updates it and restarts the shell, `scripts/dot-session`), so to hack on the shell run a clone of KubaOS instead (docs/USING.md, "Hacking on KubaOS"): `qs kill; qs -p ~/Projects/kubaos/config/quickshell`. Lock state, do-not-disturb and notifications survive a reload.

## IPC (bound in Hyprland)

| Call | Effect |
|---|---|
| `qs ipc call session lock` | Lock screen |
| `qs ipc call session locked` | `true` while locked |
| `qs ipc call powermenu toggle` | Lock / suspend / log out / reboot / shut down |
| `qs ipc call wallpaper toggle` | Wallpaper picker |
| `qs ipc call wallpaper settings` | Wallpaper picker on its Settings tab (Motion) |
| `qs ipc call wallpaper animate <always\|ac\|never>` | Wallpaper motion, remembered across restarts |
| `qs ipc call wallpaper status` | What's shown, which screens play a video (JSON) |
| `qs ipc call notifications toggle` | The notifications panel (SUPER+N) |
| `qs ipc call notifications toggleDnd` / `dnd on\|off` | Do not disturb on/off |
| `qs ipc call notifications clear` | Dismiss all notifications |
| `qs ipc call controlcenter toggle` | Wi-Fi, Bluetooth, audio, power profile |
| `qs ipc call controlcenter bluetooth` | The control center on its Bluetooth tab, searching for devices |
| `qs ipc call controlcenter keyboard` | The control center on its Keyboard tab |
| `qs ipc call recorder toggle` / `stop` | Record the focused monitor / stop any recording or replay (`dot-hypr-record`) |
| `qs ipc call clock toggle` / `settings` | The clock popup under the bar's clock / on its settings page |
| `qs ipc call clock search <text>` | The clock popup with a date search typed in |
| `qs ipc call clock timer <duration>` / `alarm <time>` / `stopwatch` | Start a timer, set an alarm, start or stop the stopwatch |
| `qs ipc call clock status` | The engine, timer, stopwatch, alarms and whether the popup is open (JSON) |
| `qs ipc call sysmon toggle` / `open` | The system monitor's popup under the bar's pill |
| `qs ipc call sysmon process <pid>` | The popup on a process's details, with End and Kill |
| `qs ipc call sysmon pill <on\|off\|toggle>` | The bar's monitor pill, remembered in `sysmon.toml` |
| `qs ipc call sysmon act <action>` | What an alert's button runs: `open`, `process:<pid>`, `files:<mount>`, `power-saver` |
| `qs ipc call sysmon status` | The engine, its sampling mode, the pill's settings, whether a sample came, whether the popup is open (JSON) |

All handlers live in `IpcHandlers.qml`. `docs/CONTRACTS.md` is the source of truth.

## File map

```
shell.qml               entry point: per screen a Wallpaper and a Frame; Lock, polkit, IPC once
IpcHandlers.qml         qs ipc targets
theme/Theme.qml         colors.json fed to the design tokens, and where the theme's state lives
settings/Settings.qml   user knobs: wallpaper folders, wallpaper motion, lock loop, kanji workspaces, security key, fingerprint, polkit
services/               singletons, no UI
  Audio                 PipeWire default sink, volume, output list
  Brightness            /sys/class/backlight (watched) + brightnessctl
  Battery               UPower battery
  Power                 the power stops, from kubaos-power's file (power-profiles-daemon's profile without it)
  Net, Bt               NetworkManager and BlueZ via Quickshell.Networking / .Bluetooth, pairing through kubaos-bluetooth
  Notifs                notification server, popups, history, unread, DND (dnd.js)
  Wallpapers            current wallpaper, picker preview, folder listing (stills, videos, posters), `dot-theme set`
  Motion                when moving wallpapers may decode: always / on AC / never (persisted), lock, idle, fullscreen
  Hypr                  focused screen, each screen's block of workspaces, workspace switching
  Panels                which panel is open, and on which screen
  MenuRequests          `kotodama toggle` / `open` an embedded KotodamaSearch got, handed on to the Command menu
  Session               lock state, suspend/logout/reboot/poweroff
  Recorder              screen recording state ($XDG_RUNTIME_DIR/dot-record.json, watched), dot-hypr-record calls
  Idle                  Keep awake and the scheduled action ($XDG_RUNTIME_DIR/kubaos-keep-awake.json,
                        kubaos-scheduled.json, watched), kubaos-idle calls
  Time                  the clock engine (`kubaos-clock serve`, line JSON), clock.toml (watched),
                        the timer, stopwatch and alarms
  Sysmon                the system monitor's stream (`kubaos-sysmon serve`, line JSON, paced by
                        the popup and the lock), the graphs' last minute, sysmon.toml (watched),
                        alerts as notifications
components/             blocks composed from KubaOS.Components
  Pill                  the bar's pill: a row of content, hover, click, right-click and wheel
  NavList               the keyboard list over K.ListRow rows
  PowerSlider           the power stops on a K.Slider
  BadgedIcon            a bar symbol with a K.Badge on its corner
  SkeletonRow           a two-line list row still being found, as K.Skeletons
  WallpaperView         a still, or a video looping over its poster (desktop, lock screen, picker)
  Countdown             a popup's time left, stopped while the pointer is on it
modules/
  frame/                Frame (one layer per screen: bar, border, every panel, input mask),
                        Exclusions (exclusive zones as four empty windows)
  bar/                  Bar, Workspaces, ActiveWindow, Clock, Tray, SysmonPill, Bell, Status, RecordingIndicator,
                        IdleIndicator
  menu/                 Menu: the Command menu on KotodamaSearch, built from KubaOS.Components (its
                        sizes for the screen, its motion). ResultRow, DetailPane + PaneContent,
                        ActionPanel, FormView, ProgressView, DetailView, IntroCard, Label (text that
                        changes with every key), words.js (glyphs, kinds, shortcuts),
                        Probe (test-only timing and state lines)
  controlcenter/        ControlCenter, QuickToggles (K.Tile), CaptureRow, Sliders, Wi-Fi/Bluetooth/Output lists in a Well,
                        KeyboardList + KeyboardAddView (its search, a KotodamaSearch in the layouts app)
  notifications/        NotificationPanel, Popups (corner, and the bell's Peek), PopupsBelow (critical, under a panel), ToastList, NotificationCard, feed.js (stacks, New and Earlier, the peek), ToastProbe (tests)
  osd/                  volume / brightness OSD
  lock/                 Lock (WlSessionLock), LockSurface, Auth (PAM)
  powermenu/            PowerMenu
  clock/                ClockPopup (the bar clock's popup), the month on K.Calendar (month and year views),
                        AnalogFace, Tools (timer, stopwatch, alarms), ClockSettings, Chip,
                        Figure (a ticking number in a box as wide as its widest digits)
  sysmon/               SysmonPopup (the monitor pill's popup), Section, Processes (top five,
                        details, End and Kill)
  wallpaper/            Wallpaper (background, crossfade, preview), WallpaperPicker,
                        WallpaperSettings (the picker's Settings tab)
  polkit/               Polkit agent + dialog (off by default)
```

### How the frame works

`modules/frame/Frame.qml` is one transparent full-screen layer (`qs-frame`, layer Top) per screen. Its input mask only covers the frame, the bar and the notification popups, so clicks elsewhere reach the windows. While a panel is open, the mask covers the whole screen, keyboard focus is exclusive, and a click outside closes the panel. The frame path (screen rectangle minus a rounded inner rectangle) and each panel's path (`K.Sheet`'s `path`) are drawn in one `Shape` inside one layer. That layer carries the opacity and a single `MultiEffect` shadow, so overlaps never double up and panel and frame merge. Panel contents sit above it at full opacity. The exclusive zones come from four empty 1x1 windows (`Exclusions.qml`, `qs-exclusion`), so the frame window never has to move. On a fullscreen workspace the frame retracts (`reveal` 1 -> 0) and comes back for an open panel, switching to the Overlay layer.

### Performance

- Paths are rebuilt only while a panel moves, and are short strings.
- The shadow layer re-renders only while something in it changes.
- Images decode at their displayed size in device pixels: `sourceSize` × `devicePixelRatio`, and half resolution for the blurred lock background.
- Nothing runs per-frame JavaScript.

In the headless test the GUI thread spent 0 ms (median) per frame on polish, animations and sync. All frame time was software rasterization, see `docs/TESTING.md`.

Imports use Quickshell's module paths: `import qs.services`, `import qs.components`, and so on.

## Keyboard

Every panel works without a mouse. Esc closes it.

- Command menu (SUPER + Space, `kotodama toggle`): type to search apps, projects, scripts and windows, Up/Down to move, Enter to run. A sum typed is answered in the list; Enter copies the answer. SUPER + F1 opens it on the Shortcuts View. Ctrl+K opens the selected row's Actions, Tab enters an app, Esc steps back (the Action panel, a View, the query, then closed), Backspace with nothing typed leaves an app. Wide screens show the selected row in a detail pane: its Actions with their keys, and its hints. `"menuMotion": "fades"` in `~/.config/kubaos/desktop/quickshell/settings.json` turns its motion down to fades. Its config is `config/kotodama/kubaos.toml`, with your changes in `~/.config/kubaos/menu/kotodama/user.toml`.
- Control center: Tab/Shift+Tab move between controls, Enter/Space toggle a tile, h/l or arrows move sliders and tabs, 1-4 jump to a tab, j/k move in lists.
- Notifications panel: ↑↓ (j/k) move, Enter opens, → (l) opens a stack and ← (h) folds it, Delete dismisses, D turns Do not disturb on or off, Esc closes.
- Clock popup (click the bar's clock): type a date or date maths to search, Enter copies the answer, Ctrl+Enter copies it as ISO. With nothing typed, Left/Right or the wheel change month and T goes to today; clicking the month name opens the year view. Esc leaves the settings page or the year view, then closes.
- System monitor (click the bar's monitor pill, right-click it to hide it): click a process for its details, then End or Kill and confirm. Esc closes.
- Power menu: h/j/k/l, arrows or Tab, then Enter.
- Wallpaper picker: h/j/k/l or arrows move (the desktop previews the selection), Tab switches folder, S opens Settings, Enter applies, Esc goes back to the current wallpaper.
- Wallpaper settings (the picker's last tab): Tab/Shift+Tab or Up/Down between rows, Left/Right change a value (Shift: 10 steps), Home/End to the ends, Space toggles, Enter activates or edits a colour, Delete resets to the default, Esc back to the wallpapers.
- Lock screen: type the password and press Enter. Esc clears the field.
- Polkit prompt: type the password and press Enter. Tab reaches Cancel and Authenticate; Esc cancels.

## Theme

The values come from the design tokens (`design/tokens.toml`, `docs/DESIGN.md`) through `KubaOS.Components`, the QML component library in `qml/KubaOS/Components`: its generated `Tokens` singleton holds every colour role, length, type style and duration. Every surface of the shell reads them directly (`import KubaOS.Components as K`); new code takes `Tokens` and the library's components too.

`Theme.qml` watches `~/.local/state/theme/colors.json` (written by `dot-theme`/matugen) and hands it to `Tokens.matugen`, so the roles follow the wallpaper live; colors crossfade where components animate them. If the file is missing, the roles fall back to the tokens' own palette (`[color.dark]` and `[color.light]`, matugen's output for the default wallpaper). Color roles are read as `K.Tokens.color.<role>` (`K.Tokens.color.primary`, `K.Tokens.color.on_surface`, ...).

`KubaOS.Components` reaches Quickshell as a QML module (`qml/package.nix`, added to Quickshell's `withModules` in `home/curated.nix`), so the shell uses the library of the release its home was built from. `QML_IMPORT_PATH` can't put a clone's `qml/` in front of it: Quickshell's Nix wrapper puts its own modules ahead of any import path you set. To try changes to the library in the shell, switch your home onto the clone (`docs/USING.md`, "Hacking on KubaOS"), which builds the library from it, then run the clone's shell with `qs -p` as above; repeat the switch after each change to the library. `nix run .#checks.x86_64-linux.qml-specimen -- shots DIR` draws the clone's components page without touching the shell, and `just design-parity` holds it to iced's. `Tokens.qml` and `Symbols.qml` come from `design/tokens.toml` in the same build; the repo doesn't keep them, and `just tokens` writes them into a checkout for an editor.

## Adding a module

1. Create `modules/<name>/<Name>.qml`. For a bar item, start from `Pill`. For a panel, start from `K.Sheet { edge: "top" | "bottom" | "left" | "right" }`. Set `contentWidth`/`contentHeight` from your content's implicit size; `align`, `flushStart`/`flushEnd` place it along the edge. Use only `KubaOS.Components` and its `Tokens` for sizes, colours and motion (`docs/DESIGN.md`).
2. Put data and side effects in a singleton under `services/` (`pragma Singleton` + `Singleton {}`), so UI files only bind to it.
3. Add it to `modules/frame/Frame.qml`, with `frame: geo` and `shown: root.panel === "<name>"`, and add a `PanelPath { drawer: <id> }` to the background `Shape`. Or add it to `Bar.qml`.
4. If Hyprland should open it, add an `IpcHandler` to `IpcHandlers.qml` that calls `Panels.toggle("<name>")`, and add the row to `docs/CONTRACTS.md`.

## PAM and polkit

The lock screen runs PAM conversations in parallel, and any one unlocks:

- `/etc/pam.d/qs-lock`: password
- `/etc/pam.d/qs-lock-u2f`: FIDO2 security key (`Settings.u2f`, on). Runs only when `/etc/u2f_mappings` has a key for you (`dot-u2f-enroll`), restarts every 4 s so a key plugged in later works, and shows "Or use your security key", then "Touch your security key" once pam_u2f has found it. A key that wants a PIN can't be answered here; the password still works.
- `/etc/pam.d/qs-lock-fingerprint`: fprintd reader (`Settings.fingerprint`, off since the security key replaced the USB reader), restarted after each failed scan, and skipped silently if there is no reader

The files come from `system/`. Quickshell is built by Nix and links Nix's libpam, which can't load Ubuntu's PAM modules, so the files use Nix's `pam_unix.so` (with Ubuntu's `unix_chkpwd` linked to `/run/wrappers/bin`) and nixpkgs' `pam_u2f.so` and `pam_fprintd.so` through root-owned links; `docs/INSTALL.md` and `docs/SECRETS.md` have the details. Password auth is verified in a container. Test it on the machine on day one: lock with `qs ipc call session lock` while a second TTY is logged in. If unlocking fails, hypridle's `|| hyprlock` fallback stays the lock screen. If the shell crashes while locked, Hyprland keeps the screen locked: switch to a TTY and start `hyprlock` against the session.

The polkit agent is in-shell but turned off (`Settings.polkitAgent: false`), and hyprpolkitagent stays the agent. To try the in-shell one, set the flag to true and set `POLKIT_AGENT = "quickshell"` at the top of `config/hypr/autostart.lua` (only one agent can register per session), then restart the session.

## Runtime dependencies

| Tool | Used for | From |
|---|---|---|
| `kotodama`, `kotodamad` | the Command menu's Daemon and CLI | Nix (`home/menu.nix`) |
| `brightnessctl` | setting brightness | apt |
| `dot-theme` | applying a wallpaper | this repo |
| `kubaos-clock`, `kubaos-sysmon` | the clock popup's dates, the system monitor's stream | this repo, Nix (`home/clock.nix`, `home/sysmon.nix`) |
| `kubaos-idle` | stopping Keep awake, cancelling a scheduled action (the bar's marker) | this repo, Nix (`home/idle.nix`) |
| `btop` | the system monitor's full picture | Nix (`home/cli.nix`) |
| `dot-hypr-record`, `dot-hypr-screenshot` | recording indicator, capture buttons | this repo |
| `dot-session leave` (then `systemctl`, `loginctl`) | power menu: log out, reboot, shut down | this repo |
| `systemctl` | power menu: suspend | system |
| NetworkManager, BlueZ, PipeWire, UPower, power-profiles-daemon | services over D-Bus | apt |
| Fonts: Geist, Geist Mono, M PLUS 1 (the icons are KubaOS Symbols, SVGs in `KubaOS.Components`) | UI, machine text and keycaps, Japanese accents | Nix (`home/desktop.nix`) |
