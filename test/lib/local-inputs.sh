# shellcheck shell=bash
# shellcheck disable=SC2034  # LOCAL_INPUT_MOUNTS and LOCAL_INPUTS are read by the callers
# Host side, sourced by test/run and install/image/build: the flake inputs
# that live on this machine (git+file:// and path: in flake.lock, such as a
# local checkout of an input), so Docker can bind-mount each one read-only at the
# same absolute path and Nix inside the container finds it where the lock
# file says. A git worktree's .git is a file pointing into the main
# repository's .git, which gets mounted too.
#
#   local_inputs FLAKE_DIR       one absolute directory per line
#   local_input_mounts FLAKE_DIR sets LOCAL_INPUT_MOUNTS (docker -v args) and
#                                LOCAL_INPUTS (the dirs, colon-separated, for
#                                the container's git safe.directory)

local_inputs() {
  local lock="$1/flake.lock" dir gitdir
  # "url": "file:///abs?ref=x" (type git) and "path": "/abs" (type path).
  {
    [[ -f $lock ]] || return 0
    sed -n 's#^ *"url": "file://\(/[^"?]*\).*#\1#p' "$lock"
    sed -n 's#^ *"path": "\(/[^"]*\)".*#\1#p' "$lock"
  } | sort -u | while IFS= read -r dir; do
    [[ -d $dir ]] || { printf 'local flake input %s is not on this machine\n' "$dir" >&2; continue; }
    printf '%s\n' "$dir"
    if [[ -f $dir/.git ]]; then
      gitdir="$(sed -n 's/^gitdir: //p' "$dir/.git")"
      [[ $gitdir == /* ]] || gitdir="$dir/$gitdir"
      # <main>/.git/worktrees/<name> -> <main>/.git
      [[ -d $gitdir/../.. ]] && (cd "$gitdir/../.." && pwd)
    fi
  done | sort -u
}

local_input_mounts() {
  local dir
  LOCAL_INPUT_MOUNTS=()
  LOCAL_INPUTS=""
  while IFS= read -r dir; do
    LOCAL_INPUT_MOUNTS+=(-v "$dir:$dir:ro")
    LOCAL_INPUTS+="${LOCAL_INPUTS:+:}$dir"
  done < <(local_inputs "$1")
}
