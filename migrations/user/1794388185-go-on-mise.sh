#!/usr/bin/env bash
# go-on-mise: installs the Go Suite's toolchain and tools with mise
# (~/.config/mise/conf.d/kubaos-go.toml) on a home that has the Suite, at the
# version its record chose, now that the Suite's Nix module installs neither
# Go nor its tools.
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

# The session the update runs in still has the GOTOOLCHAIN=local the Suite
# used to set. mise builds gopls with the chosen Go, and a gopls that needs a
# newer one fetches it for the build, which `local` refuses.
unset GOTOOLCHAIN
kubaos_suite_to_mise go
