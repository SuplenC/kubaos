# shellcheck shell=bash
# Helpers for migrations (migrations/system, migrations/user), which source
# this file. kubaos update runs each migration under bash -euo pipefail with
# KUBAOS_DIR set to the release's checkout; a user migration also gets
# KUBAOS_USER_FLAKE and KUBAOS_BACKUPS.

# kubaos_backup FILE: copies FILE, when it exists, to the backups folder under
# its own path and the time, before a migration changes it. A user's backups
# go to KUBAOS_BACKUPS, outside the user flake so a backup never becomes part
# of the flake's source; a system migration's to /var/backups/kubaos.
kubaos_backup() {
  local file="$1" dest
  [[ -e "$file" ]] || return 0
  dest="${KUBAOS_BACKUPS:-/var/backups/kubaos}/$(date +%Y%m%d-%H%M%S)$(realpath -- "$file")"
  mkdir -p -- "$(dirname -- "$dest")"
  cp -a -- "$file" "$dest"
  echo "backed up $file to $dest"
}

# kubaos_suite_to_mise NAME: moves the installed Suite NAME onto mise, for the
# release that moves its toolchain there. The switch before the migration
# has taken the Suite's Nix packages away already; `kubaos suite configure`
# with no options installs its mise tools at the version and extras its
# record names, then writes its fragment, leaving the record as it is.
# Without a record there is nothing to move. A second run finds the fragment
# as it wants it and installs nothing.
kubaos_suite_to_mise() {
  local suite="$1"
  [[ -e "$KUBAOS_USER_FLAKE/kubaos.d/suites/$suite.toml" ]] || return 0
  kubaos suite configure "$suite" --no-switch
}
