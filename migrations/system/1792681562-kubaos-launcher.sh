#!/usr/bin/env bash
# kubaos-launcher: /usr/bin/kubaos becomes the launcher a new install writes
# (install/firstboot/kubaos-launcher), which runs the kubaos each switch puts
# in the user's home. A machine installed before it has the binary of its
# install there, which no update would ever replace. That binary moves to
# /usr/lib/kubaos/kubaos, where the launcher falls back to it.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

launcher="$KUBAOS_DIR/install/firstboot/kubaos-launcher"
bin=/usr/bin/kubaos
installed=/usr/lib/kubaos/kubaos

if cmp -s "$launcher" "$bin"; then
  exit 0
fi
if [[ -f $bin && $(head -c 2 "$bin") == '#!' ]]; then
  # A script someone put there, kept in the backups.
  kubaos_backup "$bin"
elif [[ -f $bin && ! -e $installed ]]; then
  install -d -m 0755 "$(dirname "$installed")"
  cp -a "$bin" "$installed.tmp"
  mv -f "$installed.tmp" "$installed"
fi
# Through a rename: the kubaos running this update may be the file being replaced.
install -m 0755 "$launcher" "$bin.tmp"
mv -f "$bin.tmp" "$bin"
echo "/usr/bin/kubaos now runs your home's kubaos ($installed without one)"
