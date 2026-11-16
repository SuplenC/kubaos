#!/usr/bin/env bash
# claude-code-repo: Claude Code's apt source (system/files/apt/sources.list.d/
# claude-code.sources) and key put in place again on a machine that has them, so
# a changed source reaches it. The Apps catalog's helper sets the repository up
# only as it installs Claude Code; a machine without it is left alone.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

if [[ -f /etc/apt/sources.list.d/claude-code.sources ]]; then
  repo_setup claude-code
fi
