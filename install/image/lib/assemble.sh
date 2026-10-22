#!/usr/bin/env bash
# In-container half of install/image/build. Runs as root in a container of
# kubaos-image-system:<arch> with:
#   /nix   volume kubaos-image-nix-<arch>-<nix version>: the build's Nix
#          store, kept between builds (seeded by Docker from the image's /nix)
#   /work  volume kubaos-image-work-<arch>: the payload tree (/work/root),
#          synced incrementally
#   /in    read-only: src/ (the repo snapshot, unpacked), dotfiles.bundle
#   /out   the payload directory on the host
#   local flake inputs (git+file:// or path: in flake.lock), read-only at
#          their host paths, listed in LOCAL_INPUTS (colon-separated)
# Environment: ARCH HOST PAYLOAD_USER COMMIT VERSION BUILD_ID BUILT_AT
# ZSTD_LEVEL CPUS, and the OS_* names from build.
#
# Steps: put KubaOS's checkout at /usr/share/kubaos, lock the user flake,
# realise its home-manager generation (and find what a rebuild for another
# username needs), run the Nix half of system/
# (60-nix.sh, 65-recorder.sh, 70-bitwarden.sh), copy this container's / into
# /work/root, put a minimal /nix next to it (just the closure of what the
# payload needs), strip machine identity and the build user, pack squashfs,
# write manifest.json. Not meant to be run by hand.
set -euo pipefail

: "${ARCH:?}" "${HOST:?}" "${PAYLOAD_USER:?}" "${COMMIT:?}" "${VERSION:?}" "${BUILT_AT:?}"
ZSTD_LEVEL="${ZSTD_LEVEL:-15}"
CPUS="${CPUS:-6}"

NIX_BIN=/nix/var/nix/profiles/default/bin
REPO="/home/$PAYLOAD_USER/Projects/dotfiles"
# KubaOS's root-owned checkout on the machine; the user flake takes KubaOS
# from its release branch (docs/INSTALL-INTERNALS.md, "The user flake").
KUBAOS=/usr/share/kubaos
KUBAOS_ORIGIN=https://github.com/SuplenC/kubaos.git
# The build user's user flake. /home stays out of the payload; the lock ships
# as $SHARE/user-flake.lock.
USER_FLAKE="/home/$PAYLOAD_USER/.config/kubaos/home"
ROOT=/work/root
ROOTFS_NAME="${OS_ID:?}-${OS_CODENAME:?}-$ARCH.squashfs"
SHARE=usr/share/kubaos-image   # relative to the payload root

C_B=$'\e[1;34m' C_OFF=$'\e[0m'
[[ -t 1 ]] || C_B='' C_OFF=''
T_START=$SECONDS
say() { printf '%s==> [%4ds] %s%s\n' "$C_B" $((SECONDS - T_START)) "$*" "$C_OFF"; }
die() { printf 'xx  %s\n' "$*" >&2; exit 1; }

USER_PATH="$NIX_BIN:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
as_user() {
  runuser -u "$PAYLOAD_USER" -- env -i HOME="/home/$PAYLOAD_USER" USER="$PAYLOAD_USER" \
    LOGNAME="$PAYLOAD_USER" PATH="$USER_PATH" LANG=C.UTF-8 NIX_REMOTE=daemon \
    DOTFILES_IMAGE_BUILD=1 "$@"
}
nix_() { NIX_REMOTE=daemon "$NIX_BIN/nix" --option cores "$CPUS" --option max-jobs "$CPUS" "$@"; }

# ---------------------------------------------------------------------------
say "nix-daemon"
rm -f /nix/var/nix/daemon-socket/socket
# Rosetta (amd64 on Apple Silicon) can't load Nix's seccomp filter. Only
# this build daemon gets the option; the payload's nix.conf doesn't.
daemon_opts=()
if [[ "$ARCH" == x86_64 && "$(uname -m)" == x86_64 ]] && grep -qi 'VirtualApple' /proc/cpuinfo; then
  daemon_opts=(--option filter-syscalls false)
fi
"$NIX_BIN/nix-daemon" ${daemon_opts[@]+"${daemon_opts[@]}"} >/tmp/nix-daemon.log 2>&1 &
for _ in $(seq 100); do [[ -S /nix/var/nix/daemon-socket/socket ]] && break; sleep 0.1; done
[[ -S /nix/var/nix/daemon-socket/socket ]] || die "nix-daemon didn't start: $(cat /tmp/nix-daemon.log)"

say "repo snapshot -> $REPO (fresh git index, so the flake sees every file)"
rm -rf "$REPO"
install -d -o "$PAYLOAD_USER" -g "$PAYLOAD_USER" "/home/$PAYLOAD_USER/Projects" "$REPO"
cp -a /in/src/. "$REPO/"
chown -R "$PAYLOAD_USER:$PAYLOAD_USER" "$REPO"
as_user sh -c "cd '$REPO' && git init -q -b main && git add -A && \
  git -c user.name=kubaos -c user.email=kubaos@localhost commit -q --no-gpg-sign -m snapshot"

# Local flake inputs are mounted from the host, owned by root here, and git
# (Nix's fetcher too) refuses a repository owned by someone else unless it's
# listed as safe. In the build user's ~/.gitconfig: /home is left out of the
# payload, so this doesn't ship.
IFS=: read -r -a local_inputs <<<"${LOCAL_INPUTS:-}"
for d in ${local_inputs[@]+"${local_inputs[@]}"}; do
  as_user git config --global --add safe.directory "$d"
  echo "    local input $d"
done

# ---------------------------------------------------------------------------
# A clone of the commit (full history) on a local `release` branch, which
# only the updater moves, with origin on GitHub. A --allow-dirty build
# commits what it built from on top, so the release branch holds the files the generation is built from.
say "KubaOS checkout $KUBAOS (branch release, origin $KUBAOS_ORIGIN)"
kubaos_git() { git -c user.name=kubaos -c user.email=kubaos@localhost -C "$KUBAOS" "$@"; }
rm -rf "$KUBAOS"
git clone -q /in/dotfiles.bundle "$KUBAOS"
kubaos_git remote set-url origin "$KUBAOS_ORIGIN"
kubaos_git checkout -q -B release
if [[ "$COMMIT" == *-dirty ]]; then
  # The tree as it was built from, deletions included.
  find "$KUBAOS" -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
  cp -a /in/src/. "$KUBAOS/"
fi
cp "$REPO/flake.lock" "$KUBAOS/flake.lock"
if [[ -n "$(kubaos_git status --porcelain)" ]]; then
  kubaos_git add -A
  kubaos_git commit -q --no-gpg-sign -m "build: what payload $BUILD_ID was built from"
fi
chown -R root:root "$KUBAOS"
# Nix's fetcher (libgit2) refuses a repository another user owns unless
# git's system config lists it. This one ships.
git config --system --get-all safe.directory | grep -qxF "$KUBAOS" ||
  git config --system --add safe.directory "$KUBAOS"
echo "    release at $(kubaos_git rev-parse --short=12 HEAD)"

# user-flake/ for USER in DIR, the way the engine writes it
# (engine/install.rs, write_user_flake).
render_user_flake() {
  local dir=$1 user=$2
  as_user install -d "$dir"
  sed -e "s/@USER@/$user/g" -e "s/@HOST@/$HOST/g" -e "s/@SYSTEM@/$ARCH-linux/g" \
    "$KUBAOS/user-flake/flake.nix.in" | as_user tee "$dir/flake.nix" >/dev/null
  as_user cp "$KUBAOS/user-flake/home.nix" "$dir/home.nix"
}

# The user's nixpkgs starts at KubaOS's revision, so day one has one
# nixpkgs; they part when the user updates theirs.
say "user flake lock (nixpkgs at KubaOS's revision, kubaos at release)"
rm -rf "$USER_FLAKE"
render_user_flake "$USER_FLAKE" "$PAYLOAD_USER"
lock_py=/in/src/install/image/lib/user-flake-lock.py
as_user "$NIX_BIN/nix" flake lock --override-input nixpkgs "$(python3 "$lock_py" override "$KUBAOS/flake.lock")" \
  "path:$USER_FLAKE"
as_user python3 "$lock_py" pin "$KUBAOS/flake.lock" "$USER_FLAKE/flake.lock"

HM_CONFIG="$PAYLOAD_USER@$HOST"
say "home-manager generation $HM_CONFIG (the user flake)"
# --no-update-lock-file: the lock as shipped must be complete.
gen="$(as_user "$NIX_BIN/nix" --option cores "$CPUS" --option max-jobs "$CPUS" build --no-link --print-out-paths \
  --no-update-lock-file "path:$USER_FLAKE#homeConfigurations.\"$HM_CONFIG\".activationPackage")"
[[ -x "$gen/activate" ]] || die "no activate in $gen"
echo "    $gen"

# Another username at install: the engine writes the user flake with that
# name and builds <user>@<host> offline. Only the derivations that name the
# user rebuild (dotfiles, unit and session files, the activation script,
# home-path). A probe user's generation shows which ones; the store paths
# their builders need (stdenv's scripts, perl for buildEnv, ...) go into the
# payload next to the generation.
PROBE_USER=kubaos-probe
say "build inputs for another username (probe: $PROBE_USER@$HOST)"
probe_flake="/home/$PAYLOAD_USER/probe-flake"   # /home stays out of the payload
rm -rf "$probe_flake"
render_user_flake "$probe_flake" "$PROBE_USER"
as_user cp "$USER_FLAKE/flake.lock" "$probe_flake/flake.lock"
drv_of() {
  as_user "$NIX_BIN/nix" eval --raw --no-update-lock-file \
    "path:$1#homeConfigurations.\"$2@$HOST\".activationPackage.drvPath"
}
gen_drv="$(drv_of "$USER_FLAKE" "$PAYLOAD_USER")"
probe_drv="$(drv_of "$probe_flake" "$PROBE_USER")"
nix_ build --no-link "$probe_drv^out" >/dev/null
"$NIX_BIN/nix-store" -qR "$gen_drv" | sort >/tmp/gen-drvs
"$NIX_BIN/nix-store" -qR "$probe_drv" | sort >/tmp/probe-drvs
comm -13 /tmp/gen-drvs /tmp/probe-drvs | grep '\.drv$' >/tmp/differing-drvs || die "the probe generation is the payload's?"
mapfile -t differing </tmp/differing-drvs
"$NIX_BIN/nix" derivation show "${differing[@]}" |
  python3 /in/src/install/image/lib/build-inputs.py /tmp/gen-drvs /tmp/differing-drvs >/tmp/build-inputs
rebuild_paths=()
while read -r p outs; do
  if [[ -n "${outs:-}" ]]; then
    # Not in < <(...): set -e must see path-info fail on a missing output.
    out="$("$NIX_BIN/nix" path-info "$p^${outs// /,}")" || die "no output $outs of $p in the store"
    mapfile -t -O "${#rebuild_paths[@]}" rebuild_paths <<<"$out"
  else
    rebuild_paths+=("$p")
  fi
done </tmp/build-inputs
mapfile -t rebuild_paths < <(printf '%s\n' "${rebuild_paths[@]}" | sort -u)
echo "    ${#differing[@]} derivations name the user; their builds need ${#rebuild_paths[@]} store paths"
rm -rf "$probe_flake"

# The Nix half of system/: PAM module links (60-nix.sh, 70-bitwarden.sh), the
# setcap'd gsr-kms-server (65-recorder.sh). They build from KubaOS's nixpkgs
# (--inputs-from $KUBAOS, system/lib.sh) and leave GC roots in
# /nix/var/nix/gcroots/dotfiles-*.
say "system/60-nix.sh --no-switch (pam_fprintd link)"
as_user "$REPO/system/60-nix.sh" --host "$HOST" --user "$PAYLOAD_USER" --no-switch --no-gpu --no-chsh
say "system/65-recorder.sh (gsr-kms-server, cap_sys_admin)"
as_user "$REPO/system/65-recorder.sh"
say "system/70-bitwarden.sh --no-desktop --no-cli (pam_u2f link)"
as_user "$REPO/system/70-bitwarden.sh" --no-desktop --no-cli

# 60-nix.sh's GPU step, which looks for the setup tool in ~/.nix-profile:
# there is no profile yet, so run the generation's copy directly.
say "non-nixos-gpu-setup (/etc/tmpfiles.d/non-nixos-gpu.conf)"
gpu_setup="$gen/home-path/bin/non-nixos-gpu-setup"
[[ -x "$gpu_setup" ]] || die "no non-nixos-gpu-setup in the generation (targets.genericLinux.gpu?)"
"$gpu_setup"
gpu_paths="$(grep -o '/nix/store/[^ ]*' /etc/tmpfiles.d/non-nixos-gpu.conf | sort -u)"
# The conf itself is a link into the setup package's store path.
[[ -n "$gpu_paths" ]] || die "non-nixos-gpu.conf names no store path"

# The user flake's inputs (KubaOS at release and its own inputs, the user's
# nixpkgs) and the checkout's, for --inputs-from $KUBAOS.
say "flake inputs (for offline evaluation on the machine)"
flake_json="$(as_user "$NIX_BIN/nix" flake archive --json "path:$USER_FLAKE")"
flake_json+="$(as_user "$NIX_BIN/nix" flake archive --json "$KUBAOS")"
mapfile -t flake_paths < <(grep -o '/nix/store/[a-z0-9]\{32\}-[^"]*' <<<"$flake_json" | sort -u)
echo "    ${#flake_paths[@]} store paths"

say "packing tools from the flake's nixpkgs (not part of the payload)"
tools="$(as_user "$NIX_BIN/nix" build --no-link --print-out-paths --inputs-from "$KUBAOS" nixpkgs#rsync nixpkgs#squashfsTools 'nixpkgs#libjpeg_turbo^bin' | tr '\n' ' ')"
for t in $tools; do PATH="$t/bin:$PATH"; done
command -v rsync mksquashfs djpeg cjpeg >/dev/null

# The wallpaper sets (system/60-nix.sh linked them): Kōbō's
# Wallpapers screen lists them from the manifest, with a small JPEG of each
# set's first image next to it (the squashfs isn't mounted before the install).
WALLS=/usr/share/kubaos/wallpapers
say "wallpaper sets ($WALLS -> $(readlink "$WALLS" || echo missing))"
[[ -d "$WALLS/" ]] || die "no $WALLS (system/60-nix.sh, link_wallpapers)"
rm -rf /out/wallpapers
install -d /out/wallpapers
wallpaper_sets=""
for d in "$WALLS"/*/; do
  id="$(basename "$d")"
  [[ "$id" =~ ^[a-z0-9][a-z0-9-]*$ ]] || die "wallpaper set '$id': ids are a-z, 0-9 and -"
  # What dot-theme lists: images and videos, not the videos' posters.
  mapfile -t walls < <(find -L "$d" -maxdepth 1 -type f \( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' \
    -o -iname '*.webp' -o -iname '*.mp4' -o -iname '*.webm' -o -iname '*.mkv' -o -iname '*.mov' \) \
    ! -iname '*.poster.*' | LC_ALL=C sort)
  ((${#walls[@]})) || die "wallpaper set '$id' is empty"
  preview=null
  first="$(printf '%s\n' "${walls[@]}" | grep -iE '\.jpe?g$' | head -1 || true)"
  if [[ -n "$first" ]]; then
    # 3840 px wide originals -> 480 px tiles.
    djpeg -scale 1/8 "$first" | cjpeg -quality 85 -progressive -optimize >"/out/wallpapers/$id.jpg"
    preview="\"wallpapers/$id.jpg\""
  else
    echo "    $id: no JPEG to preview"
  fi
  wallpaper_sets+="${wallpaper_sets:+, }{\"id\": \"$id\", \"images\": ${#walls[@]}, \"preview\": $preview}"
  echo "    $id: ${#walls[@]} wallpapers"
done

# ---------------------------------------------------------------------------
# Roots of the payload's store: everything else in this cache store stays out.
roots=("$gen" "${flake_paths[@]}" "${rebuild_paths[@]}")
for p in $gpu_paths; do roots+=("$p"); done
declare -A gcroot_target=()
for l in /nix/var/nix/gcroots/dotfiles-*; do
  [[ -L "$l" ]] || continue
  gcroot_target["$(basename "$l")"]="$(readlink "$l")"
  roots+=("$(readlink "$l")")
done
# Every link the system side made into the store (/etc/tmpfiles.d/non-nixos-gpu.conf,
# /usr/local/lib/security/pam_*.so, ...) must resolve in the payload.
while IFS= read -r l; do
  t="$(readlink "$l")"
  roots+=("$(cut -d/ -f1-4 <<<"$t")")
done < <(find /etc /usr/local -xdev -type l -lname '/nix/store/*')
mapfile -t profile_links < <(find /nix/var/nix/profiles -maxdepth 1 -name 'default*' -type l)
for l in "${profile_links[@]}" /nix/var/nix/profiles/per-user/root/profile-*-link; do
  if [[ -L "$l" ]]; then roots+=("$(readlink -f "$l")"); fi
done

# Groups the build user ended up in (sudo, docker, i2c, ...): the engine
# gives the real user the same ones.
mapfile -t user_groups < <(id -nG "$PAYLOAD_USER" | tr ' ' '\n' | grep -vx "$PAYLOAD_USER")

# ---------------------------------------------------------------------------
# KubaOS (docs/BRAND.md): the names a person reads change, ID and
# VERSION_CODENAME stay Ubuntu's for apt and install scripts. The file is
# diverted, so a base-files upgrade puts Ubuntu's next to it as
# /usr/lib/os-release.ubuntu instead of over it.
say "os-release: $OS_NAME $OS_CODENAME_NAME"
if ! dpkg-divert --list /usr/lib/os-release | grep -q .; then
  dpkg-divert --local --rename --divert /usr/lib/os-release.ubuntu --add /usr/lib/os-release
fi
ubuntu_release=/usr/lib/os-release.ubuntu
ubuntu_get() { (. "$ubuntu_release" && eval "printf '%s' \"\${$1}\""); }
{
  printf 'NAME="%s"\n' "$OS_NAME"
  printf 'PRETTY_NAME="%s %s"\n' "$OS_NAME" "$OS_CODENAME_NAME"
  printf 'VERSION="%s (%s)"\n' "$OS_VERSION_SHORT" "$OS_CODENAME_NAME"
  printf 'VERSION_ID="%s"\n' "$(ubuntu_get VERSION_ID)"
  printf 'VERSION_CODENAME=%s\n' "$(ubuntu_get VERSION_CODENAME)"
  printf 'ID=%s\n' "$(ubuntu_get ID)"
  printf 'ID_LIKE=%s\n' "$(ubuntu_get ID_LIKE)"
  printf 'VARIANT="%s"\n' "$OS_CODENAME_NAME"
  printf 'VARIANT_ID=%s\n' "$OS_CODENAME"
  [[ -z "${OS_HOME_URL:-}" ]] || printf 'HOME_URL="%s"\n' "$OS_HOME_URL"
  printf 'UBUNTU_CODENAME=%s\n' "$(ubuntu_get UBUNTU_CODENAME)"
} >/usr/lib/os-release
chmod 0644 /usr/lib/os-release
[[ "$(readlink /etc/os-release)" == ../usr/lib/os-release ]] || ln -sfn ../usr/lib/os-release /etc/os-release
cat /usr/lib/os-release | sed 's/^/    /'
# GRUB's distributor, theme and timeout are system/25-boot.sh's
# (/etc/default/grub.d/10-kubaos.cfg). The EFI directory stays EFI/ubuntu (the
# engine's --bootloader-id): shim and the signed GRUB look for their files there.
[[ -f /etc/default/grub.d/10-kubaos.cfg ]] || die "no /etc/default/grub.d/10-kubaos.cfg (system/25-boot.sh didn't run?)"
# aarch64 is the VM payload (QEMU/UTM virt). There, with no console= the
# kernel picks the firmware's serial port (ACPI SPCR) as /dev/console, so the
# LUKS prompt only shows on serial, and the screen stays black until
# virtio_gpu loads from the root fs. Put the prompt on the screen: virtio_gpu
# in the initramfs, tty0 as the last (= prompting) console, serial kept for
# logs. Plymouth falls back to plain text whenever it sees a serial console,
# so tell it to leave serial alone and draw the splash on the screen.
if [[ "$ARCH" == aarch64 ]]; then
  printf '# KubaOS (install/image): LUKS prompt on the screen in a VM\nGRUB_CMDLINE_LINUX="$GRUB_CMDLINE_LINUX console=ttyAMA0 console=tty0 plymouth.ignore-serial-consoles"\n' \
    >/etc/default/grub.d/20-kubaos-console.cfg
  # grub-btrfs sources /etc/default/grub but not grub.d/, so the snapshot
  # entries need the same console arguments, or a snapshot boot asks for the
  # passphrase on serial only.
  btrfs_cfg=/etc/default/grub-btrfs/config
  sed -i '/^GRUB_BTRFS_SNAPSHOT_KERNEL_PARAMETERS=/d' "$btrfs_cfg"
  echo 'GRUB_BTRFS_SNAPSHOT_KERNEL_PARAMETERS="console=ttyAMA0 console=tty0 plymouth.ignore-serial-consoles"' >>"$btrfs_cfg"
  grep -qx virtio_gpu /etc/initramfs-tools/modules || echo virtio_gpu >>/etc/initramfs-tools/modules
fi

# ---------------------------------------------------------------------------
say "payload tree: / -> $ROOT (rsync, incremental)"
install -d "$ROOT"
# Excludes: mounts and runtime dirs (kept as empty dirs), the build user's
# home, build-only files. /nix is rebuilt below.
rsync -aHAX --numeric-ids --delete --delete-excluded -x \
  --exclude=/nix --exclude=/work --exclude=/in --exclude=/out \
  --exclude='/proc/*' --exclude='/sys/*' --exclude='/dev/*' --exclude='/run/*' \
  --exclude='/tmp/*' --exclude='/var/tmp/*' --exclude='/home/*' \
  --exclude=/opt/dotfiles --exclude=/.dockerenv \
  --exclude='/root/.cache' --exclude='/root/.local' \
  --exclude='/var/lib/apt/lists/*' --exclude='/var/cache/apt/*' \
  --exclude='/var/cache/debconf/*-old' --exclude='/var/lib/dpkg/*-old' \
  --exclude=/etc/hostname --exclude=/etc/hosts --exclude=/etc/resolv.conf \
  / "$ROOT/"
install -d -m 0755 "$ROOT"/{nix,proc,sys,dev,run,tmp,var/tmp,home,var/lib/apt/lists/partial,var/cache/apt/archives/partial}
chmod 1777 "$ROOT/tmp" "$ROOT/var/tmp"
# Logs of the build: keep the directories packages made, drop the files.
find "$ROOT/var/log" -type f -delete
chmod 0700 "$ROOT/var/lib/apt/lists/partial" "$ROOT/var/cache/apt/archives/partial" 2>/dev/null || true

say "payload /nix: closure of ${#roots[@]} roots"
rm -rf "$ROOT/nix"
install -d -m 0755 "$ROOT/nix"
nix_ copy --no-check-sigs --to "local?root=$ROOT" "${roots[@]}"
# State the installer made (profiles, the daemon's dirs), then GC roots.
install -d -m 0755 "$ROOT/nix/var/nix/profiles" "$ROOT/nix/var/nix/gcroots/kubaos" \
  "$ROOT/nix/var/nix/daemon-socket"
for l in "${profile_links[@]}"; do cp -a "$l" "$ROOT/nix/var/nix/profiles/"; done
# default -> per-user/root/profile -> profile-N-link -> store (the installer's chain)
if [[ -d /nix/var/nix/profiles/per-user/root ]]; then
  install -d -m 0755 "$ROOT/nix/var/nix/profiles/per-user/root"
  find /nix/var/nix/profiles/per-user/root -maxdepth 1 -name 'profile*' -type l \
    -exec cp -a {} "$ROOT/nix/var/nix/profiles/per-user/root/" \;
fi
[[ ! -L /nix/var/nix/gcroots/profiles ]] || cp -a /nix/var/nix/gcroots/profiles "$ROOT/nix/var/nix/gcroots/"
for name in "${!gcroot_target[@]}"; do ln -sfn "${gcroot_target[$name]}" "$ROOT/nix/var/nix/gcroots/$name"; done
ln -sfn "$gen" "$ROOT/nix/var/nix/gcroots/kubaos/home-generation"
i=0
for p in "${flake_paths[@]}"; do ln -sfn "$p" "$ROOT/nix/var/nix/gcroots/kubaos/flake-input-$((i++))"; done
i=0
for p in "${rebuild_paths[@]}"; do ln -sfn "$p" "$ROOT/nix/var/nix/gcroots/kubaos/rebuild-input-$((i++))"; done
i=0
for p in $gpu_paths; do ln -sfn "$p" "$ROOT/nix/var/nix/gcroots/kubaos/gpu-$((i++))"; done
chmod 1775 "$ROOT/nix/store"
chown root:nixbld "$ROOT/nix/store"

# ---------------------------------------------------------------------------
say "machine identity and the build user out"
userdel --root "$ROOT" "$PAYLOAD_USER"
groupdel --root "$ROOT" "$PAYLOAD_USER" 2>/dev/null || true
rm -f "$ROOT/etc/sudoers.d/kubaos-build" "$ROOT/var/mail/$PAYLOAD_USER" "$ROOT/var/spool/mail/$PAYLOAD_USER"
: >"$ROOT/etc/machine-id"
if [[ -e "$ROOT/var/lib/dbus/machine-id" && ! -L "$ROOT/var/lib/dbus/machine-id" ]]; then
  rm -f "$ROOT/var/lib/dbus/machine-id"
fi
rm -f "$ROOT"/etc/ssh/ssh_host_*
rm -f "$ROOT/usr/sbin/policy-rc.d" "$ROOT"/etc/apt/apt.conf.d/docker-* \
  "$ROOT/etc/apt/apt.conf.d/10kubaos-keep-cache" "$ROOT/etc/dpkg/dpkg.cfg.d/docker-apt-speedup"
rm -f "$ROOT/var/lib/systemd/random-seed" "$ROOT/var/lib/systemd/credential.secret"
rm -rf "$ROOT"/etc/*.dpkg-old "$ROOT"/etc/*- 2>/dev/null || true   # passwd-, shadow-, ...: backups with the build user
echo localhost >"$ROOT/etc/hostname"
cat >"$ROOT/etc/hosts" <<'EOF'
127.0.0.1 localhost

::1     ip6-localhost ip6-loopback
fe00::0 ip6-localnet
ff00::0 ip6-mcastprefix
ff02::1 ip6-allnodes
ff02::2 ip6-allrouters
EOF
ln -sfn ../run/systemd/resolve/stub-resolv.conf "$ROOT/etc/resolv.conf"
printf '# /etc/fstab: written by the KubaOS install.\n' >"$ROOT/etc/fstab"

# The official Nix installer only links the units when systemd runs; enable
# the socket (it starts the daemon on first use) and the service offline.
# Real files, not links into /nix: /nix is its own subvolume on the target,
# and systemd loads units before it mounts it, so a link there dangles at
# boot ("Unit nix-daemon.socket could not be found") and nothing listens.
say "nix-daemon units"
for u in nix-daemon.service nix-daemon.socket; do
  rm -f "$ROOT/etc/systemd/system/$u"
  install -m 0644 "/nix/var/nix/profiles/default/lib/systemd/system/$u" "$ROOT/etc/systemd/system/$u"
done
# What `systemctl enable` does with the installer's [Install] sections
# (systemctl --root refuses units that are themselves symlinks out of /etc).
grep -q '^WantedBy=sockets.target' "/nix/var/nix/profiles/default/lib/systemd/system/nix-daemon.socket" ||
  die "nix-daemon.socket no longer WantedBy=sockets.target; update assemble.sh"
install -d "$ROOT/etc/systemd/system/sockets.target.wants" "$ROOT/etc/systemd/system/multi-user.target.wants"
ln -sfn /etc/systemd/system/nix-daemon.socket "$ROOT/etc/systemd/system/sockets.target.wants/nix-daemon.socket"
if grep -q '^WantedBy=multi-user.target' "/nix/var/nix/profiles/default/lib/systemd/system/nix-daemon.service"; then
  ln -sfn /etc/systemd/system/nix-daemon.service "$ROOT/etc/systemd/system/multi-user.target.wants/nix-daemon.service"
fi
if [[ -f /nix/var/nix/profiles/default/lib/tmpfiles.d/nix-daemon.conf && ! -e "$ROOT/etc/tmpfiles.d/nix-daemon.conf" ]]; then
  ln -sfn /nix/var/nix/profiles/default/lib/tmpfiles.d/nix-daemon.conf "$ROOT/etc/tmpfiles.d/nix-daemon.conf"
fi
# ssh: installed for the test bench, off unless the engine turns it on.
systemctl --root="$ROOT" disable ssh.service ssh.socket 2>&1 | sed 's/^/    /' || true

# ---------------------------------------------------------------------------
say "install metadata ($SHARE)"
install -d -m 0755 "$ROOT/$SHARE"
# The engine writes it into the user flake, next to user-flake/ from $KUBAOS.
install -m 0644 "$USER_FLAKE/flake.lock" "$ROOT/$SHARE/user-flake.lock"
kernel="$(find "$ROOT/boot" -maxdepth 1 -name 'vmlinuz-*' -printf '%f\n' | sed 's/^vmlinuz-//' | sort -V | tail -1)"
[[ -n "$kernel" && -f "$ROOT/boot/initrd.img-$kernel" ]] || die "no kernel + initrd in /boot"
json_list() { local s="" x; for x in "$@"; do s+="${s:+, }\"$x\""; done; printf '[%s]' "$s"; }
cat >"$ROOT/$SHARE/image.json" <<EOF
{
  "version": "$VERSION",
  "build": "$BUILD_ID",
  "commit": "$COMMIT",
  "username": "$PAYLOAD_USER",
  "uid": 1000,
  "gid": 1000,
  "hostname": "$HOST",
  "home_generation": "$gen",
  "groups": $(json_list "${user_groups[@]}"),
  "kubaos": "$KUBAOS",
  "user_flake_lock": "/$SHARE/user-flake.lock",
  "snapper_root_config": "/$SHARE/snapper-root.conf",
  "kernel": "$kernel"
}
EOF
cat "$ROOT/$SHARE/image.json"

# ---------------------------------------------------------------------------
installed_bytes="$(du -s -B1 --apparent-size "$ROOT" | cut -f1)"
say "squashfs (zstd level $ZSTD_LEVEL), $((installed_bytes / 1024 / 1024)) MiB of files"
rm -f /out/*.squashfs /out/*.squashfs.tmp   # an older payload in the same dir
mksquashfs "$ROOT" "/out/$ROOTFS_NAME.tmp" -noappend -comp zstd -Xcompression-level "$ZSTD_LEVEL" \
  -b 1M -processors "$CPUS" -xattrs -no-progress -quiet
mv -f "/out/$ROOTFS_NAME.tmp" "/out/$ROOTFS_NAME"
rootfs_bytes="$(stat -c %s "/out/$ROOTFS_NAME")"
say "sha256"
sha="$(sha256sum "/out/$ROOTFS_NAME" | cut -d' ' -f1)"

cat >/out/manifest.json <<EOF
{
  "version": "$VERSION",
  "codename": "$OS_CODENAME_NAME",
  "arch": "$ARCH",
  "built_at": "$BUILT_AT",
  "commit": "$COMMIT",
  "username": "$PAYLOAD_USER",
  "hostname": "$HOST",
  "rootfs": "$ROOTFS_NAME",
  "rootfs_bytes": $rootfs_bytes,
  "installed_bytes": $installed_bytes,
  "sha256": "$sha",
  "kernel": "$kernel",
  "home_generation": "$gen",
  "wallpaper_sets": [$wallpaper_sets]
}
EOF
cat /out/manifest.json
say "done: $((rootfs_bytes / 1024 / 1024)) MiB archive, $((installed_bytes / 1024 / 1024)) MiB installed"
