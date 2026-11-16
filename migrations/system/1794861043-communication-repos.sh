#!/usr/bin/env bash
# communication-repos: the apt sources of the communication apps' vendor repositories
# (system/files/apt/sources.list.d/signal.sources, slack.sources, element.sources,
# mattermost.sources and zulip-desktop.sources) and their keys put in place again where KubaOS
# set the repository up for the app, so a changed source reaches it. The guard is root's journal
# of the app, never the sources file: Zulip's own package ships zulip-desktop.sources, so a Zulip
# installed by hand has it too, and that one stays the vendor's. A repository that can't be set
# up is a warning, not a failed update.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

repos=()
for pair in signal:signal slack:slack element:element mattermost:mattermost zulip:zulip-desktop; do
  if kubaos_app_set_up_repo "${pair%%:*}" "${pair#*:}"; then
    repos+=("${pair#*:}")
  fi
done
((${#repos[@]} == 0)) || repo_refresh "${repos[@]}"
