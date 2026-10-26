# shellcheck shell=bash
# The package manifest for the system scripts: system/packages.toml, the
# user's choices of optional groups and the record of what KubaOS installed
# (docs/CONTRACTS.md, "The package manifest"). system/lib.sh sources it;
# kubaos update reads the same files through kubaos_core::apt.
#
#   manifest_dump [FILE]        <table><TAB><key><TAB><value> per value, an
#                               array's items one line each
#   manifest_value TABLE KEY    the values of one key (group.docker packages)
#   manifest_groups             the groups, in the file's order
#   group_enabled GROUP         the user's choice, else the group's default
#   group_wanted GROUP          enabled, and for this architecture
#   group_packages GROUP...     the packages of the wanted groups, each once
#   choice_set GROUP on|off     record the user's choice
#   group_install GROUP...      set up the groups' repos, install what they
#                               lack, hold a held group, record what it installed
#   repo_setup REPO...          the vendor repo's key and sources file
#
# Install and record go through lib.sh's run and sudo_run, so --dry-run
# changes nothing. A group that isn't wanted is skipped with a line saying why.

MANIFEST_SYSTEM_DIR="${SYSTEM_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
APT_MANIFEST="${APT_MANIFEST:-$MANIFEST_SYSTEM_DIR/packages.toml}"
APT_CHOICES="${APT_CHOICES:-/etc/kubaos/apt-groups}"
APT_RECORD="${APT_RECORD:-/var/lib/kubaos/apt-installed}"

# The file's TOML is kept to what this reads: [tables], key = "string",
# key = true|false and arrays of strings, on one line or over several, with
# comments. kubaos_core::apt::manifest refuses anything with an escape, and
# its tests hold this reader to the Rust one.
manifest_dump() {
  awk '
    function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
    # Every "string" on the line, up to a comment outside the quotes.
    function items(s,   rest, q) {
      rest = s
      while ((q = index(rest, "\"")) > 0 && index(substr(rest, 1, q), "#") == 0) {
        rest = substr(rest, q + 1)
        q = index(rest, "\"")
        print table "\t" key "\t" substr(rest, 1, q - 1)
        rest = substr(rest, q + 1)
      }
      return rest
    }
    in_array { if (index(items($0), "]") > 0) in_array = 0; next }
    /^[ \t]*(#|$)/ { next }
    /^[ \t]*\[/ { table = trim($0); sub(/^\[/, "", table); sub(/\].*$/, "", table); next }
    {
      eq = index($0, "=")
      if (eq == 0) next
      key = trim(substr($0, 1, eq - 1))
      value = trim(substr($0, eq + 1))
      if (substr(value, 1, 1) == "[") {
        if (index(items(substr(value, 2)), "]") == 0) in_array = 1
      } else if (substr(value, 1, 1) == "\"") {
        value = substr(value, 2)
        print table "\t" key "\t" substr(value, 1, index(value, "\"") - 1)
      } else {
        sub(/[ \t]*#.*$/, "", value)
        print table "\t" key "\t" value
      }
    }
  ' "${1:-$APT_MANIFEST}"
}

manifest_value() {
  manifest_dump | awk -F'\t' -v t="$1" -v k="$2" '$1 == t && $2 == k { print $3 }'
}

manifest_groups() {
  manifest_dump | awk -F'\t' '$1 ~ /^group\./ && !seen[$1]++ { sub(/^group\./, "", $1); print $1 }'
}

group_exists() {
  [[ -n "$(manifest_value "group.$1" packages | head -1)" ]]
}

# group_choice GROUP: on or off when the user chose, else nothing.
group_choice() {
  [[ -r "$APT_CHOICES" ]] || return 0
  awk -F= -v g="$1" '$1 == g && ($2 == "on" || $2 == "off") { c = $2 } END { if (c) print c }' "$APT_CHOICES"
}

group_enabled() {
  local default choice
  default="$(manifest_value "group.$1" optional)"
  [[ -n "$default" ]] || return 0
  choice="$(group_choice "$1")"
  [[ "${choice:-$default}" == on ]]
}

# group_why_not GROUP: why the group isn't wanted here, or nothing when it is.
group_why_not() {
  local only
  only="$(manifest_value "group.$1" arch)"
  if [[ -n "$only" && "$only" != "$(arch)" ]]; then
    echo "$only only"
  elif ! group_enabled "$1"; then
    echo "turned off (kubaos packages enable $1)"
  fi
}

group_wanted() { [[ -z "$(group_why_not "$1")" ]]; }

group_packages() {
  local g
  for g in "$@"; do
    if group_wanted "$g"; then manifest_value "group.$g" packages; fi
  done | awk '!seen[$0]++'
}

choice_set() {
  local group="$1" switch="$2" tmp
  group_exists "$group" || die "no group $group in $APT_MANIFEST"
  [[ "$(group_choice "$group")" == "$switch" ]] && return 0
  tmp="$(mktemp)"
  {
    printf '%s\n' "# Optional apt package groups turned on or off (system/packages.toml)." \
      "# Change them with \`kubaos packages enable|disable <group>\`."
    if [[ -r "$APT_CHOICES" ]]; then
      awk -F= -v g="$group" '!/^#/ && NF == 2 && $1 != g' "$APT_CHOICES"
    fi
    echo "$group=$switch"
  } | awk 'NR <= 2 { print; next } { print | "sort" }' >"$tmp"
  write_root_file "$APT_CHOICES" 0644 <"$tmp"
  rm -f "$tmp"
}

repo_setup() {
  local repo sources
  for repo in "$@"; do
    sources="$MANIFEST_SYSTEM_DIR/files/apt/sources.list.d/$repo.sources"
    [[ -f "$sources" ]] || die "no $sources for repo $repo"
    [[ -d /etc/apt/keyrings ]] || sudo_run install -d -m 0755 /etc/apt/keyrings
    fetch_key "$(manifest_value "repo.$repo" key_url)" "$(manifest_value "repo.$repo" keyring)" \
      "$(manifest_value "repo.$repo" fingerprint)"
    add_apt_source "$repo" "$(render_template "$sources" "ARCH=$(arch)")"
  done
  apt_update_if_dirty
}

# record_installed GROUP PKG...: add the packages that are installed now to
# the record, under GROUP. A package already there keeps its group.
record_installed() {
  local group="$1" p lines=()
  shift
  for p in "$@"; do lines+=("$p"$'\t'"$group"); done
  record_lines "${lines[@]}"
}

# record_lines PKG<TAB>GROUP...: record_installed for several groups at once.
record_lines() {
  local line tmp lines=()
  for line in "$@"; do
    pkg_installed "${line%%$'\t'*}" && lines+=("$line")
  done
  ((${#lines[@]})) || return 0
  tmp="$(mktemp)"
  {
    printf '%s\n' "# The apt packages KubaOS installed, and the group of system/packages.toml each was for." \
      "# kubaos update removes one only when no group lists it any more."
    {
      [[ -r "$APT_RECORD" ]] && grep -v '^#' "$APT_RECORD"
      printf '%s\n' "${lines[@]}"
    } | awk -F'\t' 'NF && !seen[$1]++' | sort
  } >"$tmp"
  write_root_file "$APT_RECORD" 0644 <"$tmp"
  rm -f "$tmp"
}

# is_held PKG: PKG is in the `held` array the caller read from apt-mark.
is_held() { printf '%s\n' "${held[@]}" | grep -qxF -- "$1"; }

# group_install GROUP...: one apt-get for all the wanted groups' packages,
# after their repos and before the holds.
group_install() {
  local g why p repos=() wanted=() all=() record=() unhold=() hold=() held=() names available
  for g in "$@"; do
    group_exists "$g" || die "no group $g in $APT_MANIFEST"
    why="$(group_why_not "$g")"
    if [[ -n "$why" ]]; then
      info "skip group $g: $why"
      continue
    fi
    wanted+=("$g")
    p="$(manifest_value "group.$g" repo)"
    [[ -z "$p" ]] || repos+=("$p")
  done
  ((${#repos[@]} == 0)) || repo_setup "${repos[@]}"
  mapfile -t held < <(apt-mark showhold 2>/dev/null)
  for g in "${wanted[@]}"; do
    mapfile -t names < <(manifest_value "group.$g" packages)
    if [[ "$(manifest_value "group.$g" if_available)" == true ]]; then
      available=()
      for p in "${names[@]}"; do
        if pkg_installed "$p" || pkg_has_candidate "$p"; then available+=("$p"); else info "no $p for $(arch), skipping it"; fi
      done
      names=("${available[@]}")
    fi
    for p in "${names[@]}"; do
      all+=("$p")
      pkg_installed "$p" && continue
      record+=("$p"$'\t'"$g")
      if [[ "$(manifest_value "group.$g" hold)" == true ]] && is_held "$p"; then unhold+=("$p"); fi
    done
  done
  ((${#unhold[@]} == 0)) || sudo_run apt-mark unhold "${unhold[@]}"
  if ((${#all[@]})); then
    mapfile -t all < <(printf '%s\n' "${all[@]}" | awk '!seen[$0]++')
    apt_install "${all[@]}"
  fi
  ((DRY_RUN)) || ((${#record[@]} == 0)) || record_lines "${record[@]}"

  mapfile -t held < <(apt-mark showhold 2>/dev/null)
  for g in "${wanted[@]}"; do
    [[ "$(manifest_value "group.$g" hold)" == true ]] || continue
    while IFS= read -r p; do
      pkg_installed "$p" || ((DRY_RUN)) || continue
      is_held "$p" || hold+=("$p")
    done < <(manifest_value "group.$g" packages)
  done
  ((${#hold[@]} == 0)) || sudo_run apt-mark hold "${hold[@]}"
}
