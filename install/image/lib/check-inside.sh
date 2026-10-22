#!/usr/bin/env bash
# In-container half of install/image/check (privileged, kubaos-image-
# tools image, payload at /payload, scratch volume at /check). Unpacks the
# payload the way the engine would and checks it through chroot.
set -uo pipefail

P=/payload
R=/check/root
M="$P/manifest.json"
FAILS=0
ok() { printf '  ok    %s\n' "$*"; }
fail() { printf '  FAIL  %s\n' "$*"; FAILS=$((FAILS + 1)); }
info() { printf '        %s\n' "$*"; }
sect() { printf '\n== %s\n' "$*"; }
check() { local what="$1"; shift; if "$@" >/dev/null 2>&1; then ok "$what"; else fail "$what"; fi; }
in_root() { chroot "$R" /usr/bin/env -i PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin LANG=C.UTF-8 HOME=/root XDG_RUNTIME_DIR=/tmp "$@"; }
now() { date +%s.%N; }
since() { awk -v a="$1" -v b="$(now)" 'BEGIN { printf "%.1f", b - a }'; }

sect "manifest"
for k in version codename arch built_at commit username hostname rootfs kernel home_generation sha256; do
  v="$(jq -er ".$k | strings" "$M" 2>/dev/null)" && ok "$k = $v" || fail "$k missing or not a string"
done
for k in rootfs_bytes installed_bytes; do
  v="$(jq -er ".$k | numbers" "$M" 2>/dev/null)" && ok "$k = $v" || fail "$k missing or not a number"
done
rootfs="$P/$(jq -r .rootfs "$M")"
gen="$(jq -r .home_generation "$M")"
kernel="$(jq -r .kernel "$M")"
user="$(jq -r .username "$M")"
[[ "$(stat -c %s "$rootfs")" == "$(jq -r .rootfs_bytes "$M")" ]] && ok "rootfs_bytes matches the file" || fail "rootfs_bytes != size of $rootfs"
t="$(now)"
[[ "$(sha256sum "$rootfs" | cut -d' ' -f1)" == "$(jq -r .sha256 "$M")" ]] && ok "sha256 matches ($(since "$t") s)" || fail "sha256 mismatch"
info "$(file -b "$rootfs")"

sect "unpack (unsquashfs, as the engine would)"
rm -rf "$R"
t="$(now)"
if unsquashfs -q -n -processors 6 -d "$R" "$rootfs" >/tmp/unsquashfs.log 2>&1; then
  secs="$(since "$t")"
  ok "unpacked in $secs s ($(awk -v b="$(jq -r .installed_bytes "$M")" -v s="$secs" 'BEGIN { printf "%.0f", b / 1048576 / s }') MiB/s of files)"
else
  fail "unsquashfs failed: $(tail -5 /tmp/unsquashfs.log)"
  exit 1
fi
actual="$(du -s -B1 --apparent-size "$R" | cut -f1)"
want="$(jq -r .installed_bytes "$M")"
awk -v a="$actual" -v w="$want" 'BEGIN { d = a - w; if (d < 0) d = -d; exit !(d <= w / 100) }' &&
  ok "installed_bytes within 1% of the unpacked tree ($actual)" || fail "installed_bytes $want, tree is $actual"
info "on disk: $(du -sh "$R" | cut -f1); /nix $(du -sh "$R/nix" | cut -f1), /usr $(du -sh "$R/usr" | cut -f1), /var $(du -sh "$R/var" | cut -f1), /boot $(du -sh "$R/boot" | cut -f1)"

mount -t proc proc "$R/proc"
mount --rbind /sys "$R/sys"
mount --rbind /dev "$R/dev"
trap 'umount -R "$R/dev" "$R/sys" "$R/proc" 2>/dev/null; true' EXIT

sect "kernel and boot chain"
check "/boot/vmlinuz-$kernel" test -f "$R/boot/vmlinuz-$kernel"
check "/boot/initrd.img-$kernel" test -s "$R/boot/initrd.img-$kernel"
check "/lib/modules/$kernel/modules.dep" test -s "$R/lib/modules/$kernel/modules.dep"
check "btrfs module" in_root sh -c "modinfo -k '$kernel' btrfs"
check "dm-crypt module" in_root sh -c "modinfo -k '$kernel' dm_crypt"
for c in grub-install grub-mkconfig update-grub efibootmgr update-initramfs cryptsetup mkfs.btrfs mkfs.vfat mkfs.ext4 snapper locale-gen systemd-machine-id-setup useradd chpasswd; do
  check "$c" in_root sh -c "command -v $c"
done
case "$(jq -r .arch "$M")" in
  x86_64) efi=x86_64-efi shim=shimx64 ;;
  aarch64) efi=arm64-efi shim=shimaa64 ;;
esac
check "GRUB modules /usr/lib/grub/$efi" test -f "$R/usr/lib/grub/$efi/btrfs.mod"
check "signed GRUB /usr/lib/grub/$efi-signed" sh -c "ls $R/usr/lib/grub/$efi-signed/*.efi.signed"
check "shim /usr/lib/shim/$shim.efi.signed*" sh -c "ls $R/usr/lib/shim/$shim.efi.signed*"
check "cryptsetup initramfs hook" test -f "$R/usr/share/initramfs-tools/hooks/cryptroot"
check "grub-btrfs (/etc/grub.d/41_snapshots-btrfs)" test -x "$R/etc/grub.d/41_snapshots-btrfs"
check "snapper settings for the install" test -s "$R/usr/share/kubaos-image/snapper-root.conf"
check "no snapper config yet (made on the target)" test ! -e "$R/etc/snapper/configs/root"

sect "services (systemctl in the chroot)"
for u in NetworkManager.service nix-daemon.socket nix-daemon.service bluetooth.service docker.service \
  systemd-resolved.service snapper-timeline.timer snapper-cleanup.timer grub-btrfsd.service; do
  s="$(in_root systemctl is-enabled "$u" 2>/dev/null)"
  [[ "$s" == enabled ]] && ok "$u enabled" || fail "$u is '$s', want enabled"
done
for u in systemd-networkd.service systemd-networkd-wait-online.service ssh.socket ssh.service motd-news.timer apt-news.service; do
  s="$(in_root systemctl is-enabled "$u" 2>/dev/null)"
  [[ "$s" =~ ^(disabled|masked|not-found|)$ ]] && ok "$u ${s:-absent}" || fail "$u is '$s', want off"
done
for u in pipewire.socket pipewire-pulse.socket wireplumber.service; do
  s="$(in_root systemctl --global is-enabled "$u" 2>/dev/null)"
  [[ "$s" == enabled ]] && ok "$u enabled (user, global)" || fail "$u (user) is '$s'"
done
grep -q -- "--autologin $user" "$R/etc/systemd/system/getty@tty1.service.d/autologin.conf" 2>/dev/null &&
  ok "tty1 autologin for $user" || fail "no tty1 autologin drop-in for $user"

sect "desktop"
v="$(in_root Hyprland --version 2>&1 | head -1)"
[[ "$v" == Hyprland* ]] && ok "Hyprland --version: $v" || fail "Hyprland --version: $v"
held="$(in_root apt-mark showhold | tr '\n' ' ')"
[[ "$held" == *hyprland* ]] && ok "held: $held" || fail "hyprland not held ($held)"
for c in foot fuzzel zsh git docker bw; do check "$c" in_root sh -c "command -v $c"; done
for f in /etc/pam.d/qs-lock /etc/pam.d/qs-lock-fingerprint /etc/pam.d/qs-lock-u2f /etc/tmpfiles.d/qs-lock-unix-chkpwd.conf; do
  check "$f" test -f "$R$f"
done
grep -q pam_u2f "$R/etc/pam.d/sudo" && ok "pam_u2f line in /etc/pam.d/sudo" || fail "no pam_u2f in /etc/pam.d/sudo"

sect "Nix"
NIXB=/nix/var/nix/profiles/default/bin
v="$(in_root "$NIXB/nix" --version 2>&1)" && ok "$v" || fail "nix --version: $v"
check "home generation $gen is valid" in_root "$NIXB/nix-store" --verify-path "$gen"
check "  ... and its whole closure" in_root sh -c "$NIXB/nix-store -q --requisites '$gen' | xargs $NIXB/nix-store --verify-path"
info "closure: $(in_root "$NIXB/nix-store" -q --requisites "$gen" | wc -l) paths, $(in_root "$NIXB/nix" path-info -S "$gen" 2>/dev/null | awk '{printf "%.0f MiB", $2/1048576}')"
check "  activate script" test -x "$R$gen/activate"
home_path="$(readlink "$R$gen/home-path")"
for b in qs ghostty nvim tmux home-manager gpu-screen-recorder zsh; do
  check "  home-path/bin/$b" in_root test -x "$home_path/bin/$b"
done
check "nix-store --verify (database)" in_root "$NIXB/nix-store" --verify
for l in "$R"/nix/var/nix/gcroots/dotfiles-* "$R"/nix/var/nix/gcroots/kubaos/*; do
  tgt="$(readlink "$l")"
  [[ -e "$R$tgt" ]] && ok "gcroot ${l#"$R"} -> $tgt" || fail "gcroot ${l#"$R"} dangles ($tgt)"
done
for l in /usr/local/lib/security/pam_fprintd.so /usr/local/lib/security/pam_u2f.so; do
  tgt="$(readlink "$R$l")"
  [[ -f "$R$tgt" ]] && ok "$l -> $tgt" || fail "$l -> '$tgt' (missing)"
done
gpu="$(in_root cat /etc/tmpfiles.d/non-nixos-gpu.conf 2>/dev/null | grep -o '/nix/store/[^ ]*' | head -1)"
[[ -n "$gpu" ]] && in_root test -e "$gpu" && ok "/run/opengl-driver source $gpu" || fail "non-nixos-gpu.conf missing or dangling"
while IFS= read -r l; do
  in_root test -e "$l" && ok "link $l resolves" || fail "link $l -> $(readlink "$R$l") dangles"
done < <(cd "$R" && find etc usr/local -type l -lname '/nix/store/*' | sed 's#^#/#')
store_size="$(du -sh "$R/nix/store" | cut -f1)"
info "/nix/store: $(find "$R/nix/store" -mindepth 1 -maxdepth 1 | wc -l) entries, $store_size"

sect "file capabilities"
h=/usr/local/libexec/dotfiles/gsr-kms-server
caps="$(getcap "$R$h")"
[[ "$caps" == *cap_sys_admin=ep* ]] && ok "$h: ${caps#* }" || fail "$h has no cap_sys_admin=ep (getcap: '$caps')"
st="$(stat -c '%U:%g %a' "$R$h")"
[[ "$st" == "root:1000 750" ]] && ok "$h is root:1000 0750" || fail "$h is $st"
grep -q "$h" "$R/etc/tmpfiles.d/dotfiles-gsr.conf" && ok "/run/wrappers link in tmpfiles.d" || fail "no dotfiles-gsr.conf"
caps="$(getcap -r "$R/usr/bin" "$R/usr/sbin" "$R/usr/lib" 2>/dev/null | sed "s#^$R##" | tr '\n' ';')"
info "other file caps: $caps"

sect "no machine identity, no build leftovers"
[[ -f "$R/etc/machine-id" && ! -s "$R/etc/machine-id" ]] && ok "/etc/machine-id empty" || fail "/etc/machine-id not empty"
compgen -G "$R/etc/ssh/ssh_host_*" >/dev/null && fail "SSH host keys present" || ok "no SSH host keys"
users="$(awk -F: '$3 >= 1000 && $3 != 65534 && $1 !~ /^nixbld[0-9]+$/ {print $1}' "$R/etc/passwd" | tr '\n' ' ')"
[[ -z "$users" ]] && ok "no user accounts (uid >= 1000, other than nobody and nixbld*)" || fail "users: $users"
awk -F: '$3 == 1000' "$R/etc/passwd" "$R/etc/group" | grep -q . && fail "something has uid/gid 1000" || ok "uid and gid 1000 free"
grep -qw "$user" "$R/etc/passwd" "$R/etc/group" "$R/etc/shadow" "$R/etc/gshadow" && fail "$user still in passwd/group" || ok "$user not in passwd/group/shadow"
[[ -z "$(ls -A "$R/home")" ]] && ok "/home empty" || fail "/home: $(ls -A "$R/home")"
[[ "$(ls -A "$R/etc/sudoers.d" | grep -v README)" == "" ]] && ok "/etc/sudoers.d has no build rule" || fail "sudoers.d: $(ls "$R/etc/sudoers.d")"
for f in /.dockerenv /usr/sbin/policy-rc.d /etc/dpkg/dpkg.cfg.d/docker-apt-speedup /opt/dotfiles; do
  [[ -e "$R$f" ]] && fail "$f present" || ok "no $f"
done
compgen -G "$R/etc/apt/apt.conf.d/docker-*" >/dev/null && fail "docker-* apt confs present" || ok "no docker-* apt confs"
compgen -G "$R/var/lib/apt/lists/*_Packages*" >/dev/null && fail "apt lists present" || ok "no apt lists"
compgen -G "$R/var/cache/apt/archives/*.deb" >/dev/null && fail ".debs in /var/cache/apt" || ok "no cached .debs"
[[ "$(readlink "$R/etc/resolv.conf")" == ../run/systemd/resolve/stub-resolv.conf ]] && ok "/etc/resolv.conf -> resolved stub" || fail "/etc/resolv.conf is $(ls -l "$R/etc/resolv.conf")"
n="$(find "$R/var/log" -type f | wc -l)"
((n == 0)) && ok "no log files" || fail "$n files under /var/log"

sect "KubaOS"
osr() { (. "$R/usr/lib/os-release" && eval "printf '%s' \"\${$1}\""); }
[[ "$(osr NAME)" == KubaOS ]] && ok "os-release NAME=KubaOS" || fail "os-release NAME=$(osr NAME)"
codename="$(jq -r .codename "$M")"
version="$(jq -r .version "$M")"
[[ "$(osr PRETTY_NAME)" == "KubaOS $codename" ]] && ok "PRETTY_NAME=$(osr PRETTY_NAME)" || fail "PRETTY_NAME=$(osr PRETTY_NAME), manifest says $codename"
[[ "$(osr VERSION)" == "${version%.*} ($codename)" ]] && ok "VERSION=$(osr VERSION)" || fail "VERSION=$(osr VERSION), manifest says $version ($codename)"
[[ "$(osr ID)" == ubuntu && "$(osr VERSION_CODENAME)" == resolute && "$(osr VERSION_ID)" == 26.04 ]] &&
  ok "ID, VERSION_ID, VERSION_CODENAME stay Ubuntu's ($(osr ID) $(osr VERSION_ID) $(osr VERSION_CODENAME))" || fail "ID/VERSION_* changed"
[[ "$(readlink "$R/etc/os-release")" == ../usr/lib/os-release ]] && ok "/etc/os-release -> ../usr/lib/os-release" || fail "/etc/os-release isn't the usual link"
d="$(in_root dpkg-divert --list /usr/lib/os-release)"
[[ "$d" == *"/usr/lib/os-release.ubuntu"* ]] && ok "diverted: $d" || fail "/usr/lib/os-release not diverted"
grep -q 'GRUB_DISTRIBUTOR="KubaOS"' "$R/etc/default/grub.d/10-kubaos.cfg" && ok "GRUB_DISTRIBUTOR=KubaOS" || fail "no GRUB_DISTRIBUTOR"
[[ "$(basename "$rootfs")" == "kubaos-${codename,,}-$(jq -r .arch "$M").squashfs" ]] && ok "rootfs name $(basename "$rootfs")" || fail "rootfs name $(basename "$rootfs")"

sect "boot: splash and GRUB (system/25-boot.sh)"
for p in plymouth plymouth-label-ft; do
  [[ "$(in_root dpkg-query -W -f='${db:Status-Status}' "$p" 2>/dev/null)" == installed ]] && ok "$p installed" || fail "$p not installed"
done
alt="$(in_root update-alternatives --query default.plymouth 2>/dev/null | sed -n 's/^Value: //p')"
[[ "$alt" == /usr/share/plymouth/themes/kubaos/kubaos.plymouth ]] && ok "default.plymouth -> kubaos" || fail "default.plymouth is '$alt'"
grep -qx 'Theme=kubaos' "$R/etc/plymouth/plymouthd.conf" && grep -qx 'DeviceScale=1' "$R/etc/plymouth/plymouthd.conf" &&
  ok "plymouthd.conf: Theme=kubaos, DeviceScale=1" || fail "plymouthd.conf: $(tr '\n' ' ' <"$R/etc/plymouth/plymouthd.conf")"
for f in kubaos.plymouth kubaos.script mark.png mark@2x.png field@2x.png Geist-Regular.ttf JetBrainsMono-Regular.ttf; do
  check "theme file $f" test -s "$R/usr/share/plymouth/themes/kubaos/$f"
done
check "initramfs hook kubaos-plymouth" test -x "$R/etc/initramfs-tools/hooks/kubaos-plymouth"
# The payload's own initrd predates the theme (the engine rebuilds it on the
# target), so build one here the way the engine's update-initramfs will.
t="$(now)"
if in_root mkinitramfs -o /tmp/check-initrd.img "$kernel" >/tmp/mkinitramfs.log 2>&1; then
  ok "mkinitramfs ($(since "$t") s)"
  ird="$(in_root lsinitramfs /tmp/check-initrd.img 2>/dev/null)"
  for f in usr/share/plymouth/themes/kubaos/kubaos.script usr/share/plymouth/themes/default.plymouth \
    usr/share/fonts/Plymouth.ttf usr/share/fonts/Plymouth-monospace.ttf etc/plymouth/plymouthd.conf \
    "$(in_root plymouth --get-splash-plugin-path | sed 's#^/##')script.so" \
    "$(in_root plymouth --get-splash-plugin-path | sed 's#^/##')label-freetype.so" usr/sbin/plymouthd; do
    grep -qx "$f" <<<"$ird" && ok "  initramfs has /$f" || fail "  initramfs lacks /$f"
  done
  grep -qx 'scripts/local-bottom/kubaos-snapshot-overlay' <<<"$ird" && grep -q '/overlay.ko' <<<"$ird" &&
    ok "  initramfs has the snapshot overlay script and overlay.ko" || fail "  initramfs lacks the snapshot overlay"
  grep -q 'drivers/gpu/drm/virtio/virtio-gpu.ko' <<<"$ird" && ok "  initramfs has virtio-gpu" || fail "  initramfs lacks virtio-gpu"
  rm -f "$R/tmp/check-initrd.img"
else
  fail "mkinitramfs: $(tail -5 /tmp/mkinitramfs.log)"
fi
d="$(in_root dpkg-divert --list /etc/grub.d/10_linux)"
[[ "$d" == *"/usr/share/kubaos/grub/10_linux.ubuntu"* ]] && ok "10_linux diverted: $d" || fail "/etc/grub.d/10_linux not diverted ($d)"
check "  Ubuntu's 10_linux at the diverted path" test -x "$R/usr/share/kubaos/grub/10_linux.ubuntu"
grep -q 'KubaOS' "$R/etc/grub.d/10_linux" && ok "  /etc/grub.d/10_linux is the KubaOS wrapper" || fail "  /etc/grub.d/10_linux isn't the wrapper"
g="$R/etc/default/grub.d/10-kubaos.cfg"
for kv in 'GRUB_TIMEOUT_STYLE=hidden' 'GRUB_TIMEOUT=1' 'GRUB_RECORDFAIL_TIMEOUT=10' \
  'GRUB_CMDLINE_LINUX_DEFAULT="quiet splash"' 'GRUB_THEME="/boot/grub/themes/kubaos/theme.txt"' \
  'GRUB_BACKGROUND="/boot/grub/themes/kubaos/background.png"'; do
  grep -qxF "$kv" "$g" && ok "10-kubaos.cfg: $kv" || fail "10-kubaos.cfg lacks $kv"
done
for f in theme.txt mark.png select_c.png item_c.png terminal_box_c.png background.png \
  geist-regular-30.pf2 geist-medium-30.pf2 geist-regular-22.pf2 jetbrainsmono-regular-24.pf2; do
  check "GRUB theme /boot/grub/themes/kubaos/$f" test -s "$R/boot/grub/themes/kubaos/$f"
done
grep -q -- '--skip-login --noissue' "$R/etc/systemd/system/getty@tty1.service.d/autologin.conf" &&
  ok "tty1 autologin is silent" || fail "tty1 autologin prints issue/login lines"
check "/etc/skel/.hushlogin" test -f "$R/etc/skel/.hushlogin"

sect "packages"
in_root dpkg-query -W -f='${db:Status-Status} ${Package} ${Version} ${Architecture}\n' | sort -k2 >"$P/packages.txt"
ok "$(grep -c '^installed ' "$P/packages.txt") packages installed (packages.txt)"
bad="$(grep -v '^installed ' "$P/packages.txt" | awk '{print $2}' | tr '\n' ' ')"
[[ -z "$bad" ]] && ok "every package fully installed" || fail "not fully installed: $bad"
a="$(in_root dpkg --audit 2>&1)"
[[ -z "$a" ]] && ok "dpkg --audit clean" || fail "dpkg --audit: $a"
for p in linux-generic cryptsetup-initramfs btrfs-progs snapper network-manager docker-ce hyprland hypridle hyprlock; do
  grep -q "^installed $p " "$P/packages.txt" && ok "$p $(awk -v p="$p" '$2 == p {print $3}' "$P/packages.txt")" || fail "$p not installed"
done
K=/usr/share/kubaos
check "KubaOS checkout at $K, on release" eq "$(in_root git -C "$K" symbolic-ref --short HEAD)" release
check "  origin is GitHub" eq "$(in_root git -C "$K" remote get-url origin)" https://github.com/SuplenC/kubaos.git
check "  owned by root" eq "$(stat -c %u:%g "$R$K" "$R$K/flake.nix" | sort -u)" 0:0
# Nix's fetcher refuses a repository another user owns unless git's system
# config lists it. nobody stands in for the user the payload doesn't have.
check "  readable by another user (safe.directory in /etc/gitconfig)" \
  in_root setpriv --reuid=65534 --regid=65534 --clear-groups env HOME=/nonexistent git -C "$K" rev-parse --verify HEAD
check "  with user-flake/" test -f "$R$K/user-flake/flake.nix.in"
check "user flake lock at /usr/share/kubaos-image/user-flake.lock" test -s "$R/usr/share/kubaos-image/user-flake.lock"
check "no dotfiles checkout in the payload any more" test ! -e "$R/usr/share/kubaos-image/dotfiles"
# The user flake as the engine writes it (engine/install.rs, write_user_flake).
arch="$(jq -r .arch "$M")"
host="$(jq -r .hostname "$M")"
user_flake() {
  rm -rf "$R/tmp/$1"
  mkdir -p "$R/tmp/$1"
  sed -e "s/@USER@/$2/g" -e "s/@HOST@/$host/g" -e "s/@SYSTEM@/$arch-linux/g" \
    "$R$K/user-flake/flake.nix.in" >"$R/tmp/$1/flake.nix"
  cp "$R$K/user-flake/home.nix" "$R/usr/share/kubaos-image/user-flake.lock" "$R/tmp/$1/"
  mv "$R/tmp/$1/user-flake.lock" "$R/tmp/$1/flake.lock"
}
# nix_offline SUBCOMMAND ARGS...: --no-update-lock-file, so the shipped lock
# has to be complete.
nix_offline() {
  in_root /nix/var/nix/profiles/default/bin/nix --offline --option experimental-features 'nix-command flakes' \
    "$1" --no-update-lock-file "${@:2}"
}
# home-manager switch on the machine must work offline: every flake input
# (local ones too) is in the store from `nix flake
# archive`. The payload's user's flake evaluating to the shipped generation
# proves it, and that the shipped lock is complete.
t="$(now)"
user_flake check-home "$user"
if out="$(nix_offline eval --raw "path:/tmp/check-home#homeConfigurations.\"$user@$host\".activationPackage.outPath" 2>&1)" &&
  [[ "$out" == "$(jq -r .home_generation "$M")" ]]; then
  ok "  the user flake evaluates offline to the shipped generation ($(since "$t") s)"
else
  fail "  the user flake doesn't evaluate offline to the shipped generation: $(tail -3 <<<"$out")"
fi
# Another username: the engine builds <user>@<host> offline from the user
# flake with that name in it, no commit anywhere. The payload carries the
# build inputs for that; building one proves it.
t="$(now)"
user_flake check-other kubaos-check
if out="$(nix_offline build --option sandbox false --no-link --print-out-paths \
  "path:/tmp/check-other#homeConfigurations.\"kubaos-check@$host\".activationPackage" 2>&1)" &&
  [[ -x "$R$(tail -1 <<<"$out")/activate" ]]; then
  ok "  a generation for another username builds offline ($(since "$t") s)"
else
  fail "  a generation for another username doesn't build offline: $(tail -5 <<<"$out")"
fi
rm -rf "$R/tmp/check-home" "$R/tmp/check-other"
# Files in the system that name the build user: each must be one the engine
# renders again for the real user (engine/files.rs USER_TEMPLATES, read from
# the payload's own checkout), or a username other than the payload's gets
# the payload's (tty1 autologin into an account that isn't there).
templates="$(sed -n '/^pub const USER_TEMPLATES/,/^];/p' "$R$K/crates/kubaos-core/src/engine/files.rs" |
  grep -o '"/[^"]*"' | tr -d '"')"
[[ -n "$templates" ]] && ok "engine renders $(wc -l <<<"$templates" | tr -d ' ') system file(s) for the user" ||
  fail "no USER_TEMPLATES in the checkout's engine/files.rs"
stray=""
while IFS= read -r f; do
  grep -qxF "${f#"$R"}" <<<"$templates" || stray+=" ${f#"$R"}"
done < <(grep -rIlw --exclude-dir=kubaos-image -e "$user" "$R/etc" "$R/usr/local" "$R/usr/lib/systemd" "$R/var/lib" 2>/dev/null |
  grep -v -e '^'"$R"'/etc/\(passwd\|group\|shadow\|gshadow\|subuid\|subgid\)' )
[[ -z "$stray" ]] && ok "  no other system file names $user" || fail "  name $user, and the engine doesn't render them:$stray"
sect "wallpaper sets"
walls=/usr/share/kubaos/wallpapers
tgt="$(readlink "$R$walls")"
[[ "$tgt" == /nix/store/* && -d "$R$tgt" ]] && ok "$walls -> $tgt" || fail "$walls -> '$tgt', want a store path in the payload"
check "  pinned by a gcroot" test "$(readlink "$R/nix/var/nix/gcroots/dotfiles-wallpapers")" == "$tgt"
n="$(jq -r '.wallpaper_sets | length' "$M")"
((n > 0)) && ok "manifest lists $n set(s): $(jq -r '[.wallpaper_sets[].id] | join(", ")' "$M")" || fail "manifest lists no wallpaper_sets"
while IFS=$'\t' read -r id images preview; do
  # The same filter as assemble.sh: images and videos, not posters.
  have="$(find -L "$R$tgt/$id" -maxdepth 1 -type f \( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' -o -iname '*.webp' \
    -o -iname '*.mp4' -o -iname '*.webm' -o -iname '*.mkv' -o -iname '*.mov' \) ! -iname '*.poster.*' | wc -l)"
  [[ "$have" == "$images" ]] && ok "  $id: $images wallpapers" || fail "  $id: manifest says $images, the set has $have"
  [[ -z "$preview" || -s "$P/$preview" ]] && ok "  $id: preview ${preview:-none}" || fail "  $id: no $P/$preview"
  listed="$(jq -r --arg id "$id" '.wallpaper_sets[] | select(.id == $id) | .files | length' "$M")"
  [[ "$listed" == "$images" ]] && ok "  $id: lists its $listed wallpapers" || fail "  $id: lists $listed wallpapers of $images"
  missing="$(jq -r --arg id "$id" '.wallpaper_sets[] | select(.id == $id) | .files[] | .thumb // empty' "$M" |
    while read -r t; do [[ -s "$P/$t" ]] || printf ' %s' "$t"; done)"
  [[ -z "$missing" ]] && ok "  $id: thumbnails in the payload" || fail "  $id: missing thumbnails:$missing"
done < <(jq -r '.wallpaper_sets[] | [.id, .images, (.preview // "")] | @tsv' "$M")
info "wallpaper thumbnails: $(find "$P/wallpapers" -type f -name '*.jpg' | wc -l) files, $(du -s -B1 --apparent-size "$P/wallpapers" | awk '{printf "%.1f", $1 / 1e6}') MB"

# The ISO carries the live system (about 1.8 GB on x86_64, issue #6) next to
# this payload, and has to stay under 8 GB (docs/adr/0001, "Hosting").
sect "size"
ISO_BUDGET=8000000000 LIVE_RESERVE=2000000000
bytes="$(du -s -B1 --apparent-size "$P" --exclude=check.txt --exclude=packages.txt | cut -f1)"
info "payload $((bytes / 1000000)) MB, of which wallpaper previews $(du -s -B1 "$P/wallpapers" 2>/dev/null | awk '{printf "%.1f", $1 / 1e6}') MB; sets $(du -sL -B1 --apparent-size "$R$tgt" | awk '{printf "%.0f", $1 / 1e6}') MB unpacked"
((bytes <= ISO_BUDGET - LIVE_RESERVE)) &&
  ok "payload fits the ISO budget ($((bytes / 1000000)) MB of $(((ISO_BUDGET - LIVE_RESERVE) / 1000000)) MB)" ||
  fail "payload is $((bytes / 1000000)) MB: with the live system the ISO goes over 8 GB"

check "image.json parses" jq -e .home_generation "$R/usr/share/kubaos-image/image.json"
info "image.json groups: $(jq -c .groups "$R/usr/share/kubaos-image/image.json")"

umount -R "$R/dev" "$R/sys" "$R/proc" 2>/dev/null
[[ "${KEEP:-0}" == 1 ]] || rm -rf "$R"
printf '\n%s\n' "$( ((FAILS)) && echo "FAILED: $FAILS check(s)" || echo "all checks passed")"
((FAILS == 0))
