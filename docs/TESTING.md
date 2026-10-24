# Testing KubaOS in Docker

The point: a broken config shows up as a red FAIL on the Mac, not as a black screen on the laptop.

`test/run` builds an arm64 Ubuntu 26.04 image with the repo's own `system/*.sh`, keeps the Nix store in a volume, and on every run copies the repo in and does a real `home-manager switch` for `kuba@vm`. Only the parts that change rarely are cached. The config under test never is.

Needs Docker with BuildKit and Compose v2 (Docker Desktop on the Mac is fine). Everything the harness creates is called `dotfiles-test-*` (containers, network, volumes) or `dotfiles-nix`, so it never touches other containers.

## Commands

| Command | What |
|---|---|
| `test/run build [--progress=plain] [--no-cache]` | Build `dotfiles-test:latest` |
| `test/run warm` | Realise the `kuba@vm` generation, the home-manager CLI, nixpkgs' `pam_u2f`, `fprintd`, `pamtester` and Chromium into the `dotfiles-nix` volume, then switch once |
| `test/run e2e [--only a,b] [--keep]` | Every check below, a PASS/FAIL table, exit 1 on any failure. `--keep` leaves the container for `docker exec -it dotfiles-test-e2e bash` (logs in `/tmp/dotfiles-test/`) |
| `test/run shell` | Fresh switch, then a login zsh as kuba in `~/Projects/dotfiles` (a copy: edit on the host) |
| `test/run bitwarden` | Vaultwarden + `kubaos bitwarden` + `use bwenv` (below); builds the kubaos CLI first |
| `test/run shots [--dir NAME \| --out PATH] [--size WxH] [--scale N] [--wallpaper FOLDER/FILE] [--hyprland]` | Headless session with Quickshell, left running for screenshots |
| `test/run shots shot NAME [--webp]` | `grim` the whole output to `NAME.png` (or WebP, quality 92, `-sharp_yuv`) |
| `test/run shots run CMD...` | Run in the session as kuba: `qs ipc call controlcenter toggle`, `notify-send hi`, `dot-theme next` |
| `test/run shots record start NAME`, `record stop` | `wf-recorder` to `NAME.mp4` |
| `test/run shots log`, `shots stop` | Quickshell's log; end the session |
| `test/run clean` | Remove the harness's containers, keep the volumes |
| `test/run nuke [--yes]` | `clean`, then delete the harness volumes (asks first) |

`DOTFILES_SRC=/path/to/checkout test/run e2e` tests another copy of the repo, for example a `git worktree` of a branch. The harness scripts themselves are read from that copy (`test/lib/`), so they stay in step with it. `DOTFILES_TEST_MEM` and `DOTFILES_TEST_CPUS` set the container's limits (default 5g, 6 CPUs; Docker Desktop's VM has about 7.75 GB).

`shots --dir NAME` writes to `docs/screenshots/NAME/`. Without `--dir` or `--out` the files go to `test/out/` (git-ignored). Typical session:

```sh
test/run shots --dir v4 --size 2880x1800 --scale 1.5
test/run shots run qs ipc call controlcenter toggle
test/run shots shot cyberpunk-controlcenter-1800 --webp
test/run shots run 'dot-theme set /usr/share/kubaos/wallpapers/japanese/tokyo-tower-blue-hour.jpg'
test/run shots stop
```

## What's cached where

| Layer | Holds | Rebuilt when |
|---|---|---|
| image stage `base` | `ubuntu:26.04`, user kuba (sudo-rs, zsh), `system/10-strip.sh` and `system/20-base.sh` run for real | `system/lib.sh`, `10-strip.sh`, `20-base.sh`, `system/files/{apt,netplan,logind.conf.d}`, `system/files/systemd/{system,user}.conf.d` change |
| stage `nix` | Nix from the official installer, version, URL and sha256 read from `system/60-nix.sh`, `--daemon`, flakes on | `system/60-nix.sh` changes (or anything above) |
| stage `tooling` | apt test tools: sway, swaybg, grim, slurp, wf-recorder, Mesa llvmpipe GL/GLES/EGL, dbus, gnome-keyring, secret-tool, pamtester, shellcheck, lua5.5, hyperfine, jq, expect, cwebp, ImageMagick, Python with requests/cryptography, Node (stand-in for proto's), Plymouth with its X11 renderer and Xvfb | the Dockerfile's package list |
| stage `desktop` | `system/50-hyprland.sh` and `system/70-bitwarden.sh` run for real: Ubuntu's arm64 hyprland 0.53, lock-screen PAM files, `unix_chkpwd` link, bw CLI, pam_u2f lines for sudo/sudo-i/polkit-1 | those two scripts or `system/files/{getty,pam.d,tmpfiles.d,systemd,udev,hyprland-autostart.zsh}` |
| stage `hyprland` | placeholder, see below | |
| volume `dotfiles-nix` (`/nix`) | the whole Nix store and database. Docker seeds an empty volume from the image's `/nix` (the bare install) on first use | never by the harness; `test/run nuke` deletes it |
| volume `dotfiles-test-cache` | Nix's fetcher and eval caches, Neovim's plugins, Mason tools and treesitter parsers, GC roots for `warm` | `nuke` |
| volumes `dotfiles-test-cargo`, `dotfiles-test-target` | Cargo's registry and target dirs for the `kubaos` build `test/run e2e` does in `rust:latest`, one target subdirectory per checkout | `nuke` |
| apt `.deb`s | BuildKit cache mounts on `/var/cache/apt`, `/var/lib/apt` | `docker builder prune` |

**Local flake inputs.** `flake.lock` can point at a checkout on the Mac (`git+file://...` or `path:`). Inside the container that path doesn't exist, so `test/run` reads `flake.lock` and bind-mounts each such input read-only at the same absolute path (and a git worktree's main `.git`), and the container lists them as git `safe.directory` (`trust_local_inputs` in `harness.sh`): they're owned by root in there, and git, Nix's fetcher included, won't read a repository owned by someone else otherwise. `test/lib/local-inputs.sh` does the finding, for `install/image/build` too.

Not cached, on purpose: the repo. Each run bind-mounts it read-only at `/src`, copies it to `/home/kuba/Projects/dotfiles` (without `.git` and `docs/screenshots`), makes a fresh one-commit git repo there so the flake sees untracked files, and runs `system/60-nix.sh --host vm` (home-manager switch, the `pam_fprintd` link, the GPU setup, zsh) and `system/70-bitwarden.sh --no-desktop --no-cli` (the `pam_u2f` link) from a bash login shell, the way you would on the laptop. The home directory starts empty every run, so every run is a first activation.

Because the image never COPYs `config/`, `home/`, `scripts/`, `flake.*` or `test/`, editing them never rebuilds the image. Editing `home/*.nix` or `flake.lock` only rebuilds the derivations that changed, in the volume.

What a container has that the laptop gets from boot: a user D-Bus at `/run/user/1000/bus` (systemd --user), the system bus, `/run/user/1000`, the tmpfiles for `unix_chkpwd` and `/run/opengl-driver`, and a hand-started `nix-daemon`. `test/lib/harness.sh` does these in `boot_like`.

## Timings

Measured 2026-10-16 on an Apple Silicon Mac, Docker Desktop (arm64 VM, 8 CPUs, 7.75 GB).

| What | Time |
|---|---|
| `test/run build`, cold (empty BuildKit and apt caches) | 7 min 40 s, of which `20-base.sh` is 4 min 53 s (linux-firmware alone is about 1 GB) |
| `test/run build` after editing `config/` or `home/` | 2 s (all cached) |
| `test/run build` after editing `system/70-bitwarden.sh` | about 30 s (`desktop` stage onwards) |
| `test/run build` after changing the tooling package list | 2 min 19 s |
| `test/run warm`, empty `dotfiles-nix` volume | about 12.5 min (8.4 GB store, Chromium included) |
| `test/run warm`, warm volume | 20 s |
| `test/run e2e`, first run on an empty cache volume | about 2 min (Neovim plugins, Mason tools, treesitter parsers) |
| `test/run e2e`, warm | 42 s (switch 4 s, checks 38 s) |
| `test/run e2e` after a `home/*.nix` change and a `config/` change | 50 s (switch 8 s) |
| `test/run bitwarden` | 1 min 25 s (seeding the vault with `bw` is 57 s of it) |
| `test/run shots` until the session is ready | 11 s |

## The checks

`test/run e2e` runs these, in this order, after the switch:

| Check | What passes |
|---|---|
| `switch` | `system/60-nix.sh` and the lock-screen half of `70-bitwarden.sh` exit 0 |
| `login_shell` | `zsh -l -i` in a pty prints nothing (no warnings, zoxide doctor quiet); `y`, `eza`, `glow`, the oh-my-posh refresh are functions; atuin owns Ctrl-R; the direnv hook is in; `.zprofile` sources the autostart snippet |
| `zsh_start` | in a pty, the median of 20 `zsh -i -c exit` stays within 100 ms (the harness measures about 31 ms); deleting `~/.cache/kubaos/zsh` makes the next start write the fzf, atuin, direnv, zoxide and oh-my-posh caches and the dump again, the atuin one byte for byte what atuin prints; a second start writes nothing; a dump older than a day is rebuilt |
| `greeting` | an interactive zsh with `TERM_PROGRAM=ghostty` greets once, and a shell started in it doesn't; none under `$TMUX`, `$NVIM`, `$SSH_CONNECTION`, `$SSH_TTY`, `KUBAOS_GREETING=0`, another terminal or `zsh -c`; `dot-greeting off` and `on` turn it off and on; prints what the greeting and the full `fastfetch` cost |
| `path` | `~/.local/bin`, `~/.cargo/bin`, `~/.proto/{shims,bin}`, `~/.nix-profile/bin` on PATH, Nix's before `/usr/bin` |
| `tools` | every tool in CONTRACTS ("Packages: who installs what") that exists on arm64 resolves, the Nix ones into `~/.nix-profile/bin`, the apt ones into `/usr`, `bw` to `/usr/local/bin`; nvim is 0.12; the login shell is `/usr/bin/zsh` |
| `links` | every tracked file under `config/` is a store file with the repo's content in a real `~/.config/<app>` directory (except the ones `dot-config` seeds or merges); every `scripts/<file>` resolves into the repo copy; `~/Pictures/wallpapers` is a real folder; scripts are `dot-*`, executable, env shebang, `set -euo pipefail` |
| `theme` | `dot-theme apply --force`, `set`, `next`, `toggle-folder`; all 21 files in `~/.local/state/theme`, every M3 key in `colors.json`; `apply` with nothing changed is a no-op; the theme links resolve |
| `tmux` | the real `tmux.conf` loads with the theme and plugins, nothing complains |
| `nvim` | `Lazy! restore` from the lockfile; Mason tools and treesitter parsers installed (once, cached); starts with the matugen colorscheme; opening Markdown, Nix, shell, Lua and QML files leaves no error in `:messages` |
| `ghostty` | `ghostty +validate-config`, `foot --check-config`; the background unit (`X-RestartIfChanged=false`, its flags) and the D-Bus file are in place; under sway, the unit's command owns `com.mitchellh.ghostty` and `ghostty +new-window -e` runs its command in a window of it, in home; with no instance, `+new-window` starts one through D-Bus |
| `tms` | runs, reads the config seeded from the repo's |
| `pam` | through Nix's libpam (Nix's pamtester, as kuba, like Quickshell): `qs-lock` takes the right password and refuses a wrong one; `qs-lock-u2f` and `qs-lock-fingerprint` load their modules and fail with no device. Through Ubuntu's (apt pamtester, sudo-rs): `sudo`, `sudo-i`, `polkit-1` carry the pam_u2f line and, with a well-formed mapping for a key that isn't plugged in, fall back to the password; also with no mapping file |
| `lua` | `luac5.5 -p` on every tracked `.lua` (not the matugen templates) and on the rendered `hypr.lua`, `nvim.lua` |
| `workspaces` | `lua5.5 test/lua/workspaces_test.lua`: per-screen workspaces (`config/hypr/workspaces.lua`) against a stand-in for Hyprland's Lua API. Blocks of ten ids per screen, the laptop's first; SUPER + 2 on each screen opens its own 2; moving a window to a slot creates it on the focused screen; on unplug a screen's group moves to the laptop under its own numbers, even when Hyprland puts it on another external, an empty one vanishes, and on replug (another connector, or after a config reload) it returns; the lid does the same for the laptop. Each group keeps its first workspace and one empty workspace after its last with windows as Hyprland persistent workspace rules bound to its screen, moved along when a window lands on it and released when no longer needed, reusing the rule. Next and previous walk the groups in block order (the kept empty workspaces included, then the next group), and `SUPER + 1-0` and the bar's clicks reach a waiting group's workspaces. Then `fallback.lua` and `keybinds.lua` load: no two binds share keys, the decided per-screen, screen-focus, cycling and group keys are where #41 put them, and the three-finger swipe is Hyprland's own `workspace` gesture with `workspace_swipe_create_new` off. The two-screen run on a real Hyprland is `test/vm/workspaces-check` |
| `shutdown` | `lua5.5 test/lua/session_test.lua`: the fallback exit bind stops `graphical-session.target` under a time limit before `hyprctl dispatch 'hl.dsp.exit()'`, and the `hyprland.shutdown` hook stops it with `--no-block`. `dot-session leave reboot`, `poweroff` and `logout`, with a logging `systemctl` and `loginctl`, stop the target before the action; another action exits 2. The switch put the 5 s stop timeouts for the portal and the polkit agent in `~/.config/systemd/user`. The `fast-shutdown-boot` system migration, on a container set back to an older install (no timeouts, `NetworkManager-wait-online` enabled, stand-in Docker, containerd and setvtrgb units enabled), installs both `DefaultTimeoutStopSec=10s` files and Docker's 20 s drop-in, enables `docker.socket`, disables `docker.service`, `containerd.service` and `NetworkManager-wait-online`, masks setvtrgb, and leaves `multi-user.target` and `network-online.target` wanting none of them; a second run changes nothing. Real shutdown and boot times are measured on the laptop |
| `splash` | Plymouth's `plymouth-poweroff` and `plymouth-reboot` units are wanted by their targets, in `shutdown` and `reboot` mode, and need `splash`, which `system/files/grub/kubaos.cfg` puts on the kernel line. The theme from the repo runs in plymouthd on Xvfb (X11 renderer), and the screen is grabbed: in boot mode the sign is lit within 6 s; in shutdown and reboot mode it is lit at 0.3 s and the screen is black within 6 s. A theme script Plymouth can't run draws nothing and fails both |
| `shellcheck` | `shellcheck -x` on every shell script in `scripts/`, `system/`, `test/`, `config/`: no warnings or errors (info and style notes are listed, not fatal) |
| `flake_check` | `nix flake check` |
| `quickshell` | headless sway (pixman), Quickshell from the generation on Mesa llvmpipe: config loads, no QML error in the log, `qs ipc show` lists every target in CONTRACTS. Warnings about services a container lacks (NetworkManager, BlueZ, UPower, portals, Hyprland under sway) are printed, not fatal |
| `sysmon` | `kubaos-sysmon sample` reads the container's `/proc` (CPU, memory, processes; no GPU, battery or fan to find); headless sway with Quickshell: the shell's stream samples at the pill's pace and writes `$XDG_RUNTIME_DIR/kubaos-sysmon.json`; `qs ipc call sysmon toggle` opens the popup and the engine samples in detail, a second toggle closes it and the pace drops back; `qs ipc call sysmon pill off` hides the pill and writes `sysmon.toml`, `kubaos-sysmon set pill on` brings it back; no QML errors, by `qs ipc call sysmon status` |
| `clock` | headless sway with Quickshell: `kubaos-clock eval` answers `days until christmas`; `qs ipc call clock toggle` opens the clock popup with its engine (`kubaos-clock serve`) running, by `qs ipc call clock status`; a search goes in through `qs ipc call clock search`; a second toggle closes it; no QML errors |
| `files` | the switch put yazi's desktop entry, KubaOS's `mimeapps.list` and the Ghostty and KubaOS nautilus-python extensions in `~/.local/share`, and no FileManager1 service file there. `xdg-mime` names Nautilus for folders, zip and 7z, Loupe for PNG and HEIC, Showtime for MP4 and Papers for PDF, each an entry apt installed; GLib lists yazi among the apps for folders. Both extensions load under python3 with the Nautilus typelib. The session bus lists `org.freedesktop.FileManager1` as activatable, from Nautilus's own service file. With a `ghostty` stub on PATH that logs its arguments, `kubaos-files open` on a folder with a space in its name runs `ghostty -e yazi -- <folder>`. `kubaos-files tidy` leaves a file that arrived today and prints `0 moved`, `--scheduled` does nothing with the setting off, `tidy on|off` writes and removes the setting, and the daily timer is enabled. Opening a Nautilus window needs a display, so it is checked on the laptop |
| `automount` | the switch put the `kubaos-automount` user unit in place, running `kubaos-automount watch`. `off` and `on` write and remove the setting; ejecting a drive that isn't there exits 1 with one line, and a usage error exits 2. `import` on a card with two photos in `DCIM/` copies them into `~/Pictures/2024/07/` by mtime, and a second run copies nothing and leaves the card as it was. The container has no drives and no udisks2 daemon, so mounting and ejecting are checked on the laptop |
| `bluetooth` | the switch put the `kubaos-bluetooth` user unit in place, running `kubaos-bluetooth agent`. With no agent, `pair` fails with the one line the shell shows. The agent started by hand, with no bluetoothd on the system bus, writes `$XDG_RUNTIME_DIR/kubaos-bluetooth.json` as unregistered; it turns down an answer to no prompt, a path outside `/org/bluez/` and a scan bluetoothd can't take, and a usage error exits 2. Real pairing needs the hardware |
| `recorder` | in a headless sway session with Quickshell: Quickshell creates `$XDG_RUNTIME_DIR/dot-record.json`; `dot-hypr-record output` (wf-recorder backend, gsr needs a GPU) sets it to `recording`; `qs ipc call recorder stop` brings it back to `idle` and leaves an mp4 in `~/Videos/Recordings`; `replay save` refuses with no replay running; no QML errors |
| `session_reload` | in a headless sway session with Quickshell, switches run as kuba with a stand-in `hyprctl` in `/usr/local/bin`, which activation looks in first (`test/lib/hyprctl-stub`: one instance, logs its calls, runs what `dispatch` would exec in the session). `qs ipc call session locked` says `false`. A new bat default reaches no Hyprland and leaves Quickshell's pid alone; a new `shell.qml` default dispatches `dot-session restart-shell` and Quickshell comes back under a new pid with no QML errors, without a Hyprland reload; new `looks.lua` and `hypridle.conf` defaults run `hyprctl reload` and dispatch `restart-idle`, and leave Quickshell's pid alone. Then the repo's defaults come back with a switch that has no Hyprland to reach |
| `wallpaper_sets` | `/usr/share/kubaos/wallpapers` links into the store; the switch seeded `~/.config/kubaos/wallpapers/sets` with the sets on disk; `dot-theme folders` is `live`, every set, `yours` without the file and only the chosen (known) sets with it; `dot-theme folder` refuses an unchosen set; an image in `~/Pictures/wallpapers` sets through `folder yours`; `toggle-folder` wraps; `apply` moves an old `~/Pictures/wallpapers/<set>/` path to `/usr/share`; with `<set>/<file>` lines, `folders` and `dot-theme wallpapers` show only the picks (a missing one skipped), `next`/`prev` step over them, first boot takes the first pick when the default isn't one, and `all-sets` brings every set back whole |
| `wallpaper_video` | `dot-theme set <video>` gives the same colors as its poster and names the video in `colors.json`, `hypr.lua`, `nvim.lua`; `apply` stays a no-op; a lap of `next` over the video's folder visits stills and videos, never posters. Quickshell plays it (software decode), `qs ipc call wallpaper animate never` stops it and the mode lands in `~/.local/state/quickshell/kubaos.json`; prints Quickshell's CPU playing and not; no QML errors |
| `packages` | the `kubaos` CLI (built by `test/run e2e` in `rust:latest`, as `test/out/kubaos`) against a user flake rendered from `user-flake/`, with the repo copy as KubaOS and KubaOS's locked nixpkgs as the user's. `kubaos package add cowsay` wires a pre-state-directory `home.nix` (backup in `~/.local/state/kubaos/backups/`, none in the flake) and switches: cowsay and `suites/demo.toml`'s hello are on PATH, and the generation carries its `kubaos.d`. Two `add`s at once both install, and their switch jobs never overlap. `kubaos package restore` plus the first generation's `activate` bring back that generation's packages and `kubaos.d`. `remove` takes cowsay away and keeps hello, which the Suite file lists; deleting that file and `kubaos switch` takes hello away. Switches run `--foreground`: no systemd user manager here |
| `suites` | the test Suite (`home/suites/test.nix`; the CLI is built with `--features test-suite`) in a fresh user flake under git. `kubaos suite add test` puts `kubaos.d/suites/test.toml` in git's index (intent to add) and switches: its script, environment variable and PATH entry are there, a commented `~/.config/kubaos/nvim/plugins/test-suite.lua` is seeded, and LazyVim's opts carry the Suite's setting. An override of the same setting in that file wins. `configure --version 2 --extra greeting` applies and leaves the file alone. `kubaos suite new` scaffolds a project in `~/Projects`. `remove` shows the cache's size and keeps it, lists the user's file, leaves it and the project, takes the file out of git's index, and the new generation has no store path, session variable, PATH entry or Neovim setting of the Suite. `remove --purge-caches` deletes the cache. Reads LazyVim's opts, so it needs the `nvim` check's plugins (cached in the volume after one full run) |
| `user_layer` | `test/user-layer-check` (issue #9): every file of `~/.config/kubaos` and the seeded btop and tms configs exist; the user changes each one; newer defaults are committed and switched to; every user file is byte-identical, the new defaults are live (tmux, git, git ignore, bat) and the user's values win (tmux, git, Ghostty, Neovim, zsh, lazygit, bacon, fastfetch, yazi, btop). Commits to the checkout, so it runs before the update checks only |
| `update_edge` | `kubaos update` on edge, the way the laptop runs it: a bare clone of the repo copy is origin, a root-owned clone of it on `release` is `/usr/share/kubaos` (the real one is put back by `rollback`), the user flake takes KubaOS from it, and `apt-get` is a shim that logs its arguments. A commit on origin's main adds a system and a user migration and bumps the workspace's version. `kubaos update --check` names it in `update-available` and leaves `release`. `kubaos update -y --foreground` warns that there is no `allowed_signers`, moves `release` to main's head, runs the system migration once as root and the user one once as kuba, writes their markers, the record, a history line and the user flake's lock, and empties `update-available`. After it, the `kubaos` on PATH is the home's, at the bumped version, and `/usr/bin/kubaos` is the launcher and runs it (the `kubaos-launcher` migration). A second update moves nothing and runs no migration. `kubaos` in the checks is `/out/kubaos`, put before the home's on PATH |
| `update_stable` | after `update_edge`: a throwaway SSH key in `/usr/share/kubaos-keys/allowed_signers`, `kubaos channel stable`, a signed tag `v0.1.1` and an unsigned `v0.1.2` on origin. The update takes `v0.1.1` and says `v0.1.2` was skipped. Back on edge, with the keys there, the update refuses main's unsigned head and leaves `release` |
| `rollback` | after `update_stable`: `kubaos rollback -y --foreground` moves `release`, the record and the user flake's lock back to where the stable update started, drops its history line and keeps the migration markers. Then puts the real `/usr/share/kubaos` back and removes the shim |

`test/run bitwarden` starts `vaultwarden/server` (profile `bitwarden`) with a throwaway CA on a private network (`https://vw`), then:

| Check | What passes |
|---|---|
| `bw_server` | Vaultwarden answers; an account is registered the way the web vault does it (`test/lib/vw-register.py`), with authenticator-app two-step login on |
| `bw_seed` | folders `Personal` and `Work` and five SSH key items spread over them and "No folder": GitHub with `signing=true` (Personal), a second GitHub account `Work GitHub` with `hosts=github-work hostname=github.com` (Work), a homelab key with hostname/port/user (no folder), a later item also named GitHub with no fields (Work), and one whose `hosts` field tries to inject `ProxyCommand` (Work); folder `env/demo` with a URL, a token full of shell metacharacters, a multi-line PEM and an invalid name, plus an item elsewhere with `bwenv=demo` |
| `bw_session` | session bus, gnome-keyring unlocked, OpenSSH's agent on `~/.bitwarden-ssh-agent.sock` holding the vault's keys (stand-in for Bitwarden desktop, which has no arm64 build), and the git identity first-boot setup writes |
| `bw_setup` | `kubaos bitwarden setup` from logged out, answered by `expect`: server prompt, login choice, email, master password, the TOTP code; exits 0, reads all five SSH keys, reports the agent serving every key, warns about the second GitHub and names its `github-2.pub`, and the summary lists the keys with a Host block and the key-only one with the `hosts` hint |
| `bw_files` | a `.pub` per key, the older GitHub in `github.pub` and the newer in `github-2.pub`; three Host blocks, no `ProxyCommand`; `ssh -G` picks the right identity, host and port, `github.com` and `github-work` one key each; a commit verifies with "Good "git" signature"; no secret in plaintext under `~`; `kubaos bitwarden check` sees the agent |
| `bw_folder` | `kubaos bitwarden refresh --only ssh` with `folder=Work` (the older key name): only Work's three keys and one Host block; with `folders=Personal, Work,Archive`: a warning naming Archive, the four keys of Personal and Work, two Host blocks; with `folder=dotfiles`, a folder the vault doesn't have: warnings, then every SSH key in the vault again |
| `bw_direnv` | `kubaos bitwarden env list`: the keyring holds `demo` and its variables, not the invalid one; `cd` in an interactive zsh exports all four intact, leaving unloads them; the time per `direnv export` |
| `bw_alias` | the old names, once: `dot-bw-setup --check` and `dot-bwenv list` print their new names and do the same |

## Adding a check

1. Write `c_<name>` in `test/lib/e2e.sh` (or `bitwarden.sh`). Print `ok   ...` lines for what passed and `FAIL ...` for what didn't, return non-zero on failure. It runs in a subshell as root; `as_user`, `zrun` (login zsh) and `zlogin` (login + interactive zsh) run things as kuba, `session_start`/`qs_start`/`in_session` give you a Wayland session.
2. Add the name to the list in `e2e_main` (or `bitwarden_main`).
3. `test/run e2e --only <name>` while you work on it. Its output is in `/tmp/dotfiles-test/check-<name>.log`; `--keep` keeps the container around to look.
4. Tools it needs: apt ones go in the `tooling` stage of `test/Dockerfile`; Nix ones in the `warm` list in `test/lib/entry` (built from the flake's nixpkgs, so the version matches the generation).

No image rebuild is needed for steps 1 to 3: `test/` is read from the mounted repo.

## Hyprland (TODO)

The `hyprland` stage in `test/Dockerfile` is a placeholder (`# TODO(lead): recipe from screenshot agent`). Ubuntu's arm64 hyprland is 0.53, which predates the Lua config, and the PPA builds amd64 only. The contract the stage has to meet is written next to it: an executable `/usr/local/lib/dotfiles-test/hyprland-session` that starts Hyprland 0.56 headless with `~/.config/hypr` and writes `WAYLAND_DISPLAY` and `HYPRLAND_INSTANCE_SIGNATURE` to the file it's given. Store paths it needs must be realised at run time (`test/run warm --hyprland`, also a TODO in `test/lib/entry`), because the `dotfiles-nix` volume hides the image's `/nix`. Until then `test/run shots --hyprland` stops with a message, and everything runs under sway: no blur, no rounded corners or gradient borders, no workspace animations, and the workspace indicator stays on 1.

## Kōbō and the live ISO in a VM

The Docker harness can't boot an ISO. `test/vm/run` (QEMU on HVF, `test/vm/README.md`) boots the live ISO from `install/iso/build`, runs headless installs from an `install.toml`, boots the result, and takes screenshots and snapshots. After `just vm-install`, `just vm-user-layer` runs `test/user-layer-check` on the installed system as its user: the same promise, from a real install.

## What this can't test

- The GPU: there's no DRM device in Docker Desktop, so sway uses pixman and everything GL is Mesa llvmpipe. 120 Hz smoothness, VA-API, Vulkan and `non-nixos-gpu-setup` against real drivers need the laptop.
- Real hardware: LUKS + Btrfs subvolumes, snapper and the GRUB rollback drill (the container's `/` is overlayfs, so those steps skip), lid and idle, the USB4 dock and two monitors, Wi-Fi, Bluetooth, audio devices, the fingerprint reader.
- `system/30-hardware.sh` (TUXEDO repo and `tuxedo-drivers`, amd64 only, plus MOK) and `system/40-apps.sh` (Docker, Chrome amd64-only, SentinelOne). Neither runs in the image.
- systemd: no PID 1 systemd, so no units, timers, logind, getty autologin or `systemctl --user`; the harness fakes the bits the tests depend on (above).
- A real FIDO2 key: PAM is tested with a mapping for a key that isn't there (the fallback path), never with a touch or a fingerprint on the key. `dot-u2f-enroll` isn't exercised.
- Bitwarden desktop (no arm64 build): its SSH agent and polkit unlock. OpenSSH's agent stands in on the same socket. gh login isn't tested (it would need github.com).
- The PPA's Hyprland and the Hyprland config, until the stage above is filled in. The installed VM (`test/vm`) runs the real config on nixpkgs' Hyprland.

## Old volumes

Earlier test containers used three ad-hoc Nix store volumes. The harness uses only `dotfiles-nix` now. Once nothing uses them (`docker ps -a --filter volume=<name>` is empty; `nix-store-dotfiles` was still mounted by the `dot-v3` screenshot container on 2026-10-16), free about 78 GB with:

```sh
docker volume rm nix-store-dotfiles qs-nix nixstore
```
