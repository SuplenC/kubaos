# ukiyo: the wallpaper that's paying attention

KubaOS's default wallpaper isn't a photo or a video. It's **ukiyo** (浮世), a live scene engine for Wayland that draws on the background layer and reacts to you: the camera follows the cursor, the street drifts when you switch workspace, signs flicker while you type, the rain gets heavier when the CPU works, and the gate flashes red or green when a build fails or passes. The first scene is **Kabukichō Gate**, the red arch of Kabukichō Ichiban-gai at night, in the rain.

ukiyo is its own project (`~/Projects/ukiyo`, Rust, wgpu). KubaOS is its first user, and everything KubaOS-specific lives here, in the dotfiles. This page is how the two fit together.

## The pieces

| What | Where |
|---|---|
| Package: `ukiyo`, `ukiyo-activityd`, the scenes in `share/ukiyo/scenes` | ukiyo's `flake.nix`, the `ukiyo` input of `flake.nix` |
| Daemon unit, declarative defaults | `home/ukiyo.nix` |
| zsh hook (builds pulse the gate) | `home/shell.nix` |
| `ukiyo:<id>` as a wallpaper, starting and stopping the daemon | `scripts/dot-theme` |
| Picker, settings panel, Motion mapping, lock frame | `config/quickshell/services/Ukiyo.qml`, `modules/wallpaper/`, `modules/lock/LockSurface.qml` |
| No pop-in animation for its layer | `config/hypr/rules.lua` |
| Checks | `test/lib/e2e.sh` (`c_ukiyo`) |

## A scene is a wallpaper

`ukiyo:kabukicho-gate` is a wallpaper the way a video in `~/Pictures/wallpapers/shinjuku/` is. `dot-theme set ukiyo:kabukicho-gate` takes the colours from the scene's poster (`ukiyo scenes --json` says where it is), writes `ukiyo:kabukicho-gate` wherever a path would go (`~/.local/state/theme/wallpaper`, `colors.json`, `hypr.lua`, `nvim.lua`), then starts the daemon and switches it to the scene. `next`, `prev` and the folder commands step through scenes too: they're listed first in `shinjuku/`. It's the first-boot default (`DEFAULT_WALLPAPER`), and `config/matugen/default-colors.json` comes from its poster.

Setting a still or a loop afterwards **stops** the daemon instead of pausing it. A paused daemon keeps its layer surface on the background layer, where it can sit above Quickshell's and cover the new wallpaper, and it keeps a few hundred MB of textures on the GPU. Starting again takes a fraction of a second, and the poster covers that. The stop waits 1.5 s, so Quickshell's layer is back before ukiyo's goes.

## The daemon

`ukiyo.service` is a systemd user unit, `PartOf` and `After` `graphical-session.target`, `Restart=on-failure`, logs in `journalctl --user -u ukiyo`. It has no `[Install]` section: `dot-theme` starts it when a scene becomes the wallpaper, and the `dot-theme apply` in Hyprland's autostart starts it at login when one already is. A session with a photo as its wallpaper runs no daemon at all. Without systemd (the Docker harness), `dot-theme` runs `ukiyo daemon` in the background instead.

The scenes come from the package's `share/ukiyo/scenes`, which the unit puts on `XDG_DATA_DIRS` (with `~/.nix-profile/share`, where the session finds them too).

## Defaults and your settings

ukiyo reads two files in `~/.config/ukiyo/`. `defaults.toml` is home-manager's, read-only, written by `home/ukiyo.nix`:

| Key | Value | Why |
|---|---|---|
| `scene` | `kabukicho-gate` | The default wallpaper |
| `performance.idle_timeout` | `600` | Quickshell's `Settings.screenOffAfter`: stop when hypridle turns the screens off |
| `performance.on_battery` | `still` | Motion's first-run mode is On AC |
| `signals.typing.source` | `idle` | The compositor's idle notifications. The exact typing rate needs `ukiyo-activityd`, a system service in the input group, which KubaOS doesn't install |

`config.toml` is yours. The settings panel and `ukiyo config set` write it, it wins over the defaults, and `ukiyo config reset <key>` (or Delete in the panel) takes a key back out.

## Picker and settings panel

The wallpaper picker (SUPER + CTRL + Space) lists the scenes first in the Shinjuku tab, with their poster as the thumbnail and a **live** badge. Choosing one runs `dot-theme set ukiyo:<id>`. The desktop keeps the poster on screen until the daemon reports it's drawing on every output, then fades its own layer out and unmaps it, so there's never a black frame whichever layer ends up on top. A preview from the picker brings Quickshell's layer back for as long as it lasts.

The picker's last tab, **Settings** (Tab to it, press S, or `qs ipc call wallpaper settings`), is the one place for wallpaper settings. Motion's mode (Always / On AC / Never) sits on top. Below it is every option ukiyo has, generated from `ukiyo config schema --json` and never written by hand: one row per option, grouped by `group`, the current scene's groups first under a header with "Reset scene". Bool is a switch, float a slider with its value and unit, choice a row of segments, colour a swatch and a hex field, string and list read-only text. Changes apply as you go (a slider sends at most about 10 values a second, and the last one when you let go). Each row says where its value comes from: **yours**, **KubaOS** (`defaults.toml`) or **default**.

Keys: Tab/Shift+Tab or Up/Down between rows, Left/Right change (Shift for 10 steps), Home/End jump to the ends, Space toggles, Enter activates (or edits a colour; Enter applies, Esc cancels), Delete resets the row (on the scene header: the whole scene), Esc goes back to the wallpapers, and Esc again closes.

## Motion, one source of truth

Motion (`services/Motion.qml`) decides when moving wallpapers may move. For a scene it drives the daemon:

| Motion | ukiyo |
|---|---|
| `desktopOn(screen)` false on every screen (mode, battery, screens off, fullscreen) | `pause --reason motion` |
| true on any screen again | `resume --reason motion` |
| locked | `still` |
| unlocked | `resume` |

The modes are also ukiyo settings, so the two can't disagree:

| Motion mode | ukiyo config |
|---|---|
| Always | `performance.on_battery = full` (or `reduced`, if you chose that) |
| On AC | `performance.on_battery = still` |
| Never | `performance.motion = false` |

It works both ways. Changing the mode (the panel's top row, the control center tile, `qs ipc call wallpaper animate`) writes ukiyo's config, and changing those two options in the panel or with `ukiyo config set` changes the mode. `reduced` counts as Always: it still moves off the charger, just cheaper.

## The lock screen

The session lock covers the background layer, so a live scene can't show through it. On lock, Quickshell tells ukiyo `still`. ukiyo saves the frame that's on each screen at that moment to `$XDG_RUNTIME_DIR/ukiyo/still-<output>.png` (atomically), eases to a standstill and stops drawing. The lock screen shows that frame, blurred like any wallpaper, with the poster standing in until the file exists. Each lock has a new serial, so the image is never a stale one.

Why not draw the scene live under the lock: the still costs zero GPU while locked, it's exactly the last frame, so the blur-in is seamless, and a second live renderer in QML would mean writing the scene twice, in another shader language, and burning battery behind a lock nobody's looking at.

## Builds pulse the gate

`home/shell.nix` sources `ukiyo shell-hook zsh`, generated once at build time into the store, so no shell start runs ukiyo. After each command the hook sends the command line, exit status and duration down the daemon's socket with `zsocket`: no process per prompt, and nothing at all when no daemon runs. The daemon decides what's a build (`signals.build.commands`: cargo, just, make, nix and friends, or anything longer than `signals.build.min_duration`) and pulses `build_failed` or `build_passed`. Fail a `cargo test` and the arch burns red.

## The flake input and Docker

ukiyo comes in as a flake input from GitHub:

```nix
ukiyo = {
  url = "github:SuplenC/ukiyo/main";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

To build against a local checkout instead, set `UKIYO_OVERRIDE=~/Projects/ukiyo` (a directory; `UKIYO_REF=<branch>` picks a branch other than its HEAD) for `installer/image/build`, `test/run` or `just`; it passes `--override-input ukiyo git+file://...` and bind-mounts the checkout into the container. Any `git+file://` or `path:` input recorded in `flake.lock` is bind-mounted the same way (read-only at its own path, a worktree's main `.git` too, marked as a git `safe.directory`, because the mounts are owned by root in there). The helper is `test/lib/local-inputs.sh`.

The payload's `nix flake archive` copies the input into the store like any other, so `home-manager switch` on the installed laptop works offline, without the Mac's path. `installer/image/check` proves it by evaluating the flake with `--offline` in the unpacked tree.

## Costs

ukiyo is a GPU program. It hasn't been measured on the laptop's Radeon yet. With software rendering (the QEMU bench, the Docker harness, both llvmpipe) it's expensive: there, use `performance.scale=0.5` and `performance.fps=15`. The numbers from the VM bench are below.

Measured 2026-10-17 on the aarch64 VM bench (HVF, 4 vCPU, 4 GB, 1920x1080, llvmpipe: Homebrew QEMU has no GL), Kabukichō Gate on an idle desktop, ukiyo 0.1.0 from the `nix` branch (677cc26 plus the packaging fix). FPS is `ukiyo status`'s output line, CPU is `top` for the daemon (100% is one of the 4 vCPUs), taken over 15 s.

| | FPS | Frame time | ukiyo CPU |
|---|---|---|---|
| Defaults (`scale=1.0`, `fps=60`) | 8 | about 125 ms | about 215% |
| `performance.scale=0.5`, `performance.fps=15` | 12 | about 83 ms | about 130% |

At the defaults the 60 fps cap never bites: llvmpipe renders the full-size frame in about 125 ms. At half scale it still doesn't reach the 15 fps cap, so the lower setting saves about 40% of the daemon's CPU and gains a third more frames, but the machine is still saturated (97% busy overall, Hyprland's compositing takes the rest). These numbers say nothing about the laptop's Radeon 890M, where the cap is expected to be what limits the frame rate. They're the reason software rendering needs the two settings above.
