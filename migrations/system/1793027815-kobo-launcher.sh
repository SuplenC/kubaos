#!/usr/bin/env bash
# kobo-launcher: /usr/bin/kubaos-setup and the first-boot unit become the ones
# a new install writes (install/firstboot/), which run the kobo each switch
# puts in the user's home and fall back to the install's /usr/bin/kobo. A
# machine installed before them runs /usr/bin/kobo alone, which no update
# would ever replace, so its setup never gets a new card. A machine set up
# without Kōbō has neither file, and gets neither.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

# replace SRC DEST MODE: DEST becomes a copy of SRC, when DEST exists and differs.
replace() {
  local src=$1 dest=$2 mode=$3
  if [[ ! -e $dest ]] || cmp -s "$src" "$dest"; then
    return 0
  fi
  kubaos_backup "$dest"
  # Through a rename, so a setup starting meanwhile reads the old file or the new one.
  install -m "$mode" "$src" "$dest.tmp"
  mv -f "$dest.tmp" "$dest"
  echo "$dest now runs your home's kobo (/usr/bin/kobo without one)"
}

replace "$KUBAOS_DIR/install/firstboot/kubaos-setup" /usr/bin/kubaos-setup 0755
replace "$KUBAOS_DIR/install/firstboot/kubaos-setup.service" /usr/lib/systemd/user/kubaos-setup.service 0644
