# Using and hacking on KubaOS

Keys, updates, daily commands, how it is built, the repo map and known limitations. The front page is [`README.md`](../README.md).

## Keys that matter

Every bind is in the Command menu, found by what it does: type "move to workspace 3" and the bind shows with its keys beside it. `SUPER + F1` lists them all. These are the ones you'll use every day:

| Keys | What |
|---|---|
| `SUPER + Return` | Terminal (Ghostty) |
| `SUPER + CTRL + Return` | Terminal in tmux session `main` |
| `SUPER + Space` | Command menu (Kotodama): apps, projects, scripts, windows, binds, KubaOS's actions and the system toggles ("wifi", "volume 40", "next wallpaper"). Type a sum to calculate. It opens on the focused screen |
| `SUPER + Q` | Close window |
| `SUPER + h/j/k/l` | Focus the screen left, below, above, right |
| `SUPER + ←/↓/↑/→` | Focus the window that way. `SUPER + SHIFT + h/j/k/l` moves the window, `SUPER + CTRL + h/j/k/l` resizes it |
| `SUPER + 1..0` | Workspace 1-10 of the focused screen. `+ SHIFT` takes the window along, `+ SHIFT + ALT` sends it without following |
| `SUPER + SHIFT + CTRL + →` | Move the window to a new workspace after the screen's last |
| `SUPER + Tab` / `SUPER + CTRL + →/←` | Next / previous workspace on the focused screen (`+ SHIFT` on Tab for previous); `SUPER + CTRL + Tab` its former one |
| `SUPER + ALT + Tab` | Next window in a group (`+ SHIFT` for previous) |
| `SUPER + CTRL + A` / `B` / `W` | Control center: audio, Bluetooth, Wi-Fi |
| `SUPER + CTRL + Space` | Wallpaper picker (the colors follow) |
| `SUPER + SHIFT + Space` | Next keyboard layout, with two or more |
| `Print` / `SUPER + SHIFT + S` | Screenshot a region, copied and saved |
| `SUPER + SHIFT + R` | Record a region, same key stops |
| `SUPER + Escape` | Power menu |
| `SUPER + CTRL + Escape` | Lock |
| `SUPER + ALT + Return` / `SUPER + ALT + Space` | foot and fuzzel: the emergency terminal and app menu, for when the shell is dead |

The layouts are the ones you picked in Kōbō, the first the default (`/etc/default/keyboard`, `us` if there is no such file). The console and the disk passphrase prompt type with the default. With two or more, the bar shows the one in use ("US", "IT"); click it, press `SUPER + SHIFT + Space` or pick "Next keyboard layout" in the Command menu for the next. The Command menu has each layout ("Keyboard: Italian") to switch to it, make it the default or remove it, plus "Keyboard: add a layout" and "Keyboard: set layouts". `kubaos keyboard show|set|add|remove|default` does the same from a terminal, through `localectl`, and reloads Hyprland. Number keys are bound by keycode, so they're the top-row keys whatever the layout says.

## Updating and rolling back

Nothing updates by surprise. No popups, no "restart to finish installing", no "Later" button. Hyprland and its libraries are held in apt. Your Nix packages are pinned in your flake's lock (`~/.config/kubaos/home/flake.lock`). KubaOS's own (Quickshell, home-manager, the icons, the lock screen's PAM modules, the recorder, the GPU drivers) are pinned in KubaOS's lock and move only with a KubaOS release. unattended-upgrades only installs Ubuntu security updates. Everything else waits until you run `kubaos update`, or pick Update KubaOS in the menu (`SUPER + Space`), which opens a terminal running it. A daily timer runs `kubaos update --check`: it only looks. When there is something new it sends one notification, puts a marker in the bar, and the menu's Update KubaOS says what is waiting.

```sh
kubaos update                    # the whole update, after showing what it brings and asking
kubaos update -y                 # without asking
kubaos update --packages-only    # apt and your flake; KubaOS stays at its release
kubaos update --check            # only look; writes ~/.local/state/kubaos/update-available
kubaos rollback                  # undo the last update: KubaOS's release and your flake.lock
kubaos channel                   # which channel this machine follows
kubaos channel stable            # or edge
kubaos upgrade                   # stable only: move to the next series (Upgrade to <codename> in the menu)
```

An update, in order:

1. Refuses when another update runs, when `/` or `/nix` has less than 3 GiB free, or when `origin` can't be reached.
2. Picks the release. On **edge** (a machine installed from a build that wasn't a verified release) that's `origin/main`'s head; with `/usr/share/kubaos-keys/allowed_signers` on the machine the head must be signed by one of its keys, without it the update warns once and goes on. On **stable** it's the newest verified `v*` tag of the installed series; with no such tag it says so and only updates packages.
3. Shows the release's commits, the apt packages, a Hyprland pin change and the system migrations it will run, and asks.
4. Takes a snapper snapshot, copies your `flake.lock` to `~/.local/state/kubaos/locks/`, and adds a line to `/var/lib/kubaos/update-history`.
5. `apt-get upgrade`, then moves `/usr/share/kubaos`'s `release` branch.
6. When the release's `system/files/apt/preferences.d/kubaos-hyprland` differs from the installed one (amd64): unholds the Hyprland stack, installs the pinned versions, holds it again.
7. Runs the release's new system migrations (`migrations/system/`, as root), each once.
8. `nix flake update --flake ~/.config/kubaos/home`, `system/relink.sh` (the wallpaper, PAM and recorder links), and a switch.
9. Runs the new user migrations (`migrations/user/`, as you), each once, and writes `/var/lib/kubaos/release`.

Your Nix packages move with your flake's nixpkgs; KubaOS's keep their exact store paths until its release moves. Ways back, from smallest to largest:

- **Shell broken.** `SUPER + ALT + Return` opens foot and `SUPER + ALT + Space` opens fuzzel. Both come from apt and work without Nix or Quickshell.
- **The last update.** `kubaos rollback` (Roll back last update in the menu) moves `release` back to where that update found it, puts back the `flake.lock` it saved and switches. Migrations only go forward: what they changed stays.
- **User environment broken.** `home-manager generations`, then run the previous generation's `activate`.
- **System broken.** Boot the "before kubaos update" snapshot from GRUB's "KubaOS snapshots" menu, then swap `@` by hand ([`system/00-btrfs-subvolumes.md`](../system/00-btrfs-subvolumes.md), "Rolling back the system later"). `snapper rollback` does not work with Ubuntu's GRUB layout. apt runs take their own pre/post snapshots too. Your home isn't in the snapshot: `kubaos rollback` covers your flake.
- **Hyprland won't start.** `Ctrl+Alt+F2`, log in, `touch ~/.config/hypr/no-autostart`.

## Make it yours

Don't edit `/usr/share/kubaos` or the files in `~/.config/<app>`. Those are KubaOS's defaults: read-only, and replaced whole by every update. Your changes go in `~/.config/kubaos/<app>/`, which every app loads after ours, so what you set there wins. Updates never write to it. When we change a default you haven't touched, you get the new one; when you've set your own, you keep yours.

Each file starts as a comment and an example:

| To change | Edit | Applies |
|---|---|---|
| Hyprland: binds, gaps, monitors, rules | `~/.config/kubaos/hypr/hyprland.lua` (same Lua API as `~/.config/hypr/`) | `hyprctl reload` (`SUPER + SHIFT + CTRL + R`) |
| Idle, lock screen | `~/.config/kubaos/hypr/{hypridle,hyprlock}.conf` | restart hypridle / next lock |
| Shell aliases, exports | `~/.config/kubaos/zsh/zshrc` | new shell |
| tmux | `~/.config/kubaos/tmux/tmux.conf` | `tmux source-file ~/.config/tmux/tmux.conf` |
| Ghostty, foot, fuzzel | `~/.config/kubaos/{ghostty/config,foot/foot.ini,fuzzel/fuzzel.ini}` | reload or new window |
| git | `~/.config/kubaos/git/config`, `~/.config/kubaos/git/ignore` | at once (`ignore` after `dot-config apply`) |
| Neovim | `~/.config/kubaos/nvim/{options,keymaps,autocmds}.lua`, `plugins/*.lua`; extras with `:LazyExtras` | restart Neovim |
| lazygit, bacon, yazi | `~/.config/kubaos/{lazygit/config.yml,bacon/prefs.toml,yazi/yazi.toml,yazi/keymap.toml}` | next start (lazygit and bacon: in a new shell) |
| bat, fastfetch | `~/.config/kubaos/{bat/config,fastfetch/config.jsonc}` | after `dot-config apply` |
| btop, tms | `~/.config/btop/btop.conf`, `~/.config/tms/config.toml`: they rewrite their own file, so it's yours as is | next start |
| Command menu: your own actions, hiding KubaOS's | `~/.config/kubaos/kotodama/user.toml` (below) | on save |
| Nix packages and home-manager options | `~/.config/kubaos/home/home.nix`, e.g. `home.packages = [ pkgs.hello ];` | `home-manager switch --flake ~/.config/kubaos/home` |
| A package, without writing Nix | `kubaos package add ripgrep` (`remove`, `list`); `kubaos try cowsay` opens a shell with it and installs nothing | the switch `add` and `remove` queue |
| A whole language setup (a Suite) | `kubaos suite list`, `kubaos suite add NAME [--version V] [--extra E]`, `configure`, `remove [--purge-caches]`; `kubaos suite new NAME PROJECT` starts a project in `~/Projects` | the switch `add`, `configure` and `remove` queue |

- To see what KubaOS ships for an app, read `~/.local/share/kubaos/defaults/<app>/` (or `config/<app>/` in the repo).
- Delete a file in `~/.config/kubaos` to get the example back on the next `dot-config apply` or update.
- Quickshell's settings, the colors (matugen) and the prompt (oh-my-posh) can't be overridden yet.
- Your own wallpapers go in `~/Pictures/wallpapers/`, the picker's last tab; KubaOS's sets are in `/usr/share/kubaos/wallpapers/`.
- `~/.config/kubaos/home/` is your flake: KubaOS wrote it once, and an update never does again. Put it under git if you like (then `git add` new files of yours before a switch; the `kubaos` command does that for what it writes into `kubaos.d/`).
- The Command menu's KubaOS actions and toggles are listed in `~/.local/share/kubaos/defaults/kotodama/kubaos.toml`, each under `[extensions.kubaos.actions.<id>]`. Add your own in `user.toml` the same way, and hide any of KubaOS's, actions or toggles (`wifi`, `bluetooth`, `volume`, `brightness`, `power-saver` and so on), by id:

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

  Keys set under one of KubaOS's ids change only those keys of its action. `terminal = true` runs it in a terminal, and `delay_ms` waits that long first, so the menu is gone before a screenshot. The Extension logs what it can't use, such as an id it doesn't know (`journalctl --user -u kotodamad`).
- `kubaos package add` records the package in `~/.config/kubaos/home/kubaos.d/packages.toml`, a file the command owns, and switches. Every switch (`kubaos switch`, or one an `add` queued) runs in a systemd user unit, one at a time: closing the terminal doesn't stop it, and `kubaos job list` and `kubaos job follow <id>` show it. The first `add` on an older install adds `kubaos.stateDir = ./kubaos.d;` to your `home.nix`, after backing it up to `~/.local/state/kubaos/backups/home.nix.bak`. If your flake is under git, the command records each file it writes with `git add --intent-to-add` (and drops the ones it deletes from the index), so Nix sees them; it never commits.
- `kubaos suite add` installs a Suite: its toolchain, environment and editor settings, which load before yours in `~/.config/kubaos/nvim/`, so your settings win. It writes `kubaos.d/suites/<name>.toml` and creates any file of yours it has an example for only if that file is missing. `kubaos suite remove` takes it all away again, shows the size of its caches and keeps them unless you pass `--purge-caches`, lists your files that mention the Suite, and never touches your projects.

## Hacking on KubaOS

KubaOS itself is the root-owned checkout `/usr/share/kubaos`. To change it, clone it anywhere and point your flake at the clone for a switch:

```sh
git clone https://github.com/SuplenC/kubaos.git ~/Projects/kubaos
home-manager switch --flake ~/.config/kubaos/home --override-input kubaos git+file://$HOME/Projects/kubaos
```

`--override-input` doesn't touch your `flake.lock`, so a plain `home-manager switch --flake ~/.config/kubaos/home` goes back to the installed release. The clone's flake sees files git knows about: `git add` new ones first. `docs/CONTRACTS.md` ("Defaults and the user layer") has the details.

## Daily commands

| Command | What |
|---|---|
| `home-manager switch --flake ~/.config/kubaos/home` | Apply your `home.nix` |
| `dot-theme set <image>` | Set a wallpaper (an image or a loop) and regenerate colors everywhere |
| `dot-theme next`, `prev`, `toggle-folder` | Cycle wallpapers (stills and loops), cycle the folders: the sets you picked, your own `~/Pictures/wallpapers` |
| `~/.config/kubaos/wallpapers/sets` | What the picker shows, one per line: a set's name for all of it, `<set>/<file>` for one wallpaper (every set in `/usr/share/kubaos/wallpapers` without the file). A set a new release brings shows up once you add it; `dot-theme all-sets` (Command menu: Show every wallpaper set) lists every set again |
| `dot-theme apply [--force]` | Regenerate from the current wallpaper; without `--force` it does nothing when no input changed (login runs it) |
| `kubaos cursor style dark\|accent\|classic`, `kubaos cursor size <16-96>` | The pointer: dark with a light outline, the wallpaper's accent, or plain Bibata black and white, and its size. Applies at once and is kept in `~/.config/kubaos/cursor/cursor.toml`; the Command menu has both ("cursor") |
| `kubaos keyboard show`, `kubaos keyboard set us,it(intl)`, `kubaos keyboard add\|remove\|default <layout>` | The keyboard layouts in `/etc/default/keyboard`, the first the default. Written through `localectl`, which may ask for your password, then Hyprland reloads. At most 4, and the last one stays; the Command menu has them all ("keyboard") |
| `dot-config apply` | Create missing files in `~/.config/kubaos` and rewrite the merged configs (bat, git ignore, fastfetch) from the defaults plus yours. Every `home-manager switch` runs it |
| `qs ipc show` | List the Quickshell IPC targets Hyprland binds call |
| `dot-hypr-record region\|window\|output [--audio] [--mic]` | Start or stop a recording (gpu-screen-recorder, `~/Videos/Recordings`); `replay` keeps the last 30 s, `replay save` writes them. Run `system/65-recorder.sh` once so monitor capture needs no password |
| `dot-bw-setup [--refresh]` | Log in to Bitwarden and write SSH keys (public halves), Host blocks, git signing, gh token and project env from the vault |
| `dot-bwenv pull <project>` | Refresh one project's env secrets in the keyring; `use bwenv <project>` in `.envrc` loads them on `cd` |
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

Benchmark on `arms`, with a separate target dir per linker so they don't invalidate each other, and without sccache so clean builds aren't served from its cache:

```sh
cd ~/Projects/arms
export RUSTC_WRAPPER=   # empty: cargo skips sccache
wild='CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=--ld-path=wild"'

# Incremental: touch the binary's crate, so the run is mostly linking.
hyperfine --warmup 2 --runs 10 --prepare 'touch src/backend/rms/src/main.rs' \
  -n mold 'cargo build -q -p rms --target-dir target/bench-mold' \
  -n wild "$wild cargo build -q -p rms --target-dir target/bench-wild"

# Clean: everything rebuilt; the linker is a small share of this.
hyperfine --runs 3 \
  --prepare 'cargo clean -q --target-dir target/bench-mold' \
  --prepare 'cargo clean -q --target-dir target/bench-wild' \
  -n mold 'cargo build -q -p rms --target-dir target/bench-mold' \
  -n wild "$wild cargo build -q -p rms --target-dir target/bench-wild"

rm -rf target/bench-mold target/bench-wild
```

`cargo build --timings` on the same two target dirs shows where the time goes.

## Under the hood

For the people who care. The reasoning is in [`docs/PLAN.md`](PLAN.md), the interfaces between the parts in [`docs/CONTRACTS.md`](CONTRACTS.md).

- **Base:** Ubuntu Server 26.04 LTS ("resolute"), minimal, with snapd, cloud-init and the Ubuntu Pro adverts ripped out. apt owns the system: kernel, firmware, drivers, Hyprland, PAM, Docker, Chrome. `/etc/os-release` says KubaOS where a person reads it and stays Ubuntu where apt and install scripts look.
- **Disk:** LUKS2, then btrfs with flat subvolumes (`@ @home @snapshots @log @cache @nix @docker @swap`), snapper with timeline and apt pre/post snapshots, grub-btrfs so snapshots are bootable from GRUB.
- **Session:** autologin on tty1 after the LUKS unlock, zsh, `start-hyprland`. No display manager.
- **Compositor:** Hyprland 0.56 from `ppa:cppiber/hyprland`, held in apt, with a Lua config split into files (`config/hypr/`). Dwindle layout, Omarchy 4 keybinds moved to hjkl. hypridle locks after 5 min, turns screens off after 10, and suspends after 30 on battery only.
- **Shell:** my own, written in QML on Quickshell v0.3.2 (pinned), in the structure of Caelestia: bar, notifications, OSD, control center, power menu, wallpaper picker, lock screen with password, fingerprint and security key in parallel. The Command menu is [Kotodama](https://github.com/SuplenC/kotodama), drawn by the shell in its colors.
- **User environment:** standalone home-manager (nixos-26.05) owns everything in `$HOME`. KubaOS's configs are read-only links from the store into `~/.config`, and yours live in `~/.config/kubaos` (above, "Make it yours"). Nix apps get the GPU through home-manager's `targets.genericLinux.gpu`.
- **Theming:** matugen (`scheme-fidelity`) turns the wallpaper into Material 3 colors and renders 22 files in `~/.local/state/theme/` for every consumer, the pointer included: Bibata, recoloured. Geist, M PLUS 1 for the Japanese bits, Material Symbols, Papirus.
- **Terminal and dev:** Ghostty (foot as the fallback), tmux + tms, Neovim 0.12 (LazyVim), rustup, proto + moon, sccache, clang + mold (wild optional), Docker Engine + lazydocker, Chrome.
- **Secrets:** Bitwarden desktop's SSH agent (private keys never touch the disk), `bw` for SSH Host blocks, git signing, the gh token and per-project env through direnv. A YubiKey C Bio does sudo, polkit and the lock screen with a fingerprint; the password stays as the fallback.
- **Hardware:** `tuxedo-drivers` and power-profiles-daemon (no TUXEDO Control Center), fprintd, bolt for the dock.
- **Install:** Kōbō, an iced app, on a casper live ISO, with the Rust engine (`kubaos-core`, also behind the `kubaos` CLI) running inside it. It unpacks a prebuilt image with `/nix` and the home-manager generation already inside, which is why it needs no network.

### Repo map

| Path | What |
|---|---|
| `flake.nix`, `flake.lock` | Inputs (nixpkgs 26.05, home-manager, Quickshell v0.3.2) and the `kuba@tuxedo` / `kuba@vm` outputs |
| `home/` | home-manager modules: packages, zsh, tmux plugins, GPU wiring, GTK/Qt theme, defaults and the user layer (`links.nix`) |
| `config/` | The defaults, one dir per app, linked read-only into `~/.config`: `hypr`, `quickshell`, `nvim`, `tmux`, `tms`, `ghostty`, `foot`, `fuzzel`, `matugen`, `ohmyposh`, `lazygit`, `yazi`, `btop`, `bat`, `bacon`, `git` |
| `scripts/` | `dot-*` commands, linked to `~/.local/bin` (`dot-theme`, `dot-config`, `dot-hypr-*` including screenshots and `dot-hypr-record`, `dot-bw-setup`, `dot-bwenv`, `dot-u2f-enroll`, `dot-ssh-sk-new`) |
| `user-layer/` | The starting files of `~/.config/kubaos`, one commented example per app |
| `system/` | Ubuntu bootstrap scripts (`10-strip.sh` to `75-yubikey.sh`), `relink.sh`, which `kubaos update` runs, files for `/etc` |
| `crates/` | The Rust workspace (root `Cargo.toml`): `kubaos-core`, the engine, `kubaos-cli`, the `kubaos` command, `kubaos-kobo`, Kōbō, `kubaos-cursor`, the pointer recoloured from the wallpaper, and `kotodama-kubaos`, the Command menu's KubaOS actions and system toggles |
| `install/` | Putting KubaOS on a machine: the payload build, the live ISO, the first-boot files and the engine's end-to-end test |
| `test/` | The Docker test harness (`test/run`) |
| `docs/` | `PLAN.md` (why), `INSTALL.md` (how), `CONTRACTS.md` and `INSTALL-INTERNALS.md` (interfaces), `SECRETS.md` (Bitwarden, security key), `TESTING.md`, `BRAND.md` (name and voice), `screenshots/` |

## Testing

Two benches, both on the Mac.

- **`test/run`** builds an arm64 Ubuntu 26.04 image with the repo's own `system/*.sh`, keeps the Nix store in a volume, and on every run does a real `home-manager switch` and the checks below. `test/run e2e` takes about 42 s warm. `test/run shots` starts a headless session with Quickshell for screenshots. Commands, caches and timings: [`docs/TESTING.md`](TESTING.md).
- **`test/vm/run`** drives QEMU: boots the live ISO, runs a headless install, boots the result, takes screenshots and snapshots. See [`docs/INSTALL-INTERNALS.md`](INSTALL-INTERNALS.md), "Test bench".

Verified in an arm64 `ubuntu:26.04` container (user `kuba`, hostname `vm`, Nix installed with the official `--daemon` installer, daemon started by hand because there is no systemd):

- `system/10-strip.sh`, `20-base.sh`, `50-hyprland.sh` (Ubuntu's arm64 hyprland, not the PPA) and `60-nix.sh` run clean; systemd, netplan and snapper steps skip themselves.
- `home-manager switch --flake .#kuba@vm` builds and activates; every file in `~/.config/<dir>` and `~/.local/bin/dot-*` is a link into the store.
- zsh login shell loads without errors (zoxide doctor quiet), PATH order, atuin Ctrl-R, direnv hook, oh-my-posh prompt, `.zprofile` sources the autostart snippet.
- `dot-theme apply` and `dot-theme set` write every file in `~/.local/state/theme/`; tmux, Neovim (LazyVim from the lockfile), Ghostty config validation and tms work with the real configs.
- PAM: through Nix's libpam (the same build Quickshell links), `qs-lock` accepts the right password and rejects a wrong one as a non-root user, using Ubuntu's `unix_chkpwd` linked to `/run/wrappers/bin`. Ubuntu's own modules don't load under Nix's libpam; details in `docs/INSTALL.md`.
- Secrets, against Vaultwarden 1.37 with the official `bw` 2026.10.0 (arm64 standalone, sha256-checked by `70-bitwarden.sh`): `dot-bw-setup` from logged-out to done (server prompt, password login, unlock), writes the `.pub` files, Host blocks (`ssh -G` picks the right `IdentityFile`), git signing (a commit verifies with "Good git signature" through an agent on `~/.bitwarden-ssh-agent.sock`) and keyring entries; rejects a vault field that tries to inject an ssh_config line; leaves no secret in plaintext under `$HOME`. direnv loads 5 variables (multi-line PEM and shell metacharacters intact) on `cd` in zsh and unloads them on leave; `direnv export` takes 22 ms. The agent was OpenSSH's on Bitwarden's socket path (no arm64 desktop build); gh login was tested only up to GitHub rejecting a fake token.
- Security key PAM without a key: `sudo`, `sudo-i` and `polkit-1` with the pam_u2f line accept the right password in about 50 ms, reject a wrong one, also with no mapping or no mapping file, and through sudo-rs. Under Nix's libpam, Ubuntu's `pam_u2f.so` fails ("Module is unknown": `libfido2.so.1` not found) and nixpkgs' through the root-owned link loads and fails in 17 ms with no key. The lock screen runs `qs-lock-u2f` every 4 s while locked, stops when the mapping goes away, and shows the hint.
- Hyprland 0.56.2 (nixos-unstable) runs the Lua config unchanged in a NixOS VM (QEMU, virtio-gpu, llvmpipe): `--verify-config` passes, windows round and blur, panels open over windows without resizing them. Slow without KVM, so only a few stills.
- Quickshell v0.3.2 from Nix loads the full config in headless sway (Mesa llvmpipe OpenGL) with no QML errors, and `qs ipc show` lists every contract target. Every panel, the lock screen and the terminal stack were screenshotted there (`docs/screenshots/README.md`).
- Screen recording: `dot-hypr-record` start/stop, the state file and the bar indicator (elapsed time, gone on stop), stopped through `qs ipc call recorder stop`, with wf-recorder under headless sway; gsr's failure path without a GPU ("Recording failed" notification, state back to idle); satty opens from `dot-hypr-screenshot edit`. `system/65-recorder.sh` installs the setcap'd helper and is idempotent, but Docker refuses to exec a file capability, so gsr's KMS capture, VA-API encoding, region crop at scale 1.5 and replay need the laptop.

Needs the real machine: LUKS + Btrfs subvolumes and the rollback drill, Hyprland 0.56 from the PPA (amd64 only), the Lua config under a real compositor, GPU wiring (`non-nixos-gpu-setup`), fingerprint reader, tuxedo-drivers and power profiles, USB4 dock with two 120 Hz monitors, SentinelOne, lid and idle behaviour.

## Known limitations

- Real 120 Hz smoothness, Hyprland's blur behind the shell and the workspace animations can only be judged on the machine.
- The monitor descriptions in `config/hypr/monitors.lua` are placeholders until the UltraGears are plugged in once (`hyprctl monitors all`).
- Fingerprint unlock in the Quickshell lock screen is untested: no reader in a container. nixpkgs' `pam_fprintd.so` loads and answers like Ubuntu's does without a reader. It's off now (`Settings.fingerprint`); hyprlock is the fallback.
- Untested without hardware: a real FIDO2 touch (sudo, polkit, lock screen), Bitwarden desktop's SSH agent and its polkit unlock (amd64 only), and `bw` with real two-step login. `bw` can't do FIDO2 two-step login; `dot-bw-setup --login apikey` is the way in for a key-only account (`docs/SECRETS.md`).
- The in-shell polkit agent is off; hyprpolkitagent is the agent until the Quickshell one is tested on the machine.
- The arm64 VM can't run the Hyprland Lua config from apt (the PPA is amd64 only, Ubuntu's own build is 0.53).
- grub-btrfs is built from source (not packaged for 26.04) and isn't tracked by dpkg.
- The official Nix installer has no uninstaller; the manual steps are in `docs/INSTALL.md`.

## Credits

- [Omarchy](https://omarchy.org) for the keybinds, per-screen workspaces (`config/hypr/workspaces.lua` adapts its `monitor-workspaces.lua`, MIT) and the idea that installing a desktop should take one stick and a few minutes.
- [Caelestia](https://github.com/caelestia-dots/shell) for the shell's structure: the frame, panels that grow out of it, the Material 3 motion. No code was taken; the shell here is written from scratch.
- [Quickshell](https://quickshell.org), [Hyprland](https://hyprland.org), [matugen](https://github.com/InioX/matugen), [home-manager](https://github.com/nix-community/home-manager) and Ubuntu, which do the actual work.
- The wallpapers are by Pema G. Lama, Alex Knight, Jezael Melgoza, Marek Okon, Manuel Cosentino, LoboStudio Hamburg, ayumi kubo, Letian Zhang and Lucius Hunter on Unsplash, and Bobby Brown on Pexels (each set's `CREDITS.md` in [SuplenC/kubaos-wallpapers](https://github.com/SuplenC/kubaos-wallpapers)).
- Built with Claude. Kuba made the calls; the robots typed.
