#!/usr/bin/env bash
# Runs inside the privileged "live" container (see ../engine-e2e).
#
#   inside.sh prepare            rootfs tar on stdin -> /payload/{sqfs,tar,fail}
#   inside.sh run SCENARIO       install onto a loop disk, then check it
#
# Scenarios: sqfs (squashfs payload, snapper), tar (tar.zst payload without
# snapper: raw btrfs factory snapshot), fail (home step fails: cleanup),
# dual (free-space install next to a fake Windows, reusing its ESP),
# dual-fail (the same with a payload that fails to unpack: the free space
# comes back).
set -euo pipefail

BIN=/target/release/kubaos
GEN=/nix/store/0000000000000000000000000000test-home-manager-generation
PASS='correct horse battery'
USERPW='hunter22-e2e'
WIFIPSK='wifi-psk-e2e'
OUT=/out

say() { printf '\n==> %s\n' "$*"; }

manifest() { # DIR ROOTFS_FILE HOME_GENERATION
  local installed
  installed=$(cat /payload/installed_bytes)
  jq -n --arg rootfs "$2" --argjson bytes "$(stat -c %s "$1/$2")" --argjson installed "$installed" \
    --arg arch "$(uname -m)" --arg gen "$3" '{
      version: "e2e", arch: $arch, built_at: "2026-10-16T00:00:00Z", commit: "e2e",
      username: "kuba", hostname: "vm", rootfs: $rootfs, rootfs_bytes: $bytes,
      installed_bytes: $installed, sha256: "", kernel: "none", home_generation: $gen
    }' >"$1/manifest.json"
}

prepare() {
  rm -rf /payload/* && mkdir -p /payload/rootfs /payload/sqfs /payload/tar /payload/fail
  tar -x --numeric-owner -C /payload/rootfs
  rm -f /payload/rootfs/.dockerenv
  du -sb /payload/rootfs | cut -f1 >/payload/installed_bytes
  mksquashfs /payload/rootfs /payload/sqfs/rootfs.sqfs -comp zstd -quiet -no-progress
  tar --zstd -C /payload/rootfs --numeric-owner --xattrs --exclude=./usr/bin/snapper \
    -cf /payload/tar/rootfs.tar.zst .
  ln /payload/sqfs/rootfs.sqfs /payload/fail/rootfs.sqfs
  manifest /payload/sqfs rootfs.sqfs "$GEN"
  manifest /payload/tar rootfs.tar.zst "$GEN"
  manifest /payload/fail rootfs.sqfs /nix/store/missing-home-manager-generation
  rm -rf /payload/rootfs
  ls -la /payload/*/
}

# -- checks -----------------------------------------------------------------
results=()
fails=0
check() { # NAME CMD...
  local name="$1"
  shift
  if "$@" >/tmp/check.out 2>&1; then
    results+=("PASS  $name")
  else
    results+=("FAIL  $name: $(head -c 300 /tmp/check.out | tr '\n' ' ')")
    fails=$((fails + 1))
  fi
}
has() { grep -qE -- "$2" "$1" || { echo "no /$2/ in $1:"; head -20 "$1"; return 1; }; }
eq() { [[ "$1" == "$2" ]] || { echo "'$1' != '$2'"; return 1; }; }

# The shadow hash of kuba verifies against the password we passed.
password_matches() {
  local h
  h=$(awk -F: '$1 == "kuba" {print $2}' "$1/etc/shadow")
  [[ "$(perl -e 'print crypt($ARGV[0], $ARGV[1])' "$USERPW" "$h")" == "$h" ]]
}

# ESP with Windows' loaders, a "C:" partition, a recovery partition at the
# end, and free space in between (like a shrunk Windows install).
windows_like_disk() {
  local efi=AA64
  [[ "$(uname -m)" == x86_64 ]] && efi=X64
  sgdisk -n1:2048:+260M -t1:ef00 -c1:'EFI system partition' \
    -n2:0:+3G -t2:0700 -c2:'Basic data partition' \
    -n3:-600M:0 -t3:2700 "$LOOP" >/dev/null
  partprobe "$LOOP" 2>/dev/null || true
  sleep 1
  mkfs.vfat -F 32 "${LOOP}p1" >/dev/null
  mkfs.ext4 -q -F "${LOOP}p2"
  mkdir -p /mnt/win
  mount "${LOOP}p1" /mnt/win
  mkdir -p /mnt/win/EFI/Microsoft/Boot /mnt/win/EFI/BOOT
  echo windows >/mnt/win/EFI/Microsoft/Boot/bootmgfw.efi
  printf windows >"/mnt/win/EFI/BOOT/BOOT$efi.EFI"
  umount /mnt/win
  mount "${LOOP}p2" /mnt/win
  echo 'C: drive' >/mnt/win/marker
  umount /mnt/win
}

LOOP=''
cleanup() {
  set +e
  for m in /mnt/check/esp /mnt/check/boot /mnt/check/top; do mountpoint -q "$m" && umount "$m"; done
  [[ -e /dev/mapper/checkroot ]] && cryptsetup close checkroot
  if findmnt /mnt/kubaos >/dev/null 2>&1; then umount -R -l /mnt/kubaos; fi
  [[ -e /dev/mapper/cryptroot ]] && cryptsetup close cryptroot
  [[ -n "$LOOP" ]] && losetup -d "$LOOP"
}

run_scenario() {
  local sc="$1"
  # A real devtmpfs, so partition nodes of the loop device show up.
  mount -t devtmpfs devtmpfs /dev
  mkdir -p /dev/pts /dev/shm
  mount -t devpts -o newinstance,ptmxmode=0666 devpts /dev/pts
  mount -t tmpfs tmpfs /dev/shm
  trap cleanup EXIT

  truncate -s 12G /work/disk.img
  LOOP=$(losetup -fP --show /work/disk.img)
  say "scenario $sc on $LOOP"
  local payload="/payload/$sc" target='{ mode = "whole" }' host=vm region=''
  if [[ "$sc" == dual || "$sc" == dual-fail ]]; then
    payload=/payload/sqfs
    # A hostname of the owner's own, not the payload's: the payload's home
    # generation still activates, with nothing built.
    host=nerv-hq
    windows_like_disk
    # Kōbō's way: take a free region from probe.
    region=$(KUBAOS_TEST=1 "$BIN" probe --json --payload /payload/sqfs |
      jq -c --arg d "$LOOP" '.disks[] | select(.path == $d) | .free_regions[0]')
    echo "free region from probe: $region"
    target=$(jq -r '"{ mode = \"free_space\", start = \(.start), end = \(.end) }"' <<<"$region")
  fi
  if [[ "$sc" == dual-fail ]]; then
    # Half a squashfs, with a manifest that says so: preflight passes and
    # unsquashfs fails.
    payload=/work/broken
    mkdir -p "$payload"
    head -c "$(($(stat -c %s /payload/sqfs/rootfs.sqfs) / 2))" /payload/sqfs/rootfs.sqfs >"$payload/rootfs.sqfs"
    manifest "$payload" rootfs.sqfs "$GEN"
  fi
  cat >/work/install.toml <<EOF
[locale]
lang = "en_US.UTF-8"
keyboard_layout = ["us", "it"]
keyboard_variant = ["", "intl"]
timezone = "Europe/Rome"

[host]
hostname = "$host"

[user]
username = "kuba"
full_name = "Kuba Example"

[disk]
device = "$LOOP"
encrypt = true
target = $target

[git]
name = "Kuba Example"
email = "kuba@example.com"

[wifi]
ssid = "Home 5G"

[payload]
dir = "$payload"
EOF
  export KUBAOS_TEST=1
  check "validate" "$BIN" validate --config /work/install.toml

  say "kubaos run --events json"
  local ev="$OUT/$sc.events" rc st
  rm -f "$OUT/$sc".*
  SECONDS=0
  # Timestamp every event line as it arrives, to check the progress rate.
  set +e
  printf '{"luks_passphrase":"%s","user_password":"%s","wifi_psk":"%s"}' "$PASS" "$USERPW" "$WIFIPSK" |
    "$BIN" run --config /work/install.toml --secrets-stdin --events json --log "$OUT/$sc.live.log" \
      2>"$OUT/$sc.stderr" |
    while IFS= read -r line; do printf '%s %s\n' "$(date +%s.%N)" "$line"; done >"$ev.ts"
  st=("${PIPESTATUS[@]}")
  set -e
  rc=${st[1]}
  cut -d' ' -f2- "$ev.ts" >"$ev.jsonl"
  printf 'run took %d s, exit %s, %s events\n' "$SECONDS" "$rc" "$(wc -l <"$ev.jsonl")"
  tail -5 "$OUT/$sc.stderr"

  check "stdout is only JSON events" jq -e -s 'all(.[]; has("type"))' "$ev.jsonl"
  check "first event is plan" eq "$(head -1 "$ev.jsonl" | jq -r .type)" plan
  check "no secret in events, stderr or log" bash -c \
    "! grep -qE 'correct horse|hunter22|wifi-psk-e2e' '$ev.jsonl' '$OUT/$sc.stderr' '$OUT/$sc.live.log'"
  check "nothing left mounted" bash -c '! findmnt -R /mnt/kubaos'
  check "LUKS mapping closed" bash -c '[[ ! -e /dev/mapper/cryptroot ]]'
  check "no nix-daemon left" bash -c '! pgrep -x socat'

  if [[ "$sc" == fail ]]; then
    check "exit code 1" eq "$rc" 1
    check "last event is failed at home" eq "$(tail -1 "$ev.jsonl" | jq -r '.type + ":" + .id')" failed:home
    return
  fi
  if [[ "$sc" == dual-fail ]]; then
    check "exit code 1" eq "$rc" 1
    check "last event is failed at unpack" eq "$(tail -1 "$ev.jsonl" | jq -r '.type + ":" + .id')" failed:unpack
    sgdisk -p "$LOOP" >/work/gpt
    check "GPT: only Windows' three partitions" bash -c "[[ \$(grep -cE '^ +[0-9]+ ' /work/gpt) == 3 ]] && grep -E '^ +2 .* 0700 ' /work/gpt && grep -E '^ +3 .* 2700 ' /work/gpt"
    check "the free region is free again" eq "$(KUBAOS_TEST=1 "$BIN" probe --json --payload /payload/sqfs |
      jq -c --arg d "$LOOP" '.disks[] | select(.path == $d) | .free_regions[0]')" "$region"
    return
  fi

  check "exit code 0" eq "$rc" 0
  check "last event is done" eq "$(tail -1 "$ev.jsonl" | jq -r .type)" done
  check "every planned step started and finished in order" bash -c "
    plan=\$(head -1 '$ev.jsonl' | jq -r '[.steps[].id] | join(\" \")')
    started=\$(jq -r 'select(.type==\"step_started\") | .id' '$ev.jsonl' | xargs)
    finished=\$(jq -r 'select(.type==\"step_finished\") | .id' '$ev.jsonl' | xargs)
    [[ \"\$plan\" == \"\$started\" && \"\$plan\" == \"\$finished\" ]] || { echo \"\$plan / \$started / \$finished\"; exit 1; }"
  check "unpack sends progress at least twice a second" awk '
    { t = $1; $1 = ""; }
    /"step_started","id":"unpack"/ { on = 1; last = t; next }
    on && /"step_finished"/ { on = 0; if (gap > 0.5 || n < 2) { print "max gap " gap " s, " n " progress events"; exit 1 } exit 0 }
    on { if (t - last > gap) gap = t - last; last = t; if ($0 ~ /"progress"/) n++ }
    END { if (on) exit 1 }' "$ev.ts"

  say "re-open the disk and look"
  local P1 P2 P3 nb=2 nr=3
  [[ "$sc" == dual ]] && nb=4 nr=5
  P1="${LOOP}p1" P2="${LOOP}p$nb" P3="${LOOP}p$nr"
  sgdisk -p "$LOOP" >/work/gpt
  check "GPT: ESP 1, boot $nb, LUKS root $nr" bash -c "grep -E '^ +1 .* EF00 ' /work/gpt && grep -E '^ +$nb .* 8300 +boot' /work/gpt && grep -E '^ +$nr .* 8309 +root' /work/gpt"
  if [[ "$sc" == dual ]]; then
    check "Windows partitions untouched" bash -c "grep -E '^ +2 .* 0700 ' /work/gpt && grep -E '^ +3 .* 2700 ' /work/gpt && mount -o ro ${LOOP}p2 /mnt/win && grep -q 'C: drive' /mnt/win/marker; r=\$?; umount /mnt/win; exit \$r"
  fi
  check "LUKS2 with argon2id" bash -c "cryptsetup luksDump $P3 | grep -q 'Version:.*2' && cryptsetup luksDump $P3 | grep -q 'PBKDF:.*argon2id'"
  printf '%s' "$PASS" | cryptsetup open --key-file=- "$P3" checkroot
  mkdir -p /mnt/check/top /mnt/check/boot /mnt/check/esp
  mount -o subvolid=5 /dev/mapper/checkroot /mnt/check/top
  mount "$P2" /mnt/check/boot
  mount "$P1" /mnt/check/esp
  local T=/mnt/check/top R=/mnt/check/top/@
  btrfs subvolume list "$T" >/work/subvols
  for sv in @ @home @snapshots @log @cache @nix @docker @swap @snapshots/1/snapshot; do
    check "subvolume $sv" has /work/subvols " path $sv\$"
  done
  check "default subvolume is @" has <(btrfs subvolume get-default "$T") 'path @$'
  check "factory snapshot is read-only" eq "$(btrfs property get "$T/@snapshots/1/snapshot" ro)" ro=true
  check "factory snapshot info.xml" has "$T/@snapshots/1/info.xml" '<description>factory</description>'
  check "snapshot holds the system" test -f "$T/@snapshots/1/snapshot/etc/fstab"
  if [[ "$sc" == sqfs ]]; then
    check "snapper config root (ALLOW_USERS=kuba)" has "$R/etc/snapper/configs/root" '^ALLOW_USERS="kuba"'
  fi

  local U_ROOT U_BOOT U_ESP U_LUKS
  U_ROOT=$(blkid -c /dev/null -s UUID -o value /dev/mapper/checkroot)
  U_BOOT=$(blkid -c /dev/null -s UUID -o value "$P2")
  U_ESP=$(blkid -c /dev/null -s UUID -o value "$P1")
  U_LUKS=$(blkid -c /dev/null -s UUID -o value "$P3")
  for mp_sv in /:@ /home:@home /.snapshots:@snapshots /var/log:@log /var/cache:@cache /nix:@nix /var/lib/docker:@docker; do
    check "fstab ${mp_sv%%:*}" has "$R/etc/fstab" "^UUID=$U_ROOT +${mp_sv%%:*} +btrfs +noatime,compress=zstd:1,subvol=${mp_sv#*:} "
  done
  check "fstab /swap" has "$R/etc/fstab" "^UUID=$U_ROOT +/swap +btrfs +noatime,subvol=@swap "
  check "fstab /boot" has "$R/etc/fstab" "^UUID=$U_BOOT +/boot +ext4 "
  check "fstab /boot/efi" has "$R/etc/fstab" "^UUID=$U_ESP +/boot/efi +vfat "
  check "crypttab" has "$R/etc/crypttab" "^cryptroot UUID=$U_LUKS none luks,discard,tries=0$"
  check "hostname $host" eq "$(cat "$R/etc/hostname")" "$host"
  check "hosts" has "$R/etc/hosts" "^127.0.1.1"$'\t'"$host\$"
  check "timezone" eq "$(readlink "$R/etc/localtime")" ../usr/share/zoneinfo/Europe/Rome
  check "locale" has "$R/etc/locale.conf" '^LANG=en_US.UTF-8$'
  check "/etc/default/locale -> ../locale.conf" eq "$(readlink "$R/etc/default/locale")" ../locale.conf
  check "locale generated" bash -c "chroot $R locale -a | grep -qi en_US.utf8"
  check "keyboard layouts, the first the default" has "$R/etc/default/keyboard" '^XKBLAYOUT="us,it"$'
  check "keyboard variants" has "$R/etc/default/keyboard" '^XKBVARIANT=",intl"$'
  check "machine-id is new" has "$R/etc/machine-id" '^[0-9a-f]{32}$'
  check "user kuba: uid 1000, zsh" has "$R/etc/passwd" '^kuba:x:1000:1000:Kuba Example:/home/kuba:/usr/bin/zsh$'
  check "kuba in sudo" has "$R/etc/group" '^sudo:x:[0-9]+:.*kuba'
  check "password set (hashed)" has "$R/etc/shadow" '^kuba:\$[a-z0-9]+\$'
  check "password is the one given" password_matches "$R"
  check "home on @home" test -d "$T/@home/kuba"
  check "home activated as kuba via nix-daemon" has "$T/@home/kuba/.hm-activated" 'user=kuba uid=1000 .*nix_remote=daemon backup=hm-backup'
  local uf="$T/@home/kuba/.config/kubaos/home"
  check "user flake for kuba@$host" has "$uf/flake.nix" "^homeConfigurations.\"kuba@$host\" = mkHome \"$(uname -m)-linux\";\$"
  check "  the payload's generation, nothing built" bash -c "! grep -q 'building the home-manager generation' '$OUT/$sc.live.log'"
  check "  with the payload's lock" has "$uf/flake.lock" '^\{"version": 7\}$'
  check "  owned by kuba" eq "$(stat -c %u:%g "$uf" "$uf/flake.nix" "$uf/home.nix" "$uf/flake.lock" | sort -u)" 1000:1000
  check "no ~/Projects/dotfiles" test ! -e "$T/@home/kuba/Projects/dotfiles"
  local nm="$R/etc/NetworkManager/system-connections/Home_5G.nmconnection"
  check "Wi-Fi keyfile root:root 0600" eq "$(stat -c %u:%g:%a "$nm")" 0:0:600
  check "Wi-Fi keyfile has ssid and psk" bash -c "grep -qx 'ssid=Home 5G' '$nm' && grep -qx 'psk=$WIFIPSK' '$nm'"
  local gid="$T/@home/kuba/.config/git-identity/identity.gitconfig"
  check "git identity in ~/.config/git-identity" has "$gid" 'email = "kuba@example.com"'
  check "  owned by kuba down from ~/.config" eq "$(stat -c %u:%g "$T/@home/kuba/.config" "$T/@home/kuba/.config/git-identity" "$gid" | sort -u)" 1000:1000
  check "grub.cfg on /boot" test -s /mnt/check/boot/grub/grub.cfg
  local efi=aa64
  [[ "$(uname -m)" == x86_64 ]] && efi=x64
  check "EFI/ubuntu/grub$efi.efi" test -f "/mnt/check/esp/EFI/ubuntu/grub$efi.efi"
  if [[ "$sc" == dual ]]; then
    check "Windows' fallback loader kept" eq "$(cat "/mnt/check/esp/EFI/BOOT/BOOT${efi^^}.EFI")" windows
    check "Windows' boot manager kept" test -f /mnt/check/esp/EFI/Microsoft/Boot/bootmgfw.efi
    check "os-prober on for dual boot" has "$R/etc/default/grub.d/90-kubaos-os-prober.cfg" '^GRUB_DISABLE_OS_PROBER=false$'
  else
    check "EFI/BOOT/BOOT${efi^^}.EFI (fallback)" test -f "/mnt/check/esp/EFI/BOOT/BOOT${efi^^}.EFI"
  fi
  check "install log copied (0600)" eq "$(stat -c %a "$T/@log/kubaos/install.log")" 600
  check "install log has no secret" bash -c "! grep -qE 'correct horse|hunter22|wifi-psk-e2e' '$T/@log/kubaos/install.log'"
}

report() {
  say "results"
  printf '%s\n' "${results[@]}"
  printf '\n%d checks, %d failed\n' "${#results[@]}" "$fails"
  ((fails == 0))
}

case "${1:-}" in
  prepare) prepare ;;
  run)
    mkdir -p /work "$OUT"
    run_scenario "$2"
    report
    ;;
  *)
    echo "usage: $0 prepare | run sqfs|tar|fail|dual|dual-fail" >&2
    exit 2
    ;;
esac
