#!/usr/bin/env bash
# node-on-mise: installs the Node Suite's Node, vtsls and, with the `bun`
# extra, Bun with mise (~/.config/mise/conf.d/kubaos-node.toml) on a home that
# has the Suite, at the version and extras its record chose, now that the
# Suite's Nix module installs none of them. It runs before project-pins-on-mise,
# so that one moves a project's `use suite node` line once Node is on mise.
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

kubaos_suite_to_mise node
