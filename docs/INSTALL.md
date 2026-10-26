# Installing KubaOS

There are two ways onto the TUXEDO InfinityBook Pro 14 Gen10. The KubaOS stick takes a couple of minutes and a few keyboard-only screens. The hard way, at the end of this file, is a stock Ubuntu Server 26.04 install and nine scripts by hand, which takes an evening and is how KubaOS got built in the first place. Use the stick.

The why is in `docs/PLAN.md`, how the install works inside in `docs/INSTALL-INTERNALS.md`.

## The KubaOS stick

The stick holds a finished KubaOS: Ubuntu 26.04 after `system/10-strip.sh` through `75-yubikey.sh`, plus `/nix` with your home-manager generation already built. Kōbō, the app on the stick, partitions, encrypts, unpacks, creates your user and installs GRUB. No network needed, nothing downloaded, nothing compiled on the laptop.

### What you need

- A USB stick bigger than the ISO. Everything on it gets overwritten.
- Your SSH keys as Bitwarden "SSH key" items, in whatever folders you keep them (`docs/SECRETS.md`, "Vault conventions"). Nothing to copy by hand.
- An Ethernet cable or a Wi-Fi password. The install doesn't need a network, but the first boot setup does.

### Make the stick

Build the payload and the ISO on the Mac (both run in Docker; details in `install/image/README.md` and `install/iso/README.md`):

```sh
install/image/build --arch x86_64          # the installed system, as one squashfs
install/iso/build --arch x86_64 --payload <payload dir> --release
```

Then write the ISO to the stick with `dd`, balenaEtcher or Raspberry Pi Imager. Check the device name twice before you `dd`: it erases whatever disk you point it at.

The ISO is UEFI only and works with Secure Boot on (Canonical's signed shim, GRUB and kernel).

### BIOS

F2 at power-on opens the TUXEDO BIOS, F7 is the boot menu (choose the USB stick).

Secure Boot: on or off, as you like. The stick boots either way. With it on, `tuxedo-drivers` needs a Machine Owner Key enrolled once, after the install (below).

### The screens

Kōbō starts on its own a few seconds after GRUB. Everything works from the keyboard: Tab and Shift+Tab move, arrows pick in lists, Space toggles, Enter goes on, Esc goes back.

1. **Welcome.**
2. **Language and keyboard**, with a field to test the layout.
3. **Time zone.**
4. **Wi-Fi**, only when the live system has no network. The network you pick here is also saved on the installed system.
5. **Account.** Your full name, username and password. The username starts as the image's (`kuba`). Any other builds its home-manager config during the install, a few seconds more. The hostname starts as the image's (`tuxedo`) and can be any lowercase name of a-z, 0-9 and `-` (up to 63, no `-` at either end). It costs nothing: the home config doesn't depend on it, and the user flake's output is written as `<user>@<hostname>` so plain `home-manager switch` finds it.
6. **Disk.** Either the whole disk or the free space next to an existing system (Windows, say). KubaOS always encrypts with LUKS2: pick a passphrase you'll type at every boot, and don't lose it. There is no recovery.
7. **Git identity**, prefilled from the image, with a preview of the `~/.gitconfig` it writes.
8. **Review.** It names the disk that gets erased. Read it. Then hold the button (or Enter) to start.
9. **Installing.** Steps with timings, and a live log if you want it. Then **Done** and Restart.

If a step fails, Kōbō shows what broke, a hint, the log, Save log and Try again. The live log is `/var/log/kubaos-install.log`.

If Kōbō itself won't start, its text mode runs instead, and it's always on tty2 (`Ctrl+Alt+F2`). It asks the same questions.

### First boot

LUKS passphrase, then autologin on tty1 straight into Hyprland. The bar, Ghostty (`SUPER + Return`) and the Command menu (`SUPER + Space`) are there. snapper already has snapshot #1, `factory`, of the fresh system. The install log is in `/var/log/kubaos/install.log`.

### What the stick can't do

A few things need the real hardware or you. Do them in this order. Each script is safe to re-run.

```sh
cd ~/Projects/dotfiles

# 1. TUXEDO drivers, power profiles, dock, fingerprint reader. Then reboot.
system/30-hardware.sh
sudo reboot          # enroll the MOK on the blue screen if it asked for a password
system/30-hardware.sh   # second run: confirms modules + power profiles

# 2. Toolchains, outside Nix on purpose.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rust-analyzer clippy
bash <(curl -fsSL https://moonrepo.dev/install/proto.sh)
proto install moon
```

The MOK screen, the fingerprint enrollment and what to verify are in the hard way's sections 1, 7 and 8 below: same scripts, same steps.

Then the secrets. Bitwarden desktop, `bw` and the security-key PAM lines are already installed. Open Bitwarden (Hyprland starts it), log in, Settings -> enable "SSH agent", then run `kubaos bitwarden setup`. Set the YubiKey's PIN and fingerprints with `ykman` and run `dot-u2f-enroll` for each of your two keys. `docs/SECRETS.md` has the order, and section 11 below says how to check it all works.

Kōbō will do this part itself one day (the first-boot setup screen, `docs/screenshots/setup/`). Today it's a stub.

## Updating

```sh
kubaos update              # snapshot, apt, the release, migrations, your flake, switch
kubaos update --check      # only see what's waiting (the daily timer runs this)
kubaos update --packages-only
kubaos rollback            # back to where the last update started
kubaos channel             # edge or stable; `kubaos channel stable` to change it
kubaos upgrade             # to the next KubaOS series, on stable
```

The menu has the same: Update KubaOS, Roll back last update, Switch channel
and, when one is out, Upgrade to <codename>. The bar shows a marker while
an update waits. `docs/USING.md` ("Updating") says what each step does and
how to go back further.

Each update also reports newer Bitwarden desktop and CLI releases
(`system/70-bitwarden.sh --check-updates`) and prints the new version and
sha256 lines. Paste them into `70-bitwarden.sh` on the dev machine and
release them.

## Troubleshooting

**The install failed.** Read the error and the hint on screen, then the log
(Save log tells you where it put it; the live system also keeps it in
`/var/log/kubaos-install.log`). Try again runs the install again with the same
answers. The text mode on tty2 (`Ctrl+Alt+F2`) runs the same engine.

**Black screen / Hyprland won't start.** `Ctrl+Alt+F2`, log in. Hyprland's
log: `ls $XDG_RUNTIME_DIR/hypr/*/hyprland.log` (from another tty:
`/run/user/1000/hypr/`). To stop the autostart loop:
`touch ~/.config/hypr/no-autostart`, then `sudo systemctl restart getty@tty1`.
Remove the file to get autostart back. To run it by hand from tty1:
`start-hyprland`.

**Shell (Quickshell) dead, Hyprland fine.** The fallback binds never depend
on Nix or Quickshell: `SUPER+ALT+Return` foot, `SUPER+ALT+Space` fuzzel
(`SUPER+Return` is Ghostty, also Nix). Fix the config there; `qs` restarts it.

**Nix apps don't render (Ghostty, Quickshell crash with EGL/GL errors).**
`ls -l /run/opengl-driver` must point into `/nix/store`. If home-manager
warned "GPU drivers require an update", run
`system/60-nix.sh --no-switch --force-gpu` (or `sudo "$(readlink -f
$(command -v non-nixos-gpu-setup))"`).

**Lock screen won't take the password, finger or key.** Check
`ls -l /run/wrappers/bin/unix_chkpwd` points at `/usr/sbin/unix_chkpwd`
(if not: `sudo systemd-tmpfiles --create /etc/tmpfiles.d/qs-lock-unix-chkpwd.conf`)
and `readlink /usr/local/lib/security/pam_fprintd.so` names a `/nix/store`
path that exists (if not: `system/60-nix.sh --no-switch --no-gpu --no-chsh`).
Test without locking: `nix run nixpkgs#pamtester -- qs-lock $USER authenticate`
(and `qs-lock-u2f`, `qs-lock-fingerprint`). For the key, also
`readlink /usr/local/lib/security/pam_u2f.so` (if missing:
`system/70-bitwarden.sh --no-desktop --no-cli`). Don't switch the PAM files to Ubuntu's modules
by absolute path; Nix's libpam can't load them. If it still fails, hyprlock
stays the lock screen (hypridle's `lock_cmd` already falls back to it) and
`hyprpolkitagent` stays the polkit agent (the in-shell agent is off by
default).

**SSH, git signing or `.envrc` secrets don't work.** `kubaos bitwarden check`,
then `docs/SECRETS.md`, Troubleshooting.

**home-manager: "path ... does not exist" or a new config dir is ignored.**
Flakes only see files git knows about. `git add` new `config/<name>/` dirs and
`scripts/` files before `home-manager switch` (no commit needed).

**Broken user environment after a switch.** `home-manager generations`, pick
the previous one and run its activate script:
`/nix/store/<hash>-home-manager-generation/activate`.

**Broken system after an update.** Boot a snapshot: press F4 in the first
second after the firmware to get GRUB's menu, choose "KubaOS snapshots" -> the
"before update" one, then the kernel line under it. It boots to the full
desktop on a throwaway overlay, so nothing you do there sticks. Then make it permanent by
replacing `@` with it: `system/00-btrfs-subvolumes.md`, "Rolling back the
system later". (`snapper rollback` doesn't apply to this layout: Ubuntu's
GRUB hardcodes `subvol=@`.) Snapper also takes `apt` pre/post snapshots on
every apt run (`/etc/apt/apt.conf.d/80snapper`), so `snapper -c root list`
usually has a point right before the breaking change.
`sudo snapper -c root status <pre>..<post>` shows what changed.

**Modules missing after Secure Boot change or kernel update.**
`dkms status`; `sudo mokutil --test-key /var/lib/shim-signed/mok/MOK.der`
(with `sudo`: mokutil keeps a queued request root-only, so without it a queued
key reads as "is not enrolled"); `sudo mokutil --list-new` lists what's
queued. Re-run `system/30-hardware.sh` (it re-imports the key if needed) and
reboot.

**No network after 20-base.** `ls /etc/netplan/` (only
`01-network-manager-all.yaml` and NetworkManager's own `90-NM-*.yaml`
connections should end in `.yaml`; the installer's file was renamed
`*.disabled-by-dotfiles`), `sudo netplan apply`, `nmcli device`.

---

## The hard way: stock Ubuntu Server, by hand

How KubaOS was put together before the stick existed, and still the way to
rebuild it piece by piece or to find out which script broke something.
TUXEDO InfinityBook Pro 14 Gen10, Ubuntu Server 26.04 LTS ("resolute")
minimal, LUKS + LVM + Btrfs, Hyprland from `ppa:cppiber/hyprland`, user
environment from home-manager.

Scripts live in `system/`, run in order, as your normal user (they call
`sudo` themselves). Each is safe to re-run, checks before it changes
anything, and never reboots. `--dry-run` shows what it would do; `--help`
lists its flags. Their apt packages come from one list, `system/packages.toml`,
which `kubaos update` reads too. A flag that leaves a group out
(`40-apps.sh --no-docker`, `--no-chrome`, `70-bitwarden.sh --no-u2f`) or
adds one (`75-yubikey.sh --piv`) is kept in `/etc/kubaos/apt-groups`, so
updates follow it; `kubaos packages enable|disable <group>` changes it later.

| Step | What | Reboot after? |
|---|---|---|
| `system/00-btrfs-subvolumes.md` | manual, from the USB: `@`, `@home`, ... | yes (part of it) |
| `system/10-strip.sh` | purge snapd, cloud-init, multipath, lxd-installer, ModemManager; Pro adverts off | no |
| `system/20-base.sh` | base packages, NetworkManager, PipeWire, lid policy, snapper, grub-btrfs | recommended |
| `system/25-boot.sh` | boot splash (Plymouth, the passphrase prompt), GRUB theme, "KubaOS Shinjuku" entries, hidden menu | no (next boot shows it) |
| `system/30-hardware.sh` | TUXEDO repo + `tuxedo-drivers`, MOK, power profiles, dock, fingerprint | **yes** |
| `system/40-apps.sh` | Docker, Chrome | log out/in (docker group) |
| `system/50-hyprland.sh` | Hyprland stack (held), tty1 autologin, lock-screen PAM, keyd for the Mac-style accents | no (wait for 60) |
| `system/60-nix.sh` | Nix, flakes, home-manager switch, Nix GPU setup, zsh login shell | **yes** |
| `system/65-recorder.sh` | gpu-screen-recorder's KMS helper: root-owned copy with `cap_sys_admin`, runnable only by you, linked into `/run/wrappers/bin` | no |
| `system/70-bitwarden.sh` | Bitwarden desktop + `bw` CLI, security key (pam_u2f) for sudo, polkit, lock screen | no |
| `system/75-yubikey.sh` | YubiKey C Bio tools (`fido2-tools`), udev check, `--piv`: pcscd for the Multi-protocol Edition | no |
| `system/relink.sh` | points the wallpaper, PAM and recorder links at the installed release; `kubaos update` runs it | |

### Before you start (on the Mac)

- Ubuntu Server 26.04.1 amd64 ISO from <https://releases.ubuntu.com/26.04/>
  (`ubuntu-26.04.1-live-server-amd64.iso`), written to a USB stick
  (`dd`, balenaEtcher, or Raspberry Pi Imager).
- Your SSH keys as Bitwarden "SSH key" items, in whatever folders
  you keep them (`docs/SECRETS.md`, "Vault conventions"). Nothing to
  copy by hand.
- An Ethernet cable (Wi-Fi gets configured after NetworkManager is in).

### 1. BIOS and Secure Boot

F2 at power-on opens the TUXEDO BIOS.

- Secure Boot: on or off, as you like. **On** works: `tuxedo-drivers` is a
  DKMS module, so `30-hardware.sh` has you enroll a Machine Owner Key (MOK)
  once. On the next reboot a blue "Perform MOK management" screen appears:
  `Enroll MOK` -> `Continue` -> `Yes` -> type the one-time password you chose
  -> `Reboot`. You get about 10 seconds before it boots on without
  enrolling; if you miss it, re-run `30-hardware.sh` (it imports again) and
  reboot.
- F7 at power-on is the boot menu (choose the USB).

### 2. Install Ubuntu Server

1. Boot the USB with Ethernet plugged in. Choose **Ubuntu Server (minimized)**.
2. Network: accept the DHCP Ethernet config.
3. Storage: **Custom storage layout** on the NVMe:
   - 1 GB partition, FAT32, mounted at `/boot/efi`
   - 2 GB partition, ext4, mounted at `/boot`
   - the rest: create an LVM volume group **with "Create encrypted volume"
     ticked** (this is the LUKS passphrase you type at every boot), then one
     logical volume using all of it, formatted **btrfs**, mounted at `/`.
4. Profile: your name, hostname **`tuxedo`** (the flake's output is
   `kuba@tuxedo`, and plain `home-manager switch --flake ~/Projects/dotfiles`
   picks `<user>@<hostname>`), username `kuba`.
5. Ubuntu Pro: skip. SSH: tick OpenSSH only if you want to work over SSH.
   Featured snaps: none.
6. Reboot, remove the stick, type the LUKS passphrase, log in on the console.

Wrong hostname? `sudo hostnamectl set-hostname tuxedo`.

### 3. Btrfs subvolumes (from the USB again)

Follow `system/00-btrfs-subvolumes.md` exactly, with the device names
`lsblk` shows. It creates `@ @home @snapshots @log @cache @nix @docker @swap`,
rewrites fstab, and rebuilds GRUB from a chroot. Nothing is deleted until the
new layout has booted once.

**Verify** after the reboot: `cat /proc/cmdline` has `rootflags=subvol=@`,
`findmnt -t btrfs` lists all eight mounts, `swapon --show` shows
`/swap/swapfile`.

### 4. Get the repo

```sh
sudo apt update && sudo apt install -y git
mkdir -p ~/Projects
git clone <your dotfiles remote> ~/Projects/dotfiles
cd ~/Projects/dotfiles
```

### 5. `system/10-strip.sh`

```sh
system/10-strip.sh --dry-run
system/10-strip.sh
```

Purging snapd, cloud-init and multipath-tools also removes the
`ubuntu-server` / `ubuntu-server-minimal` metapackages (they depend on them).
The script first marks everything else those pulled in (sudo, cryptsetup,
lvm2, ...) as manually installed, so `--auto-remove` doesn't take it along.

**Verify:** `snap` is gone (`command -v snap` prints nothing),
`apt-cache policy snapd` shows `Candidate: (none)`, `pro config show apt_news`
is `False`, `systemctl status unattended-upgrades` is active.

### 6. `system/20-base.sh`

```sh
system/20-base.sh
```

Over SSH it writes the netplan file but doesn't apply it (you'd lose the
connection); reboot instead. On the console it applies it right away.

**Verify:**

- `nmcli general status` says connected; `networkctl` says networkd isn't
  running. Wi-Fi: `nmtui` -> Activate a connection, then unplug the cable.
- `wpctl status` lists PipeWire sinks (after you log in again).
- `sudo snapper -c root list` shows snapshot "clean base";
  `findmnt /.snapshots` shows `subvol=/@snapshots`.
- `ls /etc/grub.d/41_snapshots-btrfs` exists, `systemctl status grub-btrfsd`
  is active, and after the reboot GRUB has a "KubaOS snapshots" submenu
  (press F4 once the firmware logo is gone to see GRUB).

grub-btrfs isn't packaged for resolute, so the script installs the pinned
upstream release (v4.14) with `make install`. It isn't tracked by dpkg.

### 6a. `system/25-boot.sh`

```sh
system/25-boot.sh
```

Installs Plymouth with the KubaOS theme, the GRUB theme on `/boot`, and
renames the menu entries; then rebuilds the initramfs and `grub.cfg`.
What each setting does is in `docs/INSTALL-INTERNALS.md`, "Boot".

**Verify:** `update-alternatives --query default.plymouth` points at
`kubaos`; `grep -E "^(menuentry|submenu) " /boot/grub/grub.cfg` lists
"KubaOS Shinjuku"; on the next boot the passphrase prompt is the themed
field, not console text.

### 7. `system/30-hardware.sh`, then reboot

```sh
system/30-hardware.sh
sudo reboot        # enroll the MOK on the blue screen if it asked for a password
system/30-hardware.sh   # second run: confirms modules + power profiles
```

**Verify:**

- `cat /sys/firmware/acpi/platform_profile_choices` lists profiles (e.g.
  `low-power balanced performance`). Then `powerprofilesctl set performance`
  changes real limits. If the file is missing, see PLAN Phase 4 (Control
  Center).
- `lsmod | grep -E 'tuxedo|uniwill'` shows modules; `mokutil --sb-state`.
- Dock in the rear USB4 port: `boltctl list`, then
  `sudo boltctl enroll --policy auto <uuid>`.
- Fingerprint: check the reader's USB id (`lsusb`) on
  <https://fprint.freedesktop.org/supported-devices.html>, then
  `fprintd-enroll` (the right index finger unless `-f` names another, as
  `-f left-thumb`) and `fprintd-verify`. The script
  already ran `pam-auth-update --enable fprintd`, so `sudo -k; sudo true`
  asks for the finger first (10 s, then password).

### 8. `system/40-apps.sh`

```sh
system/40-apps.sh
```

**Verify:** log out and in, then `docker run --rm hello-world`.
`google-chrome --version`. Snapshot "drivers + apps" is taken at the end.
Without Docker or Chrome: `--no-docker`, `--no-chrome`, which updates keep
to (`kubaos packages enable docker` brings it back).

### 9. `system/50-hyprland.sh`

```sh
system/50-hyprland.sh
```

Installs `hyprland hyprland-guiutils xdg-desktop-portal-hyprland hypridle
hyprlock hyprpolkitagent hyprpicker` from the PPA and `apt-mark hold`s them
plus every installed `libhypr*` / `libaquamarine*` library. Writes the getty
autologin drop-in for tty1 and `/etc/pam.d/qs-lock` + `qs-lock-fingerprint`
(the Quickshell lock screen runs password and fingerprint in parallel), and
`/etc/tmpfiles.d/qs-lock-unix-chkpwd.conf`, which links
`/run/wrappers/bin/unix_chkpwd` to Ubuntu's `/usr/sbin/unix_chkpwd`. Installs
keyd with `/etc/keyd/kubaos.conf`, the `kubaos:mac-accents` XKB option
(`/etc/xkb/rules/evdev`, `/etc/xkb/symbols/kubaos`) and a polkit rule that
lets the desktop user start and stop `keyd.service`: the Mac-style accents
on Left Alt (`install_mac_accents` in `system/lib.sh`).

Why: Quickshell comes from Nix and links Nix's libpam. `pam_unix.so` by name
then means Nix's module, which runs its helper from the NixOS path
`/run/wrappers/bin/unix_chkpwd`. Ubuntu's own `pam_unix.so` / `pam_fprintd.so`
by absolute path don't work: Nix's loader can't find their libraries
(`libcrypt.so.1`, `libsystemd.so.0`) and PAM says "Module is unknown". The
fingerprint service loads nixpkgs' `pam_fprintd.so` from
`/usr/local/lib/security/pam_fprintd.so`, which talks to Ubuntu's fprintd over
D-Bus. That link is root-owned and points straight into `/nix/store`;
`60-nix.sh` creates it (step 4 below), so until then the fingerprint half
fails and the password half works. Tested in an `ubuntu:26.04` container with Nix's
`pamtester` as a non-root user: right password accepted, wrong one rejected.

How Hyprland starts: tty1 autologin -> zsh login shell -> `~/.zprofile`
(written by home-manager) execs **`start-hyprland`**. That's the launcher the
hyprland package ships and the Hyprland wiki documents; it restarts the
compositor in safe mode after a crash. The PPA doesn't ship uwsm. The
canonical snippet is `system/files/hyprland-autostart.zsh`.

**Verify:** `apt-mark showhold` lists the hypr packages;
`Hyprland --version` says 0.56.x;
`systemctl cat getty@tty1` shows the `--autologin` line;
`systemctl is-active keyd` says active.

### 10. `system/60-nix.sh`, then reboot

```sh
system/60-nix.sh                 # = --user kuba --host tuxedo
```

1. Installs Nix with the official installer, pinned to a release
   (`https://releases.nixos.org/nix/nix-<version>/install --daemon`, version
   and sha256 at the top of the script, checked before it runs). Why not
   Determinate's: since early 2026 it installs Determinate Nix only; the
   official script gives upstream Nix, which nixpkgs and home-manager target.
2. Adds `experimental-features = nix-command flakes` to `/etc/nix/nix.conf`.
3. First home-manager switch using the home-manager the flake pins
   (release-26.05), same as
   `nix run home-manager/release-26.05 -- switch --flake ~/Projects/dotfiles#kuba@tuxedo`.
   Existing dotfiles in the way are renamed `*.hm-backup`.
4. Builds `fprintd` from the flake's nixpkgs, links its `pam_fprintd.so` to
   `/usr/local/lib/security/pam_fprintd.so` for the lock screen and adds the
   GC root `/nix/var/nix/gcroots/dotfiles-pam-fprintd`, so
   `nix-collect-garbage` can't delete it. The PAM file never points into
   `$HOME`: a module path you can write to would let anything running as you
   change what the lock screen trusts.
5. GPU for Nix apps: runs `sudo non-nixos-gpu-setup` (home-manager's
   `targets.genericLinux.gpu`), which links Ubuntu-independent Mesa from the
   Nix store to `/run/opengl-driver` on every boot via
   `/etc/tmpfiles.d/non-nixos-gpu.conf`.
6. `chsh -s /usr/bin/zsh`.
7. Snapshot "home-manager base".

Reboot. You get: LUKS passphrase -> autologin on tty1 -> Hyprland. Quickshell
starts from Hyprland's autostart (`qs`, no systemd unit).

**Verify:** `home-manager generations` has one entry; `ls -l
/run/opengl-driver`; Ghostty (`SUPER+Return`) and the bar render;
`hyprctl monitors` shows every screen on the dock at its preferred mode.

Then, outside Nix on purpose:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rust-analyzer clippy
bash <(curl -fsSL https://moonrepo.dev/install/proto.sh)
proto install moon
```

### 11. `system/70-bitwarden.sh`, then `kubaos bitwarden setup`

```sh
system/70-bitwarden.sh
```

Installs the official Bitwarden desktop `.deb` and the standalone `bw` CLI
from github.com/bitwarden/clients, both pinned to a version and checked
against a sha256 in the script (no apt repo exists, and the app doesn't
self-update on Linux). Then `libsecret-tools`, `libpam-u2f`, `pamu2fcfg`,
`pamtester`, and the security key for sudo, polkit and the lock screen:

- two lines above `@include common-auth` in `/etc/pam.d/sudo`, `sudo-i` and
  `polkit-1` (polkitd ships its file in `/usr/lib/pam.d`; the copy in `/etc`
  wins), both `auth sufficient pam_u2f.so authfile=/etc/u2f_mappings cue
  nouserok`: the first with `userverification=1` (your finger on the YubiKey
  C Bio), the second with `pinverification=1` (the key's PIN, when the finger
  fails). No key, or no key enrolled, falls through to the password.
  Re-running the script replaces older versions of these lines.
- `/etc/u2f_mappings`, root:root 0644, filled by `dot-u2f-enroll`.
- a drop-in for `polkit-agent-helper@.service`, whose sandbox hides
  `/dev/hidraw*` from pam_u2f.
- `/etc/pam.d/qs-lock-u2f` and the root-owned link
  `/usr/local/lib/security/pam_u2f.so` to nixpkgs' module (Nix's libpam can't
  load Ubuntu's, same story as `pam_fprintd`), with the GC root
  `/nix/var/nix/gcroots/dotfiles-pam-u2f`.

Then `system/75-yubikey.sh` (add `--piv` for the Multi-protocol Edition's
smart card; updates keep it). As yourself, in the Hyprland session: open Bitwarden, log in,
Settings -> enable "SSH agent". Run `kubaos bitwarden setup` (SSH keys, git
signing, gh, project env). Set the key's PIN and fingerprints with `ykman`, then run
`dot-u2f-enroll` for each of your two keys. `docs/SECRETS.md`, "YubiKey C
Bio", has the order.

**Verify:** `ssh -T git@github.com` greets you; `git commit` in any repo, then
`git log --show-signature -1` says "Good git signature"; from a second tty
`sudo -k; sudo true` says "Touch the fingerprint sensor", and with the key
unplugged asks for the password. Lock with `qs ipc call session lock`: "Touch
the fingerprint sensor" shows under the password field while the key is
plugged in, "Or use your security key" while it isn't.

Undo the PAM part: `system/70-bitwarden.sh --remove-u2f`.

### Removing Nix (if ever needed)

The official installer has no uninstaller. Per the Nix manual:
`sudo systemctl disable --now nix-daemon.socket nix-daemon.service`, remove
`/etc/systemd/system/nix-daemon.*`, restore `/etc/bash.bashrc`,
`/etc/zshrc`, `/etc/profile.d/nix.sh` from their `.backup-before-nix` copies,
delete the `nixbld*` users and group, then `/nix` (here, delete the `@nix`
subvolume) and `~/.nix-*`, `~/.local/state/nix`.

### Testing in a VM first (UTM on Apple Silicon)

Worth an evening before the laptop arrives: practice the subvolume
conversion, run the scripts, try the home-manager config `kuba@vm`.

#### Setup

1. ISO: `ubuntu-26.04.1-live-server-arm64.iso` from
   <https://cdimage.ubuntu.com/releases/26.04/release/>.
2. UTM -> Create a New VM -> **Virtualize** -> Linux. Tick **Hardware OpenGL
   Acceleration** (UTM then uses the `virtio-gpu-gl-pci` display device;
   check under Display after creating). 8 GB RAM, 4+ cores, 64 GB disk.
3. Install exactly as in section 2 (same partitions, LUKS, LVM, btrfs), with
   hostname **`vm`**, then do `00-btrfs-subvolumes.md` from the ISO. The
   device is `/dev/vda` (partitions `vda1`, `vda2`, `vda3`) instead of
   `nvme0n1p*`.
4. `sudo apt install spice-vdagent` for clipboard sharing (optional).
5. Run the scripts in order. `60-nix.sh` picks `--host vm` from the hostname
   (`kuba@vm`, aarch64-linux).

#### What you can and can't test

| Works in the VM | Doesn't |
|---|---|
| Subvolume conversion, snapper, grub-btrfs, rollback drill | `tuxedo-drivers`, power profiles, MOK (TUXEDO repo is amd64 only; the script skips it) |
| 10-strip, 20-base, NetworkManager, PipeWire | Fingerprint reader, USB4 dock, two external monitors at 120 Hz |
| Docker (arm64 packages exist) | Chrome (amd64 only, skipped; use Chromium from Nix if needed) |
| Nix, home-manager `kuba@vm`, GPU wiring via `/run/opengl-driver` with virgl | |
| tty1 autologin -> zsh -> `start-hyprland` | **Hyprland from the PPA** |

**Hyprland in the VM.** `ppa:cppiber/hyprland` builds amd64 binaries only
(arm64 gets just the arch-independent data packages). On arm64,
`50-hyprland.sh` installs Ubuntu's own `hyprland` from universe instead, which
is **0.53.3** in resolute. That predates the Lua config (0.55+), so it starts
but **ignores `config/hypr/*.lua`**: you can test the session plumbing
(autologin, `start-hyprland`, portals, PipeWire, hyprlock PAM), not the
Hyprland config.

To test the Lua config in the VM, the realistic route is Hyprland from
nixpkgs, not a source build (a source build needs the whole hypr* library
stack at matching versions). nixos-26.05 has 0.55.4 (Lua config works, a
minor version behind the laptop); nixos-unstable has 0.56.2. After
`60-nix.sh` has set up `/run/opengl-driver`:

```sh
nix profile add nixpkgs/nixos-unstable#hyprland   # VM only, not in the flake
~/.nix-profile/bin/start-hyprland                 # from tty1
```

This is a test-only detour, untested here; on the laptop Hyprland always
comes from the PPA.

If Hyprland shows a black screen or fails with EGL / GBM errors in the VM:

- check that Hardware OpenGL Acceleration is on (`lspci -k` shows
  `virtio-gpu`, `eglinfo -B` from `mesa-utils` shows virgl, not llvmpipe);
- try `AQ_NO_ATOMIC=1 start-hyprland` (Aquamarine's switch for legacy KMS)
  and `AQ_NO_MODIFIERS=1`.

`WLR_RENDERER_ALLOW_SOFTWARE` does nothing here: that's a wlroots variable,
and Hyprland hasn't used wlroots since 0.42 (it uses Aquamarine, which has no
software-rendering switch). Without GL acceleration Hyprland doesn't run.
