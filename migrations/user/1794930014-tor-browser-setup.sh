#!/usr/bin/env bash
# tor-browser-setup: Tor Browser asks whether it runs on X11 or Wayland, and an update installs
# a picked app with setups only in the setup it was picked in. A Tor Browser picked before it
# had setups names none, in the picks or in its journal, so every update would skip it with a
# warning. It is recorded in `anonymity`, the Tor Project's X11, which the next update installs:
# the launcher's menu entries, its package already there. `kubaos app add` can't do it here, as
# the update running this holds the lock it takes. A setup already recorded either way stays.
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is
# their flake, under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

picks="${XDG_CONFIG_HOME:-$HOME/.config}/kubaos/apps.toml"
journal="${XDG_STATE_HOME:-$HOME/.local/state}/kubaos/apps/tor-browser/journal.toml"

[[ -f $picks ]] || exit 0
# Exits 0 when the picks name tor-browser in `apps`, before any table, and set no setup for it
# under [setups].
awk '
  /^[[:space:]]*#/ { next }
  /^[[:space:]]*\[/ { table = $0; gsub(/[[:space:]]/, "", table); next }
  table == "" && /"tor-browser"/ { picked = 1 }
  table == "[setups]" && /^[[:space:]]*"?tor-browser"?[[:space:]]*=/ { set = 1 }
  END { exit !(picked && !set) }' "$picks" || exit 0
if [[ -f $journal ]] && grep -q '^setup = ' "$journal"; then
  exit 0
fi

kubaos_backup "$picks"
tmp="$(mktemp "$picks.XXXXXX")"
# [setups] is the table `kubaos app add` writes last; the line goes at its end, else the table
# goes at the file's end.
awk '
  { lines[NR] = $0 }
  /^[[:space:]]*\[/ { table = $0; gsub(/[[:space:]]/, "", table) }
  table == "[setups]" && !/^[[:space:]]*(#.*)?$/ { at = NR }
  END {
    for (i = 1; i <= NR; i++) {
      print lines[i]
      if (i == at) print "tor-browser = \"anonymity\""
    }
    if (!at) { print ""; print "[setups]"; print "tor-browser = \"anonymity\"" }
  }' "$picks" >"$tmp"
chmod --reference="$picks" -- "$tmp"
mv -f -- "$tmp" "$picks"
echo "picked Tor Browser on X11 (anonymity) in $picks"
