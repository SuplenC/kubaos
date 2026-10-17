# Convert the installer's flat Btrfs root into subvolumes

Do this right after the first boot of the fresh install, before running any
`system/*.sh` script. It's done from the USB stick because you can't move the
running root. About 15 minutes. Nothing is deleted until the new layout has
booted once, so every step up to the end is reversible.

## Target layout

All subvolumes sit directly under the Btrfs top level (subvolid 5), side by
side (the "flat" layout). A root snapshot then contains only `@`.

| Subvolume | Mounted at | Why it's separate |
|---|---|---|
| `@` | `/` | the system; this is what snapper snapshots and what you roll back |
| `@home` | `/home` | rolling back the system must not roll back your files |
| `@snapshots` | `/.snapshots` | snapper's snapshots live outside `@`, so `@` can be replaced by one of them |
| `@log` | `/var/log` | keep logs across a rollback (you want the logs of what broke) |
| `@cache` | `/var/cache` | apt archives etc., no value in snapshots |
| `@nix` | `/nix` | the Nix store; user environments roll back with home-manager generations, not snapper |
| `@docker` | `/var/lib/docker` | images and volumes would bloat every root snapshot |
| `@swap` | `/swap` | a subvolume with an active swapfile can't be snapshotted; isolate it |

Mount options: `noatime,compress=zstd:1` (not on `@swap`).

Why not openSUSE-style `snapper rollback`: that command works by changing the
default subvolume, but Ubuntu's GRUB script always passes
`rootflags=subvol=@` when `/` is a subvolume, and fstab names `subvol=@`, so
the default subvolume is ignored. Rollback here is: boot a snapshot from the
GRUB menu (grub-btrfs) or the USB, then replace `@` with a snapshot (see
"Rolling back the system later" at the end). It's two commands and it's
predictable.

## What the installer gives you (expected)

```
nvme0n1           259:0  0  1.8T  disk
├─nvme0n1p1       259:1  0    1G  part  vfat      /boot/efi
├─nvme0n1p2       259:2  0    2G  part  ext4      /boot
└─nvme0n1p3       259:3  0  1.8T  part  crypto_LUKS
  └─dm_crypt-0    252:0  0  1.8T  crypt LVM2_member
    └─ubuntu--vg-ubuntu--lv 252:1 0 1.8T lvm btrfs  /
```

Names can differ (`nvme1n1`, a different VG name if you typed one). **Use what
`lsblk` shows on your machine everywhere below.** The commands use these shell
variables so you set them once:

```sh
DISK=/dev/nvme0n1
EFI=${DISK}p1
BOOT=${DISK}p2
LUKS=${DISK}p3
CRYPT=dm_crypt-0                 # first column of /etc/crypttab (step 0)
LV=/dev/ubuntu-vg/ubuntu-lv      # the Btrfs logical volume
```

`CRYPT` matters: step 9 rebuilds the initramfs, and its cryptsetup hook looks
up the open LUKS mapping by name in `/etc/crypttab`. Open it under any other
name (`cryptroot`, say) and the hook warns "target ... not found in
/etc/crypttab" and builds an initramfs that can't unlock the disk.

/boot is plain ext4 outside LUKS and outside Btrfs. That's what we want: GRUB
reads kernels from there, and nothing about it changes here. The LUKS
container, the LVM volume group and `/etc/crypttab` don't change either; only
what's inside the Btrfs filesystem moves.

## 0. Before rebooting into the USB

On the installed system, note these (or take a photo):

```sh
lsblk -f
cat /etc/fstab
cat /etc/crypttab
sudo btrfs subvolume list /       # expect: nothing (flat layout)
ls -la /swap.img 2>/dev/null      # does the installer's swapfile exist?
```

If `btrfs subvolume list /` already shows `@` or other subvolumes, stop: the
installer changed behaviour and these steps need adapting.

## 1. Boot the USB and get a root shell

Boot the Ubuntu Server 26.04 USB (F7 boot menu on TUXEDO). At the first
installer screen press `Ctrl+Alt+F2` for a shell (or use "Help -> Enter shell"),
then `sudo -i`. The installer environment has `cryptsetup`, `lvm` and
`btrfs-progs`. An Ubuntu Desktop live USB also works ("Try Ubuntu", open a
terminal, `sudo -i`).

## 2. Unlock and mount the top level

```sh
lsblk -f                                   # confirm DISK/EFI/BOOT/LUKS, then set the variables above
cryptsetup open "$LUKS" "$CRYPT"           # asks the LUKS passphrase; same name as in crypttab
vgchange -ay                               # activates ubuntu-vg
lvs                                        # shows ubuntu-lv
mkdir -p /mnt/top
mount -o subvolid=5 "$LV" /mnt/top
ls /mnt/top                                # bin boot etc home ... (the flat root)
btrfs subvolume list /mnt/top              # empty
```

## 3. Create `@` as a snapshot of the whole root

A snapshot of the top level is an instant, zero-copy copy of everything. The
original files stay where they are until step 10.

```sh
btrfs subvolume snapshot /mnt/top /mnt/top/@
ls /mnt/top/@/etc/fstab                    # must exist
```

## 4. Create the other subvolumes

```sh
for sv in @home @snapshots @log @cache @nix @docker @swap; do
  btrfs subvolume create "/mnt/top/$sv"
done
btrfs subvolume list /mnt/top
```

## 5. Move data out of `@` into its own subvolumes

`cp --reflink=always` shares the data blocks, so this is quick and uses no
extra space. Then empty the directories inside `@` (they stay as mount points).
The original copies at the top level are untouched.

```sh
cp -a --reflink=always /mnt/top/@/home/.      /mnt/top/@home/
cp -a --reflink=always /mnt/top/@/var/log/.   /mnt/top/@log/
cp -a --reflink=always /mnt/top/@/var/cache/. /mnt/top/@cache/

find /mnt/top/@/home      -mindepth 1 -delete
find /mnt/top/@/var/log   -mindepth 1 -delete
find /mnt/top/@/var/cache -mindepth 1 -delete

mkdir -p /mnt/top/@/.snapshots /mnt/top/@/nix /mnt/top/@/var/lib/docker /mnt/top/@/swap
ls -la /mnt/top/@home                      # your user's home dir is there
```

## 6. Swap file in `@swap`

```sh
rm -f /mnt/top/@/swap.img                  # the installer's swapfile, if any (it's still in the top level copy)
btrfs filesystem mkswapfile --size 8g /mnt/top/@swap/swapfile
```

8 GB is enough with 64 GB RAM and no hibernation. `mkswapfile` sets the
no-copy-on-write attribute that Btrfs swapfiles need.

## 7. Write the new fstab inside `@`

```sh
UUID=$(blkid -s UUID -o value "$LV")
echo "$UUID"
cp /mnt/top/@/etc/fstab /mnt/top/@/etc/fstab.pre-subvolumes
```

Edit `/mnt/top/@/etc/fstab` (`nano`). Delete the old line for `/` and any
`/swap.img` line, keep the `/boot` and `/boot/efi` lines exactly as they are,
and add (replace `<UUID>` with the value printed above):

```
UUID=<UUID>  /                btrfs  noatime,compress=zstd:1,subvol=@           0 0
UUID=<UUID>  /home            btrfs  noatime,compress=zstd:1,subvol=@home       0 0
UUID=<UUID>  /.snapshots      btrfs  noatime,compress=zstd:1,subvol=@snapshots  0 0
UUID=<UUID>  /var/log         btrfs  noatime,compress=zstd:1,subvol=@log        0 0
UUID=<UUID>  /var/cache       btrfs  noatime,compress=zstd:1,subvol=@cache      0 0
UUID=<UUID>  /nix             btrfs  noatime,compress=zstd:1,subvol=@nix        0 0
UUID=<UUID>  /var/lib/docker  btrfs  noatime,compress=zstd:1,subvol=@docker     0 0
UUID=<UUID>  /swap            btrfs  noatime,subvol=@swap                        0 0
/swap/swapfile  none  swap  defaults  0 0
```

Use names (`subvol=@`), not ids: names survive replacing `@` during a rollback.

## 8. Make `@` the default subvolume

```sh
ID=$(btrfs subvolume list /mnt/top | awk '$NF == "@" {print $2}')
echo "$ID"
btrfs subvolume set-default "$ID" /mnt/top
btrfs subvolume get-default /mnt/top       # ... path @
```

fstab and GRUB pass `subvol=@` explicitly anyway; the default only matters for
tools that mount the device without options.

## 9. Chroot into the new root, rebuild initramfs and GRUB

```sh
mkdir -p /mnt/new
mount -o subvol=@ "$LV" /mnt/new
mount -o subvol=@home "$LV" /mnt/new/home
mount -o subvol=@log  "$LV" /mnt/new/var/log
mount "$BOOT" /mnt/new/boot
mount "$EFI"  /mnt/new/boot/efi
for d in dev dev/pts proc sys run; do mount --rbind "/$d" "/mnt/new/$d"; done
mount -t efivarfs efivarfs /mnt/new/sys/firmware/efi/efivars 2>/dev/null || true

chroot /mnt/new /bin/bash
  findmnt -R /                       # / is subvol=/@ (inside the chroot)
  cat /etc/crypttab                  # unchanged, still references the LUKS UUID
  update-initramfs -u -k all         # no "not found in /etc/crypttab" warning allowed
  update-grub
  grep -m3 'rootflags=subvol=@' /boot/grub/grub.cfg   # must print linux lines
  exit
```

No `grub-install` is needed: the EFI binary didn't change, only its config on
the ext4 `/boot`.

## 10. Reboot and verify

```sh
umount -R /mnt/new
umount /mnt/top
vgchange -an ubuntu-vg
cryptsetup close "$CRYPT"
reboot
```

Remove the USB. Unlock LUKS, log in, then:

```sh
cat /proc/cmdline                    # contains rootflags=subvol=@
findmnt -t btrfs                     # /, /home, /.snapshots, /var/log, /var/cache, /nix, /var/lib/docker, /swap
sudo btrfs subvolume list /          # @ @home @snapshots @log @cache @nix @docker @swap
sudo btrfs subvolume get-default /   # @
swapon --show                        # /swap/swapfile
ls ~                                 # your files
```

All good: delete the old flat copy from the top level. It's still taking no
extra space, but leaving it around is confusing:

```sh
sudo mount -o subvolid=5 /dev/ubuntu-vg/ubuntu-lv /mnt
ls /mnt                              # @ @cache ... plus the old bin boot etc home ...
mountpoint -q /mnt && sudo find /mnt -mindepth 1 -maxdepth 1 ! -name '@*' -exec rm -rf {} +
ls /mnt                              # only @* left
sudo umount /mnt
```

Then continue with `system/10-strip.sh` (docs/INSTALL.md). `system/20-base.sh`
sets up snapper on `/` and keeps `@snapshots` mounted at `/.snapshots`.

## Undo, if the new layout doesn't boot

Until you run the delete in step 10, the original flat root is intact at the
top level. From the USB (steps 1-2):

```sh
btrfs subvolume set-default 5 /mnt/top
# GRUB still says rootflags=subvol=@; rebuild it from the old root:
mount "$BOOT" /mnt/top/boot && mount "$EFI" /mnt/top/boot/efi
for d in dev dev/pts proc sys run; do mount --rbind "/$d" "/mnt/top/$d"; done
chroot /mnt/top update-grub
```

The old root's `/etc/fstab` was never edited. Reboot and you're back where you
started; delete the `@*` subvolumes at leisure.

Typical failure: a typo in the new fstab drops you into emergency mode. Log in
there (root password not set? use the USB), `nano /etc/fstab`, reboot.

## Rolling back the system later

`@` is replaced by a read-write copy of a snapshot. Your home, logs, Nix store
and Docker data are untouched.

1. Find the snapshot number: `sudo snapper -c root list` (or pick the snapshot
   in GRUB's "KubaOS snapshots" submenu, boot it read-only, and look around).
2. From the booted snapshot or the USB (from the USB: unlock as in step 2, but
   stop before mounting). The top level goes on `/mnt`: a booted snapshot is
   read-only, so you can't create `/mnt/top` there.

```sh
sudo mount -o subvolid=5 /dev/ubuntu-vg/ubuntu-lv /mnt
N=42                                                          # the snapshot number
sudo mv /mnt/@ /mnt/@broken-$(date +%F)
sudo btrfs subvolume snapshot /mnt/@snapshots/$N/snapshot /mnt/@
ID=$(sudo btrfs subvolume list /mnt | awk '$NF == "@" {print $2}')
sudo btrfs subvolume set-default "$ID" /mnt
sudo umount /mnt
reboot
```

3. When the system works again: `sudo btrfs subvolume delete /mnt/@broken-*`
   (mount the top level on `/mnt` first).

Kernels live on the ext4 `/boot`, not in snapshots. If the snapshot predates a
kernel upgrade, its `/lib/modules` lacks the newest kernel's modules: choose the
previous kernel under "Advanced options for Ubuntu" in GRUB (Ubuntu keeps one),
then `sudo apt upgrade` brings the rolled-back system forward again.
