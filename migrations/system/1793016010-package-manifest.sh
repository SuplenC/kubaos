#!/usr/bin/env bash
# package-manifest: installs what the groups of system/packages.toml want on
# this machine and it lacks, vendor repos included, and starts the record of
# what KubaOS installed (/var/lib/kubaos/apt-installed). The kubaos of this
# release does that on every update; this covers the update that brings the
# manifest, which the release before runs.
#
# First it records the choices this machine's install made before they were
# kept: an optional group that is off by default and fully installed (PIV
# from --piv, TUXEDO's drivers from first-boot setup) is on, and one that is
# on by default and wholly absent (--no-docker, --no-chrome, --no-u2f) is
# off, so nothing the install left out comes back.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

mapfile -t groups < <(manifest_groups)
((${#groups[@]})) || { echo "no groups in $APT_MANIFEST" >&2; exit 1; }

for g in "${groups[@]}"; do
  default="$(manifest_value "group.$g" optional)"
  only="$(manifest_value "group.$g" arch)"
  [[ -n "$default" && -z "$(group_choice "$g")" ]] || continue
  [[ -z "$only" || "$only" == "$(arch)" ]] || continue
  have=0 lack=0
  while IFS= read -r p; do
    if pkg_installed "$p"; then have=$((have + 1)); else lack=$((lack + 1)); fi
  done < <(manifest_value "group.$g" packages)
  if [[ "$default" == off ]] && ((lack == 0)); then
    choice_set "$g" on
  elif [[ "$default" == on ]] && ((have == 0)); then
    choice_set "$g" off
  fi
done

group_install "${groups[@]}"
