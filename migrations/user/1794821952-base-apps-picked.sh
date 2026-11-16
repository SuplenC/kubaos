#!/usr/bin/env bash
# base-apps-picked: Google Chrome, Docker and Bitwarden desktop are apps of the
# Apps catalog, which kubaos update installs when they are picked in
# ~/.config/kubaos/apps.toml. Each of them on this machine is recorded there
# as picked, unless the user had turned its package group off (`kubaos
# packages disable docker`), so `kubaos app status` lists what the machine
# has. Nothing is installed or removed.
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

picks="${XDG_CONFIG_HOME:-$HOME/.config}/kubaos/apps.toml"

# id, the package that says it is there, and the group the user could turn off.
add=()
while read -r id package group; do
  pkg_installed "$package" || continue
  [[ $group == - || "$(group_choice "$group")" != off ]] || continue
  add+=("$id")
done <<'APPS'
google-chrome google-chrome-stable chrome
docker docker-ce docker
bitwarden bitwarden -
APPS
((${#add[@]})) || exit 0

had=()
if [[ -f $picks ]]; then
  mapfile -t had < <(grep -v '^[[:space:]]*#' "$picks" | grep -o '"[a-z0-9][a-z0-9-]*"' | tr -d '"')
fi
mapfile -t all < <(printf '%s\n' "${had[@]}" "${add[@]}" | awk 'NF' | sort -u)
if ((${#all[@]} == ${#had[@]})); then
  exit 0
fi
kubaos_backup "$picks"
list="$(printf '"%s", ' "${all[@]}")"
mkdir -p -- "$(dirname -- "$picks")"
tmp="$(mktemp "$picks.XXXXXX")"
{
  echo "# The apps of KubaOS's catalog you picked (\`kubaos app list\`). kubaos update installs a picked app"
  echo "# the machine lacks. \`kubaos app add|remove\` rewrite this file, comments too."
  echo "apps = [${list%, }]"
} >"$tmp"
mv -f -- "$tmp" "$picks"
echo "picked ${add[*]} in $picks"
