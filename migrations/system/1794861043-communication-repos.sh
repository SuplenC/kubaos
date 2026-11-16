#!/usr/bin/env bash
# communication-repos: the apt sources of the communication apps' vendor
# repositories (system/files/apt/sources.list.d/signal.sources, slack.sources,
# element.sources, mattermost.sources, zulip-desktop.sources) and their keys put
# in place again on a machine that has KubaOS's, so a changed source reaches it.
# The Apps catalog's helper sets a repository up only as it installs its app. A
# machine without one is left alone, and so is a zulip-desktop.sources that
# Zulip's own package wrote, which names its keyring, not KubaOS's.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

for repo in signal slack element mattermost zulip-desktop; do
  keyring="$(manifest_value "repo.$repo" keyring)"
  if grep -qxF "Signed-By: $keyring" "/etc/apt/sources.list.d/$repo.sources" 2>/dev/null; then
    repo_setup "$repo"
  fi
done
