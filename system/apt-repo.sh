#!/usr/bin/env bash
# Sets up vendor apt repositories of the package manifest (system/packages.toml):
# each one's signing key, refused unless it is exactly the manifest's
# fingerprint, and its sources file from system/files/apt/sources.list.d/,
# turning off any other source of the same repository, then refreshes apt's
# lists when one changed. Safe to re-run: a repo that is
# set up is left alone, offline too.
#
#   system/apt-repo.sh [--dry-run] REPO...
#   system/apt-repo.sh --keys-due REPO...
#
# kubaos update runs it as root for the repos of the groups it installs
# packages from; the install scripts call repo_setup (system/manifest.sh)
# themselves. With --keys-due it changes nothing and needs no root: it prints
# each repo whose key, or next key, setting it up would fetch again
# (repo_key_due), one a line, so an update asks for root only for those.
set -euo pipefail
# shellcheck source=system/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

usage() {
  cat <<EOF
Usage: $0 [--dry-run] REPO...
       $0 --keys-due REPO...

Sets up the vendor apt repositories named in system/packages.toml: $(manifest_dump | awk -F'\t' '$1 ~ /^repo\./ && !seen[$1]++ { sub(/^repo\./, "", $1); printf "%s ", $1 }')
$(common_usage)
EOF
}

repos=()
keys_due=0
while (($#)); do
  case "$1" in
    -n | --dry-run) DRY_RUN=1 ;;
    --keys-due) keys_due=1 ;;
    -h | --help) usage; exit 0 ;;
    -*) usage >&2; exit 2 ;;
    *) repos+=("$1") ;;
  esac
  shift
done
((${#repos[@]})) || { usage >&2; exit 2; }

if ((keys_due)); then
  # gpg reads the keyrings with a home of its own, so the user's ~/.gnupg is never made or read.
  GNUPGHOME="$(mktemp -d)"
  export GNUPGHOME
  trap 'rm -rf "$GNUPGHOME"' EXIT
  for repo in "${repos[@]}"; do
    # A repo this release's manifest no longer has is no setup's to refresh.
    [[ -n "$(manifest_value "repo.$repo" keyring)" ]] || continue
    if repo_key_due "$repo"; then echo "$repo"; fi
  done
  exit 0
fi

for repo in "${repos[@]}"; do
  [[ -n "$(manifest_value "repo.$repo" keyring)" ]] || { echo "no repo $repo in $APT_MANIFEST" >&2; exit 2; }
done

require_sudo
step "Vendor apt repositories: ${repos[*]}"
repo_setup "${repos[@]}"
