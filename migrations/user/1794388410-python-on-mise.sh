#!/usr/bin/env bash
# python-on-mise: installs the Python Suite's uv, basedpyright, ruff and
# debugpy with mise (~/.config/mise/conf.d/kubaos-python.toml) on a home that
# has the Suite, now that the Suite's Nix module installs none of them. The
# Pythons and tools uv installed stay where they are.
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

kubaos_suite_to_mise python
