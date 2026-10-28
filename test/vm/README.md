# QEMU test bench (`test/vm/run`)

Boots the KubaOS live ISO (or any ISO) and the installed disk on the development machine. A guest of the machine's own arch runs on KVM with `-cpu host` and is fast; the other arch runs on TCG and is slow, so use it only for occasional correctness checks. `--arch` defaults to the machine's. QEMU, `qemu-img`, `xorriso` (the KUBAOS_CFG disk) and `cwebp` (screenshots) come from `PATH` when they are all there, else from the flake's nixpkgs (`nix build --inputs-from .`, cached after the first run). The edk2 firmware is the one beside that QEMU, in its `share/qemu`. KVM needs read and write access to `/dev/kvm`.

One VM at a time. Its state lives in `test/vm/out/` (gitignored): `disk.qcow2` (the target, 64 GiB sparse), `vars.qcow2` (UEFI variables), `serial.log`, `qmp.sock`, `id_ed25519` (bench SSH key), `cfg.iso` (the KUBAOS_CFG disk), `shots/`, `install.log`. Downloaded ISOs go in `test/vm/cache/` (gitignored).

## Commands

| Command | What |
|---|---|
| `boot-iso [ISO] [--arch aarch64\|x86_64] [--gui] [--keep-disk]` | Boot an ISO in the background with a fresh target disk. The default ISO is the newest `iso/kubaos-*-ARCH[-vm].iso` |
| `boot-iso … --config F --secrets S \| --demo \| --after poweroff\|reboot\|shell \| --env K=V` | Also attach a KUBAOS_CFG disk (below) |
| `install CONFIG SECRETS [--iso ISO] [--demo] [--demo-fail-at STEP] [--timeout S] [--keep-running]` | Headless install. Waits for the result on serial, then powers off. Exits `0` done, `1` failed, `3` timeout. The log is in `out/install.log` |
| `install … --ssh` | After a done install, open the target from the live system, put the bench key in its `/root/.ssh` and enable `ssh.socket`, then power off. The installed system ships with sshd off; this is how the bench gets in. Bench only |
| `target-ssh CONFIG SECRETS` | The same on a live ISO that's already running (`boot-iso --keep-disk --after shell`) |
| `boot-iso …` / `install … --secure-boot` | x86_64 only: boot with Secure Boot on (below) |
| `boot-disk [--gui]` | Boot the target disk, without the ISO, in the firmware mode (plain or Secure Boot) of the last `boot-iso`/`install`. With LUKS on, type the passphrase: `test/vm/run type 'bench-luks\n'` (the prompt is on the screen, `shot` shows it) |
| `ssh [CMD…]` | `root@` the live system (dev ISO builds only) |
| `wait-serial REGEX [--timeout S]`, `wait-ssh [--timeout S]` | Block until the pattern shows up or SSH answers |
| `serial [-f]` | Print or follow the serial log (`\r` stripped) |
| `shot NAME [--png]` | QMP `screendump` to `out/shots/NAME.webp` (q92, sharp_yuv) |
| `type TEXT`, `key COMBO…` | Keyboard input via QMP `send-key`. `\n` is Enter. Combos look like `ctrl-alt-f2`, `ret`, `esc` |
| `snapshot save\|restore\|delete NAME`, `snapshot list` | qcow2 internal snapshots. A running VM is saved whole (RAM in `disk.qcow2`, EFI vars too) in 1–2 s. A stopped VM saves the disk only |
| `qmp JSON` | One raw QMP command |
| `status`, `stop`, `clean` | `clean` deletes `out/` but keeps `cache/` |

Env overrides: `VM_OUT`, `VM_MEM` (MiB, default 4096), `VM_CPUS` (4), `VM_DISK_SIZE` (64G), `VM_SSH_PORT` (2229), `VM_RES` (1920x1080; `2880x1800` is the laptop's panel, for HiDPI checks). Several worktrees run benches at once: each has its own `out/`, but give each its own `VM_SSH_PORT`, or QEMU fails with "Could not set up host forwarding rule".

## Hardware

The `virt` machine (aarch64) or `q35` (x86_64), with edk2 UEFI from pflash. The vars file is a per-VM qcow2 copy, so snapshots include it. Devices:

- The ISO is a `scsi-cd` (bootindex 1) and appears as `/dev/sr0`.
- The KUBAOS_CFG disk is a second `scsi-cd`.
- The target is `virtio-blk` and appears as `/dev/vda`.
- Network is virtio-net user mode, with `127.0.0.1:2229` forwarded to guest port 22.
- Display is `virtio-gpu-pci` at 1920×1080 (`VM_RES`), plus USB keyboard and tablet and virtio-rng. One screen: for a second one, `workspaces-check` has Hyprland make a headless output (`hyprctl output create headless NAME`, gone again with `hyprctl output remove NAME`), which plugs and unplugs like a real screen and shows in `shot` only through its windows' workspace ids.
- The serial console (`ttyAMA0` or `ttyS0`) goes to `out/serial.log`.

`--gui` opens a GTK window, otherwise it's `-display none`. QEMU runs in the background as a plain child process with a pidfile.

## Secure Boot (x86_64)

`--secure-boot` swaps QEMU's edk2 for Ubuntu's: `OVMF_CODE_4M.secboot.fd` with `OVMF_VARS_4M.ms.fd` (Microsoft's KEK and db enrolled, like a laptop out of the box), on `q35,smm=on` with the flash in secure mode. QEMU's edk2 has the secure code but no vars with keys in them. The two files come from Ubuntu's `ovmf-generic` .deb, downloaded once into `cache/ovmf/` and checked against the archive's sha256 (pinned in `run`; when Ubuntu drops that version from the pool, bump `OVMF_DEB_URL` and `OVMF_DEB_SHA256` from `dists/resolute-updates/main/binary-amd64/Packages`).

A fresh target disk always gets fresh vars, so an install never inherits an older boot entry. Check inside the guest with `mokutil --sb-state` and `dmesg | grep -i secure`.

## How config gets in: the KUBAOS_CFG disk

`boot-iso --config/--demo/…` and `install` build `out/cfg.iso` with `xorriso -as mkisofs -J` (ISO 9660 with Joliet), volume label **`KUBAOS_CFG`**, and attach it as a second CD. It holds:

```
install.toml     the InstallConfig (makes the live system run headless)
secrets.toml     the Secrets (copied to a 0600 file under /run, deleted after)
kubaos.env   KEY=value: KUBAOS_AFTER=poweroff|reboot|shell, KUBAOS_DEMO=1,
                 KUBAOS_DEMO_SPEED, KUBAOS_DEMO_FAIL_AT, KUBAOS_VERBOSE=1,
                 KOBO_ARGS
```

The same thing works on real hardware: a USB stick with a filesystem labelled `KUBAOS_CFG` and those files gives an unattended install. The live side is described in `install/iso/README.md`.

The live system prints machine-readable markers on serial. `KUBAOS-INTERACTIVE uptime=…` means Kōbō (or its text mode) started. `KUBAOS-BEGIN …` and `KUBAOS-RESULT: done` / `KUBAOS-RESULT: failed rc=N` bracket a headless run.

## SSH (dev builds only)

QEMU passes the bench's public key as `-fw_cfg name=opt/kubaos/authorized_keys`. A dev live ISO has a unit, `kubaos-bench-ssh.service`, that loads `qemu_fw_cfg`, installs the key for root, generates host keys and starts sshd, but only when that fw_cfg entry exists. Only QEMU started by this bench sets it. `install/iso/build --release` leaves out openssh-server and the unit.

## Examples

```sh
test/vm/run boot-iso                                   # newest KubaOS ISO, interactive flow
test/vm/run wait-serial KUBAOS-INTERACTIVE && test/vm/run shot kobo
test/vm/run ssh kubaos probe --json
test/vm/run install test/vm/examples/install-x86_64.toml test/vm/examples/secrets.toml --demo
test/vm/run install test/vm/examples/install-x86_64.toml test/vm/examples/secrets.toml   # the real engine
test/vm/run install test/vm/examples/install-x86_64.toml test/vm/examples/secrets.toml --ssh   # then:
test/vm/run boot-disk && sleep 15 && test/vm/run type 'bench-luks\n' && test/vm/run wait-ssh
test/vm/run ssh snapper list                           # root on the installed system
test/vm/run shot first-boot
test/vm/menu-check                                     # SUPER + Space launches an app, a sum, SUPER + F1
test/vm/workspaces-check                               # two screens (one headless): per-screen workspaces, unplug, the menu
test/vm/run snapshot save fresh; …; test/vm/run snapshot restore fresh
test/vm/run stop
```

Stock Ubuntu checks the bench itself: `test/vm/run boot-iso test/vm/cache/ubuntu-26.04.1-live-server-amd64.iso` (SHA256 checked against the official `SHA256SUMS`) brings subiquity up on serial.

## Rebuild and retest x86_64

From a clean worktree (the payload build refuses a dirty one), one build at a time:

```sh
install/image/build --arch x86_64
install/image/check --arch x86_64
install/iso/build bins --arch x86_64                  # binaries into iso/bin/x86_64
P=install/out/payload-x86_64 B=iso/bin/x86_64
install/iso/build --arch x86_64 --profile vm --payload $P --bin $B   # -> …-x86_64-vm.iso, the bench's
install/iso/build --arch x86_64 --release    --payload $P --bin $B   # -> …-x86_64.iso, the USB stick
install/test/engine-e2e                               # engine against a loop device, 4 scenarios

export VM_SSH_PORT=2286 VM_CPUS=6 VM_MEM=8192           # a port no other worktree uses
test/vm/run install test/vm/examples/install-x86_64.toml test/vm/examples/secrets.toml \
  --iso iso/kubaos-shinjuku-0.1.0-x86_64-vm.iso --arch x86_64 --secure-boot --ssh --timeout 3000
test/vm/run boot-disk                                   # Secure Boot again; LUKS prompt after ~40 s
test/vm/run type 'bench-luks\n' && test/vm/run wait-ssh --timeout 600
test/vm/run ssh 'mokutil --sb-state; systemctl --failed; findmnt -t btrfs; snapper list; Hyprland --version'
test/vm/run shot desktop                                # give the desktop a minute first
test/vm/run boot-iso iso/kubaos-shinjuku-0.1.0-x86_64.iso --arch x86_64 --secure-boot
test/vm/run wait-serial KUBAOS-INTERACTIVE --timeout 300   # the release ISO has no ssh: check serial and shot
test/vm/run stop
```

`examples/install-x86_64.toml` is `install.toml` with `hostname = "tuxedo"`, which the x86_64 payload requires; `install.toml` (`hostname = "vm"`) is for the aarch64 payload. `--iso` matters: without it `install` takes the newest `kubaos-*-x86_64*.iso`, which may be the release one, and that has no SSH for `--ssh`.

Hyprland and Quickshell render with llvmpipe here (the display is `virtio-gpu-pci`, without GL): Hyprland takes about three cores with the video wallpaper playing. Slow, never broken.
