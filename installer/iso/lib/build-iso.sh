#!/usr/bin/env bash
# Runs inside the dotinstall-iso-builder container (installer/iso/build).
#
# Mounts: /src = installer/iso (ro), /bins = kubaos and dotinstall-gui (ro),
#         /branding = branding hook dir (ro, may be empty),
#         /payload = payload dir (ro, may be empty), /out = output dir,
#         /work = cache volume (base rootfs per package-list hash).
# Env:    ARCH (aarch64|x86_64), PROFILE (hw|vm), RELEASE (0|1), NAME (output basename),
#         ZSTD_LEVEL (squashfs level), REBUILD_BASE (1 forces a fresh base).
set -euo pipefail

# shellcheck disable=SC1091
. /src/lib/brand.env

say() { printf '==> [%s] %s\n' "$(date +%T)" "$*" >&2; }

case "$ARCH" in
  aarch64) DEBARCH=arm64 EFIARCH=arm64 GRUBPLAT=arm64-efi EFI=aa64 MIRROR=http://ports.ubuntu.com/ubuntu-ports SERIAL=ttyAMA0 ;;
  x86_64) DEBARCH=amd64 EFIARCH=amd64 GRUBPLAT=x86_64-efi EFI=x64 MIRROR=http://archive.ubuntu.com/ubuntu SERIAL=ttyS0 ;;
  *) echo "bad ARCH $ARCH" >&2; exit 2 ;;
esac
# A foreign ARCH (amd64 on the Mac) works through the host's binfmt emulation:
# apt runs natively, only maintainer scripts in the chroot are emulated.
SUITE=resolute

# ---- package list ---------------------------------------------------------
pkgs=()
while IFS= read -r line; do
  [[ $line =~ ^[[:space:]]*(#|$) ]] && continue
  line="${line//@EFIARCH@/$EFIARCH}"
  name="$(echo ${line%%#*})"
  tags=""
  [[ $line == *"#"* ]] && tags="${line#*#}"
  skip=0
  for t in $tags; do
    case "$t" in
      hw | vm) [[ $PROFILE == "$t" ]] || skip=1 ;;
      amd64 | arm64) [[ $DEBARCH == "$t" ]] || skip=1 ;;
      dev) [[ $RELEASE == 1 ]] && skip=1 ;;
    esac
  done
  ((skip)) || pkgs+=("$name")
done </src/lib/packages

key="$( (printf '%s\n' "$ARCH" "${pkgs[@]}"; cat /src/lib/base-customize.sh) | sha256sum | cut -c1-16)"
BASE="/work/base-$ARCH-$PROFILE-r$RELEASE-$key"

# ---- stage 1: base rootfs (cached) -----------------------------------------
if [[ ${REBUILD_BASE:-0} == 1 || ! -f $BASE/.done ]]; then
  say "base rootfs: mmdebstrap $SUITE/$DEBARCH, ${#pkgs[@]} packages (profile $PROFILE, release $RELEASE)"
  rm -rf "$BASE"
  mkdir -p "$BASE"
  mmdebstrap --mode=root --variant=minbase --arch="$DEBARCH" \
    --components=main,restricted,universe \
    --aptopt='APT::Install-Recommends "false"' \
    --dpkgopt='path-exclude=/usr/share/doc/*' --dpkgopt='path-include=/usr/share/doc/*/copyright' \
    --dpkgopt='path-exclude=/usr/share/man/*' --dpkgopt='path-exclude=/usr/share/info/*' \
    --dpkgopt='path-exclude=/usr/share/locale/*' --dpkgopt='path-include=/usr/share/locale/locale.alias' \
    --skip=essential/unlink --skip=download/empty \
    --setup-hook='mkdir -p /work/apt-cache "$1/var/cache/apt/archives/"' \
    --setup-hook='sync-in /work/apt-cache /var/cache/apt/archives/' \
    --setup-hook='mkdir -p "$1/etc/initramfs-tools/conf.d"; echo COMPRESS=zstd > "$1/etc/initramfs-tools/conf.d/compress"' \
    --include="$(IFS=,; echo "${pkgs[*]}")" \
    --customize-hook='sync-out /var/cache/apt/archives /work/apt-cache' \
    --customize-hook='cp /src/lib/base-customize.sh "$1/tmp/" && chroot "$1" bash /tmp/base-customize.sh && rm "$1/tmp/base-customize.sh"' \
    "$SUITE" "$BASE/rootfs" \
    "deb $MIRROR $SUITE main restricted universe" \
    "deb $MIRROR $SUITE-updates main restricted universe" \
    "deb $MIRROR $SUITE-security main restricted universe"
  touch "$BASE/.done"
  # keep only the newest few bases
  ls -dt /work/base-"$ARCH"-* 2>/dev/null | tail -n +4 | xargs -r rm -rf
else
  say "base rootfs: cached ($BASE)"
fi

# ---- stage 2: live rootfs ---------------------------------------------------
W=/tmp/build
rm -rf "$W"
mkdir -p "$W/iso/casper" "$W/iso/.disk" "$W/iso/boot/grub" "$W/iso/EFI/boot"
say "live rootfs: overlay, binaries"
cp -a "$BASE/rootfs" "$W/rootfs"
R="$W/rootfs"
cp -a /src/rootfs/. "$R/"
sed -i "s/@VOLID@/$KUBAOS_VOLID/" "$R/etc/systemd/system/cdrom.mount"
install -m 0755 /bins/kubaos "$R/usr/bin/kubaos"
if [[ -f /bins/dotinstall-gui ]]; then
  install -m 0755 /bins/dotinstall-gui "$R/usr/bin/dotinstall-gui"
fi
if [[ $RELEASE == 1 ]]; then
  rm -f "$R/etc/systemd/system/dotinstall-bench-ssh.service" "$R/usr/lib/dotinstall/bench-ssh"
else
  ln -sf ../dotinstall-bench-ssh.service "$R/etc/systemd/system/multi-user.target.wants/dotinstall-bench-ssh.service"
fi
ln -sf ../dotinstall-payload.service "$R/etc/systemd/system/multi-user.target.wants/dotinstall-payload.service"
version="$(date -u +%Y%m%d-%H%M)"
cat >"$R/etc/dotinstall-live" <<EOF
DOTINSTALL_LIVE_NAME="KubaOS $KUBAOS_RELEASE $KUBAOS_VERSION"
DOTINSTALL_LIVE_VERSION=$version
DOTINSTALL_LIVE_ARCH=$ARCH
DOTINSTALL_LIVE_PROFILE=$PROFILE
DOTINSTALL_LIVE_RELEASE=$RELEASE
EOF

# What a person reads is KubaOS; ID and VERSION_CODENAME stay Ubuntu's (BRAND.md).
ubuntu_version_id="$(. "$R/usr/lib/os-release" && echo "$VERSION_ID")"
ubuntu_codename="$(. "$R/usr/lib/os-release" && echo "$VERSION_CODENAME")"
rm -f "$R/etc/os-release"
cat >"$R/etc/os-release" <<EOF
NAME="KubaOS"
PRETTY_NAME="KubaOS $KUBAOS_RELEASE (live)"
VERSION="$KUBAOS_VERSION_SHORT ($KUBAOS_RELEASE)"
VERSION_ID="$ubuntu_version_id"
VERSION_CODENAME=$ubuntu_codename
UBUNTU_CODENAME=$ubuntu_codename
ID=ubuntu
ID_LIKE=debian
VARIANT="$KUBAOS_RELEASE live"
VARIANT_ID=${KUBAOS_RELEASE_ID}-live
EOF
printf 'KubaOS %s live \\l\n\n' "$KUBAOS_RELEASE" >"$R/etc/issue"
printf 'KubaOS %s live\n' "$KUBAOS_RELEASE" >"$R/etc/issue.net"

# Branding hook (installer/iso/README.md): overlays and a script, all optional.
if [[ -d /branding/rootfs ]]; then say "branding: rootfs overlay"; cp -a /branding/rootfs/. "$R/"; fi

# kernel + initrd out of the rootfs, onto the ISO
kver="$(ls "$R/lib/modules" | sort -V | tail -1)"
cp "$R/boot/vmlinuz-$kver" "$W/iso/casper/vmlinuz"
cp "$R/boot/initrd.img-$kver" "$W/iso/casper/initrd"

# EFI: shim -> signed grub (CD build, finds /.disk/info and reads
# /boot/grub/grub.cfg) -> signed kernel. Works with Secure Boot on.
shim="$R/usr/lib/shim/shim$EFI.efi.signed"
[[ -e $shim ]] || shim="$R/usr/lib/shim/shim$EFI.efi.signed.latest"
mm="$R/usr/lib/shim/mm$EFI.efi"
gcd="$R/usr/lib/grub/$GRUBPLAT-signed/gcd$EFI.efi.signed"
[[ -f $shim && -f $gcd ]] || { echo "missing shim ($shim) or grub ($gcd)" >&2; exit 1; }
cp "$shim" "$W/iso/EFI/boot/boot$EFI.efi"
cp "$gcd" "$W/iso/EFI/boot/grub$EFI.efi"
[[ -e $mm ]] && cp "$mm" "$W/iso/EFI/boot/mm$EFI.efi"

echo "KubaOS $KUBAOS_RELEASE $KUBAOS_VERSION live $version ($ARCH)" >"$W/iso/.disk/info"
sed -e "s/@SERIAL@/$SERIAL/g" -e "s/@VERSION@/$version/g" -e "s/@RELEASE@/$KUBAOS_RELEASE/g" \
  -e "s/@KVERSION@/$KUBAOS_VERSION/g" /src/lib/grub.cfg >"$W/iso/boot/grub/grub.cfg"
if [[ -d /branding/iso ]]; then say "branding: ISO overlay"; cp -a /branding/iso/. "$W/iso/"; fi
if [[ -f /branding/hook.sh ]]; then say "branding: hook.sh"; bash /branding/hook.sh "$R" "$W/iso"; fi

# payload: on the ISO next to the squashfs, bind-mounted at /run/dotinstall/payload
if [[ -n "$(ls -A /payload 2>/dev/null)" ]]; then
  say "payload: $(du -sh /payload | cut -f1)"
  mkdir -p "$W/iso/dotinstall"
  cp -a /payload "$W/iso/dotinstall/payload"
fi

# ---- squashfs ------------------------------------------------------------
say "squashfs (zstd -${ZSTD_LEVEL})"
rm -f "$R"/boot/vmlinuz* "$R"/boot/initrd* "$R/etc/machine-id"
: >"$R/etc/machine-id"
rm -f "$R"/etc/ssh/ssh_host_*
rm -rf "$R"/var/lib/apt/lists/* "$R"/var/cache/apt/*.bin "$R"/var/cache/apt/archives/*.deb
# No apt sources: casper-bottom/41apt_build_cache_cdrom runs `apt-get update` at
# every boot, which costs ~7 s of DNS retries when there are sources and no network.
rm -f "$R"/etc/apt/sources.list.d/* && : >"$R/etc/apt/sources.list"
du -sx --block-size=1 "$R" | cut -f1 >"$W/iso/casper/filesystem.size"
mksquashfs "$R" "$W/iso/casper/filesystem.squashfs" -noappend -quiet -no-progress \
  -comp zstd -Xcompression-level "$ZSTD_LEVEL" -b 1M -e boot/efi

# ---- ESP image and ISO --------------------------------------------------------
say "ISO"
esp="$W/esp.img"
kib=$(( $(du -sk "$W/iso/EFI" | cut -f1) + 1024 ))
mkfs.vfat -C -n KUBAOS_EFI "$esp" "$kib" >/dev/null
mmd -i "$esp" ::/EFI ::/EFI/boot
mcopy -i "$esp" "$W/iso/EFI/boot/"* ::/EFI/boot/

iso="/out/$NAME.iso"
xorriso -as mkisofs -quiet \
  -r -J -joliet-long -iso-level 3 \
  -V "$KUBAOS_VOLID" -publisher KubaOS -A "KubaOS $KUBAOS_RELEASE $KUBAOS_VERSION live" \
  -partition_offset 16 \
  -append_partition 2 0xef "$esp" -appended_part_as_gpt \
  -e --interval:appended_partition_2:all:: -no-emul-boot \
  -o "$iso.tmp" "$W/iso"
mv "$iso.tmp" "$iso"
(cd /out && sha256sum "$NAME.iso" >"$NAME.iso.sha256")
{
  echo "iso:       $(du -h "$iso" | cut -f1) ($(stat -c %s "$iso") bytes)"
  echo "squashfs:  $(du -h "$W/iso/casper/filesystem.squashfs" | cut -f1)"
  echo "initrd:    $(du -h "$W/iso/casper/initrd" | cut -f1)"
  echo "kernel:    $kver ($(du -h "$W/iso/casper/vmlinuz" | cut -f1))"
  echo "rootfs:    $(numfmt --to=iec "$(cat "$W/iso/casper/filesystem.size")") unpacked"
} | tee "/out/$NAME.txt"
