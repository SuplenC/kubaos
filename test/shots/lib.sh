# shellcheck shell=bash
# test/shots/lib.sh: what `test/shots/shots` renders for a change, and the upkeep of its BEFORE
# cache. Sourced by `shots` and by `test/shots/shots-test`.
#
# - Which surfaces a change renders: `surfaces_touched`, from the table in `test/shots/surfaces`.
# - The cache, `~/.cache/kubaos/shots/<base>/`: a run holds its base's trees (`hold_base`) and
#   `prune` drops the trees no run holds, past the last few bases. The shared `target/` stays.

# The surfaces drawn by renderers of their own, not by a line in `scenarios`. The specimen's
# pages are the table's `specimen/PAGE` lines.
own_surfaces=(kobo terminals plymouth grub cursor)

# The before-trees of this many base commits are kept. Each agent renders against the main it
# branched from, and the coordinator runs about four at once, so four lets each find its own
# trees again; a base past them costs one cold BEFORE.
keep_bases=4

# In each base kept, the BEFORE shots of this many versions of the harness are kept: main's,
# and that of a branch changing the harness.
keep_harnesses=2

# scenario_surfaces SCENARIOS: the shell's surfaces, one per line.
scenario_surfaces() {
  awk '$1 !~ /^#/ && NF >= 4 { print $2 }' "$1" | sort -u
}

# specimen_pages TABLE: the specimen's pages, as `specimen/PAGE` lines.
specimen_pages() {
  awk '$1 ~ /^specimen\// { print $1 }' "$1" | sort -u
}

# every_surface TABLE SCENARIOS: every surface `shots` can render, one per line.
every_surface() {
  {
    scenario_surfaces "$2"
    printf '%s\n' "${own_surfaces[@]}"
    specimen_pages "$1"
  } | sort -u
}

# changed_paths REPO BASE: the paths that differ between BASE and REPO's working tree, untracked
# files included. A rename counts at both paths.
changed_paths() {
  git -C "$1" diff --no-renames --name-only "$2"
  git -C "$1" ls-files --others --exclude-standard
}

# surfaces_touched TABLE SCENARIOS: the surfaces the paths on stdin render, one per line. A path
# counts for the lines of TABLE that list its most specific match, so a page's own file renders
# that page alone, and the rest of its renderer every page. `*` is every surface, `shell` every
# shell surface and `specimen` every page of the specimen.
surfaces_touched() {
  local table=$1 scenarios=$2 name
  while read -r name; do
    case $name in
    '*') every_surface "$table" "$scenarios" ;;
    shell) scenario_surfaces "$scenarios" ;;
    specimen) specimen_pages "$table" ;;
    *) echo "$name" ;;
    esac
  done < <(
    awk '
      NR == FNR {
        if ($1 ~ /^#/ || NF < 2) next
        for (i = 2; i <= NF; i++) { n++; name[n] = $1; path[n] = $i }
        next
      }
      {
        best = 0
        for (k = 1; k <= n; k++)
          if (($0 == path[k] || index($0, path[k] "/") == 1) && length(path[k]) > best) best = length(path[k])
        for (k = 1; k <= n; k++)
          if (length(path[k]) == best && ($0 == path[k] || index($0, path[k] "/") == 1)) print name[k]
      }' "$table" -
  ) | sort -u
}

# table_problems TABLE SCENARIOS REPO: what is wrong with TABLE, one problem a line: a surface
# with no line of sources, a line naming no surface or group, a source not in REPO, and a path
# under a shared input (the `*` line's) listed by another line, which would take the change there
# from every surface to that line's alone.
table_problems() {
  local table=$1 scenarios=$2 repo=$3
  every_surface "$table" "$scenarios" |
    awk 'NR == FNR { if ($1 !~ /^#/ && NF >= 2) has[$1] = 1; next } !has[$1] { print $1 " has no sources" }' "$table" -
  awk -v known="$(every_surface "$table" "$scenarios" | tr '\n' ' ')" '
    BEGIN { n = split(known, k, " "); for (i = 1; i <= n; i++) ok[k[i]] = 1; ok["*"] = ok["shell"] = ok["specimen"] = 1 }
    $1 !~ /^#/ && NF >= 1 && !ok[$1] { print $1 " is no surface" }' "$table"
  local name path
  while read -r name path; do
    [[ -e $repo/$path ]] || echo "$name lists $path, which isn't in the repo"
  done < <(awk '$1 !~ /^#/ { for (i = 2; i <= NF; i++) print $1, $i }' "$table")
  awk '
    $1 ~ /^#/ { next }
    $1 == "*" { for (i = 2; i <= NF; i++) shared[$i] = 1; next }
    { for (i = 2; i <= NF; i++) { n++; name[n] = $1; path[n] = $i } }
    END {
      for (k = 1; k <= n; k++)
        for (s in shared)
          if (index(path[k], s "/") == 1)
            print name[k] " lists " path[k] ", under the shared " s ": a change there would render " name[k] " alone"
    }' "$table"
}

# hold_base DIR: a shared lock on DIR, a base commit's trees, held on the fd in `held` until the
# run ends or `release_base`, and DIR marked as just used. `prune` never drops a DIR held.
hold_base() {
  local dir=$1
  while :; do
    mkdir -p "$dir"
    exec {held}>>"$dir/.lock"
    flock -s "$held"
    # HAZARD: a prune can move DIR away between the open and the lock. The lock is then on a file
    # no longer at DIR, and a run that went on would write into a tree being deleted.
    [[ /dev/fd/$held -ef $dir/.lock ]] && break
    exec {held}>&-
  done
  touch "$dir/.used"
}

release_base() {
  exec {held}>&-
}

# by_use DIR...: the DIRs, the last used first: by their `.used` mark, or their own time when
# they have none.
by_use() {
  local dir
  for dir in "$@"; do
    echo "$(stat -c %Y "$dir/.used" 2>/dev/null || stat -c %Y "$dir") $dir"
  done | sort -rn | cut -d' ' -f2-
}

# on_a_branch REPO SHA: whether SHA is a commit of main, origin/main or a local branch.
on_a_branch() {
  git -C "$1" cat-file -e "$2^{commit}" 2>/dev/null &&
    [[ -n $(git -C "$1" for-each-ref --count=1 --contains "$2" refs/heads refs/remotes/origin/main) ]]
}

# drop DIR: moves DIR out of the way and deletes it, unless a run holds it. Fails when one does.
drop() {
  local dir=$1 lock trash
  { exec {lock}>>"$dir/.lock"; } 2>/dev/null || return 1
  if ! flock -n -x "$lock"; then
    exec {lock}>&-
    return 1
  fi
  trash=$(dirname "$dir")/.trash.$(basename "$dir").$BASHPID
  mv -T "$dir" "$trash"
  exec {lock}>&-
  rm -rf "$trash"
}

# trim DIR: in a base's trees no run holds, drops all but the `keep_harnesses` harness versions
# last used, and what an interrupted extract left.
trim() {
  local dir=$1 lock old
  { exec {lock}>>"$dir/.lock"; } 2>/dev/null || return 0
  if flock -n -x "$lock"; then
    mapfile -t old < <(
      shopt -s nullglob
      by_use "$dir"/[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]/ |
        tail -n +$((keep_harnesses + 1))
    )
    rm -rf "${old[@]}" "$dir"/.extract.*
  fi
  exec {lock}>&-
}

# prune CACHE REPO: drops from CACHE the trees of base commits no longer on main, origin/main or a
# local branch of REPO, then those of all but the `keep_bases` bases last used, then trims the
# rest. Trees a run holds stay. Then `sweep`. Nothing else in CACHE is touched, `target/`
# included. Prints how many bases it dropped and kept.
prune() {
  local cache=$1 repo=$2 dir kept=0 dropped=0
  local -a bases=()
  if [[ $cache != */kubaos/shots || ! -d $cache ]]; then
    echo "shots: $cache isn't the shots cache: nothing pruned" >&2
    return 1
  fi
  for dir in "$cache"/*/; do
    dir=${dir%/}
    [[ ${dir##*/} =~ ^[0-9a-f]{40}$ ]] && bases+=("$dir")
  done
  if ((${#bases[@]})); then
    while read -r dir; do
      if ((kept < keep_bases)) && on_a_branch "$repo" "${dir##*/}"; then
        trim "$dir"
        kept=$((kept + 1))
      elif drop "$dir"; then
        dropped=$((dropped + 1))
      else
        kept=$((kept + 1))
      fi
    done < <(by_use "${bases[@]}")
  fi
  sweep "$cache"
  echo "$dropped dropped, $kept kept"
}

# sweep CACHE: what is older than a day of CACHE's scratch: a killed run's in `tmp/`, a killed
# prune's `.trash.*`, and Qt's compiled QML. Qt keys that by the file's path, and each run stages
# the shell at a path of its own, so a run never reads another's and the folder only grows.
sweep() {
  local cache=$1
  find "$cache/tmp" -mindepth 1 -maxdepth 1 -mmin +1440 -exec rm -rf {} + 2>/dev/null || true
  find "$cache" -mindepth 1 -maxdepth 1 -name '.trash.*' -mmin +1440 -exec rm -rf {} + 2>/dev/null || true
  find "$cache/xdg/QtProject" -path '*/qmlcache/*' -type f -mmin +1440 -delete 2>/dev/null || true
}
