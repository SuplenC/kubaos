#!/usr/bin/env bash
# iac-repos: the apt sources of HashiCorp's, OpenTofu's and Trivy's repositories
# (system/files/apt/sources.list.d/hashicorp.sources, opentofu.sources and trivy.sources) and their
# keys put in place again where KubaOS set the repository up for an app, so a changed source
# reaches it: hashicorp.sources names an Ubuntu release, which a release of KubaOS moves on.
# Terraform, Packer and Vault share HashiCorp's, which is refreshed once whichever of
# them set it up. The guard is root's journal of the app, never the sources file: each vendor's
# instructions write a source of the same repository by hand, and that one stays the user's. A
# repository that can't be set up is a warning, not a failed update.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

repos=()
for pair in terraform:hashicorp packer:hashicorp vault:hashicorp opentofu:opentofu trivy:trivy; do
  repo="${pair#*:}"
  [[ " ${repos[*]} " != *" $repo "* ]] || continue
  if kubaos_app_set_up_repo "${pair%%:*}" "$repo"; then
    repos+=("$repo")
  fi
done
((${#repos[@]} == 0)) || repo_refresh "${repos[@]}"
