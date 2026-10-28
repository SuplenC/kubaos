# The KubaOS live ISO (`install/iso/build`)

A small Ubuntu 26.04 live system that boots straight into Kōbō. It holds `kubaos` and `kobo`, every tool the engine runs, and the payload.

```sh
install/iso/build                               # this machine's arch, hw profile -> iso/kubaos-shinjuku-0.1.0-x86_64.iso
install/iso/build --arch aarch64                # ARM machines
install/iso/build --profile vm                  # -> iso/kubaos-shinjuku-0.1.0-x86_64-vm.iso, for the bench
install/iso/build --payload ../image/out/payload --release    # what goes on the USB stick
install/iso/build bins --arch x86_64            # only the Rust binaries -> iso/bin/x86_64
install/iso/build clean                         # kubaos-iso-* containers, images, volumes
```

| Flag | What |
|---|---|
| `--arch x86_64\|aarch64` | Target arch, default this machine's. The builders run natively; another arch's rootfs is built through binfmt (QEMU user mode, which the machine needs registered for that arch), so it's slower |
| `--profile hw` (default) | `linux-image-generic`, `linux-firmware` and microcode (amd64): for real machines |
| `--profile vm` | No firmware, no microcode: for the bench |
| `--release` | No openssh-server and no bench SSH hook |
| `--payload DIR` | Copied onto the ISO as `/kubaos/payload` and bind-mounted read-only at `/run/kubaos/payload`. The default is `stub-payload/`, which holds only a `manifest.json`: probe and Kōbō work, but `run` refuses (wrong arch, no rootfs) |
| `--bin DIR` | Prebuilt Linux `kubaos` (and optionally `kobo`). Without it, `build bins` compiles the Cargo workspace in Docker first |
| `--branding DIR` | Branding hook (below). The default is `branding/iso` in the repo, if it exists |
| `--name NAME` | Output name. The default is `kubaos-CODENAME-VERSION-ARCH` (today `kubaos-shinjuku-0.1.0-ARCH`), plus `-vm` for the vm profile. `lib/brand.env` derives the release identity from the repo's `VERSION` and `CODENAME` |
| `--zstd N` | squashfs zstd level (default 19) |
| `--rebuild-base` | Ignore the cached base rootfs |

Everything runs in Docker. Objects are named `kubaos-iso-*`:

- images `kubaos-iso-rust:<arch>` and `kubaos-iso-iso:<arch>`, of this machine's arch (`amd64` or `arm64`)
- volumes `kubaos-iso-cargo` and `kubaos-iso-target` (cargo; each checkout of the repo builds in its own `/target/<hash of its path>`, because cargo goes by mtime and worktrees share the volume), plus `kubaos-iso-work-ARCH` (base rootfs and apt archive cache)

The limits are 6 CPUs and 8 GB (`KUBAOS_ISO_CPUS`, `KUBAOS_ISO_MEM`). The Rust binaries are built in the native `rust:1-trixie` image, and cross-compiled for the other arch (`gcc-aarch64-linux-gnu` on amd64, `gcc-x86-64-linux-gnu` on arm64). They link against glibc 2.41, and 26.04 has a newer glibc, so they run.

## How it's built, and why

**mmdebstrap + casper + shim/signed GRUB + xorriso**, scripted in `lib/build-iso.sh`:

1. **Base rootfs** (cached per package-list hash in the work volume). `mmdebstrap --variant=minbase` of `resolute` with `lib/packages`, no recommends, and no docs, man pages or locales. `lib/base-customize.sh` then sets the hostname, disables timers and wait-online, makes the journal volatile, and builds the initramfs with zstd. The downloaded `.deb`s are kept in the work volume, so a base rebuild only pays for dpkg. Times: about 3 min for vm, about 14 min for the first hw build (linux-firmware).
2. **Live rootfs**: the base plus `rootfs/` (overlay), the binaries and `/etc/kubaos-live`. apt sources are emptied because casper runs `apt-get update` on every boot, which cost 7 s of DNS retries. SSH host keys and the machine-id are removed.
3. **squashfs** (zstd), with kernel and initrd beside it in `/casper`, `/.disk/info`, and `/boot/grub/grub.cfg` from `lib/grub.cfg`.
4. **EFI**: `shimARCH.efi.signed` as `EFI/boot/bootARCH.efi`, Ubuntu's signed `gcdARCH.efi` (the CD build of GRUB, which finds `/.disk/info` and reads `/boot/grub/grub.cfg`) and `mmARCH.efi`. They go in a FAT image appended as GPT partition 2 (type EF00) and also serve as the El Torito EFI image. The ISO is hybrid: `dd` it to a USB stick or boot it as a CD. It's UEFI only, with Secure Boot working through Canonical's shim, signed GRUB and signed kernel.

Why not the alternatives:

- `livecd-rootfs` + `live-build` is what Canonical uses, but it is tied to Launchpad's build environment and drags in its own live-build fork.
- Debian's `live-build` targets Debian and handles Ubuntu badly.
- `mkosi` builds great disk images, but it has no ISO9660/El Torito output, and its Ubuntu path is less exercised.
- dracut's dmsquash-live isn't Ubuntu's default initramfs.

mmdebstrap is in the archive and runs fine in a privileged container. casper is Ubuntu's own live-boot initramfs, so medium detection, overlay and `/cdrom` behave exactly as on official ISOs.

## Branding

KubaOS Shinjuku 0.1.0 (docs/BRAND.md), from `VERSION` and `CODENAME` at the repo root. The volume label is `KUBAOS_0_1` (`KUBAOS_<major>_<minor>`) and the ESP label `KUBAOS_EFI`. GRUB entries are "Install KubaOS Shinjuku", "… (safe graphics)" and "… (verbose boot)". The live hostname is `kubaos-live`. `/etc/os-release` says KubaOS but keeps Ubuntu's `ID`, `VERSION_ID` and `VERSION_CODENAME`, and `/etc/issue` reads "KubaOS Shinjuku live". The console strings in `live-flow` and `kobo-text` follow BRAND.md's voice. The disk-erase confirmation is plain on purpose.

The hook for logo and splash assets is `--branding DIR` (default `branding/iso`). Every part is optional:

- `DIR/rootfs/` is copied over the live rootfs (fonts, a plymouth theme, a console font).
- `DIR/iso/` is copied over the ISO tree, for example `boot/grub/` theme files.
- `DIR/hook.sh` runs as `bash hook.sh ROOTFS ISO_TREE` after both. It can, say, append `theme=` lines to `ISO_TREE/boot/grub/grub.cfg`, or enable a unit in the rootfs.

## The live system

- **tty1**: root autologin. `/root/.bash_profile` runs `/usr/lib/kubaos-live/live-flow`:
  - If a filesystem labelled **`KUBAOS_CFG`** holds `install.toml`, or its `kubaos.env` sets `KUBAOS_DEMO=1`, the install runs headless: `kubaos run --config … --secrets … --events human [-v]` (or `kubaos demo`). The output is tee'd to the serial consoles and `/var/log/kubaos-live.log`, bracketed by `KUBAOS-BEGIN` and `KUBAOS-RESULT: done|failed rc=N`. Then it powers off (the default), reboots (only on success, so a failure never loops back into the live system), or drops to a shell (`KUBAOS_AFTER`). The disk format is in `test/vm/README.md`.
  - Otherwise it prints `KUBAOS-INTERACTIVE` on serial and runs `cage -s -- kobo $KOBO_ARGS` (no `--fullscreen`: cage 0.2.1 aborts on it, and cage gives the single window the whole output anyway), with stderr in `/var/log/kobo.log`. Without a DRM render node it sets `WLR_RENDERER=pixman`, and in a VM `WLR_NO_HARDWARE_CURSORS=1`. If Kōbō exits non-zero, `kobo-text` starts. cage 0.2.1 has no cursor-shape protocol, so Kōbō draws its own pointer from the XCursor theme: Bibata-Modern-Classic at size 24 (`bibata-cursor-theme`), the installed system's cursor, which live-flow passes in `XCURSOR_THEME` and `XCURSOR_SIZE` and the build makes the `default` theme. The build fails if the `default` theme is not Bibata-Modern-Classic or lacks the cursors Kōbō uses. Without it, the pointer vanishes for good the first time Kōbō hides it for keyboard use.
- **tty2**: root autologin into `kobo-text`, Kōbō's text mode. This is a bash wizard that asks for disk, host, user, locale, git and passphrases, writes `/run/kubaos/install.toml` plus a 0600 secrets file, and runs `kubaos run --events human`.
- **Payload**: `kubaos-payload.service` bind-mounts `/cdrom/kubaos/payload` (casper's mount of the medium) read-only at `/run/kubaos/payload`. ISO9660 level 3 is used, so files over 4 GiB are fine.
- **Network**: NetworkManager manages everything, including ethernet. Nothing waits for it.
- **Bench SSH** (dev builds only): see `test/vm/README.md`.
- **Shutdown**: `DefaultTimeoutStopSec=10s`. `/etc/systemd/system/cdrom.mount` replaces casper's masked unit: no default dependencies, lazy unmount. The live root runs from the medium, so stopping it failed and printed "[FAILED] Failed unmounting cdrom.mount" on every reboot.
- **Kernel cmdline**: `boot=casper noprompt fsck.mode=skip quiet … console=SERIAL console=tty1`. The GRUB menu is hidden with a 1 s timeout; press Esc for the entries "safe graphics (nomodeset)", "verbose boot" and "UEFI firmware settings".

## Sizes (aarch64)

| | vm profile | hw profile |
|---|---|---|
| ISO (Kōbō, 21.5 MB stripped, included) | 444 MB | 1.1 GB |
| squashfs / rootfs unpacked | 340 MB / 1.1 GB | 994 MB / 1.7 GB |
| initrd | 64 MB | 78 MB |

x86_64, vm profile: ISO 398 MB.

The largest items are `linux-modules` (275 MB), Mesa (LLVM 128 MB, gallium 48 MB, Vulkan drivers about 140 MB), firmware (hw) and fonts. If Kōbō ends up rendering with tiny-skia (CPU), `mesa-vulkan-drivers` can go. Nothing is read from the squashfs until it's needed, so size costs USB write time, not boot time.

## Booting it in a VM yourself

1. Build the payload, check it and build an ISO that carries it: `install/image/build`, `install/image/check`, then `install/iso/build --profile vm --payload "$PWD/install/out/payload-x86_64"` (or `just iso x86_64 vm`, which does all three). `install/iso/build` refuses a payload that hasn't passed the check. Without `--payload` the ISO has the stub, and Kōbō refuses to install.
2. `just vm x86_64 --gui` (`test/vm/run boot-iso --gui`) boots the newest ISO on KVM in a GTK window, with a fresh 64 GB disk (`test/vm/README.md`). GRUB waits 1 s (Esc shows the menu), then Kōbō comes up full screen on its own. Ctrl+Alt+F2 in the guest gives its text mode.
3. Afterwards, `test/vm/run stop`, then `test/vm/run boot-disk --gui` boots the disk. Type the disk passphrase when it asks (with "Use my account password", that's your password). tty1 logs in by itself and Hyprland starts with Quickshell on the Shinjuku wallpaper.

An aarch64 ISO boots the same way with `--arch aarch64`, on TCG: slow, which is fine for a click-through.

The bench's display is plain `virtio-gpu-pci`, without GL, so the guest renders with llvmpipe. That's enough for cage and the iced GUI, and for the installed desktop too: Hyprland and Quickshell run on llvmpipe there, at about three cores. Smoothness and GPU paths are checked on the TUXEDO.
