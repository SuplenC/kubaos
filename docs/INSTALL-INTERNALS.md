# Putting KubaOS on a machine: the engine, Kōbō and the live ISO

An Omarchy-style install for KubaOS. You boot a USB stick, answer a few screens, and a minute or two later you reboot into the finished KubaOS. That includes Ubuntu, Hyprland, Quickshell and your home-manager generation, all ready to use. The install needs no network.

This file is the contract between the parts. If you need to change an interface, change it here in the same commit.

## How it works

1. **Payload.** We build the whole installed system ahead of time as one compressed image. It contains Ubuntu 26.04, everything `system/*.sh` installs, and `/nix` with the home-manager generation for `<user>@<host>` already built. It ships with `manifest.json` (`kubaos_core::Manifest`).
2. **Live ISO.** A small Ubuntu live system holding the payload, `kubaos` and `kobo`. It boots straight into Kōbō, full screen in `cage`. There's a text fallback on tty2.
3. **Engine.** `kubaos run` partitions the disk, sets up LUKS and btrfs, unpacks the payload, configures the system, creates the user, installs the bootloader, activates home and takes the factory snapshot. It reports progress as JSON events.
4. **Kōbō.** `kobo` (工房, "workshop") asks the questions and builds the `InstallConfig` and the `Secrets`. It runs the install in its own process through `kubaos_core::engine`, the same engine `kubaos run` drives, and shows the event stream. It checks what `kubaos run` checks before it starts, writes the same log, and a signal stops and unwinds the install as it does for `kubaos run`. It runs as root under `cage` on the live system, and its Reboot button runs `systemctl reboot`.
5. **First boot.** Kōbō runs in setup mode (`kobo --setup`, `kubaos-setup`) for anything that needs you or the hardware: TUXEDO drivers, the fingerprint reader, the YubiKey, Bitwarden, the git name and email. It opens by itself at the first login. See "First boot" below.

## Layout and owners

| Path | Owner | What |
|---|---|---|
| `Cargo.toml`, `Cargo.lock` | lead | The Cargo workspace: `crates/kubaos-core`, `crates/kubaos-cli` and Kōbō in `crates/kubaos-kobo` |
| `crates/kubaos-core/src/{config,event,step,probe,lib}.rs` | lead | The contract types |
| `crates/kubaos-core/src/engine.rs`, `crates/kubaos-core/src/engine/**`, `crates/kubaos-cli/**` | engine | The real install, `probe`, `validate`, `run` |
| `crates/kubaos-core/src/demo.rs` | lead | Fake event stream for work on Kōbō |
| `crates/kubaos-kobo/**` | kobo | Kōbō, the iced app |
| `crates/kubaos-core/src/setup{.rs,/**}`, `crates/kubaos-kobo/src/setup/**`, `crates/kubaos-cli/src/setup.rs`, `crates/kubaos-core/src/engine/install/firstboot.rs`, `install/firstboot/**` | firstboot | First-boot setup |
| `install/image/**` | image | Builds the payload (rootfs image + manifest) |
| `install/iso/**` | iso | Builds the live ISO |
| `test/vm/**` | iso | QEMU test bench on the Mac |

Agents only edit files they own. If you need a change in someone else's file, put it in your report.

## CLI (`kubaos`)

```
kubaos probe [--json] [--payload DIR]    machine facts -> Probe
kubaos validate --config FILE            exit 0 / 2 with reasons
kubaos plan --config FILE [--json]       the steps that would run
kubaos run --config FILE (--secrets FILE | --secrets-stdin)
           [--events json|human] [--dry-run]
           [--target DIR] [--log FILE] [-v]
kubaos demo [--speed N] [--fail-at STEP] [--events json|human]
kubaos setup status [--json] [--demo vm|tuxedo]     first-boot cards, as the user
kubaos setup run CARD [--input-stdin] [--events json|human] [--demo vm|tuxedo]
kubaos setup done                                     write the marker
kubaos package add|remove NAME... [--no-switch] [--detach|--foreground]
kubaos package list [--json]                          what kubaos.d/ installs, and from which file
kubaos package restore GENERATION                     kubaos.d/ back from a home-manager generation
kubaos try NAME                                       nix shell with NAME, installs nothing
kubaos suite list [--json] | kubaos suite show NAME [--json]
kubaos suite add NAME [--version V] [--extra E]... [--no-switch] [--detach|--foreground]
kubaos suite configure NAME [--version V] [--extra E]... [--no-extra E]... [--no-switch] [...]
kubaos suite remove NAME [--purge-caches] [--no-switch] [--detach|--foreground]
kubaos suite new NAME PROJECT [--with SCAFFOLDER]     a project in the Projects root
kubaos switch [--detach|--foreground]                 queue a switch of the user flake
kubaos job list [--json] | kubaos job follow ID       switch jobs
```

The package, suite, switch and job commands run as the user, never root: docs/CONTRACTS.md, "The state directory and switch jobs".

- Exit codes: `0` ok, `1` a step failed (the last event is `failed`), `2` usage or config error.
- `--events json`: one `Event` per line on stdout, nothing else on stdout. Human-readable logs go to stderr and to `/var/log/kubaos-install.log` on the live system. The log is copied to `/var/log/kubaos/` on the target.
- `run` must be root. `probe`, `validate`, `plan` and `demo` work unprivileged. `run --dry-run` needs neither root nor secrets and prints every command and file it would write, as `log` events.
- `probe --payload` defaults to `/run/kubaos/payload`. `validate` reads the manifest from the config's `[payload] dir`. `run` validates again before it starts (exit 2), then re-checks in `preflight`.
- `run --target` is where the target is mounted (default `/mnt/kubaos`), `--log` the live log (default `/var/log/kubaos-install.log`), `-v` shows `debug` lines in human mode: each command and its output. SIGINT, SIGTERM and SIGHUP stop the install: the running command is killed, everything is unmounted and closed, and the last event is `failed`.

## Files

`install.toml` (`InstallConfig`):

```toml
[locale]
lang = "en_US.UTF-8"
keyboard_layout = ["us", "it"]   # 1 to 4, the first the default; "us" or "us,it" too
keyboard_variant = ["", "intl"]  # one a layout, "" for its own; may be left out
timezone = "Europe/Rome"

[host]
hostname = "tuxedo"          # any RFC 1123 label; names the user flake output <user>@<hostname>

[user]
username = "kuba"           # any; not manifest.username means a build at install (below)
full_name = "Kuba Example"

[disk]
device = "/dev/nvme0n1"
encrypt = true
target = { mode = "whole" }  # or { mode = "free_space", start = 123, end = 456 }

[git]
name = "Kuba Example"
email = "..."

[wallpapers]                 # optional; without it, every set in the payload
sets = ["cyberpunk", "shinjuku"]   # whole sets: ids from manifest.wallpaper_sets
files = ["japanese/tokyo-tower-blue-hour.jpg"]   # single wallpapers, <set>/<file>; both empty = only your own

[payload]
dir = "/run/kubaos/payload"
```

Secrets (`Secrets`), never logged: `luks_passphrase` (required when `encrypt`), `user_password`, `wifi_psk`. The LUKS passphrase never reaches the target. The user password reaches it only as its hash in `/etc/shadow`. The Wi-Fi network from `[wifi]` is written to the target as `/etc/NetworkManager/system-connections/<ssid>.nmconnection`, root:root 0600, with the psk, the way NetworkManager stores it itself, so the installed system comes up on Wi-Fi. Pass them as a TOML file with mode 0600 (`--secrets`) or as one JSON object on stdin (`--secrets-stdin`). Kōbō passes them to the engine in-process and writes neither.

## Event order

`plan`, then for each step `step_started` → (`progress` | `log`)* → `step_finished`, then `done`. On error, `failed` is the last event. Step ids and titles come from `StepId` (`step.rs`): `preflight`, `partition`, `encrypt` (only if encrypting), `filesystems`, `unpack`, `configure`, `user`, `bootloader`, `home`, `snapshot`, `finish`. `unpack` should send `progress` with a fraction at least twice a second.

## Target layout (engine)

Whole-disk mode, GPT:

| Partition | Size | Content |
|---|---|---|
| ESP | 1 GiB | vfat, `/boot/efi` |
| boot | 2 GiB | ext4, `/boot` (GRUB reads kernels here, unencrypted) |
| root | rest | LUKS2 (argon2id) → btrfs |

Btrfs subvolumes, flat, exactly as in `system/00-btrfs-subvolumes.md`: `@ @home @snapshots @log @cache @nix @docker @swap`, mount options `noatime,compress=zstd:1`. GRUB, with snapper and grub-btrfs as `system/` sets them up. The factory snapshot is snapper snapshot #1 of `@`, described as `factory`. In free-space mode, the existing ESP is reused when present, and the other partitions are created inside the region.

How the engine does it:

- Partitions with `sgdisk`, 1 MiB aligned. Type codes `ef00` (ESP, label `EFI`), `8300` (`boot`), `8309` (`root`, LUKS). Whole-disk mode wipes the disk first. Free-space mode only accepts GPT disks and a region inside one of `probe`'s `free_regions` (byte offsets, end exclusive); new partitions take the lowest free numbers.
- LUKS2, `--pbkdf argon2id`, passphrase on stdin (never argv), opened as `cryptroot`. `/etc/crypttab` uses the same name: `cryptroot UUID=<luks> none luks,discard,tries=0`. `tries=0` keeps asking until the passphrase is right; with the default of 3 the initramfs gives up on the root device and drops to a busybox shell.
- `@` is the default subvolume. fstab names everything by UUID and subvolume name. `/boot/efi` is mounted with `umask=0077`. The swap file (`/swap/swapfile`, 8 GiB) is only created when the root partition is 64 GiB or more.
- The disk needs the ESP (unless reused) + `/boot` + `installed_bytes` + a fifth of it (at least 4 GiB) as headroom.
- Configure writes `/etc/{fstab,crypttab,hostname,hosts,timezone,localtime,locale.conf,default/keyboard}` (`/etc/default/locale` is a link to `../locale.conf`, as systemd's tmpfiles on Ubuntu 26.04 makes it on every boot), runs `locale-gen <lang>` and `ssh-keygen -A` (when sshd is there) in the chroot, and writes a new random `/etc/machine-id`. An empty one would make systemd's first boot run `preset-all`, which can undo unit choices the image made. Free-space mode adds `/etc/default/grub.d/90-kubaos-os-prober.cfg` (`GRUB_DISABLE_OS_PROBER=false`).
- The chroot gets `/dev`, `/dev/pts`, `/proc`, `/sys` (and `efivarfs` when the live system has it), and a fresh tmpfs on `/run`, so nothing inside can talk to the live system's systemd or D-Bus.
- `useradd` creates the user with uid 1000, a group of the same name, shell `/usr/bin/zsh`, and whichever of `adm sudo users plugdev dip video render i2c docker` exist in the payload. Password through `chpasswd` on stdin. `[git]` becomes `~/.config/git-identity/identity.gitconfig`, the file the Git card writes (the directories are chowned to the user, because home-manager writes in `~/.config` next). `[wallpapers]` (or every set of `manifest.wallpaper_sets`) becomes `~/.config/kubaos/wallpapers/sets`, the whole sets' ids then the single wallpapers as `<set>/<file>`, one per line, which `dot-theme` and the wallpaper picker read; a payload without sets writes no file. The user flake goes to `~/.config/kubaos/home/` (below), owned by the user. There's no copy of the repo in the home: scripts, wallpapers and the defaults come from the store, and KubaOS stays in `/usr/share/kubaos`. The system files the image's scripts rendered for the build user (the tty1 autologin drop-in, the `dot-chrome-theme` units: `engine/files.rs` `USER_TEMPLATES`) are rendered again from their `system/files/*.in` templates for the real user. `install/image/check` fails on any other file in the payload's `/etc`, `/usr/local`, `/usr/lib/systemd` or `/var/lib` that names the build user.
- The bootloader step runs `update-initramfs` for each kernel in `/boot`, so the initramfs knows the new crypttab. Then `grub-install --target=x86_64-efi|arm64-efi --bootloader-id=ubuntu`, then `--removable` for firmware that forgets boot entries, then `update-grub`. Without EFI variables (VMs, containers) it passes `--no-nvram`. In free-space mode another system's `EFI/BOOT` is set aside and put back, because Ubuntu's `grub-install` writes the fallback path too when it can't write NVRAM.
- home-manager's `activate` needs the Nix store, which belongs to `nix-daemon` in a multi-user install. Nothing runs in a chroot, so the engine starts `/nix/var/nix/profiles/default/bin/nix-daemon --option sandbox false` inside it (the sandbox can't `pivot_root` in a chroot; the profile path is resolved inside the target, because its links point into `/nix/store`), waits for its socket, runs `setpriv --reuid=<uid> --regid=<gid> --init-groups env -i HOME=… USER=… PATH=<nix>:… NIX_REMOTE=daemon HOME_MANAGER_BACKUP_EXT=hm-backup sh -c 'cd ~ && exec dbus-run-session -- <home_generation>/activate'` (`dbus-run-session` when the payload has it: home-manager's dconf step needs a session bus), then stops the daemon. For a username that isn't `manifest.username`, see "Another username" below: the generation activated is the one built there. `hm-backup` keeps `/etc/skel` files out of home-manager's way. Without systemd running, home-manager skips reloading user units. Without a `nix-daemon` in the payload, activation runs against the store directly (single-user) with a warning.
- The snapshot step runs `snapper --no-dbus create-config /` with the same `@snapshots` swap as `system/20-base.sh`, applies the `20-base.sh` retention settings, then runs `snapper create --description factory` (#1). `SYNC_ACL=yes` is set last and dropped with a warning when the btrfs has no ACLs. Without snapper in the payload it takes a read-only `btrfs subvolume snapshot` of `@` to `@snapshots/1/snapshot` and writes snapper's `info.xml`. When grub-btrfs is installed, `update-grub` runs again so the snapshot is in the menu from the first boot.
- `finish` copies the live log to `/var/log/kubaos/install.log` (0600), unmounts in reverse and closes LUKS. Any failure does the same cleanup before `failed`.
- `hostname` is any RFC 1123 label (a-z, 0-9 and `-`, 1-63, no `-` at either end). Nothing in `home/` reads it, so the payload's generation activates for any hostname; the user flake names its output `<user>@<hostname>`, which plain `home-manager switch` picks. The username is anything `useradd` takes, minus the accounts the payload already has (`root`, the services', `nixbld*`).

### The user flake

Every install gets `~/.config/kubaos/home/`, the flake the user switches from then on (docs/CONTRACTS.md, "Repo location and ownership"; `docs/adr/0001-release-and-update-model.md`). The user step writes:

- `flake.nix` from `/usr/share/kubaos/user-flake/flake.nix.in`, with `@USER@`, `@HOST@` and `@SYSTEM@` (`<manifest.arch>-linux`) filled in. Its `nixpkgs` input is the user's (`nixos-26.05`); `kubaos` is `git+file:///usr/share/kubaos?ref=refs/heads/release`, with no `follows`.
- `home.nix` from `user-flake/home.nix`: where the user adds their own packages.
- `flake.lock` from `/usr/share/kubaos-image/user-flake.lock`, generated at payload build: `kubaos` at the payload's commit on `release`, and the user's `nixpkgs` at KubaOS's own revision with `original` still `nixos-26.05`. So day one has one nixpkgs, and the two only part when the user runs `nix flake update` on their flake.

The username is a literal in the user's own `flake.nix`, so nothing is committed anywhere, and `/usr/share/kubaos` stays a clean checkout that an update can move. The flake is the user's from then on. `/usr/share/kubaos` is root-owned and listed in `safe.directory` in `/etc/gitconfig`, because Nix's git fetcher refuses a repository another user owns otherwise.

### Another username

The payload's generation is built for `manifest.username` (`kuba`). home-manager bakes the user into it: the activation checks `$USER`, and the links, units and session variables all name `/home/<user>`. So renaming a template user at install can't work, and the engine builds the generation for the name it was given instead. The home step, as the user in the chroot, with the throwaway nix-daemon already up:

1. in `~/.config/kubaos/home` (the user flake, with their name in it), runs `nix --offline build -L --out-link /tmp/kubaos-home-generation '.#homeConfigurations."<user>@<host>".activationPackage'`
2. activates that, not `manifest.home_generation`

Only about 50 small derivations name the user: the `hm_*` links and files, `home-manager-files`, `home-manager-path`, the session vars, the activation script and the generation. They build in seconds. Everything else is the payload's generation, already in the store. The builders they need (stdenv's setup scripts, perl for `buildEnv`, a few tools, about 18 MB on aarch64) are in the payload because `assemble.sh` builds a probe user's generation and keeps the inputs of whatever differs (install/image/README.md). `install/image/check` builds one for `kubaos-check` offline to prove it. The payload's own username skips all of this.

What the engine needs on the live ISO: `sgdisk` (gdisk), `sfdisk` (fdisk), util-linux (`wipefs`, `blkid`, `lsblk`, `findmnt`, `losetup`, `mount`), `partprobe` (parted), `cryptsetup`, `mkfs.vfat` (dosfstools), `mkfs.ext4` (e2fsprogs), btrfs-progs 6.1 or newer (`mkswapfile`), coreutils `chroot`, and `unsquashfs` (squashfs-tools, `-percentage`) or `tar` with `zstd`. `preflight` lists any that are missing.

What it needs in the payload: `grub-efi-<arch>-bin` and `grub2-common` (plus `shim-signed` and the signed GRUB for Secure Boot), `cryptsetup-initramfs` and `btrfs-progs` in the initramfs, `snapper`, `zsh`, `locales`, util-linux (`setpriv`), `passwd`, multi-user Nix with its `nix-daemon`, KubaOS's checkout at `/usr/share/kubaos` with `user-flake/`, `/usr/share/kubaos-image/user-flake.lock`, and no user with uid 1000. File capabilities survive the unpack: `unsquashfs` keeps xattrs by default, and `tar` runs with `--xattrs --acls`.

`install/test/engine-e2e` runs real installs (squashfs, tar.zst, a failing one, and a free-space one next to a fake Windows) onto a loop device in a privileged Docker container and checks the disk afterwards. `KUBAOS_TEST=1` is for that test only: probe then offers loop devices and assumes UEFI.

## Payload (image)

`<payload dir>/manifest.json` plus `<payload dir>/<manifest.rootfs>`, a compressed archive of the root tree that the engine can unpack with progress. Format: squashfs (zstd) or a zstd-compressed tar. The image agent chooses and documents it here. The tree is a complete system without machine identity: empty `/etc/machine-id`, no SSH host keys, no user. The user's home-manager generation is in `/nix/store` and named by `manifest.home_generation`. KubaOS's checkout is at `/usr/share/kubaos` in the tree, on its `release` branch, and the user flake's lock at `/usr/share/kubaos-image/user-flake.lock`; the engine writes the user flake from them ("The user flake" above). The engine creates the user and runs that generation's `activate` as the user in a chroot. The activation creates the user layer, `~/.config/kubaos` (`dot-config apply`, `docs/CONTRACTS.md`).

`installed_bytes` is used for the disk-size check and for unpack progress.

Built by `install/image/build [--arch aarch64|x86_64]` into `install/out/payload-<arch>/` and checked by `install/image/check` (`install/image/README.md` has the details):

- **Format: squashfs, zstd** (level 15, 1 MiB blocks), file `kubaos-shinjuku-<arch>.squashfs`; `manifest.version` is the KubaOS release (`0.1.0`). `unsquashfs` decompresses on every core (about 540 MiB/s of files on the Mac's Docker VM), restores xattrs as root without extra flags (the `cap_sys_admin` file cap on `gsr-kms-server` survives), keeps hardlinks, and prints `-percentage`. `installed_bytes` is the apparent size of the unpacked tree.
- **The tree** is Ubuntu 26.04 after the real `system/10-strip.sh`, `20-base.sh`, `25-boot.sh`, `40-apps.sh`, `50-hyprland.sh`, `60-nix.sh`, `65-recorder.sh`, `70-bitwarden.sh` and `75-yubikey.sh` (in image mode, `DOTFILES_IMAGE_BUILD=1`), plus `linux-generic`, GRUB EFI (plain and signed), shim-signed, cryptsetup-initramfs and the rest of what the engine needs. Not in it: `system/30-hardware.sh` (TUXEDO repo and DKMS drivers, MOK, fingerprint PAM: first-boot setup).
- **`/nix`** holds only the closure of the generation, the store paths `system/` links to (PAM modules, gpu-screen-recorder, the GPU driver bundle), the flake's inputs and Nix itself, each with a GC root. The multi-user Nix has its `nix-daemon` socket and service enabled.
- **Identity:** no user (Docker's `ubuntu` and the build user are removed, uid and gid 1000 are free), empty `/etc/machine-id`, no SSH host keys (`openssh-server` installed but disabled), `/etc/hostname` `localhost`, no fstab or crypttab, no apt lists. `/etc/resolv.conf` links to resolved's stub.
- **KubaOS:** `/usr/lib/os-release` per `docs/BRAND.md` (diverted from base-files). The boot (splash, GRUB theme and names) is `system/25-boot.sh`, described in "Boot" below. The EFI directory stays `EFI/ubuntu` for shim.
- **`/usr/share/kubaos/`:** KubaOS's checkout (above, "The user flake").
- **`/usr/share/kubaos-image/`:** `user-flake.lock`, `image.json` (user, groups, generation, kernel), `snapper-root.conf` (`20-base.sh`'s snapper settings as `KEY=VALUE` lines).
- **Wallpapers:** `/usr/share/kubaos/wallpapers` links to the flake input `kubaos-wallpapers` (every set, read-only, GC root `dotfiles-wallpapers`; `system/60-nix.sh`). `manifest.wallpaper_sets` lists them, `[{ "id", "images", "preview", "files": [{ "name", "thumb" }] }]` sorted by id and files by name. `thumb` is `wallpapers/<id>/<file>.jpg` in the payload dir, a JPEG at least 480 px wide (djpeg scales by n/8: 480 px from a 4K or 1080p original, 640 from 2560), of the image or of a video's `<name>.poster.jpg`; a PNG, WebP or a video without a poster has none. `preview` is the set's first thumbnail. They're for Kōbō's Wallpapers screen, since the squashfs isn't mounted before the install. Kōbō decodes them off the UI thread, and falls back to the original under `/usr/share/kubaos/wallpapers` where that exists. The screen comes after Git, every wallpaper picked: each set is a section with a grid of thumbnails; Space or a click toggles a wallpaper, and on a set's header the whole set; arrows move in the grid, Tab between sets. Continue waits for at least one pick. A set whose wallpapers are all picked goes in `sets`, the rest in `files`. `install/image/check` fails when the payload is over 6 GB, which with the live system (about 1.8 GB) keeps the ISO under 8 GB.
- **No git identity:** the manifest carries no git name or email, so the builder's never ships in the ISO. The user enters theirs on the Git screen, or later on first-boot setup's Git card.

## Boot

From power on to the desktop, as `system/25-boot.sh` and `50-hyprland.sh` set it up. The payload runs both at build time. The engine's `update-initramfs` and `update-grub` in the bootloader step bring the splash and the menu onto the target. Screenshots of the release run: `docs/screenshots/release/`.

**GRUB.** The entries read "KubaOS Shinjuku", "Advanced options for KubaOS Shinjuku" and "KubaOS snapshots" (grub-btrfs). Ubuntu's `10_linux` appends "GNU/Linux" to every distributor except Ubuntu. So `25-boot.sh` diverts it (`dpkg-divert --local --rename` to `/usr/lib/kubaos/grub/10_linux.ubuntu`, outside `/etc/grub.d`, so grub-mkconfig doesn't run it twice, and outside the checkout in `/usr/share/kubaos`, which the payload build and the updater replace whole) and puts a wrapper in its place. The wrapper runs Ubuntu's script and swaps "KubaOS GNU/Linux" for os-release's `PRETTY_NAME`. Upgrades of `grub2-common` update the diverted copy and leave the wrapper alone. If the copy is missing, the wrapper fails, and grub-mkconfig keeps the old `grub.cfg` rather than writing a menu with no entries. `/etc/default/grub.d/10-kubaos.cfg` holds the rest:

| Setting | Why |
|---|---|
| `GRUB_TIMEOUT_STYLE=hidden`, `GRUB_TIMEOUT=1` | No menu on a normal boot. GRUB waits one second, invisibly, and F4 (or Esc, or Shift held where the firmware reports it) brings the menu up. F4 is the one to use: in the aarch64 VM (GRUB 2.14) an Esc that reaches the menu leaves it for the `grub>` command line, and every Esc in the hidden second ended there. `normal` and Enter gets the menu back. F4 opened the menu on every try, on aarch64 and under Secure Boot on x86_64. Measured in the VM: with `GRUB_TIMEOUT=0` no key gets through at all, and Esc pressed any earlier opens the firmware's setup. One second is the price of being able to reach the snapshots on purpose. A menu with a visible countdown would cost more and look like a PC from 2009 |
| `GRUB_RECORDFAIL_TIMEOUT=10` | GRUB sets `recordfail` in `/boot/grub/grubenv` (ext4, so GRUB can write it) when it boots an entry, and `grub2-common.service` clears it once `boot-complete.target` is reached. A boot that didn't get there (a crash, a hang, power pulled at the passphrase prompt) shows the menu next time, which boots the default after 10 s |
| `GRUB_CMDLINE_LINUX_DEFAULT="quiet splash"` | The splash, with no kernel chatter behind it |
| `GRUB_THEME`, `GRUB_BACKGROUND` | `/boot/grub/themes/kubaos/`: on `/boot`, because GRUB can't read the encrypted root. The palette, the lit neon sign, Geist for the entries and JetBrains Mono for the editor and the GRUB shell. GRUB draws in device pixels, so the sizes (30 px entries) are chosen for the laptop's 2880x1800 and still look fine at 1080p. The background image makes the editor and the shell draw on the theme's dark box instead of Ubuntu's blue |
| `GRUB_GFXMODE=auto`, `GRUB_GFXPAYLOAD_LINUX=keep` | The panel's native mode, handed on to the kernel, so the splash starts at full resolution without a mode switch |

Under Secure Boot (verified on x86_64 with shim, Ubuntu's signed GRUB and the signed kernel), the signed GRUB has every module the theme uses built in (`gfxmenu`, `gfxterm`, `png`), so the theme, the sign and the boxes load. Fonts don't: a locked-down GRUB runs font files through the shim verifier, and our `.pf2` files aren't signed, so the menu falls back to the built-in `unicode` font (16 px). At 1080p that's fine. At the laptop's native 2880x1800 the menu text will be small but legible. The splash and the passphrase prompt are Plymouth's and are unaffected. The fix, if it's too small on the real panel: under lockdown (`$lockdown = y` in grub.cfg), ask for a lower `gfxmode` before the menu. The cost is a blurrier splash until amdgpu takes over.

grub-btrfs titles the snapshot entries by date and description (`GRUB_BTRFS_TITLE_FORMAT`), because the default with the subvolume path doesn't fit the menu. A snapshot is read-only, and booted as it is, it stops at a tty: sysusers and logind can't write, so there's no session. `/etc/initramfs-tools/scripts/local-bottom/kubaos-snapshot-overlay` does what grub-btrfs's overlayfs hook does for mkinitcpio and dracut. When `rootflags` name `@snapshots/`, it lays a tmpfs overlay over the snapshot: the desktop comes up, the snapshot stays untouched, and changes vanish at reboot (`snapper rollback` makes a snapshot permanent). `snapper-boot.service` skips itself on such a boot. grub-btrfs sources `/etc/default/grub` but not `grub.d/`, so the VM payload repeats its console arguments in `GRUB_BTRFS_SNAPSHOT_KERNEL_PARAMETERS`. Without them, a snapshot boot asks for the passphrase on serial only.

**Splash.** Plymouth, theme `kubaos` (script module, `/usr/share/plymouth/themes/kubaos`). It's both the `default.plymouth` alternative (Ubuntu's initramfs hook reads it) and `Theme=` in `/etc/plymouth/plymouthd.conf`. The neon sign switches on a group of tubes at a time (ク, バ, the dakuten, Kuba, OS), each stuttering before it catches, then hums for the rest of the boot with a blink on バ every four seconds. The groups are separate layers from `branding/neon.py` (2 MB for both scales, palette PNGs), faded by the script; once all are on, one merged layer draws instead. The passphrase prompt from `cryptroot` (`plymouth ask-for-password`) becomes a field with bullets under the label "Disk passphrase". It shows a Caps Lock warning, dims to "Unlocking..." while argon2id runs, and on cryptsetup's "bad password" message it shakes and says "Wrong passphrase. Try again." crypttab's `tries=0` keeps it asking. Other messages show small at the bottom.

- HiDPI: `DeviceScale=1` in plymouthd.conf, and the script picks `@2x` assets on screens 1600 px tall or more. Plymouth's own 2x would upscale the 1x PNGs.
- Text: `label-freetype` (`plymouth-label-ft`), not Ubuntu's pango plugin and its font packages. `/etc/initramfs-tools/hooks/kubaos-plymouth` copies the plugin and links `/usr/share/fonts/Plymouth.ttf` and `Plymouth-monospace.ttf` to the theme's Geist and JetBrains Mono. Without a monospace font, plymouthd 24.004 crashes in its console viewer as soon as the passphrase prompt arrives, and the prompt falls back to an invisible console.
- Text in the theme is ASCII only: in the initramfs there's no UTF-8 locale, and label-freetype decodes with `mbrtowc`.
- GPU: Ubuntu's `UseSimpledrm=1` stays. The splash and the prompt draw on the firmware framebuffer (simpledrm) from the first moment, and the native driver takes over when it loads. In the VM that's `virtio_gpu`, which the initramfs has (`FRAMEBUFFER=y` pulls in the minimal DRM set, and the VM payload lists it in `/etc/initramfs-tools/modules`). On the laptop (Ryzen AI 9 HX 370, Radeon 890M) `amdgpu` loads from the root filesystem after the unlock. Putting it in the initramfs would add 41 MB (6.5 MB of module, 35 MB of firmware for every AMD chip) and move the simpledrm-to-amdgpu switch to the middle of the passphrase prompt, so it stays out. Not verified on the hardware yet.
- Serial consoles: when plymouth sees one it forces plain text. The VM payload boots with `console=ttyAMA0 console=tty0`, so it also gets `plymouth.ignore-serial-consoles`. The prompt is on the screen and the keyboard, as before. Headless or without a framebuffer, plymouth falls back to text on tty0 and the passphrase still goes in from the keyboard. Esc during the splash shows the boot messages.

**Into the desktop.** `plymouth-quit` ends the splash when boot finishes. tty1 then shows nothing until Hyprland's first frame: agetty skips `/etc/issue` and its automatic-login line (`--skip-login --noissue --nohostname`), `~/.hushlogin` (from `/etc/skel`) drops the motd and "Last login", and `start-hyprland` writes its banner to `$XDG_RUNTIME_DIR/start-hyprland.log`. What's left is a moment of black. `plymouth quit --retain-splash` would keep the sign up instead, but it leaves the VT in graphics mode: a tty1 shell without Hyprland (`~/.config/hypr/no-autostart`) would then be invisible.

**Shutdown and reboot.** The power menu's `dot-session leave` stops the graphical session first (docs/CONTRACTS.md, "Leaving the session"), so no session service is left to time out once Hyprland is gone. `plymouth-poweroff.service` and `plymouth-reboot.service` (Plymouth's own, wanted by `poweroff.target` and `reboot.target`, `ConditionKernelCommandLine=splash`) start plymouthd in `shutdown` or `reboot` mode. Their own ordering waits for getty@tty1 and a display manager; KubaOS has none, Hyprland runs in the login session on tty1. plymouthd asks for DRM master once, when it maps the screen (and again only on a VT switch), so started while Hyprland still holds master it draws into a buffer that never reaches the screen. A drop-in, `After=user.slice` (`system/files/systemd/plymouth-shutdown.d/kubaos-after-session.conf`, in all four of `plymouth-{poweroff,reboot,halt,kexec}.service.d`), makes plymouthd start once every session has ended: systemd runs a stop before a start ordered against it. The theme reads `Plymouth.GetMode()` and plays the sign backwards: lit, then OS, Kuba, the dakuten, バ and ク sputter out, and the glass and background fade to black, about 1.7 s at 30 fps. `kubaos-shutdown-splash.service` (enabled in the four targets) starts once the splash shows and sleeps 2 s, ordered before `systemd-poweroff.service` and its siblings, so the machine goes off once the screen is black or the rest of the shutdown is done, whichever is later. Plymouth has no "animation done" call, hence the fixed bound. With no plymouthd running (`plymouth --ping` fails: no `splash`, `plymouth.enable=0`) it is skipped and nothing waits. `install_shutdown_splash` in `system/lib.sh` (run by `25-boot.sh` and the `shutdown-splash` migration). Between Hyprland's exit and the splash, tty1's text console shows for a moment. Stop jobs that run longer than 10 s are killed (`DefaultTimeoutStopSec`), so the text console with "A stop job is running" doesn't get a chance to show.

**Boot time.** `systemd-analyze` puts the disk passphrase prompt in the kernel stage: the initramfs (cryptroot) waits there. GRUB waits one hidden second for Esc (`GRUB_TIMEOUT=1`, above), the rest of the loader stage is reading the kernel and initramfs. In userspace nothing waits for the network (CONTRACTS, "Boot").

**Locale.** The session reads the locale the engine writes: `/etc/locale.conf`, with `/etc/default/locale` a link to it. `login` loads it through `pam_env` on the tty1 autologin, and Hyprland and everything it starts inherit `LANG`. The "Locale: C" in the first VM screenshots came from an install made before the engine wrote `locale.conf` (8169a9c).

## Live ISO (iso)

`install/iso/build` (details in `install/iso/README.md`). On the live system:

- `kubaos` and `kobo` are in `/usr/bin`. The payload is mounted read-only at `/run/kubaos/payload` (on the ISO it's `/kubaos/payload`). There are no apt sources and no network wait.
- tty1 autologs in as root. When a filesystem labelled `KUBAOS_CFG` holds `install.toml`, the install runs headless: `kubaos run --config <disk>/install.toml --secrets /run/kubaos/secrets.toml --events human [-v]`, with output on the serial console framed by `KUBAOS-BEGIN` and `KUBAOS-RESULT: done|failed rc=N`, then poweroff. Otherwise it runs `cage -s -- kobo` as root (not `--fullscreen`, which trips an assertion in cage 0.2.1) inside a logind session, and falls back to `kobo-text` when Kōbō exits non-zero. tty2 always has `kobo-text`.
- Kōbō gets a logind seat (`XDG_RUNTIME_DIR=/run/user/0`) and Mesa (EGL/GLES and Vulkan, with llvmpipe/lavapipe in VMs), plus `fonts-inter` and `fonts-dejavu-core`. Exit 0 means the user finished (installed or quit on purpose). Any other exit means "use the text mode".

## First boot (firstboot)

What the payload can't carry, because it needs the hardware or the person, happens at the first login: a checklist of five cards in the install screens' look. `kobo --setup` is a separate iced application in the same crate. It shares `theme.rs`, `widgets.rs`, the key translation and `app::Message` (one variant, `Message::Setup`). The logic is in `kubaos_core::setup`, and `kubaos setup` runs the same code headless with JSON events.

| Card | Shown when | Done when | What it does |
|---|---|---|---|
| TUXEDO drivers | `/sys/class/dmi/id/{sys,board}_vendor` says TUXEDO and the arch is amd64. Otherwise "Not here" | `tuxedo-drivers` installed and its modules loaded. With Secure Boot on, also the DKMS key enrolled. A queued key shows as "Reboot": the app runs as the user and mokutil writes its request (`MokNew`) root-only, so as the user `mokutil --test-key` says "is not enrolled" for a queued key; the request's presence in efivarfs counts as queued | `system/30-hardware.sh --only tuxedo,mok`: TUXEDO repo, headers, DKMS drivers, no Control Center. With Secure Boot on (efivar `SecureBoot-…`) it asks for a one-time MokManager password (8 to 16 letters and digits, because the blue screen assumes a US keyboard), queues the key with `mokutil --import` (password on stdin, twice, as mokutil asks) and explains the blue screen. A run after which the key still isn't queued fails with that reason, never "Done" |
| Fingerprint | fprintd sees a reader (`fprintd-list`), or before fprintd is installed, a USB device from a fingerprint vendor | a finger enrolled and `pam_fprintd` in `common-auth` | `30-hardware.sh --only fingerprint` (fprintd, `pam-auth-update --enable fprintd`), then `fprintd-enroll` as the user, showing each touch with a count (`num-enroll-stages` from D-Bus) |
| YubiKey | always. "Waiting" until a key (USB vendor 1050) is plugged in | the user has a line in `/etc/u2f_mappings`, and the plugged-in key is in it (`dot-u2f-enroll --info`) | `pamu2fcfg -n -u USER -V` (finger on the key's sensor), then the helper appends the credential. A key without a PIN or fingerprints gets a terminal with the `ykman` commands, because those talk to the key interactively |
| Bitwarden | `/usr/local/bin/bw` exists | `bw status` is locked or unlocked, and a pull has written `~/.config/dot-bw/config` and `~/.ssh/config.d/bitwarden` | logs in (`bw login EMAIL --passwordenv BW_PASSWORD --method 0\|3 --code C`, or `--apikey` with `BW_CLIENTID`/`BW_CLIENTSECRET`), or only unlocks, then pulls the vault (SSH, git signing, gh, env) through `kubaos_core::bitwarden`, the code `kubaos bitwarden setup` runs, its lines in the card's log. Once pulled, the card ticks the vault's folders for the SSH keys, shows each key's Host block, hosts and signing, and what the agent serves, and logs out for another account or server. The Command menu's "Set up Bitwarden" opens setup on this card (`kubaos-setup --card bitwarden`) |
| Git | always | `git config --global --includes` answers both `user.name` and `user.email`, wherever they're set | writes `~/.config/git-identity/identity.gitconfig` (`[user]`, values quoted), which `config/git/config` includes. Not in `~/.config/git`: that holds KubaOS's read-only defaults. The form starts from git's current values, or the account's full name from `/etc/passwd`. Works offline |

Every card is detected fresh on each start, so finishing one by hand (the scripts) counts too, and re-running never redoes a finished card. Cards needing the internet (`nmcli` connectivity, or a TCP connect to 1.1.1.1/9.9.9.9 without NetworkManager) show "Waiting" while offline, unless the part that's left works offline.

**Root.** The app runs as the user. The privileged steps go through `pkexec /usr/lib/kubaos/setup-helper VERB` with three verbs: `tuxedo [--mok-password-stdin]`, `fingerprint` and `u2f-add` (credential on stdin, validated). The session's polkit agent asks: `hyprpolkitagent` (`config/hypr/autostart.lua`), or Quickshell's when switched over. Action `org.kubaos.setup` is `auth_admin_keep`. Why pkexec and not `sudo -S` with a password field in the app: the password never passes through the app, the agent already handles pam_u2f and pam_fprintd prompts, and the helper is a fixed root-owned script with three verbs, not a general root shell. The helper runs `system/30-hardware.sh` from KubaOS's root-owned checkout `/usr/share/kubaos`, never from a copy that anything running as the user could edit. It refuses files that aren't root-owned or that others can write.

**Secrets.** The MOK password goes to the helper on stdin. The master password and API key go to `bw` in its environment, and the session stays in the process, handed only to the bw calls that need it, as docs/SECRETS.md does it. None of them is logged or written. The form clears them once a run succeeds. One exception to "never in argv": the two-step code (`bw login --code`), which bw can't take any other way without a terminal. It's spent the moment bw uses it.

**On the target.** The engine's configure step (`install_setup`) copies the live system's `kobo` to `/usr/bin` and its `kubaos` to `/usr/lib/kubaos/kubaos`, and writes, from `install/firstboot/`: `/usr/bin/kubaos` (the launcher: it runs `~/.nix-profile/bin/kubaos`, which every switch installs from the release, and falls back to `/usr/lib/kubaos/kubaos`), `/usr/lib/kubaos/setup-helper`, `/usr/share/polkit-1/actions/org.kubaos.setup.policy`, `/usr/bin/kubaos-setup`, `/usr/share/applications/kobo.desktop`, and the user unit `/usr/lib/systemd/user/kubaos-setup.service`, enabled globally (linked into `/etc/systemd/user/graphical-session.target.wants/`). The unit has `ConditionPathExists=!%S/kubaos/setup-done` and runs `kobo --setup --first-boot` once `autostart.lua` starts the graphical session. Without a `kobo` next to `kubaos`, setup isn't installed, with a warning.

**A calm first login.** Setup is the only window at the first login:

- `config/hypr/rules.lua` floats the app id `kobo` in the centre at 80 × 85 % of the monitor, instead of half a tiled screen. As the newest window, it has focus. The window is one solid panel, edge to edge: no photo behind it, because the wallpaper is right there around it and Hyprland draws the border and the rounding. Only the install screens, full screen under `cage`, have the Shinjuku photo.
- `config/hypr/autostart.lua` doesn't start Bitwarden desktop while setup is pending (its unit is installed and the marker is missing). Setup starts it with `systemd-run --user` when it closes (not `hyprctl dispatch exec`: the Lua config rejects that syntax and hyprctl still exits 0), and from then on the autostart runs it as before. Nothing else autostarts a window: Quickshell, hypridle and the theme helper are silent, and the daily update check (`kubaos-update-check.timer`) only writes a file and sends a notification.
- **The keyring.** The user step writes `~/.local/share/keyrings/Default_keyring.keyring` (gnome-keyring's plaintext format, so no password) and `default` (`Default_keyring`), unless the home already has keyrings, as Omarchy does. Otherwise the first app that wants a secret (Bitwarden desktop, Chrome, `gh`, `git credential-libsecret`, `secret-tool` through `kubaos bitwarden env`) pops "Choose password for new keyring". Nothing would ever unlock a password-protected one either: the login is tty autologin right after the LUKS passphrase, so PAM never sees a password to unlock it with.

  The trade-off: the keyring file is not encrypted on its own. It's as safe as the disk, and LUKS is the boundary. With the laptop powered off it's unreadable, the same as everything else in `/home`. Anything running as you can read it, but anything running as you can read an unlocked keyring too, and an unlocked keyring is what you'd have all session anyway. What it does give up is protection for a copy of the home made while the machine runs (a backup to an unencrypted disk, say).

  The alternative that keeps a password is `pam_gnome_keyring` unlocking a login keyring with the account password, which needs a PAM login that sees the password: a display manager, or `auth optional pam_gnome_keyring.so` in hyprlock's stack plus `gnome-keyring-daemon --unlock` at the first unlock. With autologin there's no such moment at login. So the first secret would still prompt, or it would have to wait for the first screen lock, and a keyring that unlocks on the first lock is a puzzle, not security. If KubaOS ever drops tty autologin for a greeter, switch to that and give the keyring the login password.

**The marker** `~/.local/state/kubaos/setup-done` is written by "Done for now"/"All set", by closing the window, and by `kubaos setup done`. The one exception is a card waiting on a reboot (a queued MOK): then setup opens once more after the reboot, to confirm the drivers loaded. `kubaos-setup` or the launcher entry opens it any time.

Screenshots: `docs/screenshots/setup/` (from `kobo --setup --demo [--demo-machine vm|installed] --screenshot-tour DIR`; `installed` is the Bitwarden card after first boot).

## Test bench (iso)

All the commands, and how config gets in, are in `test/vm/README.md`.

`test/vm/run` drives QEMU on the Mac. aarch64 runs with HVF, which is fast. x86_64 runs under TCG, which is slow and meant for occasional correctness checks. Each command uses an EFI firmware copy, a qcow2 target disk, a serial console log, SSH via a forwarded port, screenshots via QMP, and qcow2 snapshots. Minimum commands: boot the ISO, run a headless install with a given `install.toml`, boot the installed disk, run a command over SSH, take a screenshot, snapshot and restore. You can open the same ISO in UTM to click through Kōbō yourself.

## Scope v1

Personal. The flake host comes from the payload. The username is yours to pick, and anything but the payload's builds its generation at install. Supported: whole-disk and free-space installs, LUKS on, GRUB, aarch64 for the VM and x86_64 for the TUXEDO.
