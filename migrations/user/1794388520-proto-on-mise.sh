#!/usr/bin/env bash
# proto-on-mise: installs the proto + moon Suite's proto with mise
# (~/.config/mise/conf.d/kubaos-proto.toml) on a home that has the Suite, now
# that the Suite's Nix module installs no proto, and deletes the proto that
# `proto upgrade` may have left in proto's own bin folder: proto's shims run
# that one before any on PATH, so it would stay at its version while mise's
# moves. moon and every tool proto installed stay where they are.
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

[[ -e "$KUBAOS_USER_FLAKE/kubaos.d/suites/proto.toml" ]] || exit 0
kubaos_suite_to_mise proto

bin="${PROTO_HOME:-${XDG_DATA_HOME:-$HOME/.local/share}/proto}/bin"
for exe in proto proto-shim; do
  if [[ -f $bin/$exe && ! -L $bin/$exe ]]; then
    rm -f -- "$bin/$exe"
    echo "deleted $bin/$exe: proto comes from mise, and moves with kubaos suite update"
  fi
done
