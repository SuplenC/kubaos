# install/image: the KubaOS payload

`install/image/build` produces the payload the engine installs: `manifest.json` (`kubaos_core::Manifest`) and `kubaos-shinjuku-<arch>.squashfs`, the whole installed KubaOS Shinjuku system. `install/image/check` unpacks a payload in a privileged container and checks it through chroot.

```sh
install/image/build                       # this machine's arch; x86_64 is host tuxedo -> install/out/payload-x86_64/
install/image/build --arch aarch64        # host vm, through QEMU (binfmt) -> install/out/payload-aarch64/
install/image/check [--arch aarch64]      # unpack + chroot checks; writes check.txt, packages.txt, and check.ok on a pass
```

`build` refuses a dirty worktree unless you pass `--allow-dirty`. With it, the flake is built from the working tree (tracked plus untracked, not ignored), the commit gets a `-dirty` suffix, and the shipped checkout has the working tree laid over `HEAD`. Release payloads come from a clean tree.

## What's in the payload

| | |
|---|---|
| `manifest.json` | `Manifest`. `version` and `codename` are the KubaOS release, `0.1.0` and Shinjuku, from `VERSION` and `CODENAME` in the tree being built (`--allow-dirty` reads the working tree's). The build id (`<UTC date>-<commit 12>[-dirty]`) is in `image.json`. The payload carries no git identity: the user enters it on the Git screen. `kernel` is the `uname -r` of the one installed kernel. `installed_bytes` is the apparent size of the unpacked tree, which is also what unpack progress should count |
| `kubaos-shinjuku-<arch>.squashfs` | The root tree, squashfs with zstd (level 15, 1 MiB blocks), xattrs on |
| `check.txt`, `packages.txt` | Written by `check`, not used by the engine |
| `check.ok` | Written by `check` when every check passes: the sha256 of the `manifest.json` it checked. `install/iso/build` refuses a payload without a matching one |

The tree is Ubuntu 26.04 after `system/10-strip.sh`, `20-base.sh`, `25-boot.sh`, `40-apps.sh`, `50-hyprland.sh`, `60-nix.sh`, `65-recorder.sh`, `70-bitwarden.sh` and `75-yubikey.sh`, plus what the Server installer would have put there first. That covers `ubuntu-minimal`, systemd, `linux-generic`, initramfs-tools, cryptsetup(-initramfs), btrfs-progs, dosfstools, e2fsprogs, GRUB EFI (plain and signed), shim-signed, efibootmgr, os-prober, locales, tzdata, keyboard-configuration, console-setup and openssh-server. `/nix` is a minimal store holding:

- the home-manager activation package for `kuba@<host>` (`vm` on aarch64, `tuxedo` on x86_64), built from the user flake with the shipped lock: `manifest.home_generation`, GC root `/nix/var/nix/gcroots/kubaos/home-generation`. `kuba` is the build user
- what the engine needs to build that generation for another username offline (docs/INSTALL-INTERNALS.md, "Another username"), GC roots `kubaos/rebuild-input-*`. `assemble.sh` builds the generation for a probe user (`kubaos-probe`, from the user flake written for that name) and diffs the two derivation closures. What differs is what names the user. The inputs of those derivations that aren't themselves user-specific (their outputs, through `nix derivation show` and `lib/build-inputs.py`) go into the payload. About 18 MB beyond the generation's own closure
- what `system/` links into the store: nixpkgs' `pam_fprintd` and `pam_u2f` and gpu-screen-recorder (GC roots `dotfiles-*`, as on the laptop), and the `/run/opengl-driver` bundle from `non-nixos-gpu-setup`
- the user flake's inputs and `/usr/share/kubaos`'s (`nix flake archive`), so the machine can evaluate both offline (GC roots `kubaos/flake-input-*`). That includes local ones, such as a checkout on the build machine (`git+file:///home/...`): `build` bind-mounts every `git+file://` and `path:` input from `flake.lock` read-only at its own path into the assemble container (`test/lib/local-inputs.sh`), and `assemble.sh` lists them as git `safe.directory` for the build user (in `~/.gitconfig`, which stays out of the payload with `/home`). `check` evaluates the user flake with `--offline` in the unpacked tree and compares it with `manifest.home_generation`, which is what `home-manager switch` on the machine needs
- Nix itself (the default profile from the official installer)

Nothing else from the build's store goes in.

`/usr/share/kubaos` in the tree is KubaOS's checkout: a git clone of the commit the payload was built from (full history), root-owned, on a local `release` branch, with `origin` set to `https://github.com/SuplenC/kubaos.git` and listed in `safe.directory` in `/etc/gitconfig`. With `--allow-dirty`, a commit on `release` holds the tree it was built from. The user flake takes KubaOS from it; `system/` scripts on the machine run from it.

`/usr/share/kubaos-image/` in the tree holds:

| File | What |
|---|---|
| `image.json` | `version`, `build`, `commit`, `username`, `uid`/`gid` (1000), `hostname`, `home_generation`, `groups` (supplementary groups the scripts gave the user: `sudo`, `docker`, `i2c`), `kubaos`, `user_flake_lock`, `snapper_root_config`, `kernel` |
| `user-flake.lock` | The user flake's lock: `kubaos` at `release`, the user's `nixpkgs` at KubaOS's revision (`lib/user-flake-lock.py`). The engine writes it to `~/.config/kubaos/home/flake.lock` |
| `snapper-root.conf` | `KEY=VALUE` lines from `20-base.sh`, for `snapper -c root set-config` |

## The laptop's hardware in the x86_64 payload

TUXEDO InfinityBook Pro 14 Gen10, AMD: Ryzen AI 9 HX 370 (Strix Point) with the Radeon 890M. Checked 2026-10-16 against TUXEDO's spec page and the payload's kernel (7.0.0-38-generic) and `/lib/firmware`. Everything below ships in the payload except what's marked first boot.

| Component | Driver | Firmware / userspace | In the payload |
|---|---|---|---|
| CPU microcode (Zen 5) | early initramfs | `amd64-microcode` (`intel-microcode` too, from `20-base.sh`) | yes |
| Radeon 890M (GC 11.5.0, DCN 3.5, VCN 4.0.5, PSP 14) | `amdgpu` | `linux-firmware-amd-graphics`: `gc_11_5_*`, `dcn_3_5*`, `vcn_4_0_5`, `psp_14_0_*` | yes |
| VA-API / Vulkan | radeonsi, RADV | `mesa-libgallium` (holds the VA-API drivers in 26.04), `mesa-vulkan-drivers`, `vainfo` | yes. `intel-media-va-driver` (iHD) is for Intel iGPUs: not needed |
| Wi-Fi option 1: Intel AX210 + BT 5.3 | `iwlwifi`/`iwlmvm`, `btusb`/`btintel` | `linux-firmware-intel-wireless`: `iwlwifi-ty-a0-gf-a0-*` | yes |
| Wi-Fi option 2: AMD RZ616 = MediaTek MT7922 + BT 5.2 | `mt7921e`, `btusb`/`btmtk` | `linux-firmware-mediatek`: `WIFI_MT7922_*`, `BT_RAM_CODE_MT7922_*` (MT7925 too) | yes |
| Ethernet: Motorcomm YT6801 | `dwmac-motorcomm` (stmmac glue, in the 7.0 kernel; no TUXEDO DKMS needed) | none | yes |
| Audio: HDA codec + ACP 7.x mics | `snd-hda-intel`, `snd-pci-ps`/`snd-acp70`, SOF `snd-sof-amd-acp70` | `firmware-sof-signed`, `linux-firmware-amd-misc` | yes |
| NPU (XDNA 2) | `amdxdna` | `linux-firmware-amd-misc`: `amdnpu/17f0_*` | yes (no userspace) |
| Webcam + IR camera | `uvcvideo` | none | yes |
| SD reader: Genesys GL9767 | `sdhci-pci` (gli) | none | yes |
| USB4 / DP alt mode (the dock) | `thunderbolt`, `ucsi_acpi` | none | yes |
| Power: platform profiles, PMF, s2idle | `amd-pmf`, `amd-pmc` | `power-profiles-daemon` | yes |
| Keyboard backlight, fan, TCC, Fn keys | `tuxedo-drivers` (DKMS) | TUXEDO repo | first boot (`30-hardware.sh`): DKMS, signed with a MOK for Secure Boot |
| Fingerprint reader | libfprint | `fprintd`, `libpam-fprintd` | packages yes; PAM switch-on and enrollment at first boot |
| Firmware updates | | `fwupd` | yes |
| DKMS prerequisites | | `linux-headers-generic`, `mokutil`, `sbsigntool` | yes |

`linux-firmware` costs 732 MiB in `/lib/firmware`, already zstd-compressed, so about the same in the squashfs (19% of 3.9 GB). The laptop needs about 120 MiB of it (amd-graphics, amd-misc, intel-wireless or mediatek). The rest (qualcomm-misc 163 MiB, nvidia-graphics 106 MiB, mellanox-spectrum 60 MiB, prestera, netronome, qlogic and so on) stays: the metapackage depends on all of it, a USB Wi-Fi stick or a dock may want some of it, and dropping it would save well under a second of unpack on NVMe.

## Format: squashfs, zstd

Both formats the engine supports were options. squashfs wins on four points:

- **Unpack speed.** `unsquashfs` decompresses on all cores. A `.tar.zst` decodes on one thread, and tar extraction is serial too. See the timings below.
- **Progress.** `unsquashfs -percentage` prints one integer per line. The engine can also count bytes written against `installed_bytes`. With tar you would count compressed bytes read against `rootfs_bytes`.
- **xattrs and file capabilities.** squashfs stores xattrs by default, and `unsquashfs` run as root restores them by default. That includes `security.capability` on `/usr/local/libexec/dotfiles/gsr-kms-server` (`cap_sys_admin+ep`), which `check` verifies after a real unpack. With tar, both sides have to remember `--xattrs --xattrs-include='*'`, and some tar implementations drop `security.*` quietly.
- **Hardlinks, inspection.** Hardlinks are kept. The file can be loop-mounted read-only to look inside, with no unpacking.

The cost is that the live ISO needs `squashfs-tools`, which Ubuntu's live system has anyway. The archive comes out about the same size as a `.tar.zst` at a similar level. Higher zstd levels mostly slow the build down and barely change unpack speed.

Unpack it with the target's subvolumes mounted (`@` at `/mnt`, `@home`, `@nix`, `@log`, `@cache`, ... at their places, so `/nix` lands in `@nix` and `/var/log` in `@log`):

```sh
unsquashfs -f -d /mnt -processors "$(nproc)" -percentage kubaos-shinjuku-x86_64.squashfs
```

`-f` writes into the existing directory and its mount points. Run it as root: then owners, modes, xattrs and file caps are restored.

## How the build works

`build` has two halves.

1. **`docker buildx build --target system`** of `install/image/Dockerfile`, image `kubaos-image-system:<amd64|arm64>`. The build context is the repo, but `Dockerfile.dockerignore` lets in only `system/`. Each stage copies the scripts it runs, so the layers cache the same way the test harness's do (`docs/TESTING.md`):

   | Stage | Holds | Rebuilt when |
   |---|---|---|
   | `os` | `ubuntu:26.04`, the base and boot packages above, the temporary build user `kuba` (uid 1000, NOPASSWD sudo) | the package list |
   | `base` | `system/10-strip.sh`, `20-base.sh` for real | `lib.sh`, `manifest.sh`, the package manifest `packages.toml`, those scripts, `files/{apt,netplan,logind.conf.d}`, `files/systemd/{system,user}.conf.d` |
   | `nix` | Nix from the official installer, with version, URL and sha256 read from `60-nix.sh` | `60-nix.sh` |
   | `apps` | `system/40-apps.sh` (Docker CE; Chrome on amd64) | that script, `files/chrome`, `files/systemd/docker.service.d` |
   | `desktop` | `50-hyprland.sh`, `70-bitwarden.sh`, `75-yubikey.sh` | those, `files/{getty,pam.d,tmpfiles.d,systemd,udev,keyd,xkb,polkit}`, the autostart snippet |
   | `boot` | `25-boot.sh`: Plymouth with the kubaos theme, the GRUB theme in `/boot/grub/themes/kubaos`, the `10_linux` wrapper. Last, so the splash rebuilds nothing else | that script, `files/{plymouth,grub,fonts,initramfs-tools}` |

   apt's `.deb`s and lists sit in BuildKit cache mounts (`kubaos-image-apt-{cache,lists}-<arch>`), so no layer contains apt lists. `/var/lib/apt/extended_states` (what's auto-installed) stays in the image.

2. **`lib/assemble.sh`** runs in a container `kubaos-image-assemble-<arch>` from that image, 6 CPUs and 8 GB. It mounts two volumes: `kubaos-image-nix-<arch>-<nix version>` at `/nix` (the build's store, kept between builds; Docker seeds it from the image's bare Nix install) and `kubaos-image-work-<arch>` (the payload tree). It does this:
   - writes the KubaOS os-release and GRUB distributor (below), copies the repo snapshot to `/home/kuba/Projects/dotfiles` (for `system/`), puts KubaOS's checkout at `/usr/share/kubaos`, writes the user flake for `kuba` and locks it, builds its `homeConfigurations."kuba@<host>".activationPackage`, then the probe user's to find the build inputs for another username. KubaOS's crates in the generation, `kobo` among them, build with crane (`crates/workspace.nix`, docs/CONTRACTS.md): two dependency builds, then each package on top. None of them names the user, so they're in the payload and the engine never builds them
   - runs the Nix half of `system/`, which the image stages can't run because they don't have the flake: `60-nix.sh --no-switch --no-gpu --no-chsh` (the pam_fprintd link, and `max-jobs = auto` in `/etc/nix/nix.conf`, which the payload keeps), `65-recorder.sh` (gsr-kms-server with its file cap), `70-bitwarden.sh --no-desktop --no-cli` (the pam_u2f link), and the generation's `non-nixos-gpu-setup`, which is 60-nix.sh's GPU step pointed at the generation, because there's no `~/.nix-profile` yet
   - `rsync -aHAX` copies the container's own `/` into `/work/root`, incrementally, without mounts, `/home`, apt lists or caches, `/opt/dotfiles`, `/.dockerenv` or `/nix`
   - `nix copy --to local?root=/work/root` copies the closure of the roots listed above into a fresh `/work/root/nix`, then recreates the profiles and GC roots
   - strips identity and build leftovers (next section), enables `nix-daemon.socket` and `.service` offline, disables ssh, writes `/usr/share/kubaos-image/`
   - runs `mksquashfs` (from the flake's nixpkgs, so it doesn't end up in the payload) and writes `manifest.json`

A change under `home/`, `config/`, `scripts/` or `flake.lock` only reruns step 2, and in it only the derivations that changed. apt never runs again.

Why not mkosi: it builds Ubuntu images well, but it has no per-step layer cache. Its incremental mode caches only the base tree, so any change to the package list or a script reruns everything after that. It would also need its own wrapper to run `system/*.sh` as a user with sudo. The Docker stages reuse the harness's proven way of running the real scripts, cached per script.

### What the payload leaves out, and why

| Left out | Why | Done instead by |
|---|---|---|
| The user account (`kuba`, its group, home, NOPASSWD sudo rule), and Docker's `ubuntu` user (uid 1000) | Created at install with the real password | engine, `user` step (uid and gid 1000, see below) |
| `/etc/machine-id` (empty file), `/var/lib/systemd/random-seed`, `credential.secret` | Per machine | engine writes a random machine-id |
| SSH host keys | Per machine | `ssh-keygen -A` if ssh is turned on. `openssh-server` is installed with `ssh.socket` and `ssh.service` disabled, so the VM test bench can turn it on |
| `/etc/fstab`, `/etc/crypttab`, `/etc/hostname` (`localhost`), `/etc/hosts` (localhost only) | Depend on the disk and the answers | engine, `configure` step |
| The snapper config | `snapper create-config` needs the target's btrfs and `/.snapshots` | engine, `snapshot` step |
| `system/30-hardware.sh` (TUXEDO repo, `tuxedo-drivers` DKMS, MOK, `pam-auth-update --enable fprintd`) | DKMS signs with a MOK key that would otherwise ship in every payload, MOK enrollment needs the firmware, and INSTALL-INTERNALS.md puts TUXEDO drivers in first-boot setup | first-boot setup runs `30-hardware.sh` |
| apt lists, `.deb` cache, logs, `/tmp`, `policy-rc.d`, `/.dockerenv`, Docker's `docker-*` apt and dpkg configs | Build artefacts | `apt update` before the first install |
| `home-manager switch` | Needs the user | engine, `home` step: runs `$home_generation/activate` as the user, through the payload's nix-daemon |
| The user flake `~/.config/kubaos/home` | Needs the user | engine writes it from `/usr/share/kubaos/user-flake/` and `user-flake.lock` |
| Bitwarden desktop, Chrome, the PPA's Hyprland on aarch64 | amd64 only | aarch64 has Ubuntu's hyprland 0.53 installed, as in `50-hyprland.sh`, but the session runs nixpkgs' Hyprland from the generation (`home/desktop.nix`), because 0.53 can't read the Lua config |

**KubaOS** (`docs/BRAND.md`): `/usr/lib/os-release` says `NAME="KubaOS"`, `PRETTY_NAME="KubaOS Shinjuku"`, `VERSION="0.1 (Shinjuku)"`, `VARIANT`/`VARIANT_ID` Shinjuku. `ID`, `ID_LIKE`, `VERSION_ID`, `VERSION_CODENAME` and `UBUNTU_CODENAME` are copied from Ubuntu's file, so apt, PPAs and install scripts see Ubuntu 26.04. The file is diverted (`dpkg-divert --local --rename`, Ubuntu's copy sits at `/usr/lib/os-release.ubuntu`), so a `base-files` upgrade can't put Ubuntu's back. dpkg warns that renaming a file from an Essential package is dangerous: the risk is a moment with no file during the rename, and the build writes the new one right away. `HOME_URL` comes from `KUBAOS_HOME_URL` at build time and is left out when that's unset (the repo has no public URL yet). The boot menu, its theme and the splash are `system/25-boot.sh`'s (`docs/INSTALL-INTERNALS.md`, "Boot"): `/etc/default/grub.d/10-kubaos.cfg`, the `10_linux` wrapper that names the entries "KubaOS Shinjuku" after `PRETTY_NAME`, and the Plymouth theme. The aarch64 payload adds `console=ttyAMA0 console=tty0 plymouth.ignore-serial-consoles` (`20-kubaos-console.cfg`). The EFI directory stays `EFI/ubuntu`, because shim and the signed GRUB look there.

Kept minimized: the Docker base image has the same dpkg path excludes as Ubuntu Server (minimized), so no man pages or docs. `unminimize` restores them.

## Notes for the engine

`docs/INSTALL-INTERNALS.md` describes what the engine does. Points that depend on the image:

- **gid 1000.** `/usr/local/libexec/dotfiles/gsr-kms-server` is `root:1000 0750` with `cap_sys_admin+ep`. The engine's `useradd -U` with uid 1000 gets gid 1000, because the payload leaves both free (`check` verifies this). If that ever changes, `chgrp` the helper and run `setcap cap_sys_admin+ep` on it again (a chown or chgrp clears file caps), or run `system/65-recorder.sh` on the machine.
- **Groups.** The scripts put the build user in `sudo`, `i2c` and `docker`, as `image.json` `groups` records. All three are in the engine's list.
- **initramfs.** The payload's initrd was built in a container: it has the cryptsetup hook but no crypttab. The engine's `update-initramfs` in the bootloader step is required, not optional.
- **The user flake** has to belong to the user, down from `~/.config`. `/usr/share/kubaos` stays root's: nothing on the machine writes to it except the updater.
- **Snapper.** `/usr/share/kubaos-image/snapper-root.conf` has `20-base.sh`'s settings as `KEY=VALUE` lines, written by the script itself in image mode. Read them from there to avoid a second copy.
- **ssh** is installed but off. To turn it on: `ssh-keygen -A`, then `systemctl enable ssh.socket`.
- **nix-daemon** is enabled for boot (socket and service). The engine's own daemon in the chroot runs `/nix/var/nix/profiles/default/bin/nix-daemon`, which the payload has. The build user's `nixbld` group and the `nixbld1..32` users (uid 30001+) are part of the multi-user install.

## Docker objects

All named `kubaos-image-*`: images `kubaos-image-system:<arch>` and `kubaos-image-tools:<arch>`, volumes `kubaos-image-nix-<arch>-<nix version>`, `kubaos-image-work-<arch>` and `kubaos-image-check-<arch>` (only with `check --keep`), containers `kubaos-image-assemble-<arch>` and `kubaos-image-check-<arch>` (removed after each run), and BuildKit cache ids `kubaos-image-apt-*`. The harness's `dotfiles-nix` volume is never touched. Remove everything with:

```sh
docker volume rm $(docker volume ls -q --filter name=kubaos-image-)
docker image rm $(docker image ls -q --filter reference='kubaos-image-*')
```

After a Nix version bump in `60-nix.sh`, a new store volume is made. Delete the old one by hand.

## Sizes

The assemble container is limited to 6 CPUs and 8 GB (`KUBAOS_IMAGE_CPUS`, `KUBAOS_IMAGE_MEM`).

| | aarch64 (`vm`) | x86_64 (`tuxedo`) |
|---|---|---|
| Archive (`rootfs_bytes`) | 3355 MiB | 3712 MiB |
| Installed (`installed_bytes`) | 9143 MiB (9.8 GB on disk) | 9696 MiB (11 GB on disk) |
| of which `/nix` / `/usr` | 5.8 / 4.0 GB | 5.7 / 3.7 GB |
| Packages / generation closure | 794 / 814 paths, 4.8 GB | 803 / 817 paths, 4.7 GB |

In a warm build, most of the time goes to `mksquashfs` (zstd 15 over 9 GB), then the rsync of `/` and the `nix copy` of the closure. `--zstd-level` trades build time for archive size. Unpack speed barely changes with the level.
