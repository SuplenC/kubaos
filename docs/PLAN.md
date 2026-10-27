# KubaOS plan: TUXEDO InfinityBook Pro 14 Gen10, Ubuntu + Hyprland + own Quickshell

The plan KubaOS was built from, as of 2026-10-15. The KubaOS stick (`docs/INSTALL-INTERNALS.md`) now does most of these phases in one go, all but the TUXEDO drivers. The phases still describe what it installs and why. Reference hardware: the TUXEDO InfinityBook Pro 14 Gen10, with external screens on a USB4 dock.

## Guiding rules

1. **Nothing updates by surprise.** Hyprland is held in apt at the release's versions, KubaOS's own tools are pinned in the release's `flake.lock` and yours in your flake's. `kubaos update` moves them on purpose, after a snapshot; the daily check only says what's waiting.
2. **One job, one tool.** The shell does bar, launcher, notifications, OSD. Hyprland does windows. tmux does terminal panes. No second daemon for the same job.
3. **System from apt, user from Nix.** apt owns the kernel, drivers, Hyprland, PAM, Docker, Chrome. home-manager owns everything in `$HOME`. rustup and proto own project-pinned toolchains.
4. **Defaults apart from your changes.** KubaOS's configs, scripts and wallpapers are read-only links from the Nix store; yours live in `~/.config/kubaos`, load last and are never touched by an update (issue #9, `docs/CONTRACTS.md`). The same goes for Nix: your flake (`~/.config/kubaos/home`) has its own nixpkgs and lock and takes KubaOS from the root-owned `/usr/share/kubaos` as an input, so your packages update freely and KubaOS's move only with a release (issue #12, `docs/adr/0001-release-and-update-model.md`). This replaced the original live links into the dotfiles repo and its one shared lock, which made every update a merge.
5. **Always a way back.** Btrfs snapshots for the system, home-manager generations for the user environment, a minimal fallback session that works without your shell.

## Decisions (from the grilling)

| Area | Decision |
|---|---|
| Distro | Ubuntu Server 26.04 LTS, minimal install (TUXEDO `resolute` repo) |
| Disk | LUKS, password at boot, Btrfs with snapper |
| Login | Autologin into Hyprland after LUKS unlock |
| Power | `power-profiles-daemon` + `tuxedo-drivers`, no Control Center |
| Fingerprint | USB reader for lock screen, sudo, polkit |
| Compositor | Hyprland 0.56+ (PPA), Lua config, dwindle, Omarchy 4 keybinds with hjkl |
| Workspaces | Per screen, 1-10 each (Omarchy's block of ten ids per screen, known by make, model and serial); on unplug they join the remaining screen after its own, on replug they return. `SUPER+arrows` focus screens (#41) |
| Shell | Your own Quickshell config, Caelestia-inspired, Quickshell pinned via Nix to a tagged release |
| Theme | Material You colors from wallpaper (`matugen`), Japanese and Cyberpunk wallpaper folders with a toggle |
| Idle | Lock 5 min, screens off 10 min. Battery: suspend 30 min and on lid close. AC: never suspend |
| Terminal | Ghostty (no splits) + tmux (your config, restyled, no battery/clock) + tms + vim-tmux-navigator |
| Shell (zsh) | oh-my-zsh (`git`), oh-my-posh (your config), autosuggestions, fzf, zoxide as `cd`, atuin |
| Dotfiles | Nix home-manager (standalone, flake) |
| Toolchains | rustup (Rust), proto (team-pinned tools + moon), Nix (personal tools) |
| Apps | Chrome, Docker Engine + lazydocker, Noto + CJK fonts |
| Later | Docker widget, clipboard history, extensions, Flutter/Android |

---

## Phase 0: Before the install (on another machine)

- Download Ubuntu Server 26.04 ISO and write it to a USB stick.
- Optional: run `cargo build --timings` with `RUSTFLAGS="-Zthreads=8" cargo +nightly` on a large Rust workspace to have both baselines.

## Phase 1: Install Ubuntu Server

1. In the BIOS (F2 on TUXEDO): check that Secure Boot is on or off, as you like. `tuxedo-drivers` is DKMS, so with Secure Boot on you'll enroll a MOK key once on the first reboot after installing it.
2. Boot the USB with the Ethernet cable plugged in.
3. Choose **Ubuntu Server (minimized)**.
4. Storage, custom layout:
   - 1 GB EFI partition (FAT32, `/boot/efi`)
   - 2 GB `/boot` (ext4, unencrypted, GRUB reads kernels from here)
   - The rest: LVM volume group **with encryption enabled** (this is the LUKS password), one logical volume formatted **Btrfs** mounted at `/`
5. Select OpenSSH server only if you want to SSH into the laptop. Skip all the featured server snaps.
6. Reboot, type the LUKS password, log in on the console.

**Btrfs subvolumes.** The server installer formats Btrfs without an `@` / `@home` subvolume layout, which snapshots and rollback need. Fix it right after install from the live USB: `system/00-btrfs-subvolumes.md` creates `@ @home @snapshots @log @cache @nix @docker @swap`, rewrites `/etc/fstab` with `subvol=@...`, and rebuilds GRUB. Use the device names `lsblk` shows. Do it before installing anything else, while there's nothing to lose.

## Phase 2: Strip the server parts

```sh
sudo apt purge snapd cloud-init multipath-tools lxd-installer modemmanager
sudo apt-mark hold snapd   # plus an apt preferences pin so nothing pulls it back
sudo rm -rf /etc/cloud /var/lib/cloud ~/snap
```

- Disable the Ubuntu Pro adverts: `sudo pro config set apt_news=false` and disable `motd-news`.
- Retire `unattended-upgrades` and the `apt-daily` timers (masked): `kubaos update` owns updates, and its daily check notifies about pending apt security updates.

## Phase 3: Base system, network, audio, plumbing

```sh
sudo apt install --no-install-recommends \
  network-manager pipewire pipewire-pulse pipewire-alsa wireplumber libspa-0.2-bluetooth \
  bluez dbus-user-session xdg-user-dirs xwayland qt6-wayland qtwayland5 \
  xdg-desktop-portal-gtk gnome-keyring libsecret-1-0 \
  linux-firmware mesa-vulkan-drivers mesa-libgallium vainfo fwupd \
  bolt brightnessctl upower power-profiles-daemon \
  fprintd libpam-fprintd \
  build-essential pkg-config libssl-dev clang mold git curl \
  btrfs-progs snapper \
  fonts-noto fonts-noto-color-emoji fonts-noto-cjk \
  foot fuzzel
```

- **Network:** set `renderer: NetworkManager` in `/etc/netplan/*.yaml`, `netplan apply`, then disable `systemd-networkd` and `systemd-networkd-wait-online`. Connect Wi-Fi with `nmtui` and unplug the cable.
- **`foot` and `fuzzel`** are the fallback terminal and launcher. They come from apt, so they work even if the Nix GPU wiring or your shell breaks.
- The real list is in `system/20-base.sh`. `mesa-va-drivers` no longer exists in 26.04 (VA-API is in `mesa-libgallium`).
- **Snapper:** `snapper -c root create-config /`, enable the timeline and cleanup timers, and `grub-btrfs` so snapshots show up in the boot menu (not packaged for resolute; `20-base.sh` builds the pinned upstream release). Take a first snapshot: `snapper create -d "clean base"`.
- **Rollback** is a manual `@` swap, not `snapper rollback`: Ubuntu's GRUB hardcodes `rootflags=subvol=@`, so changing the default subvolume does nothing. Boot the snapshot from the GRUB "KubaOS snapshots" menu, then follow "Rolling back the system later" in `system/00-btrfs-subvolumes.md`.
- **Fingerprint:** plug in the USB reader, `fprintd-enroll`, then `pam-auth-update` and tick fingerprint. That covers sudo and polkit. Check first that the reader is on the libfprint supported list.

## Phase 4: Hardware and apps

- **TUXEDO repo** (`deb.tuxedocomputers.com/ubuntu`, suite `resolute`), then `sudo apt install tuxedo-drivers`. Do not install `tuxedo-control-center`.
- **Check power profiles:** `cat /sys/firmware/acpi/platform_profile_choices`. If it lists profiles, `powerprofilesctl set performance` changes real CPU limits and you're done. If not, revisit TUXEDO Control Center.
- **Dock:** plug it into the back USB4 port, `boltctl list`, `boltctl enroll <uuid>`.
- **Docker:** Docker's apt repo, `docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin`, add yourself to the `docker` group.
- **Chrome:** Google's apt repo, `google-chrome-stable`.
- `snapper create -d "drivers + apps"`.

## Phase 5: Hyprland

```sh
sudo add-apt-repository ppa:cppiber/hyprland
sudo apt install --no-install-recommends hyprland xdg-desktop-portal-hyprland hyprpicker hypridle hyprlock hyprpolkitagent
sudo apt-mark hold hyprland xdg-desktop-portal-hyprland hypridle hyprlock hyprpolkitagent
```

`hyprlock` and `hyprpolkitagent` are installed as **fallbacks**, not as the main tools. See the PAM note in Phase 8.

**Autologin:** a systemd drop-in for `getty@tty1` with `--autologin <user>`. `~/.zprofile` (home-manager) sources `system/files/hyprland-autostart.zsh` last, which execs `start-hyprland` on tty1 (the PPA ships no uwsm). `touch ~/.config/hypr/no-autostart` disables it.

**Lua config** in `~/Projects/dotfiles/config/hypr/`, split into files:
- `monitors.lua`: every screen at its preferred mode, placed and scaled by Hyprland; a layout of your own goes in the user layer's `hyprland.lua`
- `workspaces.lua`: per-screen workspaces, 1-10 on each screen
- `keybinds.lua`: ported from Omarchy 4 (already Lua), window focus on `SUPER+hjkl`, screen focus on `SUPER+arrows`, move window `SUPER+SHIFT+hjkl`, resize `SUPER+CTRL+hjkl`, workspaces `SUPER+1..0`
- `fallback.lua`: `SUPER+ALT+Return` foot and `SUPER+ALT+Space` fuzzel, always bound, so you can work with the shell dead
- `autostart.lua`: starts your shell, `hypridle`, the lid sync on hotplug

**Workspace return on replug:** Hyprland moves workspaces to another screen when one disappears, and knows the screen only by its connector, which a dock renames. `workspaces.lua` renumbers them into the remaining screen's block on `monitor.removed` and returns them on `monitor.added`, by the screen's description (#41).

**Idle and lid** (hypridle + logind):
- `/etc/systemd/logind.conf.d/lid.conf`: `HandleLidSwitch=suspend`, `HandleLidSwitchExternalPower=ignore`, `HandleLidSwitchDocked=ignore`
- hypridle: 5 min lock, 10 min DPMS off, 30 min suspend wrapped in a check that skips it when `/sys/class/power_supply/AC*/online` is 1
- Hyprland lid bind: on lid close, disable the laptop panel; on open, enable it

`snapper create -d "hyprland base"`.

## Phase 6: Nix and home-manager

1. Install Nix multi-user (official installer, `system/60-nix.sh`), flakes enabled.
2. Dotfiles repo layout:

```
~/Projects/dotfiles
├── flake.nix            # inputs: nixpkgs, home-manager, quickshell (tagged)
├── flake.lock           # the pin: nothing updates until you run nix flake update
├── home/
│   ├── default.nix
│   ├── shell.nix        # zsh, omz, oh-my-posh, fzf, zoxide, atuin, direnv
│   ├── cli.nix          # the tool list below
│   ├── dev.nix          # rust helpers, sccache, proto PATH, git, delta
│   ├── terminal.nix     # ghostty, tmux, tms
│   ├── desktop.nix      # quickshell, matugen, fonts, GTK/Qt theme, gpu wiring
│   └── links.nix        # mkOutOfStoreSymlink for live configs
└── config/
    ├── nvim/  hypr/  quickshell/  tmux/  tms/  ghostty/  ohmyposh/  matugen/
```

3. **GPU wiring for Nix apps on Ubuntu:** home-manager's `targets.genericLinux.gpu` (one-time `sudo non-nixos-gpu-setup`, done by `60-nix.sh`). Test with Quickshell and Ghostty before building anything on top.
4. `home-manager switch --flake ~/Projects/dotfiles` (hostname `tuxedo` picks `kuba@tuxedo`; new files need `git add` first, flakes only see tracked files), then `snapper create -d "home-manager base"`.

**Tools in `cli.nix`:** neovim, lazygit, yazi, tmux, tmux-sessionizer (tms), ghostty, fzf, zoxide, atuin, direnv + nix-direnv, eza, bat, fd, ripgrep, delta, dust, dysk, btop, jq, yq, xh, tealdeer, just, glow, gh, fastfetch, lazydocker, bacon, cargo-nextest, hyperfine, sccache, oh-my-posh, matugen, JetBrainsMono and Cascadia Nerd Fonts, Material Symbols, Geist, M PLUS 1.

**Outside Nix, on purpose:**
- `rustup` from rustup.rs. Respects `rust-toolchain.toml`. Add `rust-analyzer` and `clippy` components.
- `proto` from its install script with `--no-profile` (`~/.zshrc` is home-manager's, read-only), then `proto install moon` and whatever your projects' `.prototools` pin. home-manager sets `PROTO_HOME` to `~/.local/share/proto` and adds its `shims` and `bin` to PATH.
- Bun through proto, so it matches the team.
- `make` from `build-essential` (already installed).

**`~/.cargo/config.toml`** (managed by home-manager):
- `linker = "clang"` with `-C link-arg=-fuse-ld=mold` for `x86_64-unknown-linux-gnu`; wild instead is one switch in `home/dev.nix` (`rustLinker`, see README "Rust")
- `rustc-wrapper = "sccache"`, `SCCACHE_CACHE_SIZE=10G` (sccache's own default; more in the user flake)

## Phase 7: Port zsh, tmux, tms, nvim

- **zsh** (`shell.nix`): oh-my-zsh with `plugins = [ "git" ]`, oh-my-posh pointed at your `config.toml` (its colors read from the matugen output), zsh-autosuggestions, `fzf --zsh` for `Ctrl+T` / `Alt+C`, **atuin** for `Ctrl+R`, `zoxide init zsh --cmd cd` last, `direnv hook`. It ships `y()` (yazi that leaves you in the folder it ended in) and the `zshrc` and `wipe` aliases; personal aliases go in the user layer's zshrc.
- **tmux:** your repo, live-linked. Changes: remove battery and clock from the status bar, source a colors file generated by matugen, add `vim-tmux-navigator`. Plugins move from TPM to Nix (`programs.tmux.plugins`), so they're pinned like everything else.
- **tms:** live-linked config, same search paths updated from `~/Projects` on macOS to the same path on Linux.
- **Neovim:** your config, live-linked, unchanged except the colorscheme, which reads the matugen-generated palette (for example through `mini.base16` or a small Lua file matugen writes). Add `vim-tmux-navigator`. Mason still works, because Ubuntu is a normal FHS system.

`snapper create -d "dev environment"`. At this point you can do a normal workday with the fallback session, before any shell exists.

## Phase 8: Your Quickshell shell, in stages

Built with Claude, inspired by Caelestia, lives in `config/quickshell/`. Each stage ends usable. Commit after each.

| Stage | Components | Notes |
|---|---|---|
| 1 | Bar: workspaces (per monitor), active window, clock, battery, network, audio, tray | One bar per monitor, follows hotplug |
| 2 | Notification daemon, volume and brightness OSD | Only start once fuzzel/no-notifications is annoying |
| 3 | Launcher with app search and calculator (`libqalculate`) | Replaces Raycast (a) and (c) |
| 4 | Lock screen with PAM (password + fingerprint) | See note below |
| 5 | Polkit agent | See note below |
| 6 | Wallpaper picker: two folders, toggle key, runs `matugen` to regenerate colors for shell, Ghostty, tmux, nvim, GTK/Qt | |
| 7 | Power menu, power-profile toggle, Bluetooth and Wi-Fi panels | |
| Later | Docker widget (containers, start/stop, logs via lazydocker), clipboard history, extensions | |

**PAM note (stages 4 and 5).** A Nix-built Quickshell on Ubuntu may fail to authenticate, because Nix's PAM library can't always load Ubuntu's PAM modules (fingerprint, `unix_chkpwd`), and polkit agents need a setuid helper that Nix can't provide outside NixOS. Test both on day one of stage 4. If they fail, the lock screen and polkit agent stay as the apt-installed `hyprlock` and `hyprpolkitagent` (themed from the same matugen colors), and the rest of the shell stays yours. You lose nothing either way; only who draws that one screen changes.

## Phase 9: Theming

- `matugen` templates in `config/matugen/` for: Quickshell, Ghostty, tmux, oh-my-posh, Neovim, GTK 3/4, Qt (`qt6ct`), hyprlock fallback, Hyprland borders.
- Wallpapers: the sets in `/usr/share/kubaos/wallpapers/` (picked at install) and your own in `~/Pictures/wallpapers/`. One key cycles within a folder, another switches folders. Each change regenerates all colors.
- Cursor and icons: Bibata recoloured from the wallpaper by `kubaos cursor` (hyprcursor and XCursor), and Papirus through home-manager.

## Update routine

Weekly or whenever the bar's marker shows: `kubaos update` (or Update KubaOS in the menu). It takes a snapper snapshot, saves your `flake.lock`, upgrades apt, moves `/usr/share/kubaos`'s `release` to the channel's next release, installs that release's Hyprland pin, runs the system migrations, updates your flake, relinks, switches, then runs your migrations. `docs/USING.md` ("Updating and rolling back") has the details and the ways back.

## Moving to Fedora or Arch later

- Install the base system with that distro's tools (Phases 1-5 change: dnf/pacman package names, TUXEDO's Fedora repo or AUR).
- Install Nix, clone `~/Projects/dotfiles`, `home-manager switch`. Phases 6-9 carry over unchanged, including your shell and its pinned Quickshell.

## Open checks to do on the machine

- [ ] Btrfs subvolume conversion done and the manual `@` swap rollback tested once
- [ ] `platform_profile` exposed by `tuxedo-drivers`
- [ ] USB fingerprint reader works with `fprintd`
- [ ] Every screen on the dock at its preferred mode (`hyprctl monitors`)
- [ ] Nix GPU wiring: Quickshell and Ghostty render with hardware acceleration
- [ ] Quickshell PAM: lock screen accepts password and fingerprint
- [ ] `cargo build --timings` on a large Rust workspace for the real comparison with the Mac
