#!/usr/bin/env bash
# Sets up vendor apt repositories of the package manifest (system/packages.toml):
# each one's signing key, refused unless it is exactly the manifest's
# fingerprint, and its sources file from system/files/apt/sources.list.d/,
# then refreshes apt's lists when one changed. Safe to re-run: a repo that is
# set up is left alone, offline too.
#
#   system/apt-repo.sh [--dry-run] REPO...
#
# kubaos update runs it as root for the repos of the groups it installs
# packages from; the install scripts call repo_setup (system/manifest.sh)
# themselves.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

usage() {
  cat <<EOF
Usage: $0 [--dry-run] REPO...

Sets up the vendor apt repositories named in system/packages.toml: $(manifest_dump | awk -F'\t' '$1 ~ /^repo\./ && !seen[$1]++ { sub(/^repo\./, "", $1); printf "%s ", $1 }')
$(common_usage)
EOF
}

repos=()
while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    -h | --help) usage; exit 0 ;;
    -*) usage >&2; exit 2 ;;
    *) repos+=("$1") ;;
  esac
  shift
done
((${#repos[@]})) || { usage >&2; exit 2; }
for repo in "${repos[@]}"; do
  [[ -n "$(manifest_value "repo.$repo" keyring)" ]] || { echo "no repo $repo in $APT_MANIFEST" >&2; exit 2; }
done

require_sudo
step "Vendor apt repositories: ${repos[*]}"
repo_setup "${repos[@]}"
