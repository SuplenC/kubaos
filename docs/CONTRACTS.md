# Contracts between the parts of KubaOS

Every part of KubaOS is built against these interfaces. If you need to change one, update this file in the same commit.

Target machine: TUXEDO InfinityBook Pro 14 Gen10 (Ryzen AI 9 HX 370, AMD iGPU), Ubuntu Server 26.04 LTS minimal, Hyprland 0.56+ (from `ppa:cppiber/hyprland`, Lua config), Quickshell pinned to a tagged release via Nix, standalone home-manager (flake). Full context: `docs/PLAN.md`.

## Repo location and ownership

- The repo lives at `~/Projects/dotfiles` on every machine. Nix code refers to it as `dotfilesDir = "${config.home.homeDirectory}/Projects/dotfiles"`.
- Linux username is a single variable in `flake.nix` (default `kuba`). Nothing else hardcodes it.
- Hostnames: `tuxedo` (laptop, x86_64) and `vm` (UTM test VM, aarch64). Flake outputs are `<user>@tuxedo` and `<user>@vm`, so plain `home-manager switch --flake ~/Projects/dotfiles` picks the right one.
- Flakes only see files git knows about: `git add` new files under `config/`, `scripts/`, `home/` before `home-manager switch` (no commit needed).

| Path | Owner | What |
|---|---|---|
| `flake.nix`, `flake.lock`, `home/default.nix`, `home/links.nix`, `home/cli.nix`, `home/dev.nix`, `home/desktop.nix` | nix | Flake, module wiring, live links, packages, cargo config (linker switch), Quickshell + GPU wiring, fonts |
| `home/shell.nix`, `home/terminal.nix`, `config/{nvim,tmux,tms,ghostty,ohmyposh,lazygit,btop,git,bat,bacon,yazi,foot,fuzzel}` | ports | zsh, tmux, tms, ghostty, ported user configs, the fallback terminal and launcher |
| `config/hypr/`, `scripts/dot-hypr-*` | hypr | Hyprland Lua config, hypridle, lid and monitor scripts, screenshots and screen recording |
| `config/quickshell/`, `scripts/dot-qs-*` | quickshell | The desktop shell |
| `home/theme.nix`, `config/matugen/`, `scripts/dot-theme*`, `wallpapers/` | theme | Wallpaper + matugen color pipeline, the wallpapers, GTK/Qt/cursor/icons |
| `system/`, `docs/INSTALL.md` | system | Ubuntu bootstrap scripts and `/etc` files |
| `home/secrets.nix`, `scripts/dot-bw*`, `scripts/dot-u2f-enroll`, `scripts/dot-ssh-sk-new`, `system/70-bitwarden.sh`, `system/75-yubikey.sh`, `system/files/pam.d/`, `docs/SECRETS.md` | secrets | Bitwarden (SSH agent, git signing, gh, project env), the security key (YubiKey C Bio: pam_u2f, ed25519-sk) |
| `README.md`, `docs/CONTRACTS.md`, `docs/PLAN.md` | lead | Overview and these contracts |

## Live links (home/links.nix)

- Every directory directly under `config/` is linked to `~/.config/<name>` with `config.lib.file.mkOutOfStoreSymlink "${dotfilesDir}/config/<name>"`. `links.nix` discovers them with `builtins.readDir ../config`, so adding a new `config/<name>/` needs no Nix change.
- Every file in `scripts/` is linked to `~/.local/bin/<name>`. Scripts are executable, start with `#!/usr/bin/env bash` (or another env shebang), use `set -euo pipefail`, and are prefixed `dot-`.
- **Never** manage a path that is live-linked with a `programs.<x>` option that writes the same file (for example `programs.tmux.extraConfig` or `programs.ghostty.settings`). Install the package with `home.packages` or `programs.<x>.enable` only if that option writes nothing under the linked path. If Nix needs to hand a store path to a live config (tmux plugins, for example), write it to a generated file outside the linked dir, such as `~/.config/tmux-nix/plugins.conf` via `xdg.configFile."tmux-nix/plugins.conf"`, and have the live config `source` it if it exists.

## Theme pipeline

- Wallpapers: `~/Pictures/wallpapers/{shinjuku,japanese,cyberpunk}/`. A wallpaper is an image (jpg, jpeg, png, webp) or a video (mp4, webm, mkv, mov). A video's still is `<name>.poster.jpg` next to it, else `~/.local/state/theme/posters/<md5 of the path>.jpg`, which `dot-theme` cuts with ffmpeg when needed. Posters are never listed as wallpapers. `~/Pictures/wallpapers` is a live link (home-manager, out-of-store) to the repo's `wallpapers/`, which holds the committed images and `CREDITS.md`. `dot-theme` keeps paths under `~/Pictures/wallpapers` as given (it does not resolve the link), so `wallpaper` and `colors.json` always name that path.
- Scheme: matugen `scheme-fidelity`, set once in `scripts/dot-theme` (`SCHEME`). First boot with no state uses `DEFAULT_WALLPAPER` (`ukiyo:kabukicho-gate`).
- Live scenes: `ukiyo:<id>` is a wallpaper too, a scene of the ukiyo daemon (`docs/UKIYO.md`). Its colors come from the scene's poster (`ukiyo scenes --json`), and `ukiyo:<id>` stands where a path would in `wallpaper`, `colors.json`, `hypr.lua` and `nvim.lua`. Scenes are listed first in the `shinjuku` folder. `dot-theme` starts `ukiyo.service` (or a plain `ukiyo daemon` without systemd) to show one and stops it when a still or a loop takes over.
- Command: `dot-theme` (owned by theme).
  - `dot-theme set <file|ukiyo:id>`: set wallpaper and regenerate colors. For a video matugen reads the poster, then `dot-theme` writes the video's path as `wallpaper` in `colors.json`, `hypr.lua` and `nvim.lua`
  - `dot-theme next` / `dot-theme prev`: next/previous wallpaper (stills and videos) in the current folder
  - `dot-theme folder <shinjuku|japanese|cyberpunk>` and `dot-theme toggle-folder` (cycles all three)
  - `dot-theme apply`: regenerate from the current wallpaper (used at login). Skips matugen and the reloads when the wallpaper (path, mtime, size; for a video also its poster's), the scheme, the matugen binary and the content of `config/matugen/config.toml`, `config/matugen/templates/*` and the oh-my-posh template all match the last successful run, and every output file exists. The record is `~/.local/state/theme/.stamp`, written after each successful run. `dot-theme apply --force` always regenerates; `set`, `next`, `prev`, `folder` and `toggle-folder` always do.
  - `dot-theme current`: print current wallpaper (path, or `ukiyo:<id>`)
- Generated output (never committed) goes to `~/.local/state/theme/`:

| File | Consumer | Format |
|---|---|---|
| `colors.json` | Quickshell | Flat JSON object, keys below, values `"#rrggbb"`, plus `"wallpaper": "<abs path or ukiyo:id>"` and `"mode": "dark"` |
| `wallpaper` | Quickshell | Plain text, absolute path of current wallpaper, or `ukiyo:<id>` for a live scene |
| `tmux.conf` | tmux (`source-file -q`) | tmux `set -g` lines defining `@thm_*` user options and status/pane/message styles |
| `ghostty` | Ghostty (`config-file = ?~/.local/state/theme/ghostty`) | Ghostty palette, background, foreground, cursor, selection |
| `nvim.lua` | Neovim | Lua file returning a table with the same keys as `colors.json`, plus `red green yellow blue magenta cyan` (harmonized syntax hues; consumers fall back to M3 roles without them) |
| `hypr.lua` | Hyprland | Lua file returning a table with the same keys as `colors.json` |
| `hyprlock.colors` | hyprlock (`source =`) | hyprlang variables, bare hex without `#`: `$primary = rrggbb` |
| `ohmyposh.toml` | oh-my-posh | Complete oh-my-posh config rendered from `config/ohmyposh/config.toml.tmpl` (or palette-only, ports and theme agree) |
| `gtk.css`, `qt6ct.colors` | GTK 3/4, qt6ct | |
| `fuzzel.ini`, `foot.ini` | fuzzel, foot | `[colors]` sections, pulled in by `include=` from `config/fuzzel/fuzzel.ini` and `config/foot/foot.ini` |
| `lazygit.yml` | lazygit | `gui.theme` only. zsh appends it to `LG_CONFIG_FILE` after `~/.config/lazygit/config.yml`, only when the file exists (lazygit refuses a missing file) |
| `lazydocker.yml` | lazydocker | `gui.theme`; `~/.config/lazydocker/config.yml` is an out-of-store link to it (`home/cli.nix`) |
| `yazi.toml` | yazi | A `theme.toml` (colors only). `config/yazi/theme.toml` is a committed relative link to it (`../../../../.local/state/theme/yazi.toml`, valid because the repo lives at `~/Projects/dotfiles`) |
| `btop.theme` | btop | btop theme file; `config/btop/themes/matugen.theme` is a relative link to it, `btop.conf` has `color_theme = "matugen"` and `theme_background = false` |
| `delta.gitconfig` | git, delta | `[delta]` syntax theme, plus/minus styles, decorations; `[include]`d by `config/git/config` (git skips a missing include) |
| `fzf` | fzf | `--color` lines for `FZF_DEFAULT_OPTS_FILE`, exported by zsh when the file exists |
| `atuin.toml` | atuin | atuin theme; `~/.config/atuin/themes/matugen.toml` links to it, `theme.name = "matugen"` (`home/shell.nix`) |
| `eza.yml` | eza | eza `theme.yml`; `~/.config/eza/theme.yml` links to it (`home/cli.nix`) |
| `glow.json` | glow | glamour style; the zsh `glow` wrapper passes `-s` when the file exists |

- Color keys (matugen Material 3 names, snake_case): `primary on_primary primary_container on_primary_container secondary on_secondary secondary_container on_secondary_container tertiary on_tertiary tertiary_container on_tertiary_container error on_error error_container on_error_container background on_background surface on_surface surface_variant on_surface_variant surface_dim surface_bright surface_container_lowest surface_container_low surface_container surface_container_high surface_container_highest outline outline_variant shadow scrim inverse_surface inverse_on_surface inverse_primary`.
- A committed fallback `config/matugen/default-colors.json` (same schema, generated from the default wallpaper) is copied to `~/.local/state/theme/colors.json` by `dot-theme apply` when no wallpaper is set yet, so everything renders before the first wallpaper. Every consumer must also tolerate the file being missing.
- After regenerating, `dot-theme` reloads consumers: tmux (`tmux source-file` if a server runs), Ghostty (its config reload signal or action), Neovim (each running server under `$XDG_RUNTIME_DIR` via `nvim --server ... --remote-send` or an autocmd that watches `nvim.lua`), Hyprland (`hyprctl eval 'dot.reload_colors()'`, falling back to `hyprctl reload`), Quickshell (watches `colors.json` itself with `FileView` + `watchChanges`, no action needed). oh-my-posh needs no hook either: a zsh `precmd` re-runs `oh-my-posh init` when `ohmyposh.toml`'s mtime changes, so open shells switch at their next prompt. The other terminal apps (lazygit, yazi, btop, bat, delta, fzf, atuin, eza, glow, bacon, foot, fuzzel) read their file or the terminal palette at start.
- Chrome: `system/40-apps.sh` installs a root-owned helper (`system/files/chrome/dot-chrome-theme` -> `/usr/local/libexec/`) and `dot-chrome-theme.path`, which watches `~/.local/state/theme/colors.json` and writes its `surface_container` color as the `BrowserThemeColor` policy to `/etc/opt/chrome/policies/managed/dot-theme.json`. Chrome reloads policy files live. Only a value matching `#rrggbb` crosses from the user's file into the policy, and the policy directory stays root-owned. Light/dark for pages follows the portal's `color-scheme` (dconf `prefer-dark`, `home/theme.nix`).
- Links from a live-linked `config/<app>/` into `~/.local/state/theme/` are committed as relative symlinks; links from paths home-manager owns (`~/.config/{eza,atuin,lazydocker,qt5ct,qt6ct}/...`) use `mkOutOfStoreSymlink`. Either may dangle before the first `dot-theme` run; each consumer then falls back to its defaults.
- Terminal backgrounds are translucent (Ghostty and foot at 85% opacity, blurred by Hyprland). Terminal apps leave the main background unpainted (Neovim `Normal` and friends `NONE`, tmux `bg=default`, btop `theme_background = false`, fzf `bg:-1`); only floats, selections, pills and diff tints are painted. Selected rows use `secondary_container` / `on_secondary_container` everywhere (Quickshell lists, fuzzel, lazygit, yazi, fzf, btop, Neovim menus).
- Quickshell exposes the roles as `Theme.m3<role>` in camelCase (`Theme.m3primary`, `Theme.m3onSurface`); the `m3` prefix avoids QML treating `on<X>` as a signal handler.

## Quickshell IPC (Hyprland binds call these)

Quickshell config lives in `config/quickshell/` with `shell.qml` at its root (the default config, started with `qs`). It exposes `IpcHandler` targets:

| Call | Effect |
|---|---|
| `qs ipc call launcher toggle` | App launcher + calculator |
| `qs ipc call session lock` | Lock screen |
| `qs ipc call powermenu toggle` | Lock / suspend / logout / reboot / shutdown |
| `qs ipc call wallpaper toggle` | Wallpaper picker |
| `qs ipc call wallpaper animate <always\|ac\|never>` | Wallpaper motion mode, remembered in `~/.local/state/quickshell/kubaos.json` (default `Settings.wallpaperAnimate`, `ac`) |
| `qs ipc call wallpaper settings` | Wallpaper picker on its Settings tab |
| `qs ipc call wallpaper status` | JSON: `wallpaper`, `video`, `scene` (the ukiyo scene id, or `""`), `ukiyo` (ukiyo draws it), `mode`, `allowed`, `onBattery`, `screensOff`, `playing` (screen name -> video path) |
| `qs ipc call notifications toggleDnd` / `clear` | Do not disturb, clear all |
| `qs ipc call controlcenter toggle` | Wi-Fi, Bluetooth, audio, power profile panel |
| `qs ipc call recorder toggle` / `stop` | Start recording the focused monitor or stop any recording or replay (through `dot-hypr-record`) |

Moving wallpapers (`services/Motion.qml`): a video decodes on a screen only when the mode allows it (`always`, or `ac` and UPower says not on battery), the session isn't locked, there has been input within `Settings.screenOffAfter` seconds (600, hypridle's DPMS timeout), and that monitor's active workspace has no fullscreen window. Otherwise the poster shows and the decoder is released. The lock screen plays the wallpaper if it's a video, else `Settings.lockLoop` (`shinjuku/station-mirror.mp4`; `""` for the blurred still), under the same rules minus the lock. One decoder per screen. A ukiyo scene follows the same rules through the daemon (`docs/UKIYO.md`): `pause`/`resume --reason motion`, `still` while locked, and the mode mirrored into ukiyo's `performance.on_battery` / `performance.motion`, both ways.

Screen recording state: `scripts/dot-hypr-record` writes `$XDG_RUNTIME_DIR/dot-record.json`, replaced atomically on every change: `{"state": "idle"|"recording"|"replay", "mode": "region"|"window"|"output"|"", "pid": <recorder pid>|null, "file": "<mp4 or replay dir>", "backend": "gsr"|"wf-recorder", "started": <unix seconds>}`. Quickshell watches it (`services/Recorder.qml`, `FileView` + `watchChanges`) and creates it as `{"state":"idle"}` when missing. A state other than `idle` whose pid is dead counts as idle; the script repairs it on its next run. Recordings go to `~/Videos/Recordings/` (`$DOT_RECORD_DIR`), screenshots to `~/Pictures/Screenshots/`; satty saves `<name>-annotated.png` next to the original.

Screen capture: gpu-screen-recorder (Nix) records through KMS. Its helper `gsr-kms-server` needs `CAP_SYS_ADMIN`, which a file in the Nix store can't carry, so `system/65-recorder.sh` copies the helper of the flake's nixpkgs build to `/usr/local/libexec/dotfiles/gsr-kms-server` (root:<user> 0750, `cap_sys_admin+ep`, GC root `/nix/var/nix/gcroots/dotfiles-gpu-screen-recorder`) and links `/run/wrappers/bin/gsr-kms-server` to it (`/etc/tmpfiles.d/dotfiles-gsr.conf`); nixpkgs' gsr wrapper looks there first. `system/update.sh --nix` re-runs it, because gsr and its helper must be the same version. Without it gsr asks for the password through polkit.

PAM: the lock screen runs PAM conversations in parallel: `/etc/pam.d/qs-lock` (password, `pam_unix`), `/etc/pam.d/qs-lock-u2f` (FIDO2 security key, `pam_u2f` with `userverification=1 pinverification=0`: an enrolled finger, never a PIN; `Settings.u2f`, on; only when `/etc/u2f_mappings` has a line for the user) and `/etc/pam.d/qs-lock-fingerprint` (`pam_fprintd`, `Settings.fingerprint`, off). They live in `system/files/pam.d/`; `system/50-hyprland.sh` installs `qs-lock` and `qs-lock-fingerprint`, `system/70-bitwarden.sh` installs `qs-lock-u2f`. Quickshell links Nix's libpam, so `pam_unix.so` resolves to Nix's module; `system/` links `/run/wrappers/bin/unix_chkpwd` to Ubuntu's helper (tmpfiles.d), and the fingerprint file loads nixpkgs' `pam_fprintd.so` from the root-owned link `/usr/local/lib/security/pam_fprintd.so`. `system/60-nix.sh` and `system/update.sh --nix` build `fprintd` from the flake's nixpkgs, point the link at its store path and pin that path with the GC root `/nix/var/nix/gcroots/dotfiles-pam-fprintd` (`link_pam_fprintd` in `system/lib.sh`). `qs-lock-u2f` does the same with nixpkgs' `pam_u2f` (`/usr/local/lib/security/pam_u2f.so`, GC root `dotfiles-pam-u2f`, `link_pam_u2f`, created by `system/70-bitwarden.sh`, relinked by `update.sh --nix`). Nothing under `$HOME` is on the PAM load path. Ubuntu's modules by absolute path do not load under Nix's libpam. hypridle's `|| hyprlock` is the fallback locker.

Security key outside the lock screen: `system/70-bitwarden.sh` adds two lines, `auth sufficient pam_u2f.so authfile=/etc/u2f_mappings cue nouserok userverification=1 pinverification=0 [cue_prompt=Touch the fingerprint sensor]` and the same with `userverification=0 pinverification=1` (PIN fallback), above `@include common-auth` in `/etc/pam.d/sudo`, `sudo-i` and `polkit-1` (Ubuntu's module, Ubuntu's libpam), plus a `polkit-agent-helper@.service` drop-in that lets the helper see `/dev/hidraw*`. `/etc/u2f_mappings` is root:root 0644 and only `dot-u2f-enroll` (through sudo) writes it. The password always stays below as the fallback.

Polkit: the in-shell agent is off by default (`Settings.polkitAgent: false`) and Hyprland starts `hyprpolkitagent` (`POLKIT_AGENT = "hyprpolkitagent"` in `config/hypr/autostart.lua`). Flip both together.

Volume and brightness keys are bound in Hyprland directly to `wpctl` and `brightnessctl`. Quickshell shows the OSD by watching PipeWire and the backlight itself.

## Secrets (docs/SECRETS.md)

- `SSH_AUTH_SOCK` is `~/.bitwarden-ssh-agent.sock` (Bitwarden desktop's agent) in the login shell (`home.sessionVariables`), hence Hyprland and its children, and in systemd --user (`systemd.user.sessionVariables`). `home/secrets.nix` masks Ubuntu's `ssh-agent.socket`/`.service` and `gcr-ssh-agent.socket`/`.service` for the user, so nothing else sets it. Hyprland's autostart starts `bitwarden`.
- `~/.ssh/config` is home-manager's (`programs.ssh`): `Include config.d/*`, then `Host *` with `IdentityAgent` set to the Bitwarden socket. `~/.ssh/config.d/bitwarden` and `~/.ssh/<name>.pub` are written by `dot-bw-setup` from the vault folder `dotfiles`; no private key is ever on disk.
- git signing: `config/git/config` includes `~/.config/git-bitwarden/signing.gitconfig` (`user.signingKey`, `gpg.format = ssh`, `gpg.ssh.allowedSignersFile`, `commit.gpgSign`), generated by `dot-bw-setup`. Missing include = unsigned commits.
- Project env: GNOME keyring items with attributes `service=dotfiles-env project=<p> name=<VAR>` and label `dotfiles-env/<p>/<VAR>` (suffix `#base64` when the value is stored base64 because it has a newline or tab). Written by `dot-bwenv` (and `dot-bw-setup`), read by `use bwenv <p>` in `.envrc` (`~/.config/direnv/lib/bwenv.sh`, generated by `home/secrets.nix`) through `dot-bwenv env <p>`, NUL-separated `NAME=VALUE` records.
- Secrets never go in argv or files: scripts pass them on stdin; `BW_SESSION` lives only in the running script and its children.

## Hyprland conventions

- Session start: tty1 autologin (getty drop-in) -> zsh login shell -> `~/.zprofile` (home-manager) sources `system/files/hyprland-autostart.zsh` as its last step, which execs `start-hyprland` (shipped with the hyprland package; no uwsm). `~/.config/hypr/no-autostart` disables it.
- Graphical session: without uwsm nothing activates `graphical-session.target`, and xdg-desktop-portal (`Requisite=graphical-session.target`) refuses to start. `config/hypr/autostart.lua` imports the session environment, then starts `hyprland-session.target` (defined in `home/desktop.nix`, `BindsTo=graphical-session.target`), and stops it on `hyprland.shutdown`.
- Panels and windows: only the frame reserves space (exclusive zones from `config/quickshell/modules/frame/Exclusions.qml`: bar 36 px, 8 px elsewhere). Panels, popups and the OSD live in the frame layer with no exclusive zone and float over windows; opening one never resizes a window. Window corners: `gaps_out = 8`, `border_size = 2`, `rounding = 10` (Hyprland rounds the border's outer edge at rounding + border = 12 = frame radius 20 - gap 8, so the corners are concentric).
- Primary terminal `ghostty`, fallback terminal `foot`. Primary launcher `qs ipc call launcher toggle`, fallback `fuzzel`. Fallback binds always exist and never depend on Quickshell or Nix GPU wiring:
  - `SUPER+Return` ghostty, `SUPER+ALT+Return` foot
  - `SUPER+Space` launcher, `SUPER+ALT+Space` fuzzel
- Movement on `SUPER+hjkl`, move window `SUPER+SHIFT+hjkl`, resize `SUPER+CTRL+hjkl`, workspaces `SUPER+1..9`, move to workspace `SUPER+SHIFT+1..9`. The rest ported from Omarchy 4.
- Monitors: laptop panel `eDP-1` at scale 1.5 (1920x1200 logical), the two externals 1920x1080 at scale 1, so both show Full HD density and every size (Quickshell tokens, terminal and UI fonts) is tuned once for it. Externals are matched by description in `config/hypr/monitors.lua`, with placeholders and a comment explaining how to fill them in from `hyprctl monitors`.
- Workspaces 1-3 on `eDP-1`, 4-6 on external 1, 7-9 on external 2. When an external disappears Hyprland moves its workspaces to the laptop; `dot-hypr-workspaces` listens on the Hyprland event socket and moves them back on `monitoradded`.
- Lock command everywhere: `loginctl lock-session`. hypridle's `lock_cmd` runs `qs ipc call session lock || hyprlock`.
- Idle: lock 5 min, DPMS off 10 min, suspend 30 min only when not on AC. On AC, never auto-suspend.
- Lid: logind suspends on lid close only on battery (`system/` owns the logind drop-in). Hyprland disables `eDP-1` on lid close and enables it on open.

## Packages: who installs what

- Pinned downloads (`system/70-bitwarden.sh`, sha256-checked): Bitwarden desktop `.deb` (amd64), `bw` standalone CLI to `/usr/local/bin/bw`.
- apt (`system/`): kernel, firmware, Mesa (`mesa-libgallium` carries VA-API; `mesa-va-drivers` no longer exists), NetworkManager, PipeWire, portals, PAM, fprintd, bolt, power-profiles-daemon, snapper, Hyprland + hyprland-guiutils + hypridle + hyprlock + hyprpolkitagent + hyprpicker (held), foot, fuzzel, zsh, Docker, Chrome, fonts-noto*, build-essential, clang, mold, unzip, openjdk-25-jre-headless (Mason, sonarlint), openssh-client, tuxedo-drivers, SentinelOne, libsecret-tools, libpam-u2f, pamu2fcfg, pamtester, fido2-tools (`system/75-yubikey.sh`; pcscd + pcsc-tools only with `--piv`). `ykman` is Nix's `yubikey-manager` (`home/cli.nix`): apt's depends on pcscd. Runtime helpers for Hyprland binds and Quickshell: wl-clipboard, grim, slurp, playerctl, socat, qalc, libnotify-bin, brightnessctl. libcap2-bin (`setcap`, `system/65-recorder.sh`).
- Built from source by `system/20-base.sh`: grub-btrfs (pinned tag, not packaged for resolute; not tracked by dpkg).
- Nix (`home/`): quickshell (tagged release), ghostty, neovim (0.12; never from apt), tmux, tms, lazygit, yazi, fzf, zoxide, atuin, direnv + nix-direnv, eza, bat, fd, ripgrep, delta, dust, dysk, btop, jq, yq, xh, tealdeer, just, glow, gh, fastfetch, lazydocker, bacon, cargo-nextest, hyperfine, sccache, wild (`wild-unwrapped`; opt-in Rust linker, mold from apt stays the default, switch is `rustLinker` in `home/dev.nix`), oh-my-posh, matugen, gpu-screen-recorder, satty, Nerd Fonts, Material Symbols, Geist, M PLUS 1 (`mplus-outline-fonts.githubRelease`), bibata cursors, Papirus (cyan folders), adw-gtk3.
- GPU for Nix apps: home-manager's `targets.genericLinux.gpu`; `sudo non-nixos-gpu-setup` once (run by `system/60-nix.sh`) links `/run/opengl-driver`. No nixGL.
- Each tool has exactly one owner. Don't add a package to both lists.
- Outside both: rustup (rustup.rs), proto (its installer; then `proto install moon` and project pins), Bun via proto.
