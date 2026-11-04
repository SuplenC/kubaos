#!/usr/bin/env bash
# kubaos-depth-queue: the wallpaper-depth Suite's job for a wallpaper. It runs
# kubaos-depth on the picture in a transient systemd user unit, at the lowest
# priority and once on AC power, into the picture's folder, which the lock
# reads. `scripts/dot-theme` calls it when a still of the user's own is set,
# and again at login; home/suites/wallpaper-depth.nix installs it.
#
#   kubaos-depth-queue PICTURE         queue PICTURE (an absolute path), unless
#                                      its folder is up to date or the settings
#                                      give it no cut-out
#   kubaos-depth-queue redo PICTURE    stop its unit, empty its folder and queue
#                                      it again
#   kubaos-depth-queue forget PICTURE  stop its unit and delete its folder
#   kubaos-depth-queue stop            stop every picture's unit
#   kubaos-depth-queue job PICTURE     the unit's command: wait for AC power, run
#                                      the tool, record what the folder was made from
#
# PICTURE's folder is ~/.cache/kubaos/wallpapers/<md5 of its path>/, and its
# unit kubaos-depth-<md5 of its path>.service, so a picture whose unit is
# running or waiting is never queued twice. The folder is up to date when its
# `source` holds the picture's size and mtime and this version of the tool;
# a failed run writes it too, so a picture the tool fails on isn't tried
# again at every login. A folder made from an older picture at the same path
# is emptied at once, so the lock never draws a stale cut-out over a new one.
# Before it queues a picture it asks `kubaos depth wants PICTURE`, which says
# no while depth is off, for a picture in the skip list
# (~/.config/kubaos/desktop/wallpapers/depth.toml), for a video and for one of
# KubaOS's sets; `kubaos depth` runs the other commands when those settings
# change.
# The contract is docs/CONTRACTS.md, "Wallpaper depth".
#
# The Nix module sets KUBAOS_DEPTH_TOOL (the kubaos-depth binary),
# KUBAOS_DEPTH_VERSION (its version) and KUBAOS_DEPTH_MODELS (the models'
# folder).
set -euo pipefail

: "${KUBAOS_DEPTH_TOOL:?KUBAOS_DEPTH_TOOL is not set}"
: "${KUBAOS_DEPTH_VERSION:?KUBAOS_DEPTH_VERSION is not set}"
: "${KUBAOS_DEPTH_MODELS:?KUBAOS_DEPTH_MODELS is not set}"

# The catalog's cache for the Suite (crates/kubaos-core/src/suites/catalog.rs),
# which `kubaos suite remove --purge-caches` deletes.
RESULTS="$HOME/.cache/kubaos/wallpapers"
# How often a job waiting on battery looks for AC power again.
AC_POLL="${KUBAOS_DEPTH_AC_POLL:-60}"

usage() {
  echo "usage: kubaos-depth-queue PICTURE | redo PICTURE | forget PICTURE | stop | job PICTURE" >&2
  exit 2
}

# The md5 of the path, as Qt.md5 in QML gives it, so the lock finds the
# folder from the wallpaper's path alone.
key_of() { printf '%s' "$1" | md5sum | cut -d' ' -f1; }

# What a folder is made from: the picture's size and mtime, and the tool.
source_of() { printf '%s %s\n' "$(stat -L -c '%s %Y' -- "$1")" "$KUBAOS_DEPTH_VERSION"; }

# Whether the folder was made from the picture as it is now, by any version.
same_picture() {
  local dir=$1 picture=$2 size mtime
  read -r size mtime _ <"$dir/source" || return 1
  [[ "$size $mtime" == "$(stat -L -c '%s %Y' -- "$picture")" ]]
}

# On AC power, or on a machine with no battery: systemd's own test, which
# ConditionACPower= uses. Without it, the job runs.
on_ac() {
  command -v systemd-ac-power >/dev/null || return 0
  systemd-ac-power
}

# Whether the settings give the picture a cut-out: `kubaos depth wants` exits
# 1 when they don't. Without kubaos, or when it fails otherwise, they are the
# defaults, which do.
wanted() {
  command -v kubaos >/dev/null || return 0
  local rc=0
  kubaos depth wants "$1" >/dev/null || rc=$?
  ((rc != 1))
}

unit_of() { printf 'kubaos-depth-%s.service\n' "$(key_of "$1")"; }

# Stops the picture's unit, waiting or running, and waits for it to end, so
# nothing writes into its folder after.
stop_unit() { systemctl --user stop --quiet "$(unit_of "$1")" 2>/dev/null || true; }

queue() {
  local picture=$1 key dir unit
  wanted "$picture" || return 0
  key=$(key_of "$picture")
  dir="$RESULTS/$key"
  unit="kubaos-depth-$key.service"
  if [[ -f $dir/source ]]; then
    [[ $(<"$dir/source") == "$(source_of "$picture")" ]] && return 0
    same_picture "$dir" "$picture" || rm -f -- "$dir/cutout.png" "$dir/depth.toml" "$dir/source"
  fi
  systemctl --user is-active --quiet "$unit" && return 0
  # The weights make the job wait on anything else that wants the CPU or the
  # disk; the tool also lowers its own niceness to 19. --collect unloads a
  # failed unit too, so the next queue of the picture can start it again.
  systemd-run --user --quiet --collect --unit="$unit" --slice=background.slice \
    --description="Wallpaper depth for ${picture##*/}" \
    -p CPUWeight=idle -p IOWeight=1 -p Nice=19 -p IOSchedulingClass=idle \
    -- "$(realpath -- "${BASH_SOURCE[0]}")" job "$picture"
}

job() {
  local picture=$1 dir made rc=0
  dir="$RESULTS/$(key_of "$picture")"
  if ! on_ac; then
    echo "waiting for AC power"
    until on_ac; do sleep "$AC_POLL"; done
  fi
  [[ -f $picture ]] || { echo "kubaos-depth-queue: $picture is gone" >&2; exit 1; }
  # Taken before the run: a picture replaced during it is stale at once.
  made=$(source_of "$picture")
  mkdir -p -- "$dir"
  "$KUBAOS_DEPTH_TOOL" run "$picture" --out "$dir" --models "$KUBAOS_DEPTH_MODELS" || rc=$?
  printf '%s\n' "$made" >"$dir/.source.partial"
  mv -f -- "$dir/.source.partial" "$dir/source"
  exit "$rc"
}

redo() {
  [[ -f $1 ]] || { echo "kubaos-depth-queue: no such file: $1" >&2; exit 1; }
  stop_unit "$1"
  rm -f -- "$RESULTS/$(key_of "$1")"/{cutout.png,depth.toml,source}
  queue "$1"
}

forget() {
  stop_unit "$1"
  rm -rf -- "${RESULTS:?}/$(key_of "$1")"
}

case ${1:-} in
  redo | forget)
    (($# == 2)) && [[ $2 == /* ]] || usage
    "$1" "$2"
    ;;
  stop)
    (($# == 1)) || usage
    systemctl --user stop --quiet 'kubaos-depth-*.service'
    ;;
  job)
    (($# == 2)) && [[ $2 == /* ]] || usage
    job "$2"
    ;;
  /*)
    (($# == 1)) || usage
    [[ -f $1 ]] || { echo "kubaos-depth-queue: no such file: $1" >&2; exit 1; }
    queue "$1"
    ;;
  *) usage ;;
esac
