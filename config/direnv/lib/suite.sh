# shellcheck shell=bash
# `use suite NAME VERSION` in an .envrc: a Project pin. That version of the
# Suite's toolchain goes on PATH, ahead of the installed one, in the folder and
# every folder below it that direnv loads the file for (a nested .envrc needs
# `source_up` to keep it). Leaving the folder gives the installed one back.
# `kubaos pin` writes the line and `kubaos unpin` removes it.
#
# home/pins.nix writes an entry for every version of every Suite, installed or
# not, at ~/.local/share/kubaos/toolchains/NAME/VERSION: the derivation, as
# `drv!output`, then the output path. The first load of a pin realises it, from
# the binary cache or by building it, and keeps it with a GC root in the
# folder's .direnv/suites/NAME/, as nix-direnv does for `use flake`; the
# Suite's other roots there go, so a re-pinned version can be collected. Every
# load after that is a file read and a readlink.
use_suite() {
  if (($# != 2)); then
    log_error "usage: use suite NAME VERSION"
    return 2
  fi
  local name=$1 version=$2
  if [[ $name == */* || $name == .* || $version == */* || $version == .* ]]; then
    log_error "kubaos: $name $version is not a Suite and version"
    return 1
  fi
  local toolchains="${XDG_DATA_HOME:-$HOME/.local/share}/kubaos/toolchains"
  local entry="$toolchains/$name/$version"
  # Watched before it is read, missing or not: a switch that adds or relinks it
  # (a new Suite, a KubaOS update moving the version to a new build) reloads
  # every shell already in the folder.
  watch_file "$entry"
  if [[ ! -f $entry ]]; then
    _kubaos_suite_missing "$toolchains" "$name" "$version"
    return 1
  fi

  local drv out
  if ! { IFS= read -r drv && IFS= read -r out; } <"$entry" || [[ -z $drv || -z $out || $drv != *'!'* ]]; then
    log_error "kubaos: $entry is not a toolchain entry; run \`kubaos switch\`"
    return 1
  fi
  local roots base root
  roots="$(direnv_layout_dir)/suites/$name"
  base="$roots/$version"
  root=$base
  # HAZARD: nix-store --add-root names the link BASE-OUTPUT for any output but
  # `out`. Comparing against the bare name would never match a `bin` output,
  # and every load would realise again.
  [[ ${drv##*!} == out ]] || root="$base-${drv##*!}"
  if [[ ! -e $out || $(readlink -- "$root" 2>/dev/null) != "$out" ]]; then
    [[ -e $out ]] || log_status "fetching $name $version into the Nix store, once"
    local nix_store=nix-store
    # The harness's shells and some login shells don't put Nix's own bin on
    # PATH; the daemon install always has it here.
    has nix-store || nix_store=/nix/var/nix/profiles/default/bin/nix-store
    mkdir -p -- "$roots"
    if ! "$nix_store" --realise "$drv" --add-root "$base" >/dev/null; then
      log_error "kubaos: could not fetch $name $version ($drv)"
      return 1
    fi
    local other
    for other in "$roots"/*; do
      [[ $other == "$root" || ! -L $other ]] || rm -f -- "$other"
    done
  fi
  PATH_add "$out/bin"
}

# _kubaos_suite_missing TOOLCHAINS NAME VERSION: says why NAME VERSION has no
# toolchain entry.
_kubaos_suite_missing() {
  local toolchains=$1 name=$2 version=$3
  if [[ -d $toolchains/$name ]]; then
    local versions=("$toolchains/$name"/*) list
    printf -v list '%s, ' "${versions[@]##*/}"
    log_error "kubaos: the $name Suite has no version $version (it has: ${list%, })"
  else
    log_error "kubaos: no Suite called $name in this home (\`kubaos suite list\`; after a KubaOS update, \`kubaos switch\`)"
  fi
}
