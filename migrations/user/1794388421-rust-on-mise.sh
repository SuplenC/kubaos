#!/usr/bin/env bash
# rust-on-mise: installs the Rust Suite's tools with mise
# (~/.config/mise/conf.d/kubaos-rust.toml) on a home that has the Suite, with
# the extras its record chose, now that the Suite's Nix module installs only
# rustup's proxies, linked to the rustup mise installs. Its toolchains stay
# where rustup keeps them.
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

kubaos_suite_to_mise rust

# nixpkgs' rustup was built without self-update, and the release mise
# installs has it: `rustup update` would end failing to update a rustup
# outside ~/.cargo/bin. A choice the user wrote into the settings stays.
settings="${RUSTUP_HOME:-$HOME/.rustup}/settings.toml"
if [[ -e "$KUBAOS_USER_FLAKE/kubaos.d/suites/rust.toml" && -f "$settings" ]] \
  && ! grep -q '^auto_self_update' "$settings"; then
  rustup set auto-self-update disable
fi
