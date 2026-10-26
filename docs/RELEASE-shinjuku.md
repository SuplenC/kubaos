# KubaOS Shinjuku 0.1: flash it, boot it, check the laptop

Read this before you put the stick in the real machine.

## What Shinjuku 0.1 is

The first KubaOS release: Ubuntu 26.04, Hyprland, Quickshell, your home-manager generation, installed from a USB stick in about a minute. Four ISOs come out of `install/iso/build`, into `iso/`:

| ISO | For |
|---|---|
| `kubaos-shinjuku-0.1.0-x86_64.iso` | The laptop. This is the one you flash |
| `kubaos-shinjuku-0.1.0-aarch64.iso` | ARM machines |
| `kubaos-shinjuku-0.1.0-x86_64-vm.iso` | The Mac's x86_64 VM bench (`test/vm/run`). Has the bench SSH unit |
| `kubaos-shinjuku-0.1.0-aarch64-vm.iso` | The Mac's aarch64 VM bench, fast on HVF |

The `-vm` builds carry the dev SSH unit. The release builds leave openssh-server out (see `test/vm/README.md`). Each ISO has a `.sha256` and a `.txt` next to it.

sha256 sums of the ISOs built 2026-10-17 from the commit that carries this table's ukiyo lock (a rebuild changes them):

| ISO | sha256 |
|---|---|
| x86_64 | `f879100ea6d308012131d6fe3d4f84d964e3f420455d56d2abcc0d65dd4f5172` |
| aarch64 | `c7f591342c7061fd5f5c60f26f6a6ffabb15d1602e64ad68655a6ae026575d1e` |
| x86_64-vm | `fb2cc0b8da60b8388e86d7f7ee30f738d67cde3db64a66cfa334fcc0bae5ab19` |
| aarch64-vm | `719c414536dab30fb53733a3d965ef52918fb49481c1563d25ecc914ce753b68` |

Check yours with `shasum -a 256 -c kubaos-shinjuku-0.1.0-x86_64.iso.sha256` in the `out/` directory.

## Flashing from macOS

**Pick the wrong disk and `dd` will erase it without asking, and there is no undo.** `dd` doesn't care that it's your backup drive or the Mac's own SSD. Read the `diskutil list` output twice. Match the size and the name to the stick. If you're not sure, unplug the stick, run `diskutil list` again, plug it in and see which entry came back.

```sh
diskutil list                                  # find the stick, say /dev/disk4 (external, physical)
diskutil unmountDisk /dev/disk4
sudo dd if=iso/kubaos-shinjuku-0.1.0-x86_64.iso of=/dev/rdisk4 bs=4m status=progress
diskutil eject /dev/disk4
```

Notes:

- `rdisk4` (the raw device) is much faster than `disk4`. The number is the same.
- `status=progress` needs a recent macOS `dd`. If yours doesn't know it, drop it and press Ctrl-T for progress.
- `dd` is silent when it's done. Wait for the prompt, then eject.
- "Resource busy" means something remounted the stick. Run `unmountDisk` again.

## Booting the TUXEDO InfinityBook Pro 14 Gen10

The AMD model, with its 2880x1800 panel.

- **Boot menu: F7.** Hold or tap it right after power on, pick the USB stick.
- **BIOS setup: F2.**

Both are what TUXEDO documents for this model and neither has been checked on this machine yet. Confirm them on the real laptop (it's on the checklist below). If a key does nothing, try the other usual suspects before blaming the stick.

### Secure Boot

Leave it on. The ISO and the installed system boot with shim and Ubuntu's signed GRUB, signed kernel included. That was tested in the VM on x86_64 with Ubuntu's OVMF and Microsoft's keys enrolled, which is what a laptop looks like out of the box (screenshot `12-x86-grub-secure-boot`). The one visible cost: under Secure Boot GRUB can't load our unsigned font files, so the menu uses its built-in 16 px font. Fine at 1080p, probably small on the 2880x1800 panel. See the checklist.

### MOK enrolment for the TUXEDO drivers

The TUXEDO DKMS drivers are kernel modules, and Secure Boot only loads signed ones. So they need a key enrolled, and that happens in the first-boot setup app (`kubaos-setup`, opens by itself at the first login):

1. Open the "TUXEDO drivers" card and run it. It installs the repo, headers and DKMS drivers.
2. It asks for a one-time password, 8 to 16 letters and digits. Pick something you'll remember for 30 seconds. It queues the key with `mokutil --import`.
3. Reboot. The blue MokManager screen appears: "Enroll MOK", Continue, Yes, type the password, reboot. The screen assumes a US keyboard, which is why the password is letters and digits only.
4. Log in. Setup opens once more to confirm the drivers loaded.

Details are in `docs/INSTALL-INTERNALS.md`, "First boot".

## What the VM release run verified

Taken 2026-10-17 on the Mac's bench (screenshots are being retaken):

- aarch64 on HVF: keyboard-only GUI install onto an encrypted disk (53 s), GRUB menu, snapshots menu, Plymouth passphrase prompt (wrong try included), first login with setup alone, "Done for now" opens nothing else, factory snapshot boots from GRUB onto its tmpfs overlay up to the desktop.
- x86_64 under TCG with Secure Boot on: headless install of the `tuxedo` payload, shim and signed GRUB load the theme, first login shows setup only with no keyring prompt, Bitwarden opens floating and centred at 1100x750.
- The installed Hyprland takes its keyboard layout from `/etc/default/keyboard`.

The GRUB menu is hidden on a normal boot. **Press F4 in the hidden second** after the firmware logo to open it. F4 worked on every try on both architectures. Esc is unreliable on aarch64 and can drop you at the `grub>` prompt (`normal` and Enter gets you out).

## Hardware checklist

None of this can be proven in a VM. Tick it on the real laptop.

- [ ] F7 opens the boot menu and F2 opens the BIOS (confirm the keys above)
- [ ] The stick boots under Secure Boot, and Kōbō comes up in `cage`
- [ ] GRUB font size at 2880x1800 under Secure Boot: legible with the built-in 16 px font? If not, apply the lower `gfxmode` fix from `docs/INSTALL-INTERNALS.md`, "Boot"
- [ ] amdgpu handover: the splash goes from GRUB's mode to the kernel's without a black flash or a corrupt frame, and Plymouth's passphrase prompt shows at native resolution
- [ ] TUXEDO DKMS drivers build and load, and the MOK enrolment works from the setup app (`mokutil --sb-state`, `lsmod | grep tuxedo`)
- [ ] Fingerprint reader: `fprintd` enrols a finger from setup, and sudo and the lock screen accept it
- [ ] YubiKey: `pamu2fcfg` registers it from setup, and it works for sudo
- [ ] Suspend and resume: lid close, lid open, the screen and Wi-Fi come back, and nothing hangs
- [ ] Wi-Fi firmware loads, scans and connects, both on the live ISO and the installed system
- [ ] Audio: speakers, headphone jack, the built-in mic
- [ ] Brightness keys and the keyboard backlight keys work, with the right OSD
- [ ] External display: HDMI or USB-C plugged in, hot-plugged, and unplugged without Hyprland losing its mind

## Known limitations

- **ukiyo** is the default wallpaper in this ISO: see `docs/UKIYO.md`.
- **TCG is slow.** x86_64 in the Mac's VM runs under TCG emulation. Quickshell needs about 70 s to load its QML, so the desktop behind setup is plain for the first two minutes, and Bitwarden shows up 79 s after "Done for now". That's emulation, not a bug. On aarch64 (HVF) and on the real laptop it's fast.
- **Bitwarden desktop is x86_64 only.** The aarch64 ISO doesn't have it, and "Done for now" opens nothing there.
- Under Secure Boot the GRUB menu uses the built-in font (see above).
- The F7 and F2 keys, and everything on the hardware checklist, are unconfirmed until someone has the real machine on the desk.
