#!/usr/bin/env bash
# yazi-drag: adds the drag-out key (Ctrl+N, ripdrag) to a yazi keymap.toml
# made before user-layer/tools/yazi/keymap.toml seeded it. A file made since has it.
# kubaos update runs it once, as the user, after the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
#
# It only appends, after a backup, and leaves the file alone when it already
# mentions ripdrag, binds Ctrl+N, or sets a keymap in a shape the appended
# [[mgr.prepend_keymap]] table would clash with (a [mgr] or [manager] table,
# or a prepend_keymap = [...] list).
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

keymap="$("$KUBAOS_DIR/scripts/dot-config" path yazi)/keymap.toml"
[[ -f $keymap ]] || exit 0

settings="$(grep -v '^[[:space:]]*#' -- "$keymap" || true)"
if grep -q 'ripdrag' <<<"$settings"; then
  exit 0
fi
if grep -qE '<C-n>|^[[:space:]]*\[(mgr|manager)\]|^[[:space:]]*\[\[manager\.|(^|[[:space:].])(prepend_|append_)?keymap[[:space:]]*=' <<<"$settings"; then
  echo "yazi-drag: $keymap sets its own keys; add the drag key by hand (docs/USING.md, \"Files\")"
  exit 0
fi

kubaos_backup "$keymap"
cat >>"$keymap" <<'TOML'

# Ctrl+N drags the selected files, or the hovered one, out of yazi: into
# Chrome, an upload form, anything that takes a drop. ripdrag opens a small
# window to drag from and closes after the drop. Added by a KubaOS update.
[[mgr.prepend_keymap]]
on = "<C-n>"
run = 'shell --orphan -- ripdrag --and-exit --all --no-click "$@"'
desc = "Drag the selected files out"
TOML
echo "yazi-drag: added the drag key to $keymap"
