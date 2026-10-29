# shellcheck shell=bash disable=SC2015,SC2016,SC2012
# test/run e2e: the full check list. Sourced by test/lib/entry after
# harness.sh and session.sh; prepare() has run (repo copied, switched).
#
# A check is a function c_<name> that prints what it finds and returns
# non-zero on failure. Add one: write the function, add a `check` line to
# e2e_main. Output goes to $LOGS/check-<name>.log; failures show its tail.

RESULTS=()
FAILED=0

check() {
  local name="$1" fn="$2" t0 rc=0 log="$LOGS/check-$1.log"
  t0="$(now)"
  ("$fn") >"$log" 2>&1 || rc=$?
  local secs
  secs="$(since "$t0")"
  if ((rc == 0)); then
    RESULTS+=("$name|PASS|$secs|$(tail -1 "$log" | cut -c1-70)")
    printf '%sPASS%s  %-14s %6ss\n' "$C_OK" "$C_OFF" "$name" "$secs"
  else
    FAILED=$((FAILED + 1))
    RESULTS+=("$name|FAIL|$secs|$(grep -m1 '^FAIL' "$log" | cut -c1-70 || true)")
    printf '%sFAIL%s  %-14s %6ss  (%s)\n' "$C_ERR" "$C_OFF" "$name" "$secs" "$log"
    tail -15 "$log" | sed 's/^/        /'
  fi
}

# expect_ok DESC CMD...: run, print "ok DESC" or "FAIL DESC" (+ output).
expect_ok() {
  local desc="$1" out rc=0
  shift
  out="$("$@" 2>&1)" || rc=$?
  if ((rc == 0)); then echo "ok   $desc"; else echo "FAIL $desc (exit $rc)"; printf '%s\n' "$out" | tail -10; return 1; fi
}

# --- checks ---------------------------------------------------------------------

c_login_shell() {
  local out
  # A pty, so zsh runs its interactive init the way a terminal does.
  out="$(as_user script -qec 'zsh -l -i -c exit' /dev/null 2>&1 | tr -d '\r')"
  if [[ -n $out ]]; then
    echo "FAIL login shell printed something (warnings?):"
    printf '%s\n' "$out"
    return 1
  fi
  echo "ok   zsh -l -i: no output, no warnings"
  out="$(as_user script -qec "zsh -l -i -c 'whence -w y eza glow _dot_omp_init; [[ -n \$ATUIN_SESSION ]] && echo atuin; (( \${+functions[_direnv_hook]} )) && echo direnv; bindkey \"^R\"'" /dev/null | tr -d '\r')"
  local want
  for want in 'y: function' 'eza: function' 'glow: function' '_dot_omp_init: function' atuin direnv 'atuin-search'; do
    grep -qF -- "$want" <<<"$out" || { echo "FAIL interactive zsh lacks: $want"; printf '%s\n' "$out"; return 1; }
  done
  echo "ok   functions, atuin on Ctrl-R, direnv hook"
  grep -q 'hyprland-autostart.zsh' "$USER_HOME/.zprofile" || { echo "FAIL .zprofile doesn't source the autostart snippet"; return 1; }
  echo "ok   .zprofile sources system/files/hyprland-autostart.zsh"
}

c_path() {
  local path_lines h="$USER_HOME" i want prev=-1 idx
  path_lines="$(zlogin 'print -l $path' 2>/dev/null)"
  for want in "$h/.local/bin" "$h/.cargo/bin" "$h/.local/share/proto/shims" "$h/.local/share/proto/bin" "$h/.nix-profile/bin" /usr/bin; do
    idx="$(grep -nxF -- "$want" <<<"$path_lines" | head -1 | cut -d: -f1)"
    [[ -n $idx ]] || { echo "FAIL PATH lacks $want"; printf '%s\n' "$path_lines"; return 1; }
    echo "ok   PATH[$idx] $want"
  done
  # Nix's tools must win over apt's where both exist (jq, hyperfine here).
  i="$(grep -nxF -- "$h/.nix-profile/bin" <<<"$path_lines" | head -1 | cut -d: -f1)"
  prev="$(grep -nxF -- /usr/bin <<<"$path_lines" | head -1 | cut -d: -f1)"
  ((i < prev)) || { echo "FAIL ~/.nix-profile/bin comes after /usr/bin"; return 1; }
  echo "ok   ~/.nix-profile/bin before /usr/bin"
}

# Every tool docs/CONTRACTS.md lists ("Packages: who installs what") that
# the image installs, resolved in a login shell, with its owner checked.
# Hyprland is the PPA's, from apt.
NIX_TOOLS=(qs ghostty nvim tmux tms lazygit yazi fzf zoxide atuin direnv eza bat fd rg delta dust dysk
  btop jq yq xh tldr just glow gh fastfetch lazydocker bacon cargo-nextest hyperfine sccache wild
  oh-my-posh matugen home-manager non-nixos-gpu-setup gpu-screen-recorder satty kubaos-files kubaos-bluetooth
  kubaos-automount kubaos-phone ripdrag)
APT_TOOLS=(foot fuzzel wl-copy grim slurp playerctl socat qalc notify-send brightnessctl clang mold
  unzip git ssh secret-tool pamtester pamu2fcfg nmcli wpctl snapper fprintd-verify boltctl
  powerprofilesctl ddcutil xdg-open hyprlock hypridle nautilus file-roller 7z unar magick udisksctl
  Hyprland)
c_tools() {
  local out bad=0 t p
  out="$(zrun "for t in ${NIX_TOOLS[*]} ${APT_TOOLS[*]} bw; do printf '%s\t%s\n' \$t \"\$(command -v \$t || echo MISSING)\"; done")"
  for t in "${NIX_TOOLS[@]}"; do
    p="$(awk -F'\t' -v t="$t" '$1 == t {print $2}' <<<"$out")"
    if [[ $p == "$USER_HOME/.nix-profile/bin/"* ]]; then echo "ok   $t  $p"; else echo "FAIL $t (nix) -> $p"; bad=1; fi
  done
  for t in "${APT_TOOLS[@]}"; do
    p="$(awk -F'\t' -v t="$t" '$1 == t {print $2}' <<<"$out")"
    if [[ $p == /usr/* ]]; then echo "ok   $t  $p"; else echo "FAIL $t (apt) -> $p"; bad=1; fi
  done
  p="$(awk -F'\t' '$1 == "bw" {print $2}' <<<"$out")"
  if [[ $p == /usr/local/bin/bw ]]; then echo "ok   bw  $p"; else echo "FAIL bw -> $p"; bad=1; fi
  zrun 'nvim --version' | head -1 | grep -q 'v0\.12' || { echo "FAIL nvim isn't 0.12"; bad=1; }
  # home-manager's programs.zsh puts its zsh on PATH; the login shell stays apt's.
  [[ "$(getent passwd "$USER_NAME" | cut -d: -f7)" == /usr/bin/zsh ]] || { echo "FAIL login shell isn't /usr/bin/zsh"; bad=1; }
  ((bad == 0)) && echo "ok   ${#NIX_TOOLS[@]} nix + ${#APT_TOOLS[@]} apt tools + bw resolve"
  return "$bad"
}

THEME_FILES=(colors.json wallpaper tmux.conf ghostty nvim.lua hypr.lua hyprlock.colors ohmyposh.toml
  gtk.css qt6ct.colors fuzzel.ini foot.ini lazygit.yml lazydocker.yml yazi.toml btop.theme
  delta.gitconfig fzf atuin.toml eza.yml glow.json cursor.toml)
M3_KEYS=(primary on_primary primary_container on_primary_container secondary on_secondary
  secondary_container on_secondary_container tertiary on_tertiary tertiary_container
  on_tertiary_container error on_error error_container on_error_container background on_background
  surface on_surface surface_variant on_surface_variant surface_dim surface_bright
  surface_container_lowest surface_container_low surface_container surface_container_high
  surface_container_highest outline outline_variant shadow scrim inverse_surface
  inverse_on_surface inverse_primary)
theme_files_ok() {
  local st="$USER_HOME/.local/state/theme" f k
  for f in "${THEME_FILES[@]}"; do
    [[ -s $st/$f ]] || { echo "FAIL missing or empty: ~/.local/state/theme/$f"; return 1; }
  done
  for k in "${M3_KEYS[@]}"; do
    jq -e --arg k "$k" '.[$k] | test("^#[0-9a-fA-F]{6}$")' "$st/colors.json" >/dev/null ||
      { echo "FAIL colors.json: $k missing or not #rrggbb"; return 1; }
  done
  jq -e '.mode == "dark" and (.wallpaper | startswith("/"))' "$st/colors.json" >/dev/null ||
    { echo "FAIL colors.json: mode/wallpaper"; return 1; }
  echo "ok   ${#THEME_FILES[@]} theme files, ${#M3_KEYS[@]} color keys"
}
c_theme() {
  local st="$USER_HOME/.local/state/theme" before after wall out
  expect_ok "dot-theme apply --force" zrun 'dot-theme apply --force' || return 1
  theme_files_ok || return 1
  before="$(jq -r .primary "$st/colors.json")"
  wall="$(zrun 'ls /usr/share/kubaos/wallpapers/japanese/*.jpg | head -1')"
  [[ -n $wall ]] || { echo "FAIL no wallpaper under /usr/share/kubaos/wallpapers/japanese"; return 1; }
  expect_ok "dot-theme set $wall" zrun "dot-theme set '$wall'" || return 1
  theme_files_ok || return 1
  [[ "$(zrun 'dot-theme current')" == "$wall" ]] || { echo "FAIL dot-theme current isn't $wall"; return 1; }
  after="$(jq -r .primary "$st/colors.json")"
  [[ $before != "$after" ]] || { echo "FAIL primary didn't change ($before) after dot-theme set"; return 1; }
  echo "ok   primary $before -> $after"
  before="$(stat -c %Y "$st/.stamp")"
  sleep 1
  out="$(zrun 'dot-theme apply' 2>&1)"
  after="$(stat -c %Y "$st/.stamp")"
  [[ $before == "$after" ]] || { echo "FAIL dot-theme apply re-ran with nothing changed: $out"; return 1; }
  echo "ok   dot-theme apply is a no-op when nothing changed"
  local l
  for l in .config/eza/theme.yml .config/atuin/themes/matugen.toml .config/lazydocker/config.yml \
    .config/qt6ct/colors/matugen.conf .config/yazi/theme.toml .config/btop/themes/matugen.theme; do
    [[ -e $USER_HOME/$l ]] || { echo "FAIL ~/$l dangles after dot-theme"; return 1; }
  done
  echo "ok   theme links resolve"
  expect_ok "dot-theme next" zrun 'dot-theme next' || return 1
  expect_ok "dot-theme toggle-folder" zrun 'dot-theme toggle-folder' || return 1
}

# The pointer in the wallpaper's colours (kubaos cursor): dot-theme writes the
# palette and the hyprcursor theme, the XCursor theme follows in the
# background, the menu's style and size apply at once, and the pointer is
# Bibata-Modern-Classic whenever the theme can't be written. There is no
# Hyprland here, so `hyprctl setcursor` is left out.
CUR_ICONS=$USER_HOME/.local/share/icons
CUR_THEME=$CUR_ICONS/KubaOS-Bibata
CUR_SETTINGS=$USER_HOME/.config/kubaos/desktop/cursor/cursor.toml
CUR_PALETTE=$USER_HOME/.local/state/theme/cursor.toml
# cursor_sizes SIZES: waits up to 60 s for the background render to leave an
# XCursor theme at SIZES ("24 30 36 48").
cursor_sizes() {
  local _
  for _ in $(seq 120); do
    grep -qx "sizes $1" "$CUR_THEME/cursors/.stamp" 2>/dev/null && return 0
    sleep 0.5
  done
  echo "FAIL no XCursor theme at sizes $1 within 60 s:"
  cat "$CUR_THEME/cursors/.stamp" "$USER_HOME/.local/state/theme/cursor.log" 2>&1 | tail -10
  return 1
}
# cursor_svg_has COLOUR: left_ptr's image in the hyprcursor theme is drawn in COLOUR.
cursor_svg_has() {
  unzip -tq "$CUR_THEME/hyprcursors/left_ptr.hlc" >/dev/null &&
    unzip -p "$CUR_THEME/hyprcursors/left_ptr.hlc" left_ptr.svg | grep -qiF "$1"
}
# cursor_classic: the theme is the stub that inherits Bibata-Modern-Classic,
# and Bibata-Modern-Classic is there to inherit.
cursor_classic() {
  grep -qx 'Inherits=Bibata-Modern-Classic' "$CUR_THEME/index.theme" && [[ ! -e $CUR_THEME/manifest.hl ]] &&
    [[ -s $CUR_ICONS/Bibata-Modern-Classic/cursors/left_ptr ]]
}
c_cursor() {
  local surface primary out rc n
  local saved
  saved="$(cat "$CUR_SETTINGS")"
  as_user tee "$CUR_SETTINGS" <<<'style = "dark"' >/dev/null

  expect_ok "dot-theme apply --force" zrun 'dot-theme apply --force' || return 1
  read -r surface primary < <(jq -r '[.surface, .primary] | join(" ")' \
    "$USER_HOME/.local/state/theme/colors.json")
  python3 -I - "$CUR_PALETTE" "$USER_HOME/.local/state/theme/colors.json" <<'PY' || return 1
import colorsys, json, sys, tomllib
with open(sys.argv[1], "rb") as f:
    p = tomllib.load(f)
with open(sys.argv[2]) as f:
    c = json.load(f)
want = {"dark": ["surface", "on_surface", "surface"], "accent": ["primary", "on_primary", "on_primary"]}
for style, roles in want.items():
    got = [p[style][k].lower() for k in ("fill", "outline", "watch")]
    if got != [c[r].lower() for r in roles]:
        sys.exit(f"FAIL ~/.local/state/theme/cursor.toml [{style}] is {got}, colors.json's {roles} are not")
def hls(hex):
    return colorsys.rgb_to_hls(*(int(hex[i:i + 2], 16) / 255 for i in (1, 3, 5)))
for style in want:
    spinner = [s.lower() for s in p[style]["spinner"]]
    hues = [round(hls(s)[0] * 360) for s in spinner]
    off = max(min(abs(h - hues[0]), 360 - abs(h - hues[0])) for h in hues)
    if spinner[0] != c["primary"].lower() or len(set(spinner)) != 4 or off > 8:
        sys.exit(f"FAIL [{style}] spinner {spinner} isn't four tones of primary {c['primary']} (hues {hues})")
print(f"ok   cursor.toml has the palette's roles, the spinner {spinner}")
PY
  [[ -s $CUR_THEME/manifest.hl ]] || { echo "FAIL no hyprcursor manifest in $CUR_THEME"; return 1; }
  local hlc=("$CUR_THEME"/hyprcursors/*.hlc)
  n=${#hlc[@]}
  ((n >= 50)) || { echo "FAIL $n hyprcursor shapes, Bibata has 56"; return 1; }
  cursor_svg_has "$surface" || { echo "FAIL left_ptr.hlc isn't a zip with left_ptr.svg filled $surface"; return 1; }
  cursor_svg_has '#00FF00' && { echo "FAIL left_ptr.svg still has Bibata's placeholder fill"; return 1; }
  echo "ok   hyprcursor theme: $n shapes, left_ptr filled $surface"
  # Bibata's rainbow spinner and corners: none of it is left in the busy cursors and the corners.
  local f svgs tones
  tones="$(grep -m1 '^spinner' "$CUR_PALETTE" | grep -oiE '#[0-9a-f]{6}' | paste -sd '|')"
  for f in wait left_ptr_watch top_left_corner bottom_right_corner; do
    # Read whole first: grep -q stopping early would fail unzip, and the pipe with it.
    svgs="$(unzip -p "$CUR_THEME/hyprcursors/$f.hlc" '*.svg')" || { echo "FAIL reading $f.hlc"; return 1; }
    grep -qiE '#(F05024|FCB813|7EBA41|32A0DA|F1613A|FDBE2A|96C865|4FADDF)' <<<"$svgs" &&
      { echo "FAIL $f.hlc still has Bibata's rainbow"; return 1; }
    grep -qiE "$tones" <<<"$svgs" || { echo "FAIL $f.hlc has none of the spinner's tones ($tones)"; return 1; }
  done
  echo "ok   the spinner and the corners are in the accent's tones, no rainbow left"

  cursor_sizes "24 30 36 48" || return 1
  python3 -I - "$CUR_THEME/cursors" "$surface" <<'PY' || return 1
import os, struct, sys
d, fill = sys.argv[1], int(sys.argv[2].lstrip("#"), 16)
for name in ("left_ptr", "default", "arrow", "watch", "pointer"):
    with open(os.path.join(d, name), "rb") as f:
        data = f.read()
    if data[:4] != b"Xcur":
        sys.exit(f"FAIL cursors/{name} isn't an XCursor file")
with open(os.path.join(d, "left_ptr"), "rb") as f:
    data = f.read()
(ntoc,) = struct.unpack_from("<I", data, 12)
sizes, opaque = set(), 0
for i in range(ntoc):
    _, nominal, pos = struct.unpack_from("<III", data, 16 + 12 * i)
    w, h = struct.unpack_from("<II", data, pos + 16)
    sizes.add(nominal)
    pixels = struct.unpack_from(f"<{w * h}I", data, pos + 36)
    opaque += sum(1 for p in pixels if p == 0xFF000000 | fill)
if sizes != {24, 30, 36, 48}:
    sys.exit(f"FAIL cursors/left_ptr has sizes {sorted(sizes)}, not 24 30 36 48")
if not opaque:
    sys.exit(f"FAIL cursors/left_ptr has no pixel of the fill #{fill:06x}")
print(f"ok   XCursor theme at 24 30 36 48, its aliases linked, left_ptr filled with {opaque} #{fill:06x} pixels")
PY

  expect_ok "kubaos cursor style accent" zrun 'kubaos cursor style accent' || return 1
  grep -qx 'style = "accent"' "$CUR_SETTINGS" || { echo "FAIL the settings don't say accent:"; cat "$CUR_SETTINGS"; return 1; }
  cursor_svg_has "$primary" || { echo "FAIL left_ptr.svg isn't filled $primary after style accent"; return 1; }
  cursor_svg_has "$surface" && { echo "FAIL left_ptr.svg still has the dark style's fill $surface"; return 1; }
  echo "ok   style accent: left_ptr filled $primary"
  expect_ok "kubaos cursor size 32" zrun 'kubaos cursor size 32' || return 1
  cursor_sizes "32 40 48 64" || return 1
  [[ $(zrun 'kubaos cursor show') == "style=accent size=32" ]] || { echo "FAIL kubaos cursor show: $(zrun 'kubaos cursor show')"; return 1; }
  rc=0; out="$(zrun 'kubaos cursor size 7' 2>&1)" || rc=$?
  ((rc == 2)) || { echo "FAIL kubaos cursor size 7 exited $rc, not 2: $out"; return 1; }
  rc=0; out="$(zrun 'kubaos cursor size 18' 2>&1)" || rc=$?
  ((rc == 2)) || { echo "FAIL kubaos cursor size 18, off the grid, exited $rc, not 2: $out"; return 1; }
  echo "ok   size 32: XCursor at 32 40 48 64; size 7 and size 18 are refused"
  expect_ok "kubaos cursor style classic" zrun 'kubaos cursor style classic' || return 1
  cursor_classic || { echo "FAIL style classic didn't leave the Bibata-Modern-Classic stub"; ls -la "$CUR_THEME"; return 1; }
  echo "ok   style classic: the theme inherits Bibata-Modern-Classic"

  # The fallback: a theme that can't be written leaves the classic stub, and
  # dot-theme carries on.
  expect_ok "kubaos cursor style dark" zrun 'kubaos cursor style dark' || return 1
  [[ -s $CUR_THEME/manifest.hl ]] || { echo "FAIL style dark wrote no hyprcursor theme"; return 1; }
  as_user install -d /var/tmp/kubaos-no-bibata
  rc=0; out="$(zrun 'KUBAOS_CURSOR_SOURCE=/var/tmp/kubaos-no-bibata kubaos cursor apply --sync' 2>&1)" || rc=$?
  ((rc == 1)) || { echo "FAIL apply without Bibata's sources exited $rc, not 1: $out"; return 1; }
  cursor_classic || { echo "FAIL without Bibata's sources the theme isn't the classic stub"; ls -la "$CUR_THEME"; return 1; }
  echo "ok   without Bibata's sources: exit 1, the pointer is Bibata-Modern-Classic"
  out="$(zrun 'KUBAOS_CURSOR_SOURCE=/var/tmp/kubaos-no-bibata dot-theme apply --force' 2>&1)" ||
    { echo "FAIL dot-theme failed with the cursor theme failing:"; printf '%s\n' "$out" | tail -10; return 1; }
  grep -q 'Bibata-Modern-Classic' <<<"$out" || { echo "FAIL dot-theme didn't say the pointer fell back: $out"; return 1; }
  cursor_classic || { echo "FAIL after dot-theme the theme isn't the classic stub"; return 1; }
  echo "ok   dot-theme carries on and says so"
  as_user mv "$CUR_PALETTE" "$CUR_PALETTE.saved"
  rc=0; out="$(zrun 'kubaos cursor apply --sync' 2>&1)" || rc=$?
  as_user mv "$CUR_PALETTE.saved" "$CUR_PALETTE"
  ((rc == 1)) && cursor_classic || { echo "FAIL without a palette: exit $rc, not 1, or no classic stub: $out"; return 1; }
  echo "ok   without a palette: exit 1, the pointer is Bibata-Modern-Classic"

  printf '%s\n' "$saved" | as_user tee "$CUR_SETTINGS" >/dev/null
  expect_ok "kubaos cursor apply --sync" zrun 'kubaos cursor apply --sync' || return 1
  rmdir /var/tmp/kubaos-no-bibata
}

c_tmux() {
  local out
  out="$(zrun 'tmux -L dotfiles-test -f ~/.config/tmux/tmux.conf new-session -d -s t -x 200 -y 50 && \
    tmux -L dotfiles-test source-file ~/.config/tmux/tmux.conf && \
    tmux -L dotfiles-test show -gv @color_primary; tmux -L dotfiles-test show-messages; \
    tmux -L dotfiles-test kill-server' 2>&1)" || { echo "FAIL tmux: $out"; return 1; }
  grep -qE '^#[0-9a-fA-F]{6}$' <<<"$out" || { echo "FAIL @color_primary unset; tmux said:"; printf '%s\n' "$out"; return 1; }
  # show-messages logs every command; anything else in it is a complaint.
  if grep -v -E ' command: |^#[0-9a-fA-F]{6}$' <<<"$out" | grep -qiE 'error|unknown|invalid|no such|bad '; then echo "FAIL tmux messages:"; printf '%s\n' "$out"; return 1; fi
  echo "ok   tmux.conf + theme + plugins load, @color_primary=$(head -1 <<<"$out")"
}

c_nvim() {
  local out
  cache_lock nvim
  # LazyVim's plugins from the lockfile (cached in the dotfiles-test-cache volume).
  expect_ok "Lazy! restore" zrun 'nvim --headless "+Lazy! restore" +qa' || return 1
  out="$(zrun "nvim --headless +'lua io.write(vim.g.colors_name or \"none\", \" \", vim.v.errmsg)' +qa" 2>&1 | tr -d '\r')"
  [[ $out == matugen* ]] || { echo "FAIL colorscheme isn't matugen: $out"; return 1; }
  [[ $out == "matugen " ]] || { echo "FAIL nvim startup error: $out"; return 1; }
  echo "ok   nvim starts clean with the matugen colorscheme"
  # LazyVim's Mason tools (LSPs, linters): installed once into the cache
  # volume; wait for them so a half-finished install doesn't show as errors.
  out="$(zrun "cd ~/Projects/dotfiles && nvim --headless README.md +'lua vim.wait(3000); local ok, r = pcall(require, \"mason-registry\"); if ok then vim.wait(900000, function() for _, p in ipairs(r.get_all_packages()) do if p:is_installing() then return false end end return true end, 1000); io.write(#r.get_installed_package_names(), \" mason packages\") end' +qa" 2>&1 | tr -d '\r' | tail -1)"
  echo "     $out"
  # Treesitter parsers LazyVim wants, compiled once into the cache volume.
  out="$(zrun "cd ~/Projects/dotfiles && nvim --headless README.md +'lua local ok, err = pcall(function() require(\"nvim-treesitter\").install(LazyVim.opts(\"nvim-treesitter\").ensure_installed):wait(900000) end); io.write(\"\\nTS \", tostring(ok), \" \", tostring(err))' +qa" 2>&1 | tr -d '\r' | tail -1)"
  echo "     treesitter: $out"
  # A file of each kind the config has extras for; Neovim's messages must
  # hold no error.
  local f msgs
  for f in README.md flake.nix scripts/dot-theme config/hypr/hyprland.lua config/quickshell/shell.qml; do
    msgs="$(zrun "cd ~/Projects/dotfiles && nvim --headless $f +'lua vim.wait(1500)' +'redir => g:m | silent messages | redir END' +'lua io.write(\"\\nFT=\", vim.bo.filetype, \"\\n\", vim.g.m)' +qa!" 2>&1 | tr -d '\r')"
    if grep -qE '^(E[0-9]+:|Error)' <<<"$msgs" || ! grep -qE '^FT=.+' <<<"$msgs"; then
      echo "FAIL nvim $f:"; printf '%s\n' "$msgs" | head -20; return 1
    fi
    echo "ok   nvim $f ($(grep -m1 '^FT=' <<<"$msgs")): no errors"
  done
}

GHOSTTY_UNIT=$USER_HOME/.config/systemd/user/app-com.mitchellh.ghostty.service
GHOSTTY_DBUS=$USER_HOME/.local/share/dbus-1/services/com.mitchellh.ghostty.service
# ghostty_owned: whether a Ghostty instance owns its name on the session bus.
ghostty_owned() {
  in_session 'dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.NameHasOwner string:com.mitchellh.ghostty' 2>/dev/null |
    grep -q 'boolean true'
}
# ghostty_wait_file FILE: wait up to 15 s for a window's command to write FILE.
ghostty_wait_file() {
  local _
  for _ in $(seq 150); do
    [[ -s $1 ]] && return 0
    sleep 0.1
  done
  return 1
}
c_ghostty() {
  local exec_start out bg f="$LOGS/ghostty-window.out" bad=0
  expect_ok "ghostty +validate-config" zrun 'ghostty +validate-config' || return 1
  expect_ok "foot --check-config" zrun 'foot --check-config' || return 1

  grep -qx 'X-RestartIfChanged=false' "$GHOSTTY_UNIT" && grep -qx 'BusName=com.mitchellh.ghostty' "$GHOSTTY_UNIT" &&
    grep -qx 'WantedBy=graphical-session.target' "$GHOSTTY_UNIT" ||
    { echo "FAIL the Ghostty unit:"; cat "$GHOSTTY_UNIT"; return 1; }
  exec_start="$(sed -n 's/^ExecStart=//p' "$GHOSTTY_UNIT")"
  [[ $exec_start == *' --gtk-single-instance=true --initial-window=false --quit-after-last-window-closed=false' ]] ||
    { echo "FAIL the unit starts '$exec_start'"; return 1; }
  grep -qx 'Name=com.mitchellh.ghostty' "$GHOSTTY_DBUS" ||
    { echo "FAIL no D-Bus file for Ghostty at $GHOSTTY_DBUS"; return 1; }
  echo "ok   the background Ghostty unit (kept across switches) and its D-Bus file"

  session_start sway 1280x720 1
  # What autostart.lua hands D-Bus activation on the laptop.
  in_session 'dbus-update-activation-environment WAYLAND_DISPLAY XDG_RUNTIME_DIR PATH' >/dev/null 2>&1
  in_session "exec $exec_start >$LOGS/ghostty-bg.log 2>&1" 2>/dev/null &
  bg=$!
  for _ in $(seq 100); do ghostty_owned && break; sleep 0.1; done
  if ghostty_owned; then
    rm -f "$f"
    out="$(in_session "cd /tmp && ghostty +new-window --working-directory=\"\$HOME\" -e sh -c 'echo \"\$TERM_PROGRAM \$PWD\" >$f'" 2>&1)"
    if ghostty_wait_file "$f" && [[ $(<"$f") == "ghostty $USER_HOME" ]]; then
      echo "ok   +new-window -e runs its command in a window of the running instance, in home"
    else
      echo "FAIL +new-window -e: $out; the window wrote '$(cat "$f" 2>/dev/null)'"
      bad=1
    fi
  else
    echo "FAIL the unit's Ghostty doesn't own com.mitchellh.ghostty:"
    tail -5 "$LOGS/ghostty-bg.log"
    bad=1
  fi
  { kill "$bg" && wait "$bg"; } 2>/dev/null || true
  pkill -u "$USER_NAME" -f 'ghostty.*--initial-window=false' 2>/dev/null
  for _ in $(seq 50); do ghostty_owned || break; sleep 0.1; done

  # With no instance running, D-Bus starts one from the linked file.
  rm -f "$f"
  out="$(in_session "ghostty +new-window -e sh -c 'echo started >$f'" 2>&1)"
  if ghostty_wait_file "$f"; then
    echo "ok   with the instance gone, +new-window starts one through D-Bus"
  else
    echo "FAIL +new-window with no instance: $out"
    bad=1
  fi
  pkill -u "$USER_NAME" -f 'ghostty.*--initial-window=false' 2>/dev/null
  session_stop
  # HAZARD: Ghostty's portal queries bus-activate xdg-desktop-portal, and the
  # bus hands it and anything activated later this session's display. Left
  # behind, the next Quickshell start in the menu check hangs on its portal
  # settings call: stop the portal and blank the display the bus hands out.
  pkill -u "$USER_NAME" -f '/usr/libexec/(xdg-desktop-portal|xdg-permission-store|gvfsd)' 2>/dev/null
  as_user dbus-update-activation-environment WAYLAND_DISPLAY= >/dev/null 2>&1
  return "$bad"
}

# zsh_pty CMD: CMD run by kuba in a pseudo-terminal, stdin passed through, CRs dropped.
zsh_pty() { as_user script -qec "$1" /dev/null 2>&1 | tr -d '\r'; }
# The greeting's logo, from branding/fastfetch.txt.
GREETING_MARK='███▀▀▀▀▀▀▀███'
# greets ENV...: how many greetings an interactive zsh shows with ENV, in a
# pty, given the commands on stdin.
greets() { zsh_pty "env $* zsh -i" | grep -c -- "$GREETING_MARK" || true; }
# An interactive start's median wall time, in ms. The harness measured 51 ms
# before the init caches and 40 ms after; 100 ms is the issue's target and
# leaves room for a loaded machine.
ZSH_START_BUDGET_MS=100
c_zsh_start() {
  local zcache="$USER_HOME/.cache/kubaos/zsh" times median f before after n bad=0
  # In a pty, so zle is on and fzf and atuin load as they do in a terminal.
  times="$(zsh_pty "zsh -fc 'zmodload zsh/datetime; repeat 22 { t=\$EPOCHREALTIME; zsh -i -c exit; print \"ms \$(( (EPOCHREALTIME - t) * 1000 ))\" }'" |
    sed -n 's/^ms //p' | tail -n 20)"
  median="$(sort -n <<<"$times" | awk '{ a[NR] = $1 } END { printf "%.1f", (a[int((NR + 1) / 2)] + a[int(NR / 2) + 1]) / 2 }')"
  echo "     zsh -i -c exit, 20 runs (ms): $(sort -n <<<"$times" | awk '{ printf "%.0f ", $1 }')"
  if awk -v m="$median" -v b="$ZSH_START_BUDGET_MS" 'BEGIN { exit !(m <= b) }'; then
    echo "ok   interactive zsh starts in $median ms (median), within the $ZSH_START_BUDGET_MS ms budget"
  else
    echo "FAIL interactive zsh starts in $median ms (median), over the $ZSH_START_BUDGET_MS ms budget"
    bad=1
  fi

  rm -rf "$zcache"
  zsh_pty 'zsh -i -c exit' >/dev/null
  for f in fzf atuin direnv zoxide omp zcompdump; do
    n="$(find "$zcache" -maxdepth 1 -name "$f-*" -size +0 2>/dev/null | wc -l)"
    ((n >= 1)) || { echo "FAIL no cached $f under ~/.cache/kubaos/zsh after deleting it"; ls -la "$zcache"; bad=1; }
  done
  ((bad)) || echo "ok   deleting ~/.cache/kubaos/zsh: the next start writes fzf, atuin, direnv, zoxide, oh-my-posh and the dump again"
  if ! cmp -s <(as_user "$USER_HOME/.nix-profile/bin/atuin" init zsh --disable-up-arrow) \
    "$(find "$zcache" -name 'atuin-*.zsh' | head -1)"; then
    echo "FAIL the cached atuin init isn't what \`atuin init zsh --disable-up-arrow\` prints"
    bad=1
  fi
  before="$(stat -c '%n %Y' "$zcache"/* | sort)"
  sleep 1.1
  zsh_pty 'zsh -i -c exit' >/dev/null
  after="$(stat -c '%n %Y' "$zcache"/* | sort)"
  if [[ $before == "$after" ]]; then
    echo "ok   the next start reads the cache and writes nothing"
  else
    echo "FAIL a second start rewrote the cache:"
    diff <(printf '%s\n' "$before") <(printf '%s\n' "$after")
    bad=1
  fi
  f="$(find "$zcache" -name 'zcompdump-*' ! -name '*.zwc' | head -1)"
  touch -d '2 days ago' "$f"
  zsh_pty 'zsh -i -c exit' >/dev/null
  if [[ -n $(find "$f" -mmin -1) ]]; then
    echo "ok   a completion dump older than a day is rebuilt"
  else
    echo "FAIL the day-old completion dump wasn't rebuilt"
    bad=1
  fi

  return "$bad"
}

c_greeting() {
  local bad=0 out t n
  n="$(printf 'exit\n' | greets TERM_PROGRAM=ghostty)"
  [[ $n == 1 ]] || { echo "FAIL a new Ghostty window's shell greets $n times, not once"; bad=1; }
  out="$(printf 'echo "level=$KUBAOS_GREETING"\nzsh -i\nexit\nexit\n' | zsh_pty 'env TERM_PROGRAM=ghostty zsh -i')"
  [[ $(grep -c -- "$GREETING_MARK" <<<"$out") == 1 ]] && grep -q 'level=0' <<<"$out" ||
    { echo "FAIL a shell started inside the window greets again:"; printf '%s\n' "$out" | tail -20; bad=1; }
  ((bad)) || echo "ok   the greeting shows once in a new Ghostty window's shell, not in a shell started from it"
  for t in TMUX=/tmp/tmux-1000/default,1,0 NVIM=/run/user/1000/nvim.1.0 SSH_CONNECTION=10.0.0.2 \
    SSH_TTY=/dev/pts/9 KUBAOS_GREETING=0 TERM_PROGRAM=foot; do
    n="$(printf 'exit\n' | greets TERM_PROGRAM=ghostty "$t")"
    [[ $n == 0 ]] || { echo "FAIL greeted with $t"; bad=1; }
  done
  n="$(zsh_pty 'env TERM_PROGRAM=ghostty zsh -i -c true' | grep -c -- "$GREETING_MARK" || true)"
  [[ $n == 0 ]] || { echo "FAIL greeted in zsh -c"; bad=1; }
  ((bad)) || echo "ok   no greeting under \$TMUX, \$NVIM, SSH, KUBAOS_GREETING=0, another terminal, or zsh -c"

  zrun 'dot-greeting off' && [[ $(zrun 'dot-greeting status') == off ]] ||
    { echo "FAIL dot-greeting off"; bad=1; }
  n="$(printf 'exit\n' | greets TERM_PROGRAM=ghostty)"
  [[ $n == 0 ]] || { echo "FAIL greeted with the setting off"; bad=1; }
  zrun 'dot-greeting on' && [[ $(zrun 'dot-greeting status') == on ]] ||
    { echo "FAIL dot-greeting on"; bad=1; }
  n="$(printf 'exit\n' | greets TERM_PROGRAM=ghostty)"
  [[ $n == 1 ]] || { echo "FAIL not greeted with the setting back on"; bad=1; }
  ((bad)) || echo "ok   dot-greeting off and on turn the greeting off and on"

  echo "     what the greeting costs, next to the full fastfetch:"
  as_user bash -c "cd ~ && ~/.nix-profile/bin/hyperfine -N --warmup 3 --runs 20 -i \
    '$USER_HOME/.nix-profile/bin/fastfetch --config $USER_HOME/.config/fastfetch/greeting.jsonc' \
    '$USER_HOME/.nix-profile/bin/fastfetch' 2>&1" | grep -E 'Benchmark|Time|Warning|Error' | sed 's/^/     /'
  return "$bad"
}

c_tms() {
  local out
  mkdir -p "$USER_HOME/Scripts"
  chown "$USER_NAME:" "$USER_HOME/Scripts"
  out="$(zrun 'tms config --help >/dev/null && tms --version' 2>&1)" || { echo "FAIL tms: $out"; return 1; }
  echo "ok   $out"
  out="$(zrun 'tmux -L tmstest new-session -d -s probe && TMUX_TMPDIR= tms sessions; tmux -L tmstest kill-server' 2>&1 || true)"
  echo "     tms sessions: ${out:0:80}"
  grep -q 'search_dirs' "$USER_HOME/.config/tms/config.toml" && echo "ok   ~/.config/tms/config.toml is the repo's" || return 1
  # tms searches the Command menu's folders: the same paths, in the same order.
  out="$(python3 -I - "$USER_HOME/.config/tms/config.toml" "$USER_HOME/.config/kotodama/kubaos.toml" <<'PY'
import sys, tomllib
tms = tomllib.load(open(sys.argv[1], "rb"))
menu = tomllib.load(open(sys.argv[2], "rb"))
searched = [d["path"] for d in tms["search_dirs"]]
folders = [f["path"] for f in menu["extensions"]["folders"]["folders"]]
if searched != folders or "~/Projects" not in folders or "~/Scripts" not in folders:
    sys.exit(f"tms searches {searched}, the menu's folders are {folders}")
print(" ".join(folders))
PY
  )" || { echo "FAIL $out"; return 1; }
  echo "ok   tms and the menu search the same folders: $out"
}

c_links() {
  local d n bad=0 target
  # Defaults: every file under config/ is a read-only store file with the
  # repo's content, except the ones dot-config seeds or merges.
  local f n_files=0 layer
  layer="$(awk '$1 != "app" {print $2}' "$USER_HOME/.local/share/kubaos/layer")"
  while IFS= read -r f; do
    f="${f#config/}"
    grep -qxF -- "$f" <<<"$layer" && continue
    target="$(readlink -f "$USER_HOME/.config/$f" || true)"
    [[ $target == /nix/store/* ]] && cmp -s "$target" "$REPO/config/$f" ||
      { echo "FAIL ~/.config/$f -> $target"; bad=1; }
    n_files=$((n_files + 1))
  done < <(as_user git -C "$REPO" ls-files config)
  # The token outputs the Nix build adds to config/ (design/package.nix), which
  # .gitignore lists: linked from the store like the rest.
  while IFS= read -r f; do
    target="$(readlink -f "$USER_HOME/.config/$f" || true)"
    [[ $target == /nix/store/* && -s $target ]] || { echo "FAIL ~/.config/$f -> $target"; bad=1; }
    n_files=$((n_files + 1))
  done < <(sed -n 's|^/config/||p' "$REPO/.gitignore")
  for d in "$REPO"/config/*/; do
    n="$(basename "$d")"
    [[ -d $USER_HOME/.config/$n && ! -L $USER_HOME/.config/$n ]] || { echo "FAIL ~/.config/$n isn't a directory"; bad=1; }
  done
  for d in "$REPO"/scripts/*; do
    n="$(basename "$d")"
    target="$(readlink -f "$USER_HOME/.local/bin/$n" || true)"
    [[ $target == /nix/store/*-kubaos-scripts/"$n" ]] && cmp -s "$target" "$d" ||
      { echo "FAIL ~/.local/bin/$n -> $target"; bad=1; }
    [[ -x $d ]] || { echo "FAIL scripts/$n isn't executable"; bad=1; }
    [[ $n == dot-* ]] || { echo "FAIL scripts/$n has no dot- prefix"; bad=1; }
    head -1 "$d" | grep -q '^#!/usr/bin/env ' || { echo "FAIL scripts/$n: no env shebang"; bad=1; }
    grep -q '^set -euo pipefail' "$d" || { echo "FAIL scripts/$n: no set -euo pipefail"; bad=1; }
  done
  [[ -d $USER_HOME/Pictures/wallpapers && ! -L $USER_HOME/Pictures/wallpapers ]] ||
    { echo "FAIL ~/Pictures/wallpapers isn't a real folder"; bad=1; }
  [[ -f $USER_HOME/.config/tmux-nix/plugins.conf ]] || { echo "FAIL no ~/.config/tmux-nix/plugins.conf"; bad=1; }
  [[ -f $USER_HOME/.config/direnv/lib/bwenv.sh ]] || { echo "FAIL no direnv bwenv.sh"; bad=1; }
  ((bad == 0)) && echo "ok   $n_files default files and $(ls "$REPO"/scripts | wc -l) scripts from the store"
  return "$bad"
}

# PAM: the lock screen's services through Nix's libpam (Nix's pamtester,
# as kuba, like Quickshell), sudo/sudo-i/polkit-1 through Ubuntu's (apt
# pamtester and sudo-rs), each with the pam_u2f line and no key present.
PAM_USER=pamtest
PAM_PW='pamtest-Pw-1'
c_pam() {
  local nixpam bad=0 svc t0 secs out
  nixpam="$(readlink -f "$CACHE/roots/pamtester" 2>/dev/null || true)"
  if [[ ! -x $nixpam/bin/pamtester ]]; then
    nixpam="$(as_user "$NIX_BIN/nix" build --no-link --print-out-paths --inputs-from "$REPO" nixpkgs#pamtester)"
  fi
  nixpam="$nixpam/bin/pamtester"
  id "$PAM_USER" >/dev/null 2>&1 || useradd -m -s /bin/bash "$PAM_USER"
  echo "$PAM_USER:$PAM_PW" | chpasswd
  echo "$PAM_USER ALL=(ALL:ALL) ALL" >/etc/sudoers.d/pamtest
  chmod 0440 /etc/sudoers.d/pamtest
  # A well-formed mapping for a key that isn't plugged in, so pam_u2f really
  # looks for a device (instead of nouserok short-cutting it).
  printf '%s:%s,%s,es256,+presence\n' "$PAM_USER" \
    "$(head -c 64 /dev/urandom | base64 -w0)" "$(head -c 65 /dev/urandom | base64 -w0)" >/etc/u2f_mappings
  printf '%s:%s,%s,es256,+presence\n' "$USER_NAME" \
    "$(head -c 64 /dev/urandom | base64 -w0)" "$(head -c 65 /dev/urandom | base64 -w0)" >>/etc/u2f_mappings
  chmod 0644 /etc/u2f_mappings

  # Lock screen (Nix libpam), as kuba (password "kuba").
  [[ "$(readlink /run/wrappers/bin/unix_chkpwd)" == /usr/sbin/unix_chkpwd ]] ||
    { echo "FAIL /run/wrappers/bin/unix_chkpwd isn't Ubuntu's helper"; bad=1; }
  out="$(printf 'kuba\n' | as_user "$nixpam" qs-lock kuba authenticate 2>&1)" &&
    echo "ok   qs-lock (Nix libpam) accepts the right password" ||
    { echo "FAIL qs-lock rejects the right password: $out"; bad=1; }
  printf 'wrong\n' | as_user "$nixpam" qs-lock kuba authenticate >/dev/null 2>&1 &&
    { echo "FAIL qs-lock accepts a wrong password"; bad=1; } || echo "ok   qs-lock rejects a wrong password"
  for svc in qs-lock-u2f qs-lock-fingerprint; do
    t0="$(now)"
    out="$(as_user timeout 20 "$nixpam" "$svc" kuba authenticate </dev/null 2>&1 || true)"
    secs="$(since "$t0")"
    if grep -qiE 'unknown|cannot open|no such file|incorrect conversation' <<<"$out" && ! grep -q 'Authentication failure' <<<"$out"; then
      echo "FAIL $svc: module didn't load: $out"; bad=1
    elif grep -q 'successfully authenticated' <<<"$out"; then
      echo "FAIL $svc authenticated with no device"; bad=1
    else
      echo "ok   $svc loads under Nix libpam, fails without a device in ${secs}s"
    fi
  done
  readlink /usr/local/lib/security/pam_u2f.so | grep -q '^/nix/store/' ||
    { echo "FAIL pam_u2f.so link doesn't point into /nix/store"; bad=1; }
  readlink /usr/local/lib/security/pam_fprintd.so | grep -q '^/nix/store/' ||
    { echo "FAIL pam_fprintd.so link doesn't point into /nix/store"; bad=1; }

  # sudo, sudo-i, polkit-1 (Ubuntu libpam): key first, password fallback.
  for svc in sudo sudo-i polkit-1; do
    grep -q '^auth .*sufficient .*pam_u2f.so authfile=/etc/u2f_mappings cue nouserok' "/etc/pam.d/$svc" ||
      { echo "FAIL /etc/pam.d/$svc has no pam_u2f line"; bad=1; continue; }
    t0="$(now)"
    if printf '%s\n' "$PAM_PW" | timeout 20 pamtester "$svc" "$PAM_USER" authenticate >/dev/null 2>&1; then
      echo "ok   $svc: no key -> password accepted in $(since "$t0")s"
    else
      echo "FAIL $svc rejects the right password with no key"; bad=1
    fi
    printf 'nope\n' | timeout 20 pamtester "$svc" "$PAM_USER" authenticate >/dev/null 2>&1 &&
      { echo "FAIL $svc accepts a wrong password"; bad=1; } || echo "ok   $svc rejects a wrong password"
  done
  # The real sudo (sudo-rs) as a user who needs a password.
  printf '%s\n' "$PAM_PW" | runuser -u "$PAM_USER" -- sudo -k -S true >/dev/null 2>&1 &&
    echo "ok   sudo-rs: password accepted with pam_u2f in the stack" ||
    { echo "FAIL sudo-rs rejected the right password"; bad=1; }
  printf 'nope\n' | runuser -u "$PAM_USER" -- sudo -k -S true >/dev/null 2>&1 &&
    { echo "FAIL sudo-rs accepted a wrong password"; bad=1; } || echo "ok   sudo-rs rejects a wrong password"
  # No mapping file at all (nouserok): still the password.
  mv /etc/u2f_mappings /etc/u2f_mappings.off
  printf '%s\n' "$PAM_PW" | timeout 20 pamtester sudo "$PAM_USER" authenticate >/dev/null 2>&1 &&
    echo "ok   sudo without /etc/u2f_mappings: password accepted" ||
    { echo "FAIL sudo without a mapping file rejects the password"; bad=1; }
  mv /etc/u2f_mappings.off /etc/u2f_mappings
  ((bad == 0)) && echo "ok   PAM: lock screen + sudo/sudo-i/polkit-1 fall back to the password"
  return "$bad"
}

c_flake_check() {
  expect_ok "nix flake check" as_user bash -c "cd '$REPO' && $NIX_BIN/nix flake check --no-warn-dirty" || return 1
}

c_shellcheck() {
  local files=() f
  cd "$REPO" || return 1
  while IFS= read -r f; do
    case "$f" in
      *.sh) files+=("$f") ;;
      *) head -1 "$f" | grep -qE '^#!.*\b(ba)?sh\b' && files+=("$f") ;;
    esac
  done < <(as_user git -C "$REPO" ls-files scripts system migrations test config crates/self-imports .claude/hooks | grep -vE '\.(md|nix|lua|qml|toml|json|jpg|png|webp)$')
  echo "     ${#files[@]} files"
  # Gate on warnings and errors; info and style notes are listed, not fatal.
  local out notes
  out="$(shellcheck -x -f gcc "${files[@]}" 2>&1 || true)"
  notes="$(grep -c . <<<"$out" || true)"
  if grep -qE ': (warning|error):' <<<"$out"; then
    printf '%s\n' "$out"
    echo "FAIL shellcheck: warnings or errors above"
    return 1
  fi
  ((notes)) && { echo "     info/style notes (not fatal):"; printf '%s\n' "$out" | sed 's/^/       /'; }
  echo "ok   shellcheck: no warnings in ${#files[@]} files ($notes info/style notes)"
}

c_lua() {
  local files=() f bad=0
  cd "$REPO" || return 1
  # matugen templates hold {{ }} placeholders; their rendered output is
  # checked instead (~/.local/state/theme/{hypr,nvim}.lua).
  mapfile -t files < <(as_user git -C "$REPO" ls-files '*.lua' | grep -v '^config/matugen/templates/')
  files+=("$USER_HOME/.local/state/theme/hypr.lua" "$USER_HOME/.local/state/theme/nvim.lua")
  for f in "${files[@]}"; do
    luac5.5 -p "$f" || { echo "FAIL $f"; bad=1; }
  done
  ((bad == 0)) && echo "ok   luac5.5 -p: ${#files[@]} files"
  # The active window's neon focus (config/hypr/focus.lua): its strike, cut
  # short by a new focus, off with the setting, and always ending lit.
  as_user lua5.5 "$REPO/test/lua/focus_test.lua" "$REPO" "$USER_HOME/.config/hypr" ||
    { echo "FAIL test/lua/focus_test.lua"; bad=1; }
  return "$bad"
}

# Per-screen workspaces (config/hypr/workspaces.lua): the block bookkeeping,
# and the module itself against a stand-in for Hyprland's Lua API, under the
# Lua 5.5 that Hyprland runs.
c_workspaces() {
  as_user lua5.5 "$REPO/test/lua/workspaces_test.lua" "$REPO" || { echo "FAIL test/lua/workspaces_test.lua"; return 1; }
  echo "ok   per-screen workspaces: blocks, slots, joining on unplug and returning on replug"
  workspaces_hotplug_bar
}

# The primary's bar after screens are plugged in: test/lib/hyprland-hotplug.py
# stands in for Hyprland and sends the groups waiting on the primary, and the
# focused window's workspace among them, to the new screens, with Quickshell
# reading the workspace list it asked for before the moves only after their
# events. The primary's bar shows its own group alone, and no title.
workspaces_hotplug_bar() {
  local signature=kubaos-e2e-hotplug state="$USER_HOME/.local/state" fake groups title bad=0
  local screens="$state/kubaos/hypr/workspace-screens"
  session_start sway 1920x1080 1
  # The stand-in reads the log from before Quickshell starts.
  : >"$QS_LOG"
  chown "$USER_NAME:" "$QS_LOG"
  as_user python3 -I "$REPO/test/lib/hyprland-hotplug.py" "$RUNTIME_DIR" "$signature" "$state" "$QS_LOG" \
    >"$LOGS/hyprland-hotplug.log" 2>&1 &
  fake=$!
  for _ in $(seq 50); do
    grep -q ready "$LOGS/hyprland-hotplug.log" && break
    sleep 0.1
  done
  if ! grep -q ready "$LOGS/hyprland-hotplug.log"; then
    echo "FAIL the stand-in for Hyprland didn't start:"
    tail -5 "$LOGS/hyprland-hotplug.log" | sed 's/^/     /'
    bad=1
  elif ! qs_start "HYPRLAND_INSTANCE_SIGNATURE=$signature" KUBAOS_WORKSPACES_PROBE=1; then
    echo "FAIL Quickshell didn't load its config:"
    tail -30 "$QS_LOG"
    bad=1
  elif ! wait "$fake"; then
    echo "FAIL the hotplug didn't play out:"
    tail -5 "$LOGS/hyprland-hotplug.log" | sed 's/^/     /'
    bad=1
  else
    groups="$(grep -o 'workspaces probe: HEADLESS-1 .*' "$QS_LOG" | tail -1)"
    groups="${groups#workspaces probe: HEADLESS-1 }"
    title="$(grep -o 'active window probe: HEADLESS-1 .*' "$QS_LOG" | tail -1)"
    title="${title#active window probe: HEADLESS-1 }"
    if [[ $groups == '[[1,2,4,5]]' ]]; then
      echo "ok   after a hotplug the primary's bar shows its own group alone, the workspace list read after the moves' events"
    else
      echo "FAIL after a hotplug the primary's bar shows $groups, not [[1,2,4,5]]"
      bad=1
    fi
    if [[ $title == '""' ]]; then
      echo "ok   the focused window's title leaves the primary's bar with its workspace"
    else
      echo "FAIL after a hotplug the primary's bar shows the title $title of a window on another screen"
      bad=1
    fi
  fi
  kill "$fake" 2>/dev/null || true
  session_stop
  rm -rf "${RUNTIME_DIR:?}/hypr/$signature" "$screens"
  return "$bad"
}

# The power stops' laptop panel (config/hypr/monitors.lua): the stop's refresh
# rate and VRR on the panel's own rule, against a stand-in for `hl`.
c_power() {
  as_user lua5.5 "$REPO/test/lua/power_panel_test.lua" "$REPO" || { echo "FAIL test/lua/power_panel_test.lua"; return 1; }
  echo "ok   power stops: the panel's rate and VRR on its own rule, no other screen touched"
}

# Keyboard layouts: config/hypr/input.lua hands Hyprland the user's layouts
# (~/.config/kubaos/desktop/keyboard/keyboard.toml), else every layout in
# /etc/default/keyboard, and `kubaos keyboard show` reads the same files.
c_keyboard() {
  as_user lua5.5 "$REPO/test/lua/input_test.lua" "$REPO" || { echo "FAIL test/lua/input_test.lua"; return 1; }
  echo "ok   input.lua: the user's layouts, else the system's as XKB's lists, US without either"
  local dir out
  install_kubaos || return 1
  dir="$(mktemp -d)"
  printf 'XKBMODEL="pc105"\nXKBLAYOUT="us,it"\nXKBVARIANT=",intl"\nXKBOPTIONS=""\n' >"$dir/keyboard"
  chown -R "$USER_NAME:" "$dir"
  chmod 644 "$dir/keyboard"
  out="$(kz "XDG_CONFIG_HOME='$dir' KUBAOS_KEYBOARD_FILE='$dir/keyboard' kubaos keyboard show" 2>&1)" ||
    { echo "FAIL kubaos keyboard show: $out"; rm -rf "$dir"; return 1; }
  [[ $out == $'us\nit(intl)' ]] || { echo "FAIL kubaos keyboard show printed: $out"; rm -rf "$dir"; return 1; }
  out="$(kz "export XDG_CONFIG_HOME='$dir' KUBAOS_KEYBOARD_FILE='$dir/keyboard'; kubaos keyboard seed && kubaos keyboard seed && kubaos keyboard show" 2>&1)"
  if [[ $out != $'us\nit(intl)' ]] || ! grep -qx 'layouts = \["us", "it(intl)"\]' "$dir/kubaos/desktop/keyboard/keyboard.toml"; then
    echo "FAIL kubaos keyboard seed: printed $out, wrote $(cat "$dir/kubaos/desktop/keyboard/keyboard.toml" 2>&1)"; rm -rf "$dir"; return 1
  fi
  printf 'layouts = ["de"]\n' >"$dir/kubaos/desktop/keyboard/keyboard.toml"
  out="$(kz "export XDG_CONFIG_HOME='$dir' KUBAOS_KEYBOARD_FILE='$dir/keyboard'; kubaos keyboard seed && kubaos keyboard show" 2>&1)"
  rm -rf "$dir"
  [[ $out == de ]] || { echo "FAIL a user list of their own: kubaos keyboard seed then show printed $out"; return 1; }
  echo "ok   kubaos keyboard show lists the system's layouts, then the user's once seeded or set, the default first"
  keyboard_changes || return 1
  keyboard_refusal || return 1
  keyboard_writer || return 1
  mac_accents || return 1
  keyboard_tab_clicks || return 1
}

# Ubuntu's own D-Bus policy turns localed's SetX11Keyboard away from everyone,
# root included, so `kubaos keyboard` never asks localed. A new default goes to
# /etc/default/keyboard through the root helper under the real pkexec and
# polkitd: in the container the user has no active session, so polkit wants
# a password, and with no terminal pkexec leaves the asking to an agent the
# session hasn't got, as on a desktop whose polkit agent isn't running.
# The user's layouts change all the same; pkexec's words are logged on a line of
# their own, then the last line, which the Keyboard tab shows, says what the
# console keeps and why.
keyboard_refusal() {
  local policy=/usr/share/dbus-1/system.d/org.freedesktop.locale1.read-only.conf dir out
  grep -q 'send_member="SetX11Keyboard"' "$policy" 2>/dev/null ||
    { echo "FAIL $policy no longer denies SetX11Keyboard: localed may write the keyboard again"; return 1; }
  [[ -x /usr/local/libexec/kubaos-keyboard-write ]] ||
    { echo "FAIL system/50-hyprland.sh installed no /usr/local/libexec/kubaos-keyboard-write"; return 1; }
  dir="$(mktemp -d)"
  printf 'XKBMODEL="pc105"\nXKBLAYOUT="us,it"\nXKBVARIANT=""\nXKBOPTIONS=""\n' >"$dir/keyboard"
  chown -R "$USER_NAME:" "$dir"
  out="$(kz "XDG_CONFIG_HOME='$dir' KUBAOS_KEYBOARD_FILE='$dir/keyboard' kubaos keyboard default it" 2>&1)" && {
    echo "FAIL kubaos keyboard default it succeeded with polkit refusing: $out"; rm -rf "$dir"; return 1
  }
  if ! grep -qx 'layouts = \["it", "us"\]' "$dir/kubaos/desktop/keyboard/keyboard.toml" 2>/dev/null; then
    echo "FAIL a refused system default left the user's layouts unchanged: $(cat "$dir/kubaos/desktop/keyboard/keyboard.toml" 2>&1)"
    rm -rf "$dir"; return 1
  fi
  rm -rf "$dir"
  [[ $(tail -1 <<<"$out") == "kubaos: the keyboard layouts changed, but the console and the disk's passphrase prompt still type with us: no polkit agent is running to ask for the password" ]] ||
    { echo "FAIL a refused pkexec printed: $out"; return 1; }
  grep -qx "kubaos: pkexec kubaos-keyboard-write failed (exit status: [0-9]*): .*No authentication agent found.*" <<<"$out" ||
    { echo "FAIL a refused pkexec logged no line with what it said: $out"; return 1; }
  echo "ok   a new default no agent can ask for still changes the user's layouts, logged with pkexec's words, said in a sentence"
}

# The root helper `kubaos keyboard` runs through pkexec, as root on the
# container's /etc/default/keyboard: it writes the four XKB fields, keeps the
# other lines, and refuses a value that could break out of its quotes. The
# console and initramfs refresh is skipped: the container has neither.
keyboard_writer() {
  local helper=/usr/local/libexec/kubaos-keyboard-write kb=/etc/default/keyboard saved bad=0
  cmp -s "$REPO/system/files/keyboard/kubaos-keyboard-write" "$helper" ||
    { echo "FAIL $helper isn't system/files/keyboard/kubaos-keyboard-write"; return 1; }
  saved="$(mktemp)"
  [[ -e $kb ]] && cp -p "$kb" "$saved"
  printf 'XKBMODEL="pc105"\nXKBLAYOUT="us"\nXKBVARIANT=""\nXKBOPTIONS=""\nBACKSPACE="guess"\n' >"$kb"
  export KUBAOS_KEYBOARD_NO_BOOT=1
  if ! "$helper" it pc105 intl caps:escape; then
    echo "FAIL kubaos-keyboard-write it pc105 intl caps:escape failed"; bad=1
  elif ! grep -qx 'XKBLAYOUT="it"' "$kb" || ! grep -qx 'XKBVARIANT="intl"' "$kb" ||
    ! grep -qx 'XKBOPTIONS="caps:escape"' "$kb" || ! grep -qx 'BACKSPACE="guess"' "$kb" ||
    [[ $(grep -c '^XKBLAYOUT=' "$kb") != 1 || $(stat -c %a "$kb") != 644 ]]; then
    echo "FAIL kubaos-keyboard-write wrote: $(cat "$kb")"; bad=1
  else
    echo "ok   the root helper writes the XKB fields and keeps the other lines"
  fi
  if "$helper" 'us"' pc105 '' '' 2>/dev/null || "$helper" "$(printf 'us\nX=1')" pc105 '' '' 2>/dev/null ||
    ! grep -qx 'XKBLAYOUT="it"' "$kb"; then
    echo "FAIL kubaos-keyboard-write took a quote or a new line: $(cat "$kb")"; bad=1
  else
    echo "ok   the root helper refuses a quote or a new line, and leaves the file as it was"
  fi
  unset KUBAOS_KEYBOARD_NO_BOOT
  if [[ -s $saved ]]; then cp -p "$saved" "$kb"; else rm -f "$kb"; fi
  rm -f "$saved"
  return "$bad"
}

# How long a check driving the shell waits for the state it asserts, in seconds.
# A bound for a run that is broken, not a pace: every wait ends as soon as the
# state is there, and a machine busy with other builds takes several seconds
# for what an idle one does at once.
PROBE_WAIT=30

# input_tools: VI, the command that runs test/lib/vinput.py (Python with
# pywayland), and VI_XML, the three protocols it needs, from the flake's
# nixpkgs: the roots an earlier run made, or built now. The virtual keyboard's
# protocol comes with wtype's source.
input_tools() {
  local roots="$CACHE/roots" sys
  sys="$(uname -m)-linux"
  [[ -x $roots/vinput-python/bin/python3 ]] ||
    as_user "$NIX_BIN/nix" build --out-link "$roots/vinput-python" --impure --expr \
      "(builtins.getFlake \"path:$REPO\").inputs.nixpkgs.legacyPackages.$sys.python3.withPackages (p: [ p.pywayland ])" \
      >/dev/null 2>&1
  [[ -f $roots/wayland-scanner/share/wayland/wayland.xml ]] ||
    as_user "$NIX_BIN/nix" build --out-link "$roots/wayland-scanner" --inputs-from "$REPO" 'nixpkgs#wayland-scanner^out' \
      >/dev/null 2>&1
  [[ -d $roots/wlr-protocols ]] ||
    as_user "$NIX_BIN/nix" build --out-link "$roots/wlr-protocols" --inputs-from "$REPO" nixpkgs#wlr-protocols >/dev/null 2>&1
  [[ -d $roots/wtype-src ]] ||
    as_user "$NIX_BIN/nix" build --out-link "$roots/wtype-src" --inputs-from "$REPO" nixpkgs#wtype.src >/dev/null 2>&1
  VI=("$roots/vinput-python/bin/python3" -I "$REPO/test/lib/vinput.py")
  VI_XML=("$roots/wayland-scanner/share/wayland/wayland.xml"
    "$roots/wtype-src/protocol/virtual-keyboard-unstable-v1.xml"
    "$roots/wlr-protocols/share/wlr-protocols/unstable/wlr-virtual-pointer-unstable-v1.xml")
  VI_SOCK="$RUNTIME_DIR/vinput.sock"
  [[ -x ${VI[0]} && -f ${VI_XML[0]} && -f ${VI_XML[1]} && -f ${VI_XML[2]} ]]
}

# input_up WxH: the session's keyboard and pointer (test/lib/vinput.py), on
# an output of that size. Started before Quickshell, so it binds both with the
# seat and nothing typed or clicked comes before it can take it.
input_up() {
  local server
  rm -f "$VI_SOCK"
  in_session "exec ${VI[*]} serve $VI_SOCK ${VI_XML[*]} ${1%x*} ${1#*x}" >>"$LOGS/vinput.log" 2>&1 &
  server=$!
  for _ in $(seq $((PROBE_WAIT * 10))); do
    [[ -S $VI_SOCK ]] && return 0
    kill -0 "$server" 2>/dev/null || break
    sleep 0.1
  done
  echo "FAIL test/lib/vinput.py didn't start: $(tail -3 "$LOGS/vinput.log")"
  return 1
}

# input STEP...: the steps (test/lib/vinput.py) through the session's keyboard
# and pointer; it returns once the compositor has taken them.
input() { as_user "${VI[@]}" send "$VI_SOCK" "$@" 2>>"$LOGS/vinput.log"; }

# kb_probe_at NAME: "X,Y", where the keyboard tab's probe last put NAME
# (config/quickshell/modules/controlcenter/KeyboardList.qml).
kb_probe_at() {
  grep -o "keyboard probe: at .*" "$QS_LOG" | tail -1 | grep -oE "$1 x=[0-9]+ y=[0-9]+" |
    sed -E 's/.* x=([0-9]+) y=([0-9]+)/\1,\2/'
}

# kb_probe_wait PATTERN: up to PROBE_WAIT for the probe to log a line matching PATTERN (ERE).
kb_probe_wait() {
  for _ in $(seq $((PROBE_WAIT * 10))); do
    grep -qE "keyboard probe: $1" "$QS_LOG" && return 0
    sleep 0.1
  done
  return 1
}

# kb_probe_last PATTERN: up to PROBE_WAIT for the probe's latest `at` line to
# match PATTERN (ERE): where everything is now, not where it once was.
kb_probe_last() {
  for _ in $(seq $((PROBE_WAIT * 10))); do
    grep -o "keyboard probe: at .*" "$QS_LOG" | tail -1 | grep -qE "$1" && return 0
    sleep 0.1
  done
  return 1
}

# kb_layouts_are FILE LAYOUTS: up to PROBE_WAIT for keyboard.toml FILE to list
# LAYOUTS, as `layouts = [...]` spells them.
kb_layouts_are() {
  for _ in $(seq $((PROBE_WAIT * 10))); do
    grep -qxF "layouts = [$2]" "$1" && return 0
    sleep 0.1
  done
  return 1
}

# The control center's Keyboard tab under a real pointer (a virtual one, as
# the headless session has none), with the Daemon, which the add view searches
# through. A double-click on the toolbar's + opens the search and adds
# nothing. Once the search is in the "Add a keyboard layout" app, a click on
# its first match adds it to the user's layouts with no privilege asked. A
# drag of the last row up one place moves it there, through one
# `kubaos keyboard move`.
keyboard_tab_clicks() {
  local dir at to x y added moves daemon mine="$USER_HOME/.config/kubaos/desktop/keyboard/keyboard.toml" saved bad=0
  input_tools || { echo "FAIL no pywayland or Wayland protocols from the flake's nixpkgs"; return 1; }
  dir="$(mktemp -d)"
  printf 'XKBMODEL="pc105"\nXKBLAYOUT="us,it"\nXKBVARIANT=","\nXKBOPTIONS=""\n' >"$dir/keyboard"
  chown -R "$USER_NAME:" "$dir"
  saved="$dir/keyboard.toml.saved"
  [[ -f $mine ]] && cp -p "$mine" "$saved"
  as_user mkdir -p "$(dirname "$mine")"
  printf 'layouts = ["us", "it"]\n' | as_user tee "$mine" >/dev/null
  session_start sway 1920x1080 1
  in_session "exec kotodamad >>$LOGS/kotodamad-keyboard.log 2>&1" &
  daemon=$!
  input_up 1920x1080 || bad=1
  if ((!bad)) && ! qs_start "KUBAOS_KEYBOARD_FILE=$dir/keyboard" KUBAOS_KEYBOARD_PROBE=1; then
    echo "FAIL Quickshell didn't load"; tail -30 "$QS_LOG"; bad=1
  fi
  # The probe logs where things are once the sheet has slid in.
  if ((!bad)); then
    in_session 'qs ipc call controlcenter keyboard' >/dev/null 2>&1
    kb_probe_wait 'at .*add-button' || { echo "FAIL qs ipc call controlcenter keyboard showed no + in the toolbar"; bad=1; }
  fi
  at="$(kb_probe_at add-button)"

  # The second click comes well inside the double-click interval; a second after
  # it is the time it has to change something it shouldn't.
  if ((!bad)); then
    input move:"$at" click wait:0.12 click wait:1 || { echo "FAIL test/lib/vinput.py: $(tail -3 "$LOGS/vinput.log")"; bad=1; }
    if ! grep -qx 'layouts = \["us", "it"\]' "$mine"; then
      echo "FAIL a double-click on + changed the layouts: $(cat "$mine")"; bad=1
    elif ! grep -q 'keyboard probe: adding true' "$QS_LOG" || grep -q 'keyboard probe: adding false' "$QS_LOG"; then
      echo "FAIL the double-click at $at didn't leave the search open:"
      grep -o 'keyboard probe: .*' "$QS_LOG" | tail -5 | sed 's/^/     /'
      bad=1
    else
      echo "ok   a double-click on + opens the search and adds nothing"
    fi
  fi

  if ((!bad)); then
    if ! kb_probe_wait 'entered true'; then
      echo "FAIL the search never entered the \"Add a keyboard layout\" app:"
      grep -o 'keyboard probe: .*' "$QS_LOG" | tail -5 | sed 's/^/     /'
      tail -5 "$LOGS/kotodamad-keyboard.log" | sed 's/^/     /'
      bad=1
    elif ! kb_probe_last 'first-match '; then
      echo "FAIL the search showed no first match:"
      grep -o 'keyboard probe: .*' "$QS_LOG" | tail -5 | sed 's/^/     /'
      bad=1
    else
      at="$(kb_probe_at first-match)"
      input move:"$at" click || echo "FAIL test/lib/vinput.py: $(tail -3 "$LOGS/vinput.log")"
      kb_probe_wait 'change add ' || true
      added="$(grep -o 'keyboard probe: change add .*' "$QS_LOG" | tail -1 | sed 's/.*change add //')"
      if [[ -n $added ]] && kb_layouts_are "$mine" "\"us\", \"it\", \"$added\""; then
        echo "ok   a click on the search's first match adds it ($added) to the user's layouts, no privilege asked"
      else
        echo "FAIL a click on the first match at '$at' didn't add it: added '$added', layouts $(cat "$mine")"
        bad=1
      fi
    fi
  fi

  if ((!bad)); then
    kb_probe_last 'row-2 ' || { echo "FAIL the list never showed a third row"; bad=1; }
    at="$(kb_probe_at row-2)"
    to="$(kb_probe_at row-1)"
    x="${at%,*}"
    y="${at#*,}"
    # Up past the drag threshold and on to the row above, in steps, as a hand moves. A second
    # after the drop is the time a second move has to show up.
    input move:"$at" press wait:0.1 move:"$x,$((y - 8))" wait:0.05 move:"$x,$((y - 20))" wait:0.05 \
      move:"$x,$((y - 36))" wait:0.05 move:"$to" wait:0.05 release wait:1
    kb_layouts_are "$mine" "\"us\", \"$added\", \"it\"" || true
    moves="$(grep -c 'keyboard probe: change move' "$QS_LOG")"
    if ! grep -qxF "layouts = [\"us\", \"$added\", \"it\"]" "$mine"; then
      echo "FAIL dragging the last row from $at to $to didn't move it up one place: $(cat "$mine")"
      grep -o 'keyboard probe: .*' "$QS_LOG" | tail -5 | sed 's/^/     /'
      bad=1
    elif ((moves != 1)); then
      echo "FAIL a drag ran kubaos keyboard move $moves times, not once"; bad=1
    else
      echo "ok   dragging a row onto the one above moves it there, with one kubaos keyboard move"
    fi
  fi

  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  menu_stop "$daemon"
  if [[ -f $saved ]]; then cp -p "$saved" "$mine"; else rm -f "$mine"; fi
  rm -rf "$dir"
  return "$bad"
}

# `kubaos keyboard move`, `add` and `list` with nothing stood in: a change that
# keeps the default is the user's file alone, and asks for no privilege.
keyboard_changes() {
  local dir out
  dir="$(mktemp -d)"
  printf 'XKBMODEL="pc105"\nXKBLAYOUT="us,it,de"\nXKBVARIANT=",intl,"\nXKBOPTIONS="caps:escape"\n' >"$dir/keyboard"
  cp -p "$dir/keyboard" "$dir/keyboard.before"
  chown -R "$USER_NAME:" "$dir"
  out="$(kz "export XDG_CONFIG_HOME='$dir' KUBAOS_KEYBOARD_FILE='$dir/keyboard'; kubaos keyboard move de 2 && kubaos keyboard add pl" 2>&1)" ||
    { echo "FAIL kubaos keyboard move de 2, add pl: $out"; rm -rf "$dir"; return 1; }
  if ! grep -qx 'layouts = \["us", "de", "it(intl)", "pl"\]' "$dir/kubaos/desktop/keyboard/keyboard.toml" ||
    ! cmp -s "$dir/keyboard" "$dir/keyboard.before"; then
    echo "FAIL kubaos keyboard move de 2, add pl wrote $(cat "$dir/kubaos/desktop/keyboard/keyboard.toml" 2>&1), system $(cat "$dir/keyboard")"
    rm -rf "$dir"
    return 1
  fi
  echo "ok   kubaos keyboard move and add change the user's layouts alone while the default stays"
  out="$(kz "XDG_CONFIG_HOME='$dir' KUBAOS_KEYBOARD_FILE='$dir/keyboard' kubaos keyboard move de 5" 2>&1)" && {
    echo "FAIL kubaos keyboard move de 5 of 4 succeeded: $out"
    rm -rf "$dir"
    return 1
  }
  rm -rf "$dir"
  echo "ok   a move past the last place is refused"
  out="$(kz "kubaos keyboard list" 2>&1)" || { echo "FAIL kubaos keyboard list: $out"; return 1; }
  if ! grep -qxP 'fr\tFrench' <<<"$out" || ! grep -qP '^us\(intl\)\t' <<<"$out"; then
    echo "FAIL kubaos keyboard list lacks fr or us(intl): $(head -5 <<<"$out")"
    return 1
  fi
  echo "ok   kubaos keyboard list names every layout and variant ($(wc -l <<<"$out") of them)"
}

# The Mac-style accents on Left Alt: keyd's config and the XKB option as
# system/50-hyprland.sh installed them, the option in a keymap with two
# layouts and the user's own option, the setting through `kubaos keyboard
# accents` with a stand-in systemctl, and Hyprland's cheat sheet. Real keys
# through keyd need /dev/uinput: the laptop checklist covers them.
mac_accents() {
  local dir out key keymap
  cmp -s "$REPO/system/files/keyd/kubaos.conf" /etc/keyd/kubaos.conf ||
    { echo "FAIL /etc/keyd/kubaos.conf isn't system/files/keyd/kubaos.conf"; return 1; }
  out="$(keyd.rvaiya list-keys 2>&1)" || { echo "FAIL keyd list-keys: $out"; return 1; }
  for key in leftalt move edit bassboost question iso setup deletefile closecd; do
    grep -qx "$key" <<<"$out" || { echo "FAIL keyd knows no key $key"; return 1; }
  done
  echo "ok   keyd's config is installed, and keyd knows every key it names"
  keymap="$(as_user xkbcli compile-keymap --layout fr,ru --options grp:alt_shift_toggle,kubaos:mac-accents 2>&1)" ||
    { echo "FAIL compiling a keymap with kubaos:mac-accents: $(tail -5 <<<"$keymap")"; return 1; }
  for key in 'I183>.*dead_acute' 'I184>.*dead_grave' 'I217>.*dead_diaeresis' 'I222>.*dead_circumflex' \
    'I178>.*dead_tilde' 'I149>.*acute, *grave' 'I154>.*diaeresis, *U02C6' 'I168>.*U02DC' 'ISO_Next_Group'; do
    grep -qE "$key" <<<"$keymap" || { echo "FAIL the keymap lacks $key"; return 1; }
  done
  echo "ok   kubaos:mac-accents joins grp:alt_shift_toggle in a French and Russian keymap"

  dir="$(mktemp -d)"
  install -m 0755 /dev/stdin "$dir/systemctl" <<'SH'
#!/bin/sh
# Answers the unit-file query and logs the rest.
case "$1" in
  list-unit-files) echo "keyd.service enabled enabled" ;;
  *) echo "$*" >>"$(dirname "$0")/calls" ;;
esac
SH
  chown -R "$USER_NAME:" "$dir"
  out="$(kz "export PATH='$dir':\$PATH XDG_CONFIG_HOME='$dir'; kubaos keyboard accents off && kubaos keyboard accents" 2>&1)" ||
    { echo "FAIL kubaos keyboard accents off: $out"; rm -rf "$dir"; return 1; }
  if [[ $(tail -1 <<<"$out") != off ]] || ! grep -qx 'mac_accents = false' "$dir/kubaos/desktop/keyboard/keyboard.toml" ||
    ! grep -qx 'stop --no-ask-password keyd.service' "$dir/calls"; then
    echo "FAIL accents off: printed $out, wrote $(cat "$dir/kubaos/desktop/keyboard/keyboard.toml" 2>&1), ran $(cat "$dir/calls" 2>&1)"
    rm -rf "$dir"
    return 1
  fi
  out="$(kz "PATH='$dir':\$PATH XDG_CONFIG_HOME='$dir' kubaos keyboard accents on" 2>&1)" ||
    { echo "FAIL kubaos keyboard accents on: $out"; rm -rf "$dir"; return 1; }
  if ! grep -qx 'mac_accents = true' "$dir/kubaos/desktop/keyboard/keyboard.toml" ||
    [[ $(tail -1 "$dir/calls") != 'start --no-ask-password keyd.service' ]]; then
    echo "FAIL accents on: wrote $(cat "$dir/kubaos/desktop/keyboard/keyboard.toml"), ran $(cat "$dir/calls")"
    rm -rf "$dir"
    return 1
  fi
  rm -rf "$dir"
  echo "ok   kubaos keyboard accents off and on write the setting and stop and start keyd"

  as_user lua5.5 "$REPO/test/lua/accents_test.lua" "$REPO" || { echo "FAIL test/lua/accents_test.lua"; return 1; }
  echo "ok   accents.lua: the cheat sheet in a submap, gone with the setting off"
}

c_quickshell() {
  local out t
  session_start sway 1920x1080 1
  if ! qs_start; then
    echo "FAIL Quickshell didn't load its config:"
    tail -30 "$QS_LOG"
    session_stop
    return 1
  fi
  sleep 5   # let the panels, services and the wallpaper settle
  local ipc
  ipc="$(in_session 'qs ipc show' 2>&1 || true)"
  # The Wi-Fi and Bluetooth tabs, as the Command menu opens them: each scans while it shows.
  in_session 'qs ipc call controlcenter wifi' >/dev/null 2>&1
  sleep 1
  in_session 'qs ipc call controlcenter bluetooth' >/dev/null 2>&1
  sleep 1
  in_session 'qs ipc call controlcenter toggle' >/dev/null 2>&1
  sleep 1
  out="$(qs_problems)"
  session_stop
  if [[ -n $out ]]; then
    echo "FAIL QML errors/warnings ($QS_LOG):"
    printf '%s\n' "$out" | head -30
    return 1
  fi
  echo "ok   Quickshell loaded with no QML errors, the Wi-Fi and Bluetooth tabs opened and closed"
  echo "     environment warnings (services a container doesn't have):"
  qs_env_warnings | cut -c1-120 | sed 's/^/       /'
  
  for t in session powermenu wallpaper notifications controlcenter recorder clock sysmon; do
    grep -qE "target $t\b" <<<"$ipc" || { echo "FAIL qs ipc show lacks target $t"; printf '%s\n' "$ipc"; return 1; }
  done
  echo "ok   qs ipc show lists every contract target"
}

# The clock popup: it opens under the bar's clock through the clock IPC, its
# engine (kubaos-clock serve) runs behind it, and a search sent to it is
# answered with no QML errors on the way. Esc closes it.
c_clock() {
  local out bad=0 i
  session_start sway 1920x1080 1
  if ! qs_start; then
    echo "FAIL Quickshell didn't load its config:"
    tail -30 "$QS_LOG"
    session_stop
    return 1
  fi
  sleep 3

  out="$(in_session 'kubaos-clock eval "days until christmas"' 2>&1)" || { echo "FAIL kubaos-clock eval: $out"; bad=1; }
  [[ $out == [0-9]*" day"* ]] && echo "ok   kubaos-clock answers 'days until christmas': ${out%%$'\n'*}" ||
    { echo "FAIL kubaos-clock eval printed: $out"; bad=1; }

  in_session 'qs ipc call clock toggle' >/dev/null 2>&1
  for i in $(seq 30); do
    out="$(in_session 'qs ipc call clock status' 2>&1 || true)"
    [[ $out == *'"open":true'* && $out == *'"available":true'* ]] && break
    sleep 0.2
  done
  if [[ $out == *'"open":true'* && $out == *'"available":true'* ]]; then
    echo "ok   qs ipc call clock toggle opens the popup, its engine running"
  else
    echo "FAIL the popup isn't open with its engine: $out"
    bad=1
  fi

  in_session "qs ipc call clock search 'next friday'" >/dev/null 2>&1
  sleep 1
  in_session 'qs ipc call clock toggle' >/dev/null 2>&1
  for i in $(seq 30); do
    out="$(in_session 'qs ipc call clock status' 2>&1 || true)"
    [[ $out == *'"open":false'* ]] && break
    sleep 0.2
  done
  [[ $out == *'"open":false'* ]] && echo "ok   toggling again closes it" || { echo "FAIL the popup stayed open: $out"; bad=1; }

  out="$(qs_problems)"
  session_stop
  if [[ -n $out ]]; then
    echo "FAIL QML errors/warnings with the popup open ($QS_LOG):"
    printf '%s\n' "$out" | head -30
    bad=1
  fi
  ((bad)) || echo "ok   no QML errors with the popup open"
  return "$bad"
}

# The system monitor: kubaos-sysmon reads the container's /proc with no GPU,
# battery or fan, the shell's stream feeds the pill, the panel opens through
# the sysmon IPC with the engine sampling in detail, and the pill hides and
# comes back through the IPC, with no QML errors on the way.
c_sysmon() {
  local out bad=0 i
  out="$(zrun 'kubaos-sysmon sample | jq -r '\''select(.type == "sample" and .cpu.count > 0 and .memory.total > 0 and (.top.cpu | length) > 0) | "\(.cpu.count) cores, RAM \(.memory.percent | floor)%, \(.gpus | length) GPUs, battery \(.battery != null)"'\''' 2>&1)" || true
  if [[ $out == *" cores, RAM "* ]]; then
    echo "ok   kubaos-sysmon sample reads the machine: $out"
  else
    echo "FAIL kubaos-sysmon sample has no CPU, memory or processes: ${out:0:300}"
    bad=1
  fi

  session_start sway 1920x1080 1
  if ! qs_start; then
    echo "FAIL Quickshell didn't load its config:"
    tail -30 "$QS_LOG"
    session_stop
    return 1
  fi
  for i in $(seq 30); do
    out="$(in_session 'qs ipc call sysmon status' 2>&1 || true)"
    [[ $out == *'"sample":true'* && $out == *'"mode":"pill"'* ]] && break
    sleep 0.2
  done
  if [[ $out == *'"available":true'* && $out == *'"sample":true'* && $out == *'"mode":"pill"'* ]]; then
    echo "ok   the shell's stream samples for the pill: $out"
  else
    echo "FAIL no pill samples from the stream: $out"
    bad=1
  fi
  if [[ -s "$(in_session 'printf %s "$XDG_RUNTIME_DIR"')/kubaos-sysmon.json" ]]; then
    echo "ok   the stream file the menu reads is written"
  else
    echo "FAIL no stream file in \$XDG_RUNTIME_DIR"
    bad=1
  fi

  in_session 'qs ipc call sysmon toggle' >/dev/null 2>&1
  for i in $(seq 30); do
    out="$(in_session 'qs ipc call sysmon status' 2>&1 || true)"
    [[ $out == *'"open":true'* && $out == *'"detail":true'* ]] && break
    sleep 0.2
  done
  if [[ $out == *'"open":true'* && $out == *'"mode":"popup"'* && $out == *'"detail":true'* ]]; then
    echo "ok   qs ipc call sysmon toggle opens the panel, sampling in detail"
  else
    echo "FAIL the panel isn't open with detailed samples: $out"
    bad=1
  fi
  in_session 'qs ipc call sysmon toggle' >/dev/null 2>&1
  for i in $(seq 30); do
    out="$(in_session 'qs ipc call sysmon status' 2>&1 || true)"
    [[ $out == *'"open":false'* && $out == *'"mode":"pill"'* ]] && break
    sleep 0.2
  done
  [[ $out == *'"open":false'* && $out == *'"mode":"pill"'* ]] && echo "ok   toggling again closes it, back to the pill's pace" ||
    { echo "FAIL the panel stayed open or the pace stayed fast: $out"; bad=1; }

  in_session 'qs ipc call sysmon pill off' >/dev/null 2>&1
  for i in $(seq 30); do
    out="$(in_session 'qs ipc call sysmon status' 2>&1 || true)"
    [[ $out == *'"pill":false'* ]] && break
    sleep 0.2
  done
  if [[ $out == *'"pill":false'* ]] && grep -qx 'pill = false' "$USER_HOME/.config/kubaos/desktop/sysmon/sysmon.toml"; then
    echo "ok   qs ipc call sysmon pill off hides it and writes sysmon.toml"
  else
    echo "FAIL the pill didn't hide: $out"
    bad=1
  fi
  zrun "kubaos-sysmon set pill on" >/dev/null 2>&1
  for i in $(seq 30); do
    out="$(in_session 'qs ipc call sysmon status' 2>&1 || true)"
    [[ $out == *'"pill":true'* ]] && break
    sleep 0.2
  done
  [[ $out == *'"pill":true'* ]] && echo "ok   kubaos-sysmon set pill on, as the menu runs it, shows it again" ||
    { echo "FAIL the pill stayed hidden after kubaos-sysmon set pill on: $out"; bad=1; }

  out="$(qs_problems)"
  session_stop
  if [[ -n $out ]]; then
    echo "FAIL QML errors/warnings with the monitor ($QS_LOG):"
    printf '%s\n' "$out" | head -30
    bad=1
  fi
  ((bad)) || echo "ok   no QML errors with the pill and the panel"
  return "$bad"
}

# toast_wait DESC PATTERN: up to 5 s for the toast probe to log, after line
# $TOAST_MARK of the log, a line matching PATTERN (ERE).
toast_wait() {
  for _ in $(seq 50); do
    if tail -n +"$TOAST_MARK" "$QS_LOG" | grep -q -E -- "toast probe: .*$2"; then
      echo "ok   $1"
      return 0
    fi
    sleep 0.1
  done
  echo "FAIL $1: the probe logged no /$2/; it logged:"
  tail -n +"$TOAST_MARK" "$QS_LOG" | grep -o 'toast probe: .*' | tail -3 | sed 's/^/     /'
  return 1
}

# toast_overlaps: every probe line where a stack drawing cards meets an open
# panel's rectangle (empty when none did).
toast_overlaps() {
  grep -o 'toast probe: .*' "$QS_LOG" | awk '{
    np = 0
    for (i = 3; i <= NF; i++) {
      split($i, kv, "="); split(kv[2], v, ",")
      if (kv[1] == "panel") { np++; px[np] = v[1]; py[np] = v[2]; pw[np] = v[3]; ph[np] = v[4] }
    }
    for (i = 3; i <= NF; i++) {
      split($i, kv, "="); split(kv[2], v, ",")
      if (kv[1] != "stack" || v[2] == 0) continue
      for (j = 1; j <= np; j++)
        if (v[3] < px[j] + pw[j] && px[j] < v[3] + v[5] && v[4] < py[j] + ph[j] && py[j] < v[4] + v[6]) { print; next }
    }
  }'
}

# Notification popups and the panels in their corner, through the toast probe
# (config/quickshell/modules/notifications/ToastProbe.qml), which logs the
# stacks' and panels' rectangles as they change. A toast on screen when the
# system monitor opens leaves the corner, one sent while it is open waits, and
# so does a critical one, as the panel reaches the bottom edge; under the power
# menu, which leaves room below it, the critical one shows. Closing it brings
# all three back to the corner. At no logged frame, the glides included, does a
# stack drawing cards meet an open panel.
c_notifications() {
  local out bad=0 i frames
  session_start sway 1920x1080 1
  if ! qs_start KUBAOS_TOAST_PROBE=1; then
    echo "FAIL Quickshell didn't load its config:"
    tail -30 "$QS_LOG"
    session_stop
    return 1
  fi
  sleep 3

  TOAST_MARK=$(($(wc -l <"$QS_LOG") + 1))
  in_session "notify-send -t 60000 Earlier 'On screen before the panel opens'" >/dev/null 2>&1
  toast_wait "a toast shows in the corner" 'stack=top,1,' || bad=1

  TOAST_MARK=$(($(wc -l <"$QS_LOG") + 1))
  in_session 'qs ipc call sysmon toggle' >/dev/null 2>&1
  for i in $(seq 30); do
    out="$(in_session 'qs ipc call sysmon status' 2>&1 || true)"
    [[ $out == *'"open":true'* ]] && break
    sleep 0.2
  done
  sleep 1
  out="$(tail -n +"$TOAST_MARK" "$QS_LOG" | grep -o 'toast probe: panel=.*' | head -1)"
  if [[ $out == *' stack=top,0,'* ]]; then
    echo "ok   the system monitor opening sends it away at once"
  else
    echo "FAIL the toast was still drawn in the panel's first frame: $out"
    bad=1
  fi

  in_session "notify-send -t 60000 Held 'Sent while the panel is open'" >/dev/null 2>&1
  sleep 1.5
  out="$(grep -o 'toast probe: .*' "$QS_LOG" | tail -1)"
  if [[ $out == *panel=*' stack=top,0,'*' stack=right,0,'* ]]; then
    echo "ok   one sent while the panel is open waits"
  else
    echo "FAIL a toast showed with the panel open: $out"
    bad=1
  fi

  in_session "notify-send -u critical Critical 'Shows under the power menu'" >/dev/null 2>&1
  sleep 1.5
  out="$(grep -o 'toast probe: .*' "$QS_LOG" | tail -1)"
  if [[ $out == *panel=*' stack=top,0,'*' stack=right,0,'* ]]; then
    echo "ok   a critical one waits too, with no room under the full-height panel"
  else
    echo "FAIL a critical toast showed with the full-height panel open: $out"
    bad=1
  fi

  TOAST_MARK=$(($(wc -l <"$QS_LOG") + 1))
  in_session 'qs ipc call powermenu toggle' >/dev/null 2>&1
  toast_wait "the critical one shows under the power menu" 'panel=.* stack=top,0,.* stack=right,1,' || bad=1

  TOAST_MARK=$(($(wc -l <"$QS_LOG") + 1))
  in_session 'qs ipc call powermenu toggle' >/dev/null 2>&1
  toast_wait "closing the power menu brings all three back to the corner" 'stack=top,3,.* stack=right,0,' || bad=1

  # The notifications panel: the three are unread until it has shown them, and it covers the
  # corner as the other panels do.
  out="$(in_session 'qs ipc call notifications status' 2>&1 || true)"
  if [[ $out == *'"count":3'*'"unread":3'* ]]; then
    echo "ok   the three are unread before the panel shows them"
  else
    echo "FAIL the status before the panel: $out"
    bad=1
  fi
  TOAST_MARK=$(($(wc -l <"$QS_LOG") + 1))
  in_session 'qs ipc call notifications toggle' >/dev/null 2>&1
  toast_wait "the notifications panel sends the toasts away" 'panel=.* stack=top,0,' || bad=1
  in_session 'qs ipc call notifications toggle' >/dev/null 2>&1
  sleep 1
  out="$(in_session 'qs ipc call notifications status' 2>&1 || true)"
  if [[ $out == *'"count":3'*'"unread":0'*'"open":false'* ]]; then
    echo "ok   closing the panel marks what it showed read and keeps it"
  else
    echo "FAIL the status after the panel: $out"
    bad=1
  fi

  # Do not disturb: on by hand, a new one lands in the list with no popup, and the state file
  # the Command menu reads says so.
  in_session 'qs ipc call notifications clear' >/dev/null 2>&1
  in_session 'qs ipc call notifications dnd on' >/dev/null 2>&1
  sleep 1
  TOAST_MARK=$(($(wc -l <"$QS_LOG") + 1))
  in_session "notify-send -t 60000 Quiet 'Sent while Do not disturb is on'" >/dev/null 2>&1
  sleep 1.5
  out="$(in_session 'qs ipc call notifications status' 2>&1 || true)"
  local dnd_file
  dnd_file="$(in_session 'cat "$XDG_RUNTIME_DIR/kubaos-dnd.json"' 2>&1 || true)"
  if [[ $out == *'"count":1'*'"dnd":true'*'"reason":"manual"'* ]] && [[ $dnd_file == *'"on":true'* ]] \
    && ! tail -n +"$TOAST_MARK" "$QS_LOG" | grep -q 'stack=top,[1-9]'; then
    echo "ok   Do not disturb holds the popup back, keeps the notification and writes its state"
  else
    echo "FAIL Do not disturb: $out, state file $dnd_file"
    bad=1
  fi
  TOAST_MARK=$(($(wc -l <"$QS_LOG") + 1))
  in_session "notify-send -u critical Urgent 'Breaks through Do not disturb'" >/dev/null 2>&1
  toast_wait "a critical one pops up through Do not disturb" 'stack=top,1,' || bad=1
  in_session 'qs ipc call notifications dnd off' >/dev/null 2>&1
  sleep 0.5
  dnd_file="$(in_session 'cat "$XDG_RUNTIME_DIR/kubaos-dnd.json"' 2>&1 || true)"
  if [[ $dnd_file == *'"on":false'* ]]; then
    echo "ok   Do not disturb off writes its state"
  else
    echo "FAIL Do not disturb off: state file $dnd_file"
    bad=1
  fi

  out="$(toast_overlaps)"
  frames="$(grep -c 'toast probe: panel=' "$QS_LOG")"
  if [[ -n $out ]]; then
    echo "FAIL a toast drew over an open panel:"
    head -5 <<<"$out" | sed 's/^/     /'
    bad=1
  elif ((frames < 5)); then
    echo "FAIL the probe logged the panels open in $frames frames only"
    bad=1
  else
    echo "ok   no toast met a panel in any of $frames frames with one on screen"
  fi

  out="$(qs_problems)"
  session_stop
  if [[ -n $out ]]; then
    echo "FAIL QML errors/warnings with the toasts ($QS_LOG):"
    printf '%s\n' "$out" | head -30
    bad=1
  fi
  ((bad)) || echo "ok   no QML errors with the toasts and the panels"
  return "$bad"
}

# stub_screenshot_tools DIR: stand-ins for what dot-hypr-screenshot calls, in DIR. Two monitors
# (1920x1200 at scale 1.5, 1920x1080 at 1), so grim's frame is 5760x1800 with a red 150x75 box at
# pixel 300,150. Every call is logged to DIR/calls; slurp answers 200,100 100x50 (that box in
# logical coordinates) or cancels when DIR/cancel exists.
stub_screenshot_tools() {
  local d=$1 tool
  for tool in notify-send xdg-user-dir; do printf '#!/bin/sh\nexit 0\n' >"$d/$tool"; done
  cat >"$d/hyprctl" <<'STUB'
#!/bin/bash
D=$(dirname "$0")
case "$*" in
  "-j monitors") echo '[{"x":0,"y":0,"scale":1.5},{"x":1920,"y":0,"scale":1}]' ;;
  "-j layers")
    if [[ -e $D/picker.pid ]]; then
      echo '{"A":{"levels":{"3":[{"namespace":"hyprpicker"}]}},"B":{"levels":{"3":[{"namespace":"hyprpicker"}]}}}'
    else
      echo '{}'
    fi ;;
esac
STUB
  cat >"$d/grim" <<'STUB'
#!/bin/bash
D=$(dirname "$0")
echo "grim $*" >>"$D/calls"
magick -size 5760x1800 xc:white -fill red -draw 'rectangle 300,150 449,224' "${@: -1}"
STUB
  cat >"$d/hyprpicker" <<'STUB'
#!/bin/bash
D=$(dirname "$0")
echo "hyprpicker $*" >>"$D/calls"
trap 'echo ended >"$D/picker.ended"; rm -f "$D/picker.pid"; exit 0' TERM
echo $$ >"$D/picker.pid"
while :; do sleep 0.05; done
STUB
  cat >"$d/slurp" <<'STUB'
#!/bin/bash
D=$(dirname "$0")
echo "slurp $*" >>"$D/calls"
[[ ! -e $D/picker.pid ]] || echo "slurp saw the freeze" >>"$D/calls"
[[ ! -e $D/cancel ]] || exit 1
echo "200,100 100x50"
STUB
  for tool in wl-copy satty; do
    cat >"$d/$tool" <<'STUB'
#!/bin/bash
{ printf '%s' "$(basename "$0")"; printf ' [%s]' "$@"; echo; } >>"$(dirname "$0")/calls"
STUB
  done
  chmod +x "$d"/*
}

wait_for_file() {
  local _
  for _ in $(seq 40); do [[ -e $1 ]] && return; sleep 0.05; done
  return 1
}

c_screenshot() {
  local t dir bad=0 rc shot
  t="$(mktemp -d)"
  mkdir "$t/bin"
  stub_screenshot_tools "$t/bin"
  run_shot() { PATH="$t/bin:$PATH" HOME="$t" DOT_SCREENSHOT_DIR="$t/shots" bash "$REPO/scripts/dot-hypr-screenshot" "$@"; }

  run_shot region || { echo "FAIL region exited non-zero"; bad=1; }
  shot="$(ls "$t"/shots/Screenshot_*.png 2>/dev/null | head -1)"
  [[ -n $shot ]] && [[ "$(magick "$shot" -format '%wx%h %[pixel:p{0,0}] %[pixel:p{149,74}]' info:)" == "150x75 srgb(255,0,0) srgb(255,0,0)" ]] \
    && echo "ok   region crops the selection at scale 1.5 from the frame taken before slurp" \
    || { echo "FAIL region crop: ${shot:-no file}"; bad=1; }
  grep -q 'hyprpicker --render-inactive --no-zoom' "$t/bin/calls" && grep -q 'slurp saw the freeze' "$t/bin/calls" \
    && echo "ok   hyprpicker's overlay is up when slurp starts" || { echo "FAIL freeze not up before slurp"; bad=1; }
  wait_for_file "$t/bin/picker.ended" && echo "ok   the trap ended the freeze" || { echo "FAIL hyprpicker was left running"; bad=1; }
  grep -q '^wl-copy \[--type\] \[image/png\]' "$t/bin/calls" && echo "ok   wl-copy gets the image type" || { echo "FAIL wl-copy without a type"; bad=1; }

  rm -f "$t/bin/picker.ended" "$t/bin/calls"
  rm -rf "$t/shots"
  touch "$t/bin/cancel"
  rc=0
  run_shot region || rc=$?
  [[ $rc == 0 && -z "$(compgen -G "$t/shots/Screenshot_*")" ]] && wait_for_file "$t/bin/picker.ended" && ! grep -q '^wl-copy' "$t/bin/calls" \
    && echo "ok   Esc ends the freeze and saves nothing" || { echo "FAIL cancel (exit $rc)"; bad=1; }

  rm -f "$t/bin/picker.ended" "$t/bin/cancel" "$t/bin/calls"
  mkdir "$t/failing"
  printf '#!/bin/sh\ncase "$*" in *-crop*) exit 1 ;; esac\nPATH=${PATH#*failing:}\nexec magick "$@"\n' >"$t/failing/magick"
  chmod +x "$t/failing/magick"
  rc=0
  PATH="$t/failing:$t/bin:$PATH" HOME="$t" DOT_SCREENSHOT_DIR="$t/shots" bash "$REPO/scripts/dot-hypr-screenshot" region 2>/dev/null || rc=$?
  [[ $rc != 0 ]] && wait_for_file "$t/bin/picker.ended" && echo "ok   an error ends the freeze too" || { echo "FAIL error path (exit $rc)"; bad=1; }

  rm -f "$t/bin/calls"
  run_shot region --edit
  grep -q '^satty .*\[--copy-command\] \[wl-copy --type image/png\]' "$t/bin/calls" \
    && echo "ok   satty copies through 'wl-copy --type image/png'" || { echo "FAIL satty copy command"; bad=1; }

  rm -rf "$t"
  return "$bad"
}

# dot-hypr-record's start/stop and state file, with wf-recorder (gsr needs a
# GPU and KMS), stopped through Quickshell's IPC like the bar's click does.
c_recorder() {
  local st dir="$USER_HOME/Videos/Recordings" state f bad=0
  st="/run/user/$(id -u "$USER_NAME")/dot-record.json"
  session_start sway 1280x720 1
  if ! qs_start; then
    echo "FAIL Quickshell didn't load"; session_stop; return 1
  fi
  sleep 2
  [[ -e $st ]] && echo "ok   Quickshell created the state file" || { echo "FAIL no $st after Quickshell start"; bad=1; }
  expect_ok "dot-hypr-record output starts" in_session 'DOT_RECORD_BACKEND=wf-recorder dot-hypr-record output' || bad=1
  sleep 3
  state="$(jq -r .state "$st" 2>/dev/null)"
  [[ $state == recording ]] && echo "ok   state file says recording" || { echo "FAIL state is '$state'"; cat "${st%/*}/dot-record.log" 2>/dev/null; bad=1; }
  expect_ok "qs ipc call recorder stop" in_session 'qs ipc call recorder stop' || bad=1
  for _ in $(seq 50); do [[ "$(jq -r .state "$st" 2>/dev/null)" == idle ]] && break; sleep 0.2; done
  state="$(jq -r .state "$st" 2>/dev/null)"
  [[ $state == idle ]] && echo "ok   state back to idle" || { echo "FAIL state stayed '$state'"; bad=1; }
  f="$(find "$dir" -name 'Recording_*.mp4' -size +1k 2>/dev/null | head -1)"
  [[ -n $f ]] && echo "ok   saved ${f#"$USER_HOME"/} ($(stat -c %s "$f") bytes)" || { echo "FAIL no recording in $dir"; ls -la "$dir" 2>&1; bad=1; }
  if in_session 'dot-hypr-record replay save' >/dev/null 2>&1; then
    echo "FAIL replay save succeeded with no replay running"; bad=1
  else
    echo "ok   replay save refuses when no replay runs"
  fi
  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  session_stop
  return "$bad"
}

# A switch run from a terminal in the session reloads or restarts only what
# its new defaults changed (scripts/dot-session). Sway has no Hyprland, so a
# stand-in hyprctl (test/lib/hyprctl-stub) in /usr/local/bin, which comes
# before the home's and apt's hyprctl on activation's PATH, runs what Hyprland
# would exec.
RELOAD_DIR=/var/tmp/dotfiles-test-hyprctl
RELOAD_CALLS=$RELOAD_DIR/calls.log
RELOAD_STUB=/usr/local/bin/hyprctl

# reload_switch: home-manager switch as kuba, the stand-in's calls in
# $RELOAD_CALLS.
reload_switch() {
  : >"$RELOAD_CALLS"
  zrun ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh; \
    export HYPRCTL_STUB_LOG=$RELOAD_CALLS HYPRCTL_STUB_ENV=$SESSION_ENV HYPRCTL_STUB_OUT=$QS_LOG; \
    home-manager switch -b hm-backup --flake '$REPO#$USER_NAME@vm'" \
    >>"$LOGS/session-reload-switch.log" 2>&1
}

# reload_edit FILE MARK: a comment line, MARK and a timestamp, appended to FILE
# in the repo copy.
reload_edit() {
  as_user sh -c "printf '\n%s e2e %s\n' '$2' \"\$(date +%s%N)\" >>'$REPO/$1'"
}

# qs_pids: kuba's Quickshell processes, space separated.
qs_pids() { pgrep -u "$USER_NAME" -x 'qs|quickshell|\.quickshell-wra' | paste -sd ' ' || true; }

c_session_reload() {
  local pid now bad=0
  [[ -e $RELOAD_STUB ]] && { echo "FAIL $RELOAD_STUB is taken"; return 1; }
  install -d -o "$USER_NAME" "$RELOAD_DIR"
  ln -s "$SRC/test/lib/hyprctl-stub" "$RELOAD_STUB"
  : >"$RELOAD_CALLS"
  chown "$USER_NAME:" "$RELOAD_CALLS"
  : >"$LOGS/session-reload-switch.log"
  chown "$USER_NAME:" "$LOGS/session-reload-switch.log"
  session_start sway 1280x720 1
  if ! qs_start; then
    echo "FAIL Quickshell didn't load"; session_stop; rm -f "$RELOAD_STUB"; return 1
  fi
  pid="$(qs_pids)"
  now="$(in_session 'qs ipc call session locked' 2>/dev/null || true)"
  [[ $now == false ]] && echo "ok   qs ipc call session locked says false" ||
    { echo "FAIL qs ipc call session locked said '$now'"; bad=1; }

  reload_edit config/bat/config '#'
  reload_switch || { echo "FAIL switch with a new bat default"; tail -20 "$LOGS/session-reload-switch.log"; bad=1; }
  sleep 2
  if [[ -s $RELOAD_CALLS ]] && grep -qv '^instances' "$RELOAD_CALLS"; then
    echo "FAIL a switch changing only bat reached Hyprland:"; cat "$RELOAD_CALLS"; bad=1
  fi
  now="$(qs_pids)"
  [[ $now == "$pid" ]] && echo "ok   a switch leaving the shell's defaults alone leaves Quickshell ($pid) running" ||
    { echo "FAIL Quickshell went from '$pid' to '$now' on a bat change"; bad=1; }

  reload_edit config/quickshell/shell.qml '//'
  reload_switch || { echo "FAIL switch with a new shell default"; tail -20 "$LOGS/session-reload-switch.log"; bad=1; }
  grep -q 'dispatch .*dot-session.* restart-shell' "$RELOAD_CALLS" && echo "ok   the switch asked Hyprland to restart the shell" ||
    { echo "FAIL no restart-shell dispatch:"; cat "$RELOAD_CALLS"; bad=1; }
  grep -qx -- '-i e2e reload' "$RELOAD_CALLS" && { echo "FAIL Hyprland reloaded with its config unchanged"; bad=1; }
  for _ in $(seq 100); do
    now="$(qs_pids)"
    [[ -n $now && $now != "$pid" && $(grep -c 'Configuration Loaded' "$QS_LOG") -ge 2 ]] && break
    sleep 0.2
  done
  [[ -n $now && $now != "$pid" && $now != *' '* ]] && echo "ok   Quickshell restarted on its new defaults ($pid -> $now)" ||
    { echo "FAIL Quickshell is '$now', was '$pid'"; tail -20 "$QS_LOG"; bad=1; }
  pid="$now"

  reload_edit config/hypr/looks.lua '--'
  reload_edit config/hypr/hypridle.conf '#'
  reload_switch || { echo "FAIL switch with new Hyprland defaults"; tail -20 "$LOGS/session-reload-switch.log"; bad=1; }
  grep -qx -- '-i e2e reload' "$RELOAD_CALLS" && echo "ok   a new Hyprland default reloads Hyprland" ||
    { echo "FAIL no hyprctl reload:"; cat "$RELOAD_CALLS"; bad=1; }
  grep -q 'dispatch .* restart-idle' "$RELOAD_CALLS" && echo "ok   a new hypridle default restarts hypridle" ||
    { echo "FAIL no restart-idle dispatch:"; cat "$RELOAD_CALLS"; bad=1; }
  sleep 2
  now="$(qs_pids)"
  [[ $now == "$pid" ]] && echo "ok   and leaves Quickshell running" ||
    { echo "FAIL Quickshell went from '$pid' to '$now' on a Hyprland change"; bad=1; }
  # The restarted shell logs in color: `dot-session` starts a plain `qs`.
  sed -i 's/\x1b\[[0-9;]*m//g' "$QS_LOG"
  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  session_stop
  rm -f "$RELOAD_STUB"

  as_user git -C "$REPO" checkout -q -- config
  reload_switch && echo "ok   a switch back with no Hyprland running succeeds" ||
    { echo "FAIL switch back to the repo's defaults"; tail -20 "$LOGS/session-reload-switch.log"; bad=1; }
  return "$bad"
}

# menu_stop PID: the Daemon (PID is its in_session job), its Extensions, the
# keyboard and pointer, and the session. Waiting on the job keeps the shell's
# "Terminated" out of the log.
menu_stop() {
  pkill -u "$USER_NAME" -x kotodamad 2>/dev/null || true
  pkill -u "$USER_NAME" -f '/kotodama-[a-z-]+$' 2>/dev/null || true
  pkill -u "$USER_NAME" -f 'vinput\.py serve' 2>/dev/null || true
  wait "$1" 2>/dev/null || true
  session_stop
}

# menu_example [--remove]: Kotodama's example shell Extension, from the
# Kotodama in flake.lock, as the user's own Extension: bookmarks in a list
# View, a timer with its forms, its job and its progress View, an app with an
# intro card. --remove takes it away again.
menu_example() {
  local src ext="$USER_HOME/.config/kotodama/extensions"
  rm -rf "$ext/shell-example" "$USER_HOME/.config/kotodama/shell-example.toml"
  [[ ${1:-} == --remove ]] && return 0
  src="$(as_user "$NIX_BIN/nix" eval --raw --impure \
    --expr "(builtins.getFlake \"path:$REPO\").inputs.kotodama.outPath")" || return 1
  as_user mkdir -p "$ext"
  as_user cp -r "$src/examples/shell-extension" "$ext/shell-example"
  as_user chmod -R u+w "$ext/shell-example"
  as_user tee "$USER_HOME/.config/kotodama/shell-example.toml" >/dev/null <<'EOF'
[extensions.shell-example]
bookmarks = [
  { title = "Kotodama", url = "https://github.com/SuplenC/kotodama" },
  { title = "Quickshell", url = "https://quickshell.org" },
  { title = "Hyprland wiki", url = "https://wiki.hypr.land" },
]
EOF
}

# menu_up SIZE: a session at SIZE, the Daemon (its job in MENU_DAEMON), the
# keyboard, and Quickshell with the menu's probe, which logs `menu probe: ...`
# lines (config/quickshell/modules/menu/Probe.qml), once the menu's search has
# reached the Daemon, which `kotodama toggle` needs.
menu_up() {
  session_start sway "$1" 1
  in_session "exec kotodamad >>$LOGS/kotodamad.log 2>&1" &
  MENU_DAEMON=$!
  input_up "$1" || return 1
  if ! qs_start KUBAOS_MENU_PROBE=1; then
    echo "FAIL Quickshell didn't load:"
    tail -30 "$QS_LOG"
    return 1
  fi
  menu_now 'connected true' || { echo "FAIL the menu's search never reached the Daemon"; return 1; }
}

# menu_until PATTERN CMD...: CMD, then up to PROBE_WAIT for the probe to log a
# line matching PATTERN (ERE) after it.
menu_until() {
  local pattern="$1" mark
  shift
  mark=$(($(wc -l <"$QS_LOG") + 1))
  "$@" >>"$LOGS/menu-steps.log" 2>&1 || { echo "FAIL $* failed: $(tail -3 "$LOGS/menu-steps.log")"; return 1; }
  for _ in $(seq $((PROBE_WAIT * 10))); do
    tail -n +"$mark" "$QS_LOG" | grep -q -E -- "menu probe: $pattern" && return 0
    sleep 0.1
  done
  return 1
}

# menu_now STATE: up to PROBE_WAIT for the probe's latest line of STATE's kind
# (its first word) to be STATE: how the menu is now, however long ago it got so.
menu_now() {
  for _ in $(seq $((PROBE_WAIT * 10))); do
    [[ $(grep -o "menu probe: ${1%% *} .*" "$QS_LOG" | tail -1) == "menu probe: $1" ]] && return 0
    sleep 0.1
  done
  return 1
}

# menu_step DESC PATTERN CMD...: CMD, then up to PROBE_WAIT for the probe to
# log a line matching PATTERN (ERE).
menu_step() {
  local desc="$1" pattern="$2"
  shift 2
  if menu_until "$pattern" "$@"; then
    echo "ok   $desc"
    return 0
  fi
  echo "FAIL $desc: the probe logged no /$pattern/; it logged:"
  grep -o 'menu probe: .*' "$QS_LOG" | tail -5 | sed 's/^/     /'
  return 1
}

# menu_keys PLACE: up to PROBE_WAIT for the menu's keys to go to PLACE
# (`search`, `filter`, `form`), so nothing typed next lands elsewhere.
menu_keys() {
  menu_now "keys $1" && return 0
  echo "FAIL the menu's keys never went to its $1: $(grep -o 'menu probe: keys.*' "$QS_LOG" | tail -1)"
  return 1
}

# menu_typed QUERY STEP...: the keys (`input`), then up to PROBE_WAIT for the
# Results for QUERY to reach the screen; what a step typed after it acts on.
menu_typed() {
  local query="$1"
  shift
  menu_until "frame query=\"$query\" " input "$@" && return 0
  echo "FAIL the Results for \"$query\" never reached the screen; the probe logged:"
  grep -o 'menu probe: .*' "$QS_LOG" | tail -5 | sed 's/^/     /'
  return 1
}

# menu_kotodama ARGS...: the `kotodama` command in the session.
menu_kotodama() { in_session "kotodama $*"; }

# menu_layout PANE MIN_ROWS: the window's last layout has the pane or not, and
# at least MIN_ROWS rows.
menu_layout() {
  local line rows
  line="$(grep -o 'menu probe: layout .*' "$QS_LOG" | tail -1)"
  rows="$(sed -n 's/.* rows=\([0-9]*\).*/\1/p' <<<"$line")"
  if [[ $line == *" pane=$1" && ${rows:-0} -ge $2 ]]; then
    echo "ok   ${line#menu probe: }"
  else
    echo "FAIL want pane=$1 and at least $2 rows, got '${line#menu probe: }'"
    return 1
  fi
}

# menu_latency: the median of Kotodama's latency (query to Results in the
# model) and of query to the frame with them on screen, over what was typed.
menu_latency() {
  local frames
  frames="$(grep -o 'menu probe: frame .*' "$QS_LOG")"
  [[ -n $frames ]] || { echo "FAIL the probe timed no frames"; return 1; }
  median() { grep -o "$1=[0-9]*" | cut -d= -f2 | sort -n | awk '{a[NR]=$1} END {print a[int((NR+1)/2)]}'; }
  echo "ok   $(wc -l <<<"$frames") keystrokes: median latency_us=$(median latency_us <<<"$frames") pixels_ms=$(median pixels_ms <<<"$frames")"
}

# The Command menu: Kotodama's Daemon, started by hand as its socket unit
# would, and KubaOS's own menu inside Quickshell, driven by the session's
# keyboard (test/lib/vinput.py). `kotodama toggle` succeeds only when a menu
# answers it. KubaOS's settings reach the Daemon, which finds nothing wrong
# with them. The container has no logind and no Hyprland, so what an Extension
# reports (`extension <name>: ...`) is listed, not fatal, and the Shortcuts View
# is test/vm/menu-check's.
#
# At 1280x800 the menu has no detail pane and more rows than Kotodama's own
# menu (seven); at 1920x1080 it has the pane, and every View is walked through
# with Kotodama's example shell Extension. Each key is typed once the probe says
# where the keys go and the Results it acts on are on screen.
c_menu() {
  local out bad=0 log="$LOGS/kotodamad.log" downs=()
  for _ in {1..12}; do downs+=(key:Down); done
  : >"$log"
  chown "$USER_NAME:" "$log"
  input_tools || { echo "FAIL no pywayland or Wayland protocols from the flake's nixpkgs"; return 1; }
  menu_example || { echo "FAIL Kotodama's example Extension isn't in flake.lock's Kotodama"; return 1; }

  menu_up 1280x800 || { menu_stop "$MENU_DAEMON"; return 1; }
  menu_step "kotodama toggle opens the menu in Quickshell" 'visible true' menu_kotodama toggle || bad=1
  menu_layout false 8 || bad=1
  menu_step "kotodama toggle closes it" 'visible false' menu_kotodama toggle || bad=1
  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors at 1280x800:"; qs_problems | head; bad=1; }
  menu_stop "$MENU_DAEMON"

  menu_up 1920x1080 || { menu_stop "$MENU_DAEMON"; return 1; }
  out="$(in_session 'kotodama check' 2>&1 || true)"
  printf '%s\n' "$out" | sed 's/^/     kotodama check: /'
  if grep -v -e '^extension ' -e '^No problems\.$' <<<"$out" | grep -q .; then
    echo "FAIL the Daemon finds problems with the config"
    bad=1
  else
    echo "ok   no problems with KubaOS's settings"
  fi

  out="$(in_session 'kotodama config' 2>&1)" || { echo "FAIL kotodama config: $out"; bad=1; }
  local want
  for want in 'terminal = \["ghostty", "-e"\]  # .*/kotodama/kubaos\.toml' \
    'hidden = \[.*"org\.codeberg\.dnkl\.footclient".*\]  # .*/kotodama/kubaos\.toml' \
    'nvim = "Neovim"  # .*/kotodama/kubaos\.toml' \
    'in_search = true  # .*/kotodama/kubaos\.toml'; do
    grep -qE -- "^$want$" <<<"$out" || { echo "FAIL kotodama config lacks /$want/"; bad=1; }
  done
  ((bad)) || echo "ok   the apps' hidden, renamed apps and terminal, and the binds in the search, come from kubaos.toml"

  menu_step "kotodama toggle opens the menu" 'visible true' menu_kotodama toggle || bad=1
  menu_layout true 8 || bad=1
  menu_keys search &&
    menu_step "typing searches, and the Results reach the screen" 'frame query="tim" ' input --delay 120 type:tim || bad=1
  menu_step "Ctrl+K opens the Action panel on Timer" 'actions [2-9]' input down:ctrl type:k up:ctrl || bad=1
  menu_step "Esc closes only the Action panel" 'actions 0' input key:Escape || bad=1
  menu_step "Enter on Timer pushes its form" 'view kind=form title="Start timer" depth=1' input key:Return || bad=1
  menu_keys form &&
    menu_step "the form's Enter starts a job, shown in a progress View" \
      'view kind=progress title="2 second timer" .*job=running' input --delay 50 type:2 key:Return || bad=1
  menu_step "the job runs to the end" 'view kind=progress .*job=succeeded' : || bad=1
  menu_step "Esc pops the View" 'view kind= title="" depth=0' input key:Escape || bad=1
  menu_step "kotodama open --view opens a list View" 'view kind=list title="Bookmarks" depth=1' \
    menu_kotodama open --view shell-example/bookmarks || bad=1
  menu_keys filter && menu_step "the list View's filter narrows it" 'listed 1$' input --delay 50 type:quick || bad=1
  menu_step "Esc clears the filter first" 'listed 3$' input key:Escape || bad=1
  menu_step "Esc again leaves the list View" 'view kind= title="" depth=0' input key:Escape || bad=1
  menu_step "kotodama open --app enters an app, with its intro card" 'intro ".+"' menu_kotodama open --app shell || bad=1
  menu_keys search && menu_step "Backspace with nothing typed leaves the app" 'scope ""' input key:BackSpace || bad=1
  menu_step "Esc clears the query" 'frame query="" ' input key:Escape || bad=1
  menu_typed 'power off' --delay 50 'type:power off' &&
    menu_step "Enter on Power Off asks first" 'confirming "Power Off\? ' input key:Return || bad=1
  menu_step "Esc cancels the confirmation" 'confirming ""' input key:Escape || bad=1
  menu_typed '' key:Escape && menu_typed timer --delay 50 type:timer &&
    menu_step "Forget on Timer leaves a Toast" 'toast ".+"' input down:ctrl down:shift key:BackSpace up:shift up:ctrl || bad=1
  menu_step "Ctrl+Z undoes it" 'toast ""' input down:ctrl type:z up:ctrl || bad=1
  menu_typed '' key:Escape &&
    menu_step "a date query is answered in the top row" 'top "[0-9]+ days? · until .* 25 December ' \
      input --delay 50 'type:days until christmas' || bad=1
  menu_typed '' key:Escape && menu_typed s type:s &&
    menu_step "Down past the last row on screen scrolls the selection into view" 'selected row=12 shown=true' \
      input --delay 60 "${downs[@]}" || bad=1
  menu_step "PageUp brings the top row back into view" 'selected row=[0-4] shown=true' input key:Prior || bad=1
  menu_typed '' key:Escape && menu_step "Esc clears, and Esc again closes the menu" 'visible false' input key:Escape || bad=1
  menu_latency || bad=1

  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  if ((bad)); then tail -20 "$log"; fi
  menu_stop "$MENU_DAEMON"
  menu_example --remove
  ((bad)) || echo "ok   the menu answers kotodama toggle, takes KubaOS's settings and shows every View"
  return "$bad"
}

# The KubaOS Extension (crates/kotodama-kubaos): it runs under the
# Daemon and its Items are found by typing, with the user layer's action shown
# and the ones it hides gone. test/lib/menu-query.py is the Front end, since
# the `kotodama` command can't query. The Extension's Kotodama crates must be
# the Daemon's commit (Cargo.lock against flake.lock).
c_menu_kubaos() {
  local out bad=0 log="$LOGS/kotodamad-kubaos.log" daemon
  local layer="$USER_HOME/.config/kubaos/menu/kotodama/user.toml" saved="$LOGS/user.toml.saved"
  local query="python3 -I $REPO/test/lib/menu-query.py \$XDG_RUNTIME_DIR/kotodama.sock"

  out="$(python3 -I - "$REPO/Cargo.lock" "$REPO/flake.lock" <<'PY'
import json, sys, tomllib
lock = tomllib.load(open(sys.argv[1], "rb"))
pinned = json.load(open(sys.argv[2]))["nodes"]["kotodama"]["locked"]["rev"]
revs = {p["source"].rsplit("#", 1)[1] for p in lock["package"] if p["name"].startswith("kotodama-") and "source" in p}
if revs != {pinned}:
    sys.exit(f"Cargo.lock has Kotodama at {sorted(revs)}, flake.lock at {pinned}")
print(pinned)
PY
  )" || { echo "FAIL $out"; bad=1; }
  ((bad)) || echo "ok   the Extension's Kotodama crates are the Daemon's commit, $out"

  [[ -e $layer ]] && cp -p "$layer" "$saved"
  # The keyboard Items are the layouts of this file, which the Extension reads at start.
  local kb=/etc/default/keyboard kb_saved="$LOGS/keyboard.saved"
  [[ -e $kb ]] && cp -p "$kb" "$kb_saved"
  printf 'XKBMODEL="pc105"\nXKBLAYOUT="us,it"\nXKBVARIANT=","\nXKBOPTIONS=""\n' >"$kb"
  install -D -o "$USER_NAME" -g "$USER_NAME" -m 644 /dev/stdin "$layer" <<'TOML'
[extensions.kubaos]
hide = ["theme-next", "bluetooth"]

[extensions.kubaos.actions.harness-notes]
title = "Back up harness notes"
run = ["true"]
TOML

  # A user folder the Extension sends an Item for, and a mounted drive, as the
  # automount service lists it.
  install -d -o "$USER_NAME" -g "$USER_NAME" "$USER_HOME/Downloads"
  local drives="$RUNTIME_DIR/kubaos-drives"
  printf '/org/freedesktop/UDisks2/drives/SanDisk_Ultra_1234\tSTICK\t/media/%s/STICK\n' "$USER_NAME" >"$drives"
  chown "$USER_NAME:" "$drives"

  # Google Chrome installed: its desktop entry, as the Apps Extension finds it, and its package
  # group on. The terminal greeting on. The pairs' halves follow these files.
  local groups=/etc/kubaos/apt-groups groups_saved="$LOGS/apt-groups.saved"
  local chrome="$USER_HOME/.local/share/applications/google-chrome.desktop" chrome_saved="$LOGS/google-chrome.desktop.saved"
  local no_greeting="$USER_HOME/.config/kubaos/terminal/zsh/no-greeting" greeting_was_off=0
  local kbset="$USER_HOME/.config/kubaos/desktop/keyboard/keyboard.toml" kbset_saved="$LOGS/keyboard.toml.saved"
  [[ -e $groups ]] && cp -p "$groups" "$groups_saved"
  [[ -e $chrome ]] && cp -p "$chrome" "$chrome_saved"
  [[ -e $no_greeting ]] && greeting_was_off=1
  install -D -m 644 /dev/stdin "$groups" <<<'chrome=on'
  install -d -o "$USER_NAME" -g "$USER_NAME" "${chrome%/*}"
  install -o "$USER_NAME" -g "$USER_NAME" -m 644 /dev/stdin "$chrome" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Google Chrome
GenericName=Web Browser
Exec=true %U
DESKTOP
  # A Chrome web app: the browser's name is the first word of its desktop id, which finds it
  # for "chrome" but never ranks it with the browser.
  local webapp="${chrome%/*}/chrome-aghfnjkcajfp-Default.desktop"
  install -o "$USER_NAME" -g "$USER_NAME" -m 644 /dev/stdin "$webapp" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Harness Mail
Exec=true %U
DESKTOP
  rm -f "$no_greeting"
  session_start sway 1280x720 1
  : >"$log"
  chown "$USER_NAME:" "$log"
  in_session "exec kotodamad >$log 2>&1" &
  daemon=$!

  # The Daemon starts its Extensions when it starts; give them a moment.
  local i
  for i in $(seq 20); do
    out="$(in_session "$query Wi-Fi" 2>&1 || true)"
    grep -q 'kubaos/wifi' <<<"$out" && break
    sleep 1
  done
  if grep -q 'kubaos/wifi' <<<"$out"; then
    echo "ok   the Extension runs under the Daemon: Wi-Fi is found (${i}s)"
  else
    echo "FAIL the Wi-Fi toggle isn't found: $out"
    bad=1
  fi

  # Typing "chrome" with Chrome installed lists the app first, its removal below it, and no
  # install. Turning the group off swaps the pair, through the choices file alone.
  local ranked
  # The wait above is for the KubaOS Extension; the Apps Extension may still be scanning.
  for i in $(seq 20); do
    ranked=" $(in_session "$query chrome" 2>&1 | cut -f2 || true) "
    [[ $ranked == " apps/google-chrome "* ]] && break
    sleep 1
  done
  if [[ $ranked == " apps/google-chrome "* && $ranked == *" kubaos/packages-chrome-off "* &&
    $ranked != *" kubaos/packages-chrome-on "* ]]; then
    echo "ok   typing 'chrome' lists the Chrome app first, then its removal, and no install:$ranked"
  else
    echo "FAIL typing 'chrome' with Chrome installed lists:$ranked"
    bad=1
  fi
  if [[ $ranked == " apps/google-chrome "* && $ranked == *" apps/chrome-"* ]]; then
    echo "ok   a Chrome web app is found by its desktop id, but ranks below the browser:$ranked"
  else
    echo "FAIL typing 'chrome' ranks a web app, found by its desktop id, with the browser:$ranked"
    bad=1
  fi
  printf 'chrome=off\n' >"$groups.new"
  mv "$groups.new" "$groups"
  for i in $(seq 20); do
    ranked=" $(in_session "$query chrome" 2>&1 | cut -f2 || true) "
    [[ $ranked == *" kubaos/packages-chrome-on "* && $ranked != *" kubaos/packages-chrome-off "* ]] && break
    sleep 1
  done
  if [[ $ranked == *" kubaos/packages-chrome-on "* && $ranked != *" kubaos/packages-chrome-off "* ]]; then
    echo "ok   with the chrome group off, Install Google Chrome is listed instead of Remove (${i}s)"
  else
    echo "FAIL with the chrome group off, typing 'chrome' lists:$ranked"
    bad=1
  fi

  out="$(in_session "$query 'Back up harness notes' 'Previous wallpaper' 'Control center' 'Set volume' 'Next wallpaper' Bluetooth" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  printf '%s\n' "$out" | sed 's/^/     found: /'
  local want
  for want in 'Back up harness notes:harness-notes' 'Previous wallpaper:theme-previous' \
    'Control center:control-center' 'Set volume:volume'; do
    grep -qE -- "^${want%%:*}[[:blank:]](.* )?kubaos/${want#*:}( |$)" <<<"$out" ||
      { echo "FAIL typing '${want%%:*}' doesn't find kubaos/${want#*:}"; bad=1; }
  done
  ((bad)) || echo "ok   KubaOS's actions, the toggles and the user layer's action are found"
  if grep -qE 'kubaos/(theme-next|bluetooth)( |$)' <<<"$out"; then
    echo "FAIL the user layer hides theme-next and bluetooth, and the menu shows them"
    bad=1
  else
    echo "ok   the actions the user layer hides aren't found"
  fi

  # The settings folder and each of its groups, by title and by the word "settings".
  out="$(in_session "$query 'Settings folder' 'Settings: Desktop' 'Settings: Terminal' 'Settings: Editor' 'Settings: Tools' 'Settings: Menu' settings" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  local typed id
  for want in 'Settings folder=settings-folder' 'Settings: Desktop=settings-desktop' \
    'Settings: Terminal=settings-terminal' 'Settings: Editor=settings-editor' \
    'Settings: Tools=settings-tools' 'Settings: Menu=settings-menu' 'settings=settings-folder'; do
    typed="${want%%=*}" id="${want#*=}"
    grep -E -- "^${typed}[[:blank:]]" <<<"$out" | grep -qE -- "kubaos/${id}( |$)" ||
      { echo "FAIL typing '$typed' doesn't find kubaos/$id"; bad=1; }
  done
  ((bad)) || echo "ok   the settings folder and its five groups are found"

  out="$(in_session "$query 'Keyboard: add a layout' 'Keyboard: set layouts' Keyboard 'next layout'" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  for want in 'Keyboard: add a layout:keyboard-add' \
    'Keyboard: set layouts:keyboard-set' 'Keyboard:keyboard:us' 'Keyboard:keyboard:it'; do
    grep -E -- "^${want%%:keyboard*}[[:blank:]]" <<<"$out" | grep -qE -- "kubaos/keyboard${want#*:keyboard}( |$)" ||
      { echo "FAIL typing '${want%%:keyboard*}' doesn't find kubaos/keyboard${want#*:keyboard}"; bad=1; }
  done
  ((bad)) || echo "ok   the keyboard Items are found: add, set, and one a layout"
  # The "Add a keyboard layout" app (crates/kotodama-kubaos/layouts/): its
  # entry is found by its word, its layouts only inside it, and the ones in
  # use aren't offered.
  out="$(in_session "$query layout Czech" 2>&1)" || { echo "FAIL querying the menu: $out"; bad=1; }
  if ! grep -E '^layout[[:blank:]]' <<<"$out" | grep -qE 'kotodama/app:kubaos-layouts( |$)' ||
    grep -qE 'kubaos-layouts/' <<<"$out"; then
    echo "FAIL the layouts app's entry isn't found by 'layout', or its layouts show outside it: $out"; bad=1
  fi
  out="$(in_session "$query --app layout czech italian" 2>&1)" || { echo "FAIL entering the layouts app: $out"; bad=1; }
  if grep -E '^czech[[:blank:]]' <<<"$out" | grep -qE 'kubaos-layouts/layout:cz( |$)' &&
    grep -E '^italian[[:blank:]]' <<<"$out" | grep -qE 'kubaos-layouts/layout:it\(' &&
    ! grep -E '^italian[[:blank:]]' <<<"$out" | grep -qE 'kubaos-layouts/layout:it( |$)'; then
    echo "ok   the layouts app is entered by its word and searches the layouts not in use"
  else
    echo "FAIL in the layouts app, czech and italian found: $out"; bad=1
  fi
  # The accents' pair follows the keyboard settings: with them off, then on, typing either name
  # finds only the half that applies.
  local accents shown hidden found
  [[ -e $kbset ]] && cp -p "$kbset" "$kbset_saved"
  install -d -o "$USER_NAME" -g "$USER_NAME" "${kbset%/*}"
  for accents in false true; do
    shown=mac-accents-on hidden=mac-accents-off
    [[ $accents == true ]] && shown=mac-accents-off hidden=mac-accents-on
    { grep -vE '^[[:blank:]]*mac_accents[[:blank:]]*=' "$kbset_saved" 2>/dev/null || true; echo "mac_accents = $accents"; } >"$kbset.new"
    chown "$USER_NAME:" "$kbset.new"
    mv "$kbset.new" "$kbset"
    for i in $(seq 20); do
      out="$(in_session "$query 'Mac accents on Left Alt' accents" 2>&1 || true)"
      found="$(grep -cE "kubaos/${shown}( |\$)" <<<"$out" || true)"
      [[ $found == 2 ]] && ! grep -qE "kubaos/${hidden}( |\$)" <<<"$out" && break
      sleep 1
    done
    if [[ $found == 2 ]] && ! grep -qE "kubaos/${hidden}( |\$)" <<<"$out"; then
      echo "ok   with mac_accents = $accents, typing 'Mac accents on Left Alt' or 'accents' finds $shown alone (${i}s)"
    else
      echo "FAIL with mac_accents = $accents, typing the accents' names finds: $out"
      bad=1
    fi
  done
  # The next layout is the SUPER + SHIFT + Space bind's Item, from the hyprland Extension, alone.
  if grep -E -- "^next layout[[:blank:]]" <<<"$out" | grep -qE 'kubaos/keyboard-next( |$)'; then
    echo "FAIL the KubaOS Extension sends a next layout Item beside the bind's"
    bad=1
  else
    echo "ok   no KubaOS Item doubles the next layout bind"
  fi

  out="$(in_session "$query 'Wallpaper set: Shinjuku' 'wallpaper sets'" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  if grep -E -- "^Wallpaper set: Shinjuku[[:blank:]]" <<<"$out" | grep -qE 'kubaos/wallpaper-set:shinjuku( |$)' &&
    grep -E -- "^wallpaper sets[[:blank:]]" <<<"$out" | grep -qE 'kubaos/wallpaper-set:'; then
    echo "ok   the wallpaper sets are Items, found by name and by 'wallpaper sets'"
  else
    echo "FAIL the wallpaper set Items aren't found: $out"
    bad=1
  fi

  out="$(in_session "$query 'Terminal greeting' fastfetch" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  for want in 'Terminal greeting:greeting-off' 'fastfetch:greeting-off'; do
    grep -E -- "^${want%%:*}[[:blank:]]" <<<"$out" | grep -qE -- "kubaos/${want#*:}( |$)" ||
      { echo "FAIL typing '${want%%:*}' doesn't find kubaos/${want#*:}"; bad=1; }
  done
  if grep -qE 'kubaos/greeting-on( |$)' <<<"$out"; then
    echo "FAIL the greeting is on, and 'Terminal greeting on' is listed"
    bad=1
  fi
  ((bad)) || echo "ok   with the greeting on, only 'Terminal greeting off' is found"
  install -D -o "$USER_NAME" -g "$USER_NAME" -m 644 /dev/null "$no_greeting"
  for i in $(seq 20); do
    out="$(in_session "$query 'Terminal greeting'" 2>&1 || true)"
    grep -qE 'kubaos/greeting-on( |$)' <<<"$out" && ! grep -qE 'kubaos/greeting-off( |$)' <<<"$out" && break
    sleep 1
  done
  if grep -qE 'kubaos/greeting-on( |$)' <<<"$out" && ! grep -qE 'kubaos/greeting-off( |$)' <<<"$out"; then
    echo "ok   turning the greeting off lists 'Terminal greeting on' in its place (${i}s)"
  else
    echo "FAIL the greeting's pair didn't follow its file: $out"
    bad=1
  fi

  out="$(in_session "$query Files 'Open folder' 'Files: Downloads' 'Recent files'" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  for want in 'Files:files' 'Open folder:files-open' 'Files: Downloads:files:download' 'Recent files:files-recent'; do
    grep -E -- "^${want%:files*}[[:blank:]]" <<<"$out" | grep -qE -- "kubaos/files${want##*:files}( |$)" ||
      { echo "FAIL typing '${want%:files*}' doesn't find kubaos/files${want##*:files}"; bad=1; }
  done
  ((bad)) || echo "ok   Files, Open folder, Recent files and one Item a user folder are found"

  out="$(in_session "$query Yazi 'Tidy Downloads' 'Eject STICK' 'eject drive'" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  for want in 'Yazi:yazi' 'Tidy Downloads:tidy-downloads' 'Eject STICK:drive:SanDisk_Ultra_1234' \
    'eject drive:drive:SanDisk_Ultra_1234'; do
    grep -E -- "^${want%%:*}[[:blank:]]" <<<"$out" | grep -qE -- "kubaos/${want#*:}( |$)" ||
      { echo "FAIL typing '${want%%:*}' doesn't find kubaos/${want#*:}"; bad=1; }
  done
  ((bad)) || echo "ok   Yazi, Tidy Downloads and the mounted drive's Eject Item are found"

  # The clock popup's Items, and a date query answered in the top row by its
  # engine (crates/kubaos-clock), which the calculator can't read.
  out="$(in_session "$query 'Open calendar' 'Add countdown' 'Add time zone' 'Calendar settings'" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  for want in 'Open calendar:clock-open' 'Add countdown:clock-add-countdown' \
    'Add time zone:clock-add-zone' 'Calendar settings:clock-settings'; do
    grep -E -- "^${want%%:*}[[:blank:]]" <<<"$out" | grep -qE -- "kubaos/${want#*:}( |$)" ||
      { echo "FAIL typing '${want%%:*}' doesn't find kubaos/${want#*:}"; bad=1; }
  done
  ((bad)) || echo "ok   Open calendar, Add countdown, Add time zone and Calendar settings are found"
  out="$(in_session "$query --top 'days until christmas'" 2>&1)" || { echo "FAIL querying the menu: $out"; bad=1; }
  if [[ $out == "days until christmas"$'\t'"kubaos/days until christmas"$'\t'[0-9]*" day"*$'\t'"until "*" 25 December "* ]]; then
    echo "ok   'days until christmas' is answered on top: $(cut -f3 <<<"$out"), $(cut -f4 <<<"$out")"
  else
    echo "FAIL 'days until christmas' has no KubaOS answer on top: $out"
    bad=1
  fi

  # The system monitor's Items, and `ram` answered on top from /proc by the
  # Extension, with no shell running to write the stream file.
  out="$(in_session "$query 'Open system monitor' 'Show/hide the bar pill' 'Pick bar metrics'" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  for want in 'Open system monitor:sysmon-open' 'Show/hide the bar pill:sysmon-pill' 'Pick bar metrics:sysmon-metrics'; do
    grep -E -- "^${want%%:*}[[:blank:]]" <<<"$out" | grep -qE -- "kubaos/${want#*:}( |$)" ||
      { echo "FAIL typing '${want%%:*}' doesn't find kubaos/${want#*:}"; bad=1; }
  done
  ((bad)) || echo "ok   Open system monitor, Show/hide the bar pill and Pick bar metrics are found"
  out="$(in_session "$query --top ram" 2>&1)" || { echo "FAIL querying the menu: $out"; bad=1; }
  if [[ $out == "ram"$'\t'"kubaos/ram"$'\t'"RAM "[0-9]*" %"$'\t'*" used · "* ]]; then
    echo "ok   'ram' is answered on top: $(cut -f3 <<<"$out"), $(cut -f4 <<<"$out")"
  else
    echo "FAIL 'ram' has no KubaOS answer on top: $out"
    bad=1
  fi

  # The helper writes the file through a temporary one and a rename; the Items follow it.
  printf 'XKBMODEL="pc105"\nXKBLAYOUT="us,de"\nXKBVARIANT=","\nXKBOPTIONS=""\n' >"$kb.new"
  mv "$kb.new" "$kb"
  for i in $(seq 20); do
    out="$(in_session "$query Keyboard" 2>&1 || true)"
    grep -qE 'kubaos/keyboard:de( |$)' <<<"$out" && ! grep -qE 'kubaos/keyboard:it( |$)' <<<"$out" && break
    sleep 1
  done
  if grep -qE 'kubaos/keyboard:de( |$)' <<<"$out" && ! grep -qE 'kubaos/keyboard:it( |$)' <<<"$out"; then
    echo "ok   a change to $kb reaches the keyboard Items (${i}s)"
  else
    echo "FAIL the keyboard Items kept the old layouts after $kb changed: $out"
    bad=1
  fi

  # ~/.config/kotodama/user.toml is a link to the user layer's file, and an
  # edit to the file it points at reaches the running Daemon with no restart.
  cat >"$layer" <<'TOML'
[extensions.kubaos]
hide = ["theme-next"]

[extensions.kubaos.actions.harness-notes]
title = "Restore harness notes"
run = ["true"]
TOML
  for i in $(seq 20); do
    out="$(in_session "$query 'Restore harness notes' Bluetooth" 2>&1 || true)"
    grep -qE '^Restore harness notes[[:blank:]].*kubaos/harness-notes' <<<"$out" &&
      grep -qE 'kubaos/bluetooth( |$)' <<<"$out" && break
    sleep 1
  done
  if grep -qE '^Restore harness notes[[:blank:]].*kubaos/harness-notes' <<<"$out" &&
    grep -qE 'kubaos/bluetooth( |$)' <<<"$out"; then
    echo "ok   an edit to the linked user layer reaches the running Daemon (${i}s)"
  else
    echo "FAIL the running Daemon kept the old user layer after an edit: $out"
    bad=1
  fi

  if ((bad)); then tail -20 "$log"; fi
  menu_stop "$daemon"
  if [[ -e $saved ]]; then mv "$saved" "$layer"; else rm -f "$layer"; fi
  if [[ -e $kb_saved ]]; then mv "$kb_saved" "$kb"; else rm -f "$kb"; fi
  if [[ -e $groups_saved ]]; then mv "$groups_saved" "$groups"; else rm -f "$groups"; fi
  if [[ -e $chrome_saved ]]; then mv "$chrome_saved" "$chrome"; else rm -f "$chrome"; fi
  rm -f "$webapp"
  if [[ -e $kbset_saved ]]; then mv "$kbset_saved" "$kbset"; else rm -f "$kbset"; fi
  ((greeting_was_off)) || rm -f "$no_greeting"
  rm -f "$drives"
  return "$bad"
}

# menu_closes_as_is DESC STEP...: the steps, then up to PROBE_WAIT for the menu to
# close. It closes from what it showed: no other View, and no Action panel closing,
# reaches the screen between the steps and the menu being gone.
menu_closes_as_is() {
  local desc="$1" mark shown before
  shift
  mark=$(($(wc -l <"$QS_LOG") + 1))
  before="$(grep -o 'menu probe: screen .* visible=' "$QS_LOG" | tail -1)"
  input "$@" 2>>"$LOGS/menu-steps.log" || { echo "FAIL $desc: the keys didn't go"; return 1; }
  for _ in $(seq $((PROBE_WAIT * 10))); do
    tail -n +"$mark" "$QS_LOG" | grep -q 'menu probe: visible false' && break
    sleep 0.1
  done
  shown="$(tail -n +"$mark" "$QS_LOG" | grep -o 'menu probe: .*' | sed '/^menu probe: visible false$/q')"
  if [[ $shown != *"menu probe: visible false" ]]; then
    echo "FAIL $desc: the menu didn't close"
    return 1
  fi
  if grep -E '^menu probe: screen .* visible=true$' <<<"$shown" | grep -qvF "${before}true" ||
    grep -qx 'menu probe: actions 0' <<<"$shown"; then
    echo "FAIL $desc: the menu changed on screen before it closed:"
    grep -E '^menu probe: (screen|actions)' <<<"$shown" | sed 's/^/     /'
    return 1
  fi
  echo "ok   $desc"
}

# menu_form_is PATTERN DESC: up to PROBE_WAIT for the probe's latest form line, once the form rests,
# to match PATTERN (ERE): the fields drawn, and what they hold.
menu_form_is() {
  # The scroll glides for a moment after a field takes the keys: the form is read once it rests.
  sleep 1
  for _ in $(seq $((PROBE_WAIT * 10))); do
    grep -o 'menu probe: form .*' "$QS_LOG" | tail -1 | grep -qE -- "$1" && { echo "ok   $2"; return 0; }
    sleep 0.1
  done
  echo "FAIL $2: the form is $(grep -o 'menu probe: form .*' "$QS_LOG" | tail -1)"
  return 1
}

# The Command menu's file and folder fields, in Quickshell under the Daemon: Ctrl+O closes the
# menu for the system's picker and opens it again on the form, the path chosen in its field and
# every other field as it was, and Enter runs the action with it, the menu closing from the form
# with nothing else on screen first, as it does from the Action panel. A cancelled pick brings
# the form back as it was, and Esc still pops a form on screen. A stub stands in for `kubaos-files pick`, as the container has no portal:
# it answers after a second, as a person would, with a file, a folder, or nothing.
c_menu_files() {
  local bad=0 daemon log="$LOGS/kotodamad-files.log" out
  local layer="$USER_HOME/.config/kubaos/menu/kotodama/user.toml" saved="$LOGS/user.toml.files-saved"
  local stub="$LOGS/pick-stub" cancel="$LOGS/pick-cancel" ran="$LOGS/files-ran"
  local config="$USER_HOME/Downloads/work.conf" folder="$USER_HOME/Projects/acme"
  input_tools || { echo "FAIL no pywayland or Wayland protocols from the flake's nixpkgs"; return 1; }

  [[ -e $layer ]] && cp -p "$layer" "$saved"
  install -D -o "$USER_NAME" -g "$USER_NAME" -m 644 /dev/stdin "$layer" <<TOML
[extensions.kubaos.actions.harness-file]
title = "Harness file"
keyword = "harness-file"
run = ["sh", "-c", "printf '%s\\\\n' \"\$1\" >$ran", "sh", "{path}"]
arguments = [{ name = "path", kind = "file", accept = ["conf"] }]

[extensions.kubaos.actions.harness-close]
title = "Harness close"
run = ["true"]

[extensions.kubaos.actions.harness-pair]
title = "Harness pair"
keyword = "harness-pair"
run = ["sh", "-c", "printf '%s %s\\\\n' \"\$1\" \"\$2\" >$ran", "sh", "{suite}", "{folder}"]
arguments = [{ name = "suite", placeholder = "node" }, { name = "folder", kind = "folder" }]
TOML
  install -d -o "$USER_NAME" -g "$USER_NAME" "${config%/*}" "$folder"
  install -o "$USER_NAME" -g "$USER_NAME" -m 644 /dev/null "$config"
  install -d -o "$USER_NAME" -g "$USER_NAME" "$stub"
  install -o "$USER_NAME" -g "$USER_NAME" -m 755 /dev/stdin "$stub/kubaos-files" <<SH
#!/bin/sh
sleep 1
[ -e $cancel ] && exit 1
case " \$* " in
  *" --folder "*) echo $folder ;;
  *) echo $config ;;
esac
SH
  rm -f "$cancel" "$ran"

  session_start sway 1920x1080 1
  : >"$log"
  chown "$USER_NAME:" "$log"
  in_session "exec kotodamad >$log 2>&1" &
  daemon=$!
  input_up 1920x1080 || { menu_stop "$daemon"; return 1; }
  if ! qs_start KUBAOS_MENU_PROBE=1 "PATH=$stub:\$PATH"; then
    echo "FAIL Quickshell didn't load:"
    tail -30 "$QS_LOG"
    menu_stop "$daemon"
    return 1
  fi
  menu_now 'connected true' || { echo "FAIL the menu's search never reached the Daemon"; bad=1; }

  # A file field: picked, then run.
  menu_step "kotodama toggle opens the menu" 'visible true' menu_kotodama toggle || bad=1
  menu_keys search && menu_typed 'harness file' --delay 50 'type:harness file' &&
    menu_step "Enter on an action with a file argument asks for it in a form" 'view kind=form' input key:Return || bad=1
  menu_keys form && menu_form_is 'fields=1 shown=1 past=0 values=\[""\]' "the form shows its file field" || bad=1
  menu_step "Ctrl+O closes the menu for the picker" 'visible false' input down:ctrl type:o up:ctrl || bad=1
  menu_step "the menu opens again once the picker answers" 'visible true' : || bad=1
  menu_form_is 'fields=1 shown=1 past=0 values=\["~/Downloads/work.conf"\]' "the form is back, the file picked in its field" || bad=1
  menu_keys form && menu_closes_as_is "Enter runs the action, and the menu closes from the form" key:Return || bad=1
  for _ in $(seq $((PROBE_WAIT * 10))); do [[ -s $ran ]] && break; sleep 0.1; done
  out="$(cat "$ran" 2>/dev/null)"
  if [[ $out == "$config" ]]; then
    echo "ok   Enter runs the action with the path picked: $out"
  else
    echo "FAIL Enter ran the action with '$out', not $config"
    bad=1
  fi
  rm -f "$ran"

  # Two fields: what was typed in one stays through a cancelled pick and a folder picked in the
  # other.
  menu_now 'visible false' || { echo "FAIL the menu stayed open after running the action"; bad=1; }
  menu_step "kotodama toggle opens the menu again" 'visible true' menu_kotodama toggle || bad=1
  menu_keys search && menu_typed 'harness pair' --delay 50 'type:harness pair' &&
    menu_step "Enter on an action with a text and a folder argument asks for them" 'view kind=form' input key:Return || bad=1
  menu_keys form && input --delay 50 type:node key:Tab
  menu_form_is 'fields=2 shown=2 past=0 values=\["node",""\]' "the suite is typed in the first field" || bad=1
  install -o "$USER_NAME" -g "$USER_NAME" -m 644 /dev/null "$cancel"
  menu_step "Ctrl+O in the folder field closes the menu for the picker" 'visible false' input down:ctrl type:o up:ctrl || bad=1
  menu_step "a cancelled pick opens the menu again" 'visible true' : || bad=1
  menu_form_is 'fields=2 shown=2 past=0 values=\["node",""\]' "the form is back as it was after a cancel" || bad=1
  rm -f "$cancel"
  menu_keys form &&
    menu_step "Ctrl+O again closes the menu for the picker" 'visible false' input down:ctrl type:o up:ctrl || bad=1
  menu_step "the menu opens again with the folder" 'visible true' : || bad=1
  menu_form_is 'fields=2 shown=2 past=0 values=\["node","~/Projects/acme"\]' "the folder is picked, the suite still typed" || bad=1
  menu_keys form && menu_closes_as_is "Enter runs it, and the menu closes from the form" key:Return || bad=1
  for _ in $(seq $((PROBE_WAIT * 10))); do [[ -s $ran ]] && break; sleep 0.1; done
  out="$(cat "$ran" 2>/dev/null)"
  if [[ $out == "node $folder" ]]; then
    echo "ok   Enter runs the action with both: $out"
  else
    echo "FAIL Enter ran the action with '$out', not 'node $folder'"
    bad=1
  fi

  # The Action panel: the Action run from it closes the menu with the panel still on it.
  menu_now 'visible false' || { echo "FAIL the menu stayed open after running the action"; bad=1; }
  menu_step "kotodama toggle opens the menu for the Action panel" 'visible true' menu_kotodama toggle || bad=1
  menu_keys search && menu_typed 'harness close' --delay 50 'type:harness close' &&
    menu_step "Ctrl+K opens the Action panel" 'actions [1-9]' input down:ctrl type:k up:ctrl || bad=1
  menu_closes_as_is "Enter in the Action panel runs it, and the menu closes with the panel on it" key:Return || bad=1

  # Esc still goes back one View, on screen.
  menu_step "kotodama toggle opens the menu once more" 'visible true' menu_kotodama toggle || bad=1
  menu_keys search && menu_typed 'harness pair' --delay 50 'type:harness pair' &&
    menu_step "Enter asks for the suite and the folder again" 'view kind=form' input key:Return || bad=1
  menu_keys form && menu_step "Esc pops the form, on screen" 'screen kind= title="" visible=true' input key:Escape || bad=1

  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  if ((bad)); then tail -20 "$log"; fi
  menu_stop "$daemon"
  if [[ -e $saved ]]; then mv "$saved" "$layer"; else rm -f "$layer"; fi
  rm -rf "$stub" "$cancel" "$ran" "$config" "$folder"
  return "$bad"
}

# Wallpaper sets: every set read-only in /usr/share/kubaos/wallpapers,
# the picker's folders are the chosen ones (~/.config/kubaos/desktop/wallpapers/sets,
# all of them without the file) plus ~/Pictures/wallpapers, a real folder.
c_wallpaper_sets() {
  local sys=/usr/share/kubaos/wallpapers sets="$USER_HOME/.config/kubaos/desktop/wallpapers/sets"
  local own="$USER_HOME/Pictures/wallpapers" st="$USER_HOME/.local/state/theme" got bad=0 wall
  [[ "$(readlink "$sys")" == /nix/store/* ]] && echo "ok   $sys -> $(readlink "$sys")" ||
    { echo "FAIL $sys isn't a link into the store"; return 1; }
  [[ -d $own && ! -L $own ]] && echo "ok   ~/Pictures/wallpapers is a real folder" ||
    { echo "FAIL ~/Pictures/wallpapers isn't a real folder"; return 1; }
  folders() { zrun 'dot-theme folders' | cut -f1 | tr '\n' ' '; }

  # The switch seeded the file (no install here) with today's sets.
  [[ "$(grep -v '^#' "$sets" 2>/dev/null | tr '\n' ' ')" == "cyberpunk japanese shinjuku " ]] &&
    echo "ok   activation seeded the sets file with the sets on disk" ||
    { echo "FAIL sets file after the switch: $(cat "$sets" 2>&1)"; bad=1; }
  rm -f "$sets"
  got="$(folders)"
  [[ $got == "cyberpunk japanese shinjuku yours " ]] && echo "ok   no sets file: every set ($got)" ||
    { echo "FAIL no sets file: folders are '$got'"; bad=1; }
  as_user mkdir -p "$(dirname "$sets")"
  printf '# picked at install\nshinjuku\nvaporwave\n' | as_user tee "$sets" >/dev/null
  got="$(folders)"
  [[ $got == "shinjuku yours " ]] && echo "ok   sets file: the chosen set, unknown ones skipped ($got)" ||
    { echo "FAIL sets file 'shinjuku vaporwave': folders are '$got'"; bad=1; }
  zrun 'dot-theme folder japanese' >/dev/null 2>&1 && { echo "FAIL dot-theme folder japanese works, but it isn't chosen"; bad=1; } ||
    echo "ok   dot-theme folder refuses a set that isn't chosen"

  as_user cp "$sys/japanese/tokyo-tower-blue-hour.jpg" "$own/mine.jpg"
  expect_ok "dot-theme folder yours" zrun 'dot-theme folder yours' || bad=1
  [[ "$(zrun 'dot-theme current')" == "$own/mine.jpg" && "$(<"$st/folder")" == yours ]] &&
    echo "ok   your own image is the wallpaper, folder yours" || { echo "FAIL current $(zrun 'dot-theme current'), folder $(<"$st/folder")"; bad=1; }
  expect_ok "dot-theme toggle-folder" zrun 'dot-theme toggle-folder' || bad=1
  [[ "$(<"$st/folder")" == shinjuku ]] && echo "ok   toggle-folder wraps from yours to shinjuku" ||
    { echo "FAIL toggle-folder from yours went to $(<"$st/folder")"; bad=1; }

  # An older machine's wallpaper was under ~/Pictures/wallpapers/<set>/:
  # apply finds it in /usr/share.
  wall="$sys/shinjuku/train-neon-streak.jpg"
  printf '%s\n' "$own/shinjuku/train-neon-streak.jpg" >"$st/wallpaper"
  expect_ok "dot-theme apply (old path)" zrun 'dot-theme apply' || bad=1
  [[ "$(zrun 'dot-theme current')" == "$wall" ]] && echo "ok   the old ~/Pictures/wallpapers/shinjuku path moves to $wall" ||
    { echo "FAIL old path: current is $(zrun 'dot-theme current')"; bad=1; }

  wall="$sys/shinjuku/kubaos-neon-sign.jpg"
  : >"$st/wallpaper"
  expect_ok "dot-theme apply (first boot)" zrun 'dot-theme apply' || bad=1
  [[ "$(zrun 'dot-theme current')" == "$wall" ]] && echo "ok   first boot: the default is $wall" ||
    { echo "FAIL first boot: current is $(zrun 'dot-theme current')"; bad=1; }

  # Machines from earlier releases may have a live scene, `ukiyo:<id>`, as the
  # wallpaper: apply moves them to the default.
  printf '%s\n' ukiyo:kabukicho-gate >"$st/wallpaper"
  expect_ok "dot-theme apply (old live scene)" zrun 'dot-theme apply' || bad=1
  [[ "$(zrun 'dot-theme current')" == "$wall" ]] && echo "ok   the old live scene ukiyo:kabukicho-gate moves to $wall" ||
    { echo "FAIL old live scene: current is $(zrun 'dot-theme current')"; bad=1; }

  # Single wallpapers, as Kōbō writes them: a set shows only its picks, and
  # the default isn't among them.
  rm -f "$own/mine.jpg"
  printf '# picked one by one\njapanese/tokyo-tower-blue-hour.jpg\nshinjuku/skyline-fuji-dusk.jpg\nshinjuku/train-neon-streak.jpg\nshinjuku/gone.jpg\n' |
    as_user tee "$sets" >/dev/null
  got="$(folders)"
  [[ $got == "japanese shinjuku yours " ]] && echo "ok   single wallpapers: their sets are the folders ($got)" ||
    { echo "FAIL single wallpapers: folders are '$got'"; bad=1; }
  got="$(zrun 'dot-theme wallpapers' | tr '\t\n' ': ')"
  [[ $got == "japanese:$sys/japanese/tokyo-tower-blue-hour.jpg shinjuku:$sys/shinjuku/skyline-fuji-dusk.jpg shinjuku:$sys/shinjuku/train-neon-streak.jpg " ]] &&
    echo "ok   dot-theme wallpapers: only the picks, a missing one skipped" ||
    { echo "FAIL dot-theme wallpapers: '$got'"; bad=1; }
  expect_ok "dot-theme folder shinjuku" zrun 'dot-theme folder shinjuku' || bad=1
  zrun 'dot-theme next' >/dev/null 2>&1
  got="$(zrun 'dot-theme current')"
  zrun 'dot-theme next' >/dev/null 2>&1
  got+=" $(zrun 'dot-theme current')"
  zrun 'dot-theme prev' >/dev/null 2>&1
  got+=" $(zrun 'dot-theme current')"
  [[ $got == "$sys/shinjuku/train-neon-streak.jpg $sys/shinjuku/skyline-fuji-dusk.jpg $sys/shinjuku/train-neon-streak.jpg" ]] &&
    echo "ok   next and prev step over shinjuku's two picks only" ||
    { echo "FAIL next, next, prev over shinjuku's picks: $got"; bad=1; }
  wall="$sys/japanese/tokyo-tower-blue-hour.jpg"
  : >"$st/wallpaper"
  rm -f "$st/folder"
  expect_ok "dot-theme apply (first boot, default not picked)" zrun 'dot-theme apply' || bad=1
  [[ "$(zrun 'dot-theme current')" == "$wall" ]] && echo "ok   first boot without the default: the first pick, $wall" ||
    { echo "FAIL first boot without the default: current is $(zrun 'dot-theme current')"; bad=1; }
  expect_ok "dot-theme all-sets" zrun 'dot-theme all-sets' || bad=1
  got="$(grep -v '^#' "$sets" | tr '\n' ' ') / $(folders) / $(zrun 'dot-theme wallpapers' | grep -c "^shinjuku")"
  [[ $got == "cyberpunk japanese shinjuku  / cyberpunk japanese shinjuku yours  / 6" ]] &&
    echo "ok   all-sets: every set again, whole ($got)" || { echo "FAIL all-sets: $got"; bad=1; }

  # The Command menu's set Items: hide-set and show-set rewrite the sets file, and the last set
  # in the picker stays.
  rm -f "$sets"
  expect_ok "dot-theme hide-set japanese (no sets file)" zrun 'dot-theme hide-set japanese' || bad=1
  got="$(grep -v '^#' "$sets" | tr '\n' ' ') / $(folders)"
  [[ $got == "cyberpunk shinjuku  / cyberpunk shinjuku yours " ]] &&
    echo "ok   hide-set without a sets file: every set but that one ($got)" || { echo "FAIL hide-set: $got"; bad=1; }
  expect_ok "dot-theme hide-set cyberpunk" zrun 'dot-theme hide-set cyberpunk' || bad=1
  zrun 'dot-theme hide-set shinjuku' >/dev/null 2>&1 && { echo "FAIL hide-set took the last set out of the picker"; bad=1; }
  got="$(grep -v '^#' "$sets" | tr '\n' ' ')"
  [[ $got == "shinjuku " ]] && echo "ok   hide-set refuses the last set in the picker" ||
    { echo "FAIL hide-set on the last set: '$got'"; bad=1; }
  printf 'shinjuku/train-neon-streak.jpg\n' | as_user tee "$sets" >/dev/null
  zrun 'dot-theme hide-set shinjuku' >/dev/null 2>&1 && { echo "FAIL hide-set took the last pick out of the picker"; bad=1; }
  expect_ok "dot-theme show-set shinjuku" zrun 'dot-theme show-set shinjuku' || bad=1
  expect_ok "dot-theme show-set japanese" zrun 'dot-theme show-set japanese' || bad=1
  got="$(grep -v '^#' "$sets" | tr '\n' ' ') / $(zrun 'dot-theme wallpapers' | grep -c "^shinjuku")"
  [[ $got == "shinjuku japanese  / 6" ]] && echo "ok   show-set shows the whole set, its picks gone ($got)" ||
    { echo "FAIL show-set: $got"; bad=1; }
  zrun 'dot-theme show-set yours' >/dev/null 2>&1 && { echo "FAIL show-set takes yours as a set"; bad=1; }

  rm -f "$sets" "$own/mine.jpg"
  return "$bad"
}

# Moving wallpapers. dot-theme takes a video's colors from its poster and
# names the video as the wallpaper; next/prev step over stills and videos
# alike. Quickshell plays it (software decode here, llvmpipe) and stops when
# motion is set to never. Prints Quickshell's CPU with the video playing and
# with it stopped.
c_wallpaper_video() {
  local st="$USER_HOME/.local/state/theme" vid poster a b out seen n i bad=0 pid
  vid="$(zrun 'ls /usr/share/kubaos/wallpapers/*/*.mp4 ~/Pictures/wallpapers/*.mp4 2>/dev/null | head -1')"
  # None of the sets has a video wallpaper.
  [[ -n $vid ]] || { echo "skip no video wallpaper in the sets or ~/Pictures/wallpapers"; return 0; }
  poster="${vid%.*}.poster.jpg"
  [[ -f $poster ]] || { echo "FAIL $vid has no poster ($poster)"; return 1; }
  expect_ok "dot-theme set <poster>" zrun "dot-theme set '$poster'" || return 1
  a="$(jq -S 'del(.wallpaper)' "$st/colors.json")"
  expect_ok "dot-theme set ${vid#"$USER_HOME"/}" zrun "dot-theme set '$vid'" || return 1
  b="$(jq -S 'del(.wallpaper)' "$st/colors.json")"
  [[ $a == "$b" ]] && echo "ok   colors come from the poster" || { echo "FAIL colors differ from the poster's"; bad=1; }
  jq -e --arg v "$vid" '.wallpaper == $v' "$st/colors.json" >/dev/null &&
    grep -qF "\"$vid\"" "$st/hypr.lua" && grep -qF "\"$vid\"" "$st/nvim.lua" &&
    echo "ok   colors.json, hypr.lua, nvim.lua name the video" ||
    { echo "FAIL the generated files don't name the video"; grep -h wallpaper "$st/colors.json" "$st/hypr.lua"; bad=1; }
  [[ "$(zrun 'dot-theme current')" == "$vid" ]] || { echo "FAIL dot-theme current isn't the video"; bad=1; }
  out="$(zrun 'dot-theme apply' 2>&1)"
  [[ $(jq -S 'del(.wallpaper)' "$st/colors.json") == "$b" && -z $out ]] &&
    echo "ok   dot-theme apply on a video is a no-op when nothing changed" || { echo "FAIL apply: $out"; bad=1; }
  # A full lap of `next` over the video's folder visits stills and videos
  # and comes back.
  n="$(find -L "$(dirname "$vid")" -maxdepth 1 -type f -regextype posix-extended -iregex '.*\.(jpe?g|png|webp|mp4|webm|mkv|mov)' ! -iname '*.poster.*' | wc -l)"
  seen=""
  for ((i = 0; i < n; i++)); do
    zrun 'dot-theme next' >/dev/null 2>&1 || { echo "FAIL dot-theme next"; bad=1; break; }
    seen+="$(zrun 'dot-theme current')"$'\n'
  done
  grep -q '\.mp4$' <<<"$seen" && grep -q '\.jpg$' <<<"$seen" && [[ "$(zrun 'dot-theme current')" == "$vid" ]] &&
    ! grep -q '\.poster\.' <<<"$seen" &&
    echo "ok   next: $n wallpapers in $(basename "$(dirname "$vid")")/, stills and videos, no posters, back to the start" ||
    { echo "FAIL next didn't lap $(dirname "$vid") over stills and videos:"; printf '%s' "$seen"; bad=1; }

  session_start sway 1280x800 1
  if ! qs_start; then
    echo "FAIL Quickshell didn't load"; tail -30 "$QS_LOG"; session_stop; return 1
  fi
  out=""
  for _ in $(seq 60); do
    out="$(in_session 'qs ipc call wallpaper status' 2>/dev/null)"
    jq -e --arg v "$vid" '.playing | to_entries | any(.value == $v)' <<<"$out" >/dev/null 2>&1 && break
    sleep 0.5
  done
  jq -e --arg v "$vid" '.playing | to_entries | any(.value == $v)' <<<"$out" >/dev/null 2>&1 &&
    echo "ok   Quickshell plays the video: $out" || { echo "FAIL video not playing: $out"; bad=1; }
  pid="$(pgrep -u "$USER_NAME" -n -f 'quickshell|bin/qs' || true)"
  if [[ -n $pid ]]; then
    echo "     Quickshell CPU (100% = one core), video playing: $(cpu_pct "$pid" 10)%"
  fi
  expect_ok "qs ipc call wallpaper animate never" in_session 'qs ipc call wallpaper animate never' || bad=1
  for _ in $(seq 20); do
    out="$(in_session 'qs ipc call wallpaper status' 2>/dev/null)"
    jq -e '.mode == "never" and (.playing | length == 0)' <<<"$out" >/dev/null 2>&1 && break
    sleep 0.5
  done
  jq -e '.mode == "never" and (.playing | length == 0)' <<<"$out" >/dev/null 2>&1 &&
    echo "ok   motion off: the poster stays, nothing decodes" || { echo "FAIL still playing with motion off: $out"; bad=1; }
  if [[ -n $pid ]]; then
    sleep 1
    echo "     Quickshell CPU, poster only: $(cpu_pct "$pid" 5)%"
  fi
  jq -e '.animate == "never"' "$USER_HOME/.local/state/quickshell/kubaos.json" >/dev/null 2>&1 &&
    echo "ok   the mode is remembered in ~/.local/state/quickshell/kubaos.json" ||
    { echo "FAIL kubaos.json: $(cat "$USER_HOME/.local/state/quickshell/kubaos.json" 2>&1)"; bad=1; }
  in_session 'qs ipc call wallpaper animate ac' >/dev/null 2>&1 || true
  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  session_stop
  return "$bad"
}

# cpu_pct PID SECONDS: the process's CPU use over SECONDS, in percent of one core.
cpu_pct() {
  local pid=$1 secs=$2 t0 t1 hz
  hz="$(getconf CLK_TCK)"
  t0="$(awk '{print $14 + $15}' "/proc/$pid/stat")"
  sleep "$secs"
  t1="$(awk '{print $14 + $15}' "/proc/$pid/stat")"
  awk -v d=$((t1 - t0)) -v hz="$hz" -v s="$secs" 'BEGIN { printf "%.0f", d * 100 / (hz * s) }'
}

# has_bin NAME: whether the home profile has NAME.
has_bin() { [[ -x $USER_HOME/.nix-profile/bin/$1 ]]; }
# kz CMD: CMD in a login zsh with Nix's own bin on PATH, which home-manager
# runs `nix` from; the harness's login zsh doesn't source nix-daemon.sh.
# /usr/local/bin goes first, so `kubaos` is /out/kubaos, built with the test
# Suite, and not the home's.
kz() { zrun ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh; PATH=/usr/local/bin:\$PATH; $1"; }

# The state directory and switch jobs, through the kubaos binary
# test/run built (/out/kubaos) and a user flake like the installer's, with the
# repo copy as KubaOS and KubaOS's own locked nixpkgs as the user's, so
# nothing is downloaded but the packages. The packages are ones the home
# doesn't ship (ripgrep is in home/cli.nix). Switches use --foreground: the
# container has no systemd user manager, so the queue runs in the calling
# process, behind the same lock.
# install_kubaos: /out/kubaos, which test/run built, as the kubaos on PATH.
install_kubaos() {
  [[ -x /out/kubaos ]] || { echo "FAIL no /out/kubaos (test/run e2e builds it)"; return 1; }
  install -m 0755 /out/kubaos /usr/local/bin/kubaos
}

# user_flake DIR [KUBAOS_URL]: a fresh user flake in DIR, rendered from
# user-flake/ with KUBAOS_URL (the repo copy by default) as KubaOS and
# KubaOS's own locked nixpkgs as the user's, and no home.nix yet.
user_flake() {
  local u="$1" kubaos="${2:-git+file://$REPO}" nixpkgs
  nixpkgs="$(as_user "$NIX_BIN/nix" flake archive --dry-run --json "$REPO" | jq -r .inputs.nixpkgs.path)"
  [[ $nixpkgs == /nix/store/* ]] || { echo "FAIL KubaOS's nixpkgs: $nixpkgs"; return 1; }
  rm -rf "$u"
  as_user install -d "$u"
  sed -e 's/@USER@/kuba/g; s/@HOST@/vm/g' -e "s/@SYSTEM@/$(uname -m)-linux/" \
    -e "s#github:NixOS/nixpkgs/nixos-26.05#path:$nixpkgs#" \
    -e "s#git+file:///usr/share/kubaos?ref=refs/heads/release#$kubaos#" \
    "$REPO/user-flake/flake.nix.in" | as_user tee "$u/flake.nix" >/dev/null
}

c_packages() {
  local u="$USER_HOME/.config/kubaos/home" jobs="$USER_HOME/.local/state/kubaos/jobs" gen1 out
  local backups="$USER_HOME/.local/state/kubaos/backups"
  install_kubaos || return 1
  user_flake "$u" || return 1
  rm -rf "$backups"
  as_user install -d "$u/kubaos.d/suites"
  # home.nix as installs before the state directory wrote it, for the migration.
  printf '{ pkgs, ... }:\n{\n  home.packages = [ ];\n}\n' | as_user tee "$u/home.nix" >/dev/null
  echo 'packages = ["hello"]' | as_user tee "$u/kubaos.d/suites/demo.toml" >/dev/null

  out="$(kz 'kubaos package add cowsay --foreground' 2>&1)" ||
    { echo "FAIL kubaos package add cowsay"; printf '%s\n' "$out" | tail -20; return 1; }
  echo "ok   kubaos package add cowsay, then a switch"
  grep -q '^  kubaos.stateDir = ./kubaos.d;$' "$u/home.nix" && cmp -s "$backups/home.nix.bak" \
    <(printf '{ pkgs, ... }:\n{\n  home.packages = [ ];\n}\n') ||
    { echo "FAIL home.nix not wired, or no backup in $backups"; cat "$u/home.nix"; ls -la "$backups"; return 1; }
  [[ -z $(find "$u" -maxdepth 1 -name 'home.nix?*') ]] ||
    { echo "FAIL a backup inside the user flake, which would reach the Nix store"; ls -la "$u"; return 1; }
  echo "ok   home.nix wired to kubaos.d/, the old one in ~/.local/state/kubaos/backups, outside the flake"
  has_bin cowsay && zrun 'cowsay hi' >/dev/null || { echo "FAIL cowsay not on PATH"; return 1; }
  has_bin hello || { echo "FAIL hello, from suites/demo.toml, not on PATH"; return 1; }
  echo "ok   cowsay (packages.toml) and hello (suites/demo.toml) on PATH: the union"
  gen1="$(readlink -f "$USER_HOME/.local/state/nix/profiles/home-manager")"
  grep -q cowsay "$gen1/home-files/.local/share/kubaos/kubaos.d/packages.toml" ||
    { echo "FAIL the generation carries no copy of kubaos.d"; return 1; }
  echo "ok   the generation carries its kubaos.d"

  kz 'kubaos package add figlet --foreground' >"$LOGS/packages-figlet.log" 2>&1 &
  local a=$!
  kz 'kubaos package add sl --foreground' >"$LOGS/packages-sl.log" 2>&1 &
  local b=$!
  wait "$a" || { echo "FAIL concurrent add figlet"; tail -20 "$LOGS/packages-figlet.log"; return 1; }
  wait "$b" || { echo "FAIL concurrent add sl"; tail -20 "$LOGS/packages-sl.log"; return 1; }
  has_bin figlet && has_bin sl || { echo "FAIL figlet or sl not on PATH after both adds"; return 1; }
  # The jobs, by start: each ends before the next starts.
  local overlap
  overlap="$(jq -s 'map(select(.state == "succeeded")) | sort_by(.started_at)
    | [range(1; length) as $i | select(.[$i - 1].finished_at > .[$i].started_at)] | length' "$jobs"/*.json)"
  [[ $overlap == 0 ]] || { echo "FAIL $overlap switch jobs overlapped"; jq -c . "$jobs"/*.json; return 1; }
  echo "ok   two adds at once: both installed, their switches one after the other ($(ls "$jobs"/*.json | wc -l) jobs)"

  kz "kubaos package restore '$gen1' && '$gen1/activate'" >"$LOGS/packages-rollback.log" 2>&1 ||
    { echo "FAIL restore + activate the first generation"; tail -20 "$LOGS/packages-rollback.log"; return 1; }
  grep -q '^packages = \["cowsay"\]$' "$u/kubaos.d/packages.toml" || { echo "FAIL kubaos.d not restored"; cat "$u/kubaos.d/packages.toml"; return 1; }
  ! has_bin figlet && has_bin cowsay || { echo "FAIL rollback: the first generation's packages aren't the ones on PATH"; return 1; }
  echo "ok   rolling back: kubaos package restore + the generation's activate bring back both"

  out="$(kz 'kubaos package remove cowsay hello --foreground' 2>&1)" ||
    { echo "FAIL kubaos package remove"; printf '%s\n' "$out" | tail -20; return 1; }
  ! has_bin cowsay || { echo "FAIL cowsay still on PATH after remove"; return 1; }
  has_bin hello && grep -q 'hello stays installed: the demo Suite lists it' <<<"$out" ||
    { echo "FAIL hello, still in suites/demo.toml, should stay (and say so)"; printf '%s\n' "$out"; return 1; }
  echo "ok   kubaos package remove cowsay: gone; hello stays, the demo Suite lists it"
  rm "$u/kubaos.d/suites/demo.toml"
  kz 'kubaos switch --foreground' >"$LOGS/packages-switch.log" 2>&1 ||
    { echo "FAIL kubaos switch"; tail -20 "$LOGS/packages-switch.log"; return 1; }
  ! has_bin hello || { echo "FAIL hello still on PATH with no file listing it"; return 1; }
  echo "ok   kubaos switch after deleting the Suite file: hello gone"
  c_packages_unfree "$u"
}

# c_packages_unfree FLAKE: unfree packages in the user flake of c_packages.
# unrar is unfree in nixpkgs and is only recorded, never built; home.nix's
# kubaos-unfree-test is an unfree package built from nothing, so the switch
# downloads nothing.
c_packages_unfree() {
  local u="$1" out
  out="$(kz 'kubaos package add unrar --no-switch' 2>&1)" && grep -qx 'added unrar (unfree)' <<<"$out" ||
    { echo "FAIL kubaos package add unrar doesn't say it is unfree"; printf '%s\n' "$out"; return 1; }
  out="$(kz 'kubaos package list' 2>&1)" && grep -qx 'unrar  (packages.toml)  unfree' <<<"$out" ||
    { echo "FAIL kubaos package list doesn't mark unrar unfree"; printf '%s\n' "$out"; return 1; }
  out="$(kz 'kubaos package list --json' 2>&1)" && jq -e '.unrar.unfree == true' <<<"$out" >/dev/null ||
    { echo "FAIL kubaos package list --json: unrar isn't unfree"; printf '%s\n' "$out"; return 1; }
  echo "ok   kubaos package add and list mark unrar unfree"
  out="$(kz 'kubaos package unfree off --no-switch' 2>&1)" &&
    { echo "FAIL kubaos package unfree off went ahead with unrar recorded"; printf '%s\n' "$out"; return 1; }
  grep -q 'unrar is unfree (packages.toml)' <<<"$out" && [[ ! -e $u/kubaos.d/nixpkgs.toml ]] ||
    { echo "FAIL kubaos package unfree off with unrar recorded"; printf '%s\n' "$out"; return 1; }
  kz 'kubaos package remove unrar --no-switch' >/dev/null 2>&1 || { echo "FAIL kubaos package remove unrar"; return 1; }
  echo "ok   kubaos package unfree off refuses while unrar is recorded"

  as_user tee "$u/home.nix" >/dev/null <<'EOF'
{ pkgs, ... }:
{
  home.packages = [
    (pkgs.runCommand "kubaos-unfree-test" { meta.license = pkgs.lib.licenses.unfree; } ''
      mkdir -p $out/bin
      printf '#!/bin/sh\necho unfree\n' >$out/bin/kubaos-unfree-test
      chmod +x $out/bin/kubaos-unfree-test
    '')
  ];
  kubaos.stateDir = ./kubaos.d;
}
EOF
  kz 'kubaos switch --foreground' >"$LOGS/packages-unfree.log" 2>&1 && has_bin kubaos-unfree-test ||
    { echo "FAIL a switch with an unfree package in home.nix"; tail -20 "$LOGS/packages-unfree.log"; return 1; }
  echo "ok   unfree packages are allowed by default: a switch installs one"

  out="$(kz 'kubaos package unfree off --no-switch && kubaos package unfree' 2>&1)" && [[ ${out##*$'\n'} == off ]] ||
    { echo "FAIL kubaos package unfree off"; printf '%s\n' "$out"; return 1; }
  grep -qx 'allow_unfree = false' "$u/kubaos.d/nixpkgs.toml" ||
    { echo "FAIL kubaos.d/nixpkgs.toml"; cat "$u/kubaos.d/nixpkgs.toml"; return 1; }
  ! kz 'kubaos switch --foreground' >"$LOGS/packages-free-only.log" 2>&1 &&
    grep -q 'unfree license' "$LOGS/packages-free-only.log" ||
    { echo "FAIL with unfree packages off, the switch built home.nix's unfree package"; tail -20 "$LOGS/packages-free-only.log"; return 1; }
  echo "ok   with unfree packages off, the home build refuses one"
  out="$(kz 'kubaos package add unrar --no-switch' 2>&1)" &&
    { echo "FAIL kubaos package add unrar went ahead with unfree packages off"; printf '%s\n' "$out"; return 1; }
  grep -q 'unrar is unfree, and unfree packages are turned off (allow_unfree in kubaos.d/nixpkgs.toml): turn them back on with `kubaos package unfree on`' <<<"$out" &&
    ! grep -q unrar "$u/kubaos.d/packages.toml" ||
    { echo "FAIL kubaos package add unrar with unfree packages off"; printf '%s\n' "$out"; return 1; }
  echo "ok   with unfree packages off, kubaos package add refuses unrar, naming the setting"

  kz 'kubaos package unfree on --foreground' >"$LOGS/packages-unfree-on.log" 2>&1 && has_bin kubaos-unfree-test ||
    { echo "FAIL kubaos package unfree on"; tail -20 "$LOGS/packages-unfree-on.log"; return 1; }
  echo "ok   kubaos package unfree on switches, and the unfree package installs again"
}

# A live preview: the home switched to another KubaOS checkout and back to
# the release (the repo copy), through /out/kubaos and a user flake like the
# installer's. The checkout is a worktree of a clone of the repo copy under a
# Projects root of its own, on a branch, with an uncommitted line in a file the
# home links (branding/fastfetch.txt), so only that file builds anew. Switches
# run --foreground, as in packages.
c_preview() {
  local u="$USER_HOME/.config/kubaos/home" record="$USER_HOME/.local/state/kubaos/preview.json"
  local root="$USER_HOME/Projects/preview-root" linked="$USER_HOME/.config/fastfetch/kubaos.txt"
  local env="KUBAOS_PROJECTS=$USER_HOME/Projects/preview-root" out lock marked broken
  marked="$root/marked"
  broken="$root/broken"
  install_kubaos || return 1
  user_flake "$u" || return 1
  as_user cp "$REPO/user-flake/home.nix" "$u/home.nix"
  rm -rf "$root" "$record"
  as_user git clone -q "$REPO" "$root/kubaos"
  as_user git -C "$root/kubaos" worktree add -q -b feature/marked "$marked"
  as_user git -C "$root/kubaos" worktree add -q -b feature/broken "$broken"
  echo 'kubaos-preview-marker' | as_user tee -a "$marked/branding/fastfetch.txt" >/dev/null
  echo '{' | as_user tee "$broken/flake.nix" >/dev/null

  kz 'kubaos switch --foreground' >"$LOGS/preview-release.log" 2>&1 ||
    { echo "FAIL the first switch, on the release"; tail -20 "$LOGS/preview-release.log"; return 1; }
  ! grep -q kubaos-preview-marker "$linked" || { echo "FAIL the marker is in the release"; return 1; }
  lock="$(sha256sum <"$u/flake.lock")"

  out="$(kz "$env kubaos switch --from feature/marked --foreground" 2>&1)" ||
    { echo "FAIL kubaos switch --from feature/marked"; printf '%s\n' "$out" | tail -20; return 1; }
  grep -q kubaos-preview-marker "$linked" ||
    { echo "FAIL the home isn't built from the checkout: no marker in $linked"; return 1; }
  [[ $(jq -r '.path, .branch, .dirty' "$record" | paste -sd' ') == "$marked feature/marked true" ]] &&
    [[ $(jq -r .commit "$record") == "$(as_user git -C "$marked" rev-parse HEAD)" ]] ||
    { echo "FAIL the preview record"; cat "$record"; return 1; }
  [[ $(sha256sum <"$u/flake.lock") == "$lock" ]] || { echo "FAIL the preview wrote flake.lock"; return 1; }
  echo "ok   kubaos switch --from a branch: its worktree built, uncommitted change included; recorded; flake.lock untouched"

  out="$(kz 'kubaos switch --foreground' 2>&1)" && { echo "FAIL a plain switch during a preview ran"; return 1; }
  grep -q -- '--release' <<<"$out" && grep -q -- "--from $marked" <<<"$out" ||
    { echo "FAIL a plain switch during a preview doesn't name both ways on"; printf '%s\n' "$out"; return 1; }
  echo "ok   a plain kubaos switch during a preview is refused, naming --from and --release"

  out="$(kz "kubaos switch --from '$broken' --foreground" 2>&1)" &&
    { echo "FAIL a checkout that doesn't evaluate was switched to"; return 1; }
  grep -q "doesn't evaluate" <<<"$out" && [[ $(jq -r .path "$record") == "$marked" ]] &&
    grep -q kubaos-preview-marker "$linked" ||
    { echo "FAIL a broken checkout: not refused as it should be"; printf '%s\n' "$out" | tail -20; return 1; }
  echo "ok   a checkout that doesn't evaluate is refused, and the preview stays"

  out="$(kz 'kubaos switch --release --foreground' 2>&1)" ||
    { echo "FAIL kubaos switch --release"; printf '%s\n' "$out" | tail -20; return 1; }
  ! grep -q kubaos-preview-marker "$linked" && [[ ! -s $record ]] ||
    { echo "FAIL --release: still on the preview, or still recorded"; cat "$record"; return 1; }
  [[ $(sha256sum <"$u/flake.lock") == "$lock" ]] || { echo "FAIL --release changed flake.lock"; return 1; }
  echo "ok   kubaos switch --release: back on the release, the record emptied"
}

# The Suite framework, through the test Suite (home/suites/test.nix,
# in a kubaos built with the test-suite feature) and a user flake under git,
# so every file kubaos writes into kubaos.d/ must reach git's index for Nix to
# see it. Add, configure, scaffold and remove it, with a user override in
# ~/.config/kubaos/editor/nvim/ that must win throughout, and nothing of the Suite
# left in the generation afterwards. Switches run --foreground, as in packages.
c_suites() {
  local u="$USER_HOME/.config/kubaos/home" layer="$USER_HOME/.config/kubaos" out gen
  local mine="$USER_HOME/.config/kubaos/editor/nvim/plugins/test-suite.lua"
  local cache="$USER_HOME/.cache/kubaos-test-suite" project="$USER_HOME/Projects/test-suite-demo"
  install_kubaos || return 1
  user_flake "$u" || return 1
  as_user cp "$REPO/user-flake/home.nix" "$u/home.nix"
  rm -rf "$mine" "$cache" "$project"
  as_user git -C "$u" init -q
  as_user git -C "$u" add -A
  as_user git -C "$u" -c user.name=t -c user.email=t@t commit -q -m 'the user flake'
  # opts NAME: LazyVim's opts.kubaos_test_suite as "<greeting> <version>", as
  # the Defaults plus the user layer give them.
  opts() {
    cache_lock nvim
    zrun "nvim --headless +'lua local o = LazyVim.opts(\"LazyVim\").kubaos_test_suite or {}; io.write(\"\\nOPTS \", tostring(o.greeting), \" \", tostring(o.version))' +qa" 2>&1 |
      tr -d '\r' | sed -n 's/^OPTS //p'
  }

  out="$(kz 'kubaos suite add test --foreground' 2>&1)" ||
    { echo "FAIL kubaos suite add test"; printf '%s\n' "$out" | tail -20; return 1; }
  echo "ok   kubaos suite add test, then a switch"
  as_user git -C "$u" ls-files --error-unmatch kubaos.d/suites/test.toml >/dev/null 2>&1 ||
    { echo "FAIL kubaos.d/suites/test.toml not in git's index"; as_user git -C "$u" status --short; return 1; }
  echo "ok   the Suite file is in git's index (intent to add), so the flake sees it"
  has_bin kubaos-test-suite && [[ $(zrun 'kubaos-test-suite version') == 1 ]] ||
    { echo "FAIL kubaos-test-suite (version 1) not on PATH"; return 1; }
  [[ $(zrun 'print -r -- "$KUBAOS_TEST_SUITE_VERSION"') == 1 ]] ||
    { echo "FAIL KUBAOS_TEST_SUITE_VERSION is not 1"; return 1; }
  zrun 'print -r -- $PATH' | tr ':' '\n' | grep -qx "$USER_HOME/.local/share/kubaos-test-suite/bin" ||
    { echo "FAIL the Suite's PATH entry is missing"; return 1; }
  echo "ok   its package, environment variable and PATH entry"
  [[ -f $mine ]] && grep -q 'created ~/.config/kubaos/editor/nvim/plugins/test-suite.lua' <<<"$out" ||
    { echo "FAIL the missing user-layer file was not seeded"; printf '%s\n' "$out"; return 1; }
  out="$(opts)"
  [[ $out == "from the test Suite 1" ]] || { echo "FAIL the Suite's Neovim setting: $out"; return 1; }
  echo "ok   a commented example seeded in the user layer; the Suite's Neovim setting is in the Defaults"

  printf 'return {\n  { "LazyVim/LazyVim", opts = { kubaos_test_suite = { greeting = "mine" } } },\n}\n' |
    as_user tee "$mine" >/dev/null
  as_user cp "$mine" "$LOGS/suites-mine.lua"
  out="$(opts)"
  [[ $out == "mine 1" ]] || { echo "FAIL the user's override in $mine doesn't win: $out"; return 1; }
  echo "ok   the user's override of the same setting wins"

  out="$(kz 'kubaos suite configure test --version 2 --extra greeting --foreground' 2>&1)" ||
    { echo "FAIL kubaos suite configure test"; printf '%s\n' "$out" | tail -20; return 1; }
  [[ $(zrun 'kubaos-test-suite version') == 2 ]] && has_bin kubaos-test-suite-greet ||
    { echo "FAIL configure: not version 2 with kubaos-test-suite-greet"; return 1; }
  cmp -s "$mine" "$LOGS/suites-mine.lua" && grep -q 'kept ~/.config/kubaos/editor/nvim/plugins/test-suite.lua' <<<"$out" ||
    { echo "FAIL configure wrote to the user's file"; printf '%s\n' "$out"; return 1; }
  out="$(opts)"
  [[ $out == "mine 2" ]] || { echo "FAIL after configure: $out (want the user's greeting, the Suite's version 2)"; return 1; }
  echo "ok   kubaos suite configure --version 2 --extra greeting: applied, the user's file untouched and still winning"

  out="$(kz 'kubaos suite new test test-suite-demo' 2>&1)" ||
    { echo "FAIL kubaos suite new"; printf '%s\n' "$out" | tail -20; return 1; }
  grep -q 'Made by the test Suite 2' "$project/README.md" ||
    { echo "FAIL no scaffolded project in $project"; return 1; }
  echo "ok   kubaos suite new test test-suite-demo: a project in the Projects root"
  zrun 'kubaos-test-suite run' && [[ -s $cache/build.log ]] || { echo "FAIL kubaos-test-suite run made no cache"; return 1; }

  out="$(kz 'kubaos suite remove test --foreground' 2>&1)" ||
    { echo "FAIL kubaos suite remove test"; printf '%s\n' "$out" | tail -20; return 1; }
  grep -qE '^cache ~/.cache/kubaos-test-suite \([0-9.]+ [KMG]?i?B\): kept' <<<"$out" && [[ -s $cache/build.log ]] ||
    { echo "FAIL remove: the cache's size not shown, or the cache not kept"; printf '%s\n' "$out"; return 1; }
  grep -qx '  ~/.config/kubaos/editor/nvim/plugins/test-suite.lua' <<<"$out" ||
    { echo "FAIL remove doesn't list the user-layer file that mentions the Suite"; printf '%s\n' "$out"; return 1; }
  cmp -s "$mine" "$LOGS/suites-mine.lua" && [[ -f $project/README.md ]] ||
    { echo "FAIL remove touched the user's file or the project"; return 1; }
  echo "ok   kubaos suite remove test: the cache kept with its size shown, the user's file listed, the project untouched"
  ! as_user git -C "$u" ls-files --error-unmatch kubaos.d/suites/test.toml >/dev/null 2>&1 ||
    { echo "FAIL the deleted Suite file is still in git's index"; return 1; }
  gen="$(readlink -f "$USER_HOME/.local/state/nix/profiles/home-manager")"
  out="$("$NIX_BIN/nix-store" -qR "$gen" | grep kubaos-test-suite || true)"
  [[ -z $out ]] || { echo "FAIL the Suite's packages are still in the generation:"; printf '%s\n' "$out"; return 1; }
  ! grep -qs -e KUBAOS_TEST_SUITE -e kubaos-test-suite "$gen/home-path/etc/profile.d/hm-session-vars.sh" ||
    { echo "FAIL the Suite's environment is still in hm-session-vars.sh"; return 1; }
  ! grep -qs kubaos_test_suite "$gen/home-files/.config/nvim/lua/kubaos/suites.lua" ||
    { echo "FAIL the Suite's Neovim setting is still in the Defaults"; return 1; }
  [[ -f $gen/home-files/.config/nvim/lua/kubaos/suites.lua && ! -e $gen/home-files/.local/share/kubaos/kubaos.d/suites/test.toml ]] ||
    { echo "FAIL the generation's suites.lua is missing, or its kubaos.d still has test.toml"; return 1; }
  ! has_bin kubaos-test-suite && ! has_bin kubaos-test-suite-greet || { echo "FAIL the Suite's commands are still on PATH"; return 1; }
  out="$(opts)"
  [[ $out == "mine nil" ]] || { echo "FAIL after remove, Neovim has: $out"; return 1; }
  echo "ok   no leftovers in the generation: packages, environment, PATH, Neovim setting and Suite file all gone"

  out="$(kz 'kubaos suite add test --no-switch && kubaos suite remove test --purge-caches --no-switch' 2>&1)" ||
    { echo "FAIL add + remove --purge-caches"; printf '%s\n' "$out" | tail -20; return 1; }
  [[ ! -e $cache ]] && grep -qE '^cache ~/.cache/kubaos-test-suite \([0-9.]+ [KMG]?i?B\): deleted' <<<"$out" ||
    { echo "FAIL --purge-caches: the cache is still there"; printf '%s\n' "$out"; return 1; }
  [[ -f $project/README.md ]] && cmp -s "$mine" "$LOGS/suites-mine.lua" ||
    { echo "FAIL --purge-caches touched the project or the user's file"; return 1; }
  echo "ok   kubaos suite remove --purge-caches: the cache gone, the project and the user's file still there"
  rm -rf "$mine" "$project" "$u/.git"
}

# Project pins: `kubaos pin` and the `use suite` direnv function
# (config/direnv/lib/suite.sh), with the test Suite installed at version 1 and
# folders under ~/Projects/acme pinned to others. Version 3 is in no
# generation, so its first use fetches it into the store. Each cd is in an
# interactive zsh, whose direnv hook loads the folder as a terminal would.
c_pins() {
  local u="$USER_HOME/.config/kubaos/home" acme="$USER_HOME/Projects/acme" out entry drv3 out3
  local typo="$USER_HOME/Projects/typo"
  install_kubaos || return 1
  user_flake "$u" || return 1
  as_user cp "$REPO/user-flake/home.nix" "$u/home.nix"
  rm -rf "$acme" "$typo"
  as_user install -d "$acme/old-app/src" "$acme/new-app" "$acme/other-app" "$typo"
  # at DIR: direnv's log and `V=<kubaos-test-suite version>` in DIR, then `H=<…>` back in the home.
  at() {
    zsh_pty "zsh -l -i -c 'cd $1; print -r -- V=\$(kubaos-test-suite version); cd; print -r -- H=\$(kubaos-test-suite version)'"
  }
  # version_at DIR: kubaos-test-suite's version in DIR.
  version_at() { at "$1" | sed -n 's/^V=//p'; }

  out="$(kz 'kubaos suite add test --foreground' 2>&1)" ||
    { echo "FAIL kubaos suite add test"; printf '%s\n' "$out" | tail -20; return 1; }
  entry="$USER_HOME/.local/share/kubaos/toolchains/test/3"
  { IFS= read -r drv3 && IFS= read -r out3; } <"$entry" && [[ $drv3 == /nix/store/*.drv!bin && $out3 == /nix/store/*-bin ]] ||
    { echo "FAIL no toolchain entry for the test Suite's version 3 at $entry"; return 1; }
  "$NIX_BIN/nix-store" --delete "$out3" >/dev/null 2>&1 || true
  [[ ! -e $out3 ]] || { echo "FAIL $out3 is still in the store, so version 3 can't show a first fetch"; return 1; }
  [[ -e ${drv3%!bin} ]] || { echo "FAIL the generation doesn't keep ${drv3%!bin}"; return 1; }
  echo "ok   the test Suite installed at 1; version 3 has a toolchain entry for its bin output, its derivation kept, the output not in the store"

  out="$(kz "kubaos pin test 3 $acme" 2>&1)" || { echo "FAIL kubaos pin test 3"; printf '%s\n' "$out"; return 1; }
  [[ $(cat "$acme/.envrc") == 'use suite test 3' ]] && grep -q 'fetches it the first time' <<<"$out" ||
    { echo "FAIL kubaos pin wrote:"; cat "$acme/.envrc"; printf '%s\n' "$out"; return 1; }
  out="$(kz "kubaos pins $acme/old-app/src" 2>&1)"
  grep -qF 'test 3  from ~/Projects/acme/.envrc  (installed: 1; fetched on first use)' <<<"$out" ||
    { echo "FAIL kubaos pins below the pin:"; printf '%s\n' "$out"; return 1; }
  echo "ok   kubaos pin test 3 ~/Projects/acme: the .envrc line, and kubaos pins sees it two folders down"

  out="$(at "$acme/old-app/src")"
  [[ $(sed -n 's/^V=//p' <<<"$out") == 3 && $(sed -n 's/^H=//p' <<<"$out") == 1 ]] ||
    { echo "FAIL cd ~/Projects/acme/old-app/src should give 3, the home 1:"; printf '%s\n' "$out"; return 1; }
  grep -q 'fetching test 3 into the Nix store' <<<"$out" && [[ -e $out3 ]] ||
    { echo "FAIL version 3 was not fetched on first use"; printf '%s\n' "$out"; return 1; }
  local roots="$acme/.direnv/suites/test" inode
  [[ $(readlink "$roots/3-bin") == "$out3" ]] ||
    { echo "FAIL no GC root ~/Projects/acme/.direnv/suites/test/3-bin for version 3"; ls -la "$roots" 2>&1; return 1; }
  echo "ok   cd ~/Projects/acme/old-app/src: version 3, fetched into the store on first use and kept by a GC root; leaving gives 1 back"
  # nix-store --add-root replaces the link, so an unchanged inode means the load realised nothing.
  inode="$(stat -c %i "$roots/3-bin")"
  out="$(at "$acme/old-app")"
  [[ $(sed -n 's/^V=//p' <<<"$out") == 3 && $(stat -c %i "$roots/3-bin") == "$inode" ]] ||
    { echo "FAIL the second cd should use version 3 as it is, realising nothing"; printf '%s\n' "$out"; return 1; }
  echo "ok   the next cd uses it as it is, without asking Nix"

  kz "kubaos pin test 2 $acme" >/dev/null 2>&1 && [[ $(version_at "$acme/old-app") == 2 && $(ls "$roots") == 2-bin ]] ||
    { echo "FAIL re-pinning to 2 should leave only the 2-bin root"; ls -la "$roots"; return 1; }
  kz "kubaos pin test 3 $acme" >/dev/null 2>&1 && [[ $(version_at "$acme/old-app") == 3 && $(ls "$roots") == 3-bin ]] ||
    { echo "FAIL re-pinning to 3 should leave only the 3-bin root"; ls -la "$roots"; return 1; }
  echo "ok   re-pinning drops the old version's GC root"

  printf 'source_up\n' | as_user tee "$acme/new-app/.envrc" >/dev/null
  out="$(kz "kubaos pin test 2 $acme/new-app" 2>&1)" || { echo "FAIL kubaos pin test 2"; printf '%s\n' "$out"; return 1; }
  [[ $(version_at "$acme/new-app") == 2 ]] || { echo "FAIL the nearer pin (2) doesn't win in new-app"; at "$acme/new-app"; return 1; }
  printf 'export OTHER=1\n' | as_user tee "$acme/other-app/.envrc" >/dev/null
  zrun "direnv allow $acme/other-app" >/dev/null 2>&1
  out="$(kz "kubaos pins $acme/other-app" 2>&1)"
  [[ $(version_at "$acme/other-app") == 1 ]] && grep -qF "doesn't reach here: ~/Projects/acme/other-app/.envrc has no \`source_up\`" <<<"$out" ||
    { echo "FAIL other-app's .envrc without source_up: want 1, said so by kubaos pins"; printf '%s\n' "$out"; return 1; }
  out="$(kz 'kubaos pins --all' 2>&1)"
  # shellcheck disable=SC2088 # kubaos prints paths under the home with a literal ~.
  grep -qF '~/Projects/acme  test 3' <<<"$out" && grep -qF '~/Projects/acme/new-app  test 2' <<<"$out" ||
    { echo "FAIL kubaos pins --all:"; printf '%s\n' "$out"; return 1; }
  echo "ok   nested: new-app's own pin (2, after source_up) wins; other-app's .envrc without source_up keeps 1, and kubaos pins says why"

  printf 'use suite test 9\n' | as_user tee "$typo/.envrc" >/dev/null
  zrun "direnv allow $typo" >/dev/null 2>&1
  out="$(at "$typo")"
  grep -qF 'the test Suite has no version 9 (it has: 1, 2, 3)' <<<"$out" && [[ $(sed -n 's/^V=//p' <<<"$out") == 1 ]] ||
    { echo "FAIL use suite test 9 should say which versions there are and leave 1"; printf '%s\n' "$out"; return 1; }
  echo "ok   use suite test 9: the versions there are, and the installed one stays"
  # The entry appearing while a shell is in the folder, as a switch would add it, reloads it.
  out="$(zsh_pty "zsh -l -i -c 'cd $typo; cp $USER_HOME/.local/share/kubaos/toolchains/test/3 $USER_HOME/.local/share/kubaos/toolchains/test/9; _direnv_hook; print -r -- V=\$(kubaos-test-suite version)'")"
  rm -f "$USER_HOME/.local/share/kubaos/toolchains/test/9"
  [[ $(sed -n 's/^V=//p' <<<"$out") == 3 ]] ||
    { echo "FAIL a shell in the folder should reload once the entry for 9 appears"; printf '%s\n' "$out"; return 1; }
  echo "ok   a shell already in the folder reloads when the missing entry appears"

  out="$(kz "kubaos unpin test $acme" 2>&1)" || { echo "FAIL kubaos unpin"; printf '%s\n' "$out"; return 1; }
  [[ ! -e $acme/.envrc && ! -e $acme/.direnv && $(version_at "$acme/old-app") == 1 && $(version_at "$acme/new-app") == 2 ]] ||
    { echo "FAIL after unpin: want no ~/Projects/acme/.envrc nor .direnv, 1 in old-app, 2 in new-app"; printf '%s\n' "$out"; return 1; }
  echo "ok   kubaos unpin test ~/Projects/acme: the .envrc and its GC roots gone, old-app back to 1, new-app keeps its own 2"

  rm -rf "$acme" "$typo"
  out="$(kz 'kubaos suite remove test --foreground' 2>&1)" ||
    { echo "FAIL kubaos suite remove test"; printf '%s\n' "$out" | tail -20; return 1; }
}

# The user layer: test/user-layer-check, which the VM bench also
# runs on an installed system. Commits to the checkout, so it runs after the
# other checks but the update ones, which start from a clone of it.
# The file manager (home/files.nix, crates/kubaos-files): Files (Nautilus)
# is the default for folders and archives, with the viewers KubaOS ships for
# the rest; "Show in folder" goes to Nautilus's own FileManager1 service, and
# yazi stays under "Open With" and on its key. Nautilus needs a display to
# start, so opening a window is checked on the laptop; `ghostty` is a stub on
# PATH that logs its arguments, so yazi opens no window either.
files_wait() { # files_wait LOG PATTERN: up to 10 s for a line matching PATTERN
  local _
  for _ in $(seq 50); do
    grep -qxF -- "$2" "$1" 2>/dev/null && return 0
    sleep 0.2
  done
  return 1
}

c_files() {
  local bad=0 out f want stub="$LOGS/files-stub" log="$LOGS/files-ghostty.log" path
  local data="$USER_HOME/.local/share" dir="$USER_HOME/Downloads/Two words"
  local bus=(dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus)
  local unit="$USER_HOME/.config/systemd/user/kubaos-tidy-downloads.service"
  local bin="$USER_HOME/.nix-profile/bin/kubaos-files"

  for f in applications/kubaos-files.desktop applications/mimeapps.list \
    nautilus-python/extensions/ghostty.py nautilus-python/extensions/kubaos.py; do
    [[ -f $data/$f ]] || { echo "FAIL the switch left no ~/.local/share/$f"; bad=1; }
  done
  if [[ -e $data/dbus-1/services/org.freedesktop.FileManager1.service ]]; then
    echo "FAIL a FileManager1 service file in ~/.local/share shadows Nautilus's"
    bad=1
  fi

  for want in inode/directory:org.gnome.Nautilus application/zip:org.gnome.Nautilus \
    application/x-7z-compressed:org.gnome.Nautilus image/png:org.gnome.Loupe image/heic:org.gnome.Loupe \
    video/mp4:org.gnome.Showtime application/pdf:org.gnome.Papers; do
    out="$(as_user xdg-mime query default "${want%%:*}" 2>&1)"
    if [[ $out != "${want#*:}.desktop" ]]; then
      echo "FAIL xdg-mime's default for ${want%%:*} is '$out', not ${want#*:}.desktop"
      bad=1
    elif [[ ! -f /usr/share/applications/$out ]]; then
      echo "FAIL ${want%%:*} defaults to $out, which apt didn't install"
      bad=1
    fi
  done
  ((bad)) || echo "ok   folders and archives open in Files, images, video and PDFs in the viewers KubaOS ships"
  # Open With lists GLib's apps for the type; the image has no `gio` command.
  if as_user /usr/bin/python3 -I -c '
from gi.repository import Gio
print("\n".join(app.get_id() for app in Gio.AppInfo.get_all_for_type("inode/directory")))
' 2>&1 | tee "$LOGS/files-open-with.log" | grep -qx 'kubaos-files.desktop'; then
    echo "ok   yazi is under Open With for folders"
  else
    echo "FAIL kubaos-files.desktop isn't among the apps for folders: $(tr "\n" " " <"$LOGS/files-open-with.log")"
    bad=1
  fi

  # nautilus-python loads each extension into Nautilus's process; one that
  # doesn't load takes its right-click actions with it.
  out="$(as_user /usr/bin/python3 -I -c '
import sys
from gi.repository import Nautilus
for path in sys.argv[1:]:
    exec(compile(open(path).read(), path, "exec"), {"__name__": path.rsplit("/", 1)[1][:-3], "__file__": path})
' "$data/nautilus-python/extensions/ghostty.py" "$data/nautilus-python/extensions/kubaos.py" 2>&1 | grep -v PyGIWarning || true)"
  [[ -z $out ]] && echo "ok   the Ghostty and KubaOS right-click extensions load" ||
    { echo "FAIL the right-click extensions don't load: $out"; bad=1; }

  as_user "${bus[@]}" org.freedesktop.DBus.ReloadConfig >/dev/null 2>&1 || true
  if as_user "${bus[@]}" org.freedesktop.DBus.ListActivatableNames | grep -q '"org.freedesktop.FileManager1"' &&
    grep -q '^Exec=/usr/bin/nautilus ' /usr/share/dbus-1/services/org.freedesktop.FileManager1.service; then
    echo "ok   the session bus starts Nautilus for org.freedesktop.FileManager1"
  else
    echo "FAIL org.freedesktop.FileManager1 isn't Nautilus's to start"
    bad=1
  fi

  install -d -o "$USER_NAME" -g "$USER_NAME" "$stub" "$USER_HOME/Downloads" "$dir"
  printf '#!/bin/sh\nprintf "%%s\\n" "$*" >>%s\n' "$log" >"$stub/ghostty"
  chmod 755 "$stub/ghostty"
  : >"$log"
  chown "$USER_NAME:" "$log"
  path="$stub:$USER_HOME/.nix-profile/bin:/usr/local/bin:/usr/bin:/bin"
  as_user env PATH="$path" WAYLAND_DISPLAY=wayland-stub "$bin" open "$dir" >"$LOGS/files-yazi.log" 2>&1 || true
  if files_wait "$log" "-e yazi -- $dir"; then
    echo "ok   kubaos-files open, the yazi key's command, opens yazi on the folder"
  else
    echo "FAIL kubaos-files open '$dir' didn't run ghostty -e yazi on it:"
    cat "$log" "$LOGS/files-yazi.log"
    bad=1
  fi

  # Files that arrived today stay put, and the daily run does nothing with
  # the setting off. Moving older ones is unit-tested: a file's ctime can't be
  # set back here. stderr is notify-send's, with no notification server.
  as_user touch "$USER_HOME/Downloads/report.pdf"
  out="$(as_user "$bin" tidy --scheduled 2>/dev/null)" && [[ -z $out ]] ||
    { echo "FAIL the daily tidy with the setting off printed: $out"; bad=1; }
  out="$(as_user "$bin" tidy 2>/dev/null)" && [[ $out == "0 moved" && -f $USER_HOME/Downloads/report.pdf ]] ||
    { echo "FAIL tidying Downloads moved today's file, or printed: $out"; bad=1; }
  as_user "$bin" tidy on >/dev/null && [[ -e $USER_HOME/.config/kubaos/desktop/files/tidy-downloads ]] &&
    as_user "$bin" tidy off >/dev/null && [[ ! -e $USER_HOME/.config/kubaos/desktop/files/tidy-downloads ]] ||
    { echo "FAIL kubaos-files tidy on|off doesn't turn the daily tidy on and off"; bad=1; }
  grep -qE '^ExecStart=.*/bin/kubaos-files tidy --scheduled$' "$unit" 2>/dev/null &&
    [[ -e $USER_HOME/.config/systemd/user/timers.target.wants/kubaos-tidy-downloads.timer ]] ||
    { echo "FAIL the daily timer isn't in place to run kubaos-files tidy --scheduled"; bad=1; }
  ((bad)) || echo "ok   Tidy Downloads leaves today's files, and the daily timer runs only when it's on"

  rm -rf "$stub" "$dir" "$USER_HOME/Downloads/report.pdf"
  return "$bad"
}

# The automount for removable drives (home/automount.nix,
# crates/kubaos-automount). The container has no drives and no udisks2
# daemon, so mounting and ejecting are checked on the laptop; here the unit,
# the setting, the eject command's refusal and the photo import.
c_automount() {
  local bad=0 out rc
  local bin="$USER_HOME/.nix-profile/bin/kubaos-automount" flag="$USER_HOME/.config/kubaos/desktop/files/no-automount"
  local unit="$USER_HOME/.config/systemd/user/kubaos-automount.service"
  local card="$LOGS/automount-card" pictures="$USER_HOME/Pictures"

  grep -qE '^ExecStart=.*/bin/kubaos-automount watch$' "$unit" 2>/dev/null &&
    echo "ok   the user unit runs kubaos-automount watch" ||
    { echo "FAIL $unit doesn't run kubaos-automount watch"; bad=1; }
  [[ -x $bin ]] || { echo "FAIL the switch left no $bin"; return 1; }

  as_user "$bin" off >/dev/null && [[ -e $flag ]] && as_user "$bin" on >/dev/null && [[ ! -e $flag ]] &&
    echo "ok   kubaos-automount off|on turns mounting off and on" ||
    { echo "FAIL kubaos-automount off|on doesn't change $flag"; bad=1; }
  rc=0
  out="$(as_user "$bin" eject /org/freedesktop/UDisks2/drives/Nothing 2>&1)" || rc=$?
  ((rc == 1)) && [[ $out == kubaos-automount:* ]] &&
    echo "ok   ejecting a drive that isn't there fails with the reason in one line" ||
    { echo "FAIL eject of no drive exited $rc: $out"; bad=1; }
  rc=0
  as_user "$bin" mount >/dev/null 2>&1 || rc=$?
  ((rc == 2)) || { echo "FAIL a usage error exits $rc, not 2"; bad=1; }

  # A card with two photos from July 2024, by mtime as they carry no EXIF.
  install -d -o "$USER_NAME" -g "$USER_NAME" "$card/DCIM/100CANON"
  as_user sh -c "printf one >'$card/DCIM/100CANON/IMG_0001.JPG'; printf two >'$card/DCIM/100CANON/IMG_0002.JPG'"
  as_user touch -d '2024-07-15 12:00:00Z' "$card/DCIM/100CANON/IMG_0001.JPG" "$card/DCIM/100CANON/IMG_0002.JPG"
  out="$(as_user "$bin" import "$card" 2>&1)" && [[ $out == "2 copied, 0 already there, 0 failed" ]] &&
    [[ $(cat "$pictures/2024/07/IMG_0002.JPG" 2>/dev/null) == two ]] ||
    { echo "FAIL importing the card into ~/Pictures/2024/07: $out"; bad=1; }
  out="$(as_user "$bin" import "$card" 2>&1)" && [[ $out == "0 copied, 2 already there, 0 failed" ]] &&
    [[ -f $card/DCIM/100CANON/IMG_0001.JPG ]] ||
    { echo "FAIL a second import copied again, or touched the card: $out"; bad=1; }
  ((bad)) || echo "ok   Import photos copies a card's DCIM by month, once, and leaves the card as it was"

  rm -rf "$card" "$pictures/2024"
  return "$bad"
}

# The Bluetooth pairing agent (home/bluetooth.nix, crates/kubaos-bluetooth),
# started as its user unit would be, against the container's system bus,
# where no bluetoothd runs: it takes its name on the session bus, writes the
# state file the shell watches, and turns down what the shell can't ask of
# it. Pairing a device needs an adapter, so that is checked on the laptop.
c_bluetooth() {
  local bad=0 out i rc
  local bin="$USER_HOME/.nix-profile/bin/kubaos-bluetooth" state="$RUNTIME_DIR/kubaos-bluetooth.json"
  local unit="$USER_HOME/.config/systemd/user/kubaos-bluetooth.service"

  if grep -qE '^ExecStart=.*/bin/kubaos-bluetooth agent$' "$unit" 2>/dev/null; then
    echo "ok   the user unit runs kubaos-bluetooth agent"
  else
    echo "FAIL $unit doesn't run kubaos-bluetooth agent"
    bad=1
  fi
  [[ -x $bin ]] || { echo "FAIL the switch left no $bin"; return 1; }

  out="$(as_user "$bin" pair /org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF 2>&1)" &&
    { echo "FAIL pair succeeded with no agent running"; bad=1; }
  if [[ $out == "The pairing agent isn't running"* ]]; then
    echo "ok   with no agent, pair says so in one line"
  else
    echo "FAIL pair with no agent printed: $out"
    bad=1
  fi

  rm -f "$state"
  as_user "$bin" agent >"$LOGS/bluetooth-agent.log" 2>&1 &
  for i in $(seq 50); do
    [[ -s $state ]] && break
    sleep 0.1
  done
  if [[ $(cat "$state" 2>/dev/null) == '{"agent":false,"prompt":null,"pairing":[],"errors":{}}' ]]; then
    echo "ok   the agent writes its state, unregistered with no bluetoothd (${i}00 ms)"
  else
    echo "FAIL the agent wrote no state to $state:"
    cat "$state" "$LOGS/bluetooth-agent.log" 2>/dev/null
    bad=1
  fi

  out="$(as_user "$bin" answer 1 yes 2>&1)" && { echo "FAIL answering no prompt succeeded"; bad=1; }
  [[ $out == "There's no prompt 1 to answer"* ]] || { echo "FAIL answer with no prompt printed: $out"; bad=1; }
  out="$(as_user "$bin" pair /org/freedesktop/DBus 2>&1)" && { echo "FAIL pairing a non-BlueZ object succeeded"; bad=1; }
  [[ $out == *"isn't a Bluetooth adapter or device" ]] || { echo "FAIL pair on a non-BlueZ object printed: $out"; bad=1; }
  out="$(as_user "$bin" scan on /org/bluez/hci0 2>&1)" && { echo "FAIL scanning with no bluetoothd succeeded"; bad=1; }
  [[ $out == "Couldn't scan"* ]] || { echo "FAIL scan with no bluetoothd printed: $out"; bad=1; }
  rc=0
  as_user "$bin" scan maybe /org/bluez/hci0 >/dev/null 2>&1 || rc=$?
  ((rc == 2)) || { echo "FAIL a usage error exits $rc, not 2"; bad=1; }
  ((bad)) || echo "ok   the agent turns down a stale answer, a foreign object and a scan bluetoothd can't take"

  pkill -u "$USER_NAME" -f 'kubaos-bluetooth agent' || true
  rm -f "$state"
  return "$bad"
}

# The phone link (home/phone.nix, crates/kubaos-phone). No phone and no
# systemd user manager here, so pairing and sending are checked on the laptop;
# here the unit behind its flag, that nothing of KDE Connect's sits where a bus
# or a session would start it, and the command's opt-in.
c_phone() {
  local bad=0 out rc f
  local bin="$USER_HOME/.nix-profile/bin/kubaos-phone" flag="$USER_HOME/.config/kubaos/desktop/phone/phone-link"
  local unit="$USER_HOME/.config/systemd/user/kubaos-phone.service"

  grep -qE '^ExecStart=/nix/store/[^ ]+-kdeconnect-kde-[^/]+/bin/kdeconnectd$' "$unit" 2>/dev/null &&
    grep -qxF "ConditionPathExists=$flag" "$unit" &&
    grep -qE '^ExecStartPost=.*/bin/kubaos-phone restrict$' "$unit" &&
    echo "ok   the user unit runs kdeconnectd only while $flag exists, and restricts its plugins" ||
    { echo "FAIL $unit doesn't run kdeconnectd behind the flag"; bad=1; }
  [[ -x $bin ]] || { echo "FAIL the switch left no $bin"; return 1; }

  for f in "$USER_HOME/.nix-profile/bin/kdeconnectd" "$USER_HOME/.nix-profile/bin/kdeconnect-cli" \
    "$USER_HOME/.nix-profile/share/dbus-1/services/org.kde.kdeconnect.service" \
    "$USER_HOME/.local/share/dbus-1/services/org.kde.kdeconnect.service" \
    "$USER_HOME/.nix-profile/etc/xdg/autostart/org.kde.kdeconnect.daemon.desktop" \
    "$USER_HOME/.config/autostart/org.kde.kdeconnect.daemon.desktop"; do
    [[ -e $f ]] && { echo "FAIL $f could start KDE Connect behind the flag"; bad=1; }
  done
  ((bad)) || echo "ok   no activation file, autostart entry or command of KDE Connect's in the home"

  rm -f "$flag"
  out="$(as_user "$bin" status --json 2>&1)"
  [[ $out == *'"on": false'* && $out == *'"running": false'* && $out == *'"devices": []'* ]] &&
    echo "ok   status says off, and lists nothing" ||
    { echo "FAIL status with the link off printed: $out"; bad=1; }
  rc=0
  out="$(as_user "$bin" ring 2>&1)" || rc=$?
  ((rc == 1)) && [[ $out == "kubaos-phone: the phone link is off"* ]] &&
    echo "ok   ringing with the link off fails, saying it's off" ||
    { echo "FAIL ring with the link off exited $rc: $out"; bad=1; }

  # With no user manager, starting the unit fails; the flag still follows on and off.
  as_user "$bin" on >/dev/null 2>&1
  [[ -e $flag ]] || { echo "FAIL kubaos-phone on left no $flag"; bad=1; }
  as_user "$bin" off >/dev/null 2>&1
  [[ -e $flag ]] && { echo "FAIL kubaos-phone off left $flag"; bad=1; }
  rc=0
  as_user "$bin" pair >/dev/null 2>&1 || rc=$?
  ((rc == 2)) || { echo "FAIL a usage error exits $rc, not 2"; bad=1; }
  ((bad)) || echo "ok   kubaos-phone on|off writes and removes the flag, and a usage error exits 2"

  rm -f "$flag"
  return "$bad"
}

c_user_layer() {
  zrun "'$REPO/test/user-layer-check' '$REPO'"
}

# scripts/dot-config on its own, in a scratch home: seeding, the README
# links, and moving a flat user layer into its groups without losing a file.
# The Lua side of the same table (config/hypr/user_layer.lua) beside it, and
# every user-layer path a default names literally, as a file include must,
# is one the switch created.
c_dot_config() {
  local bad=0 path missing=()
  as_user bash "$REPO/test/dot-config-test" "$REPO" || { echo "FAIL test/dot-config-test"; bad=1; }
  as_user lua5.5 "$REPO/test/lua/user_layer_test.lua" "$REPO" || { echo "FAIL test/lua/user_layer_test.lua"; bad=1; }
  while IFS= read -r path; do
    [[ -e $USER_HOME/.config/kubaos/$path ]] || missing+=("$path")
  done < <(as_user git -C "$REPO" grep -hoE '[~]/\.config/kubaos/[a-z]+/[a-z]+/[A-Za-z0-9_.-]+' -- \
    config/foot config/ghostty config/git config/tmux config/hypr/hypridle.conf config/hypr/hyprlock.conf config/fuzzel |
    sed 's|^~/\.config/kubaos/||' | sort -u)
  if ((${#missing[@]})); then
    echo "FAIL the defaults include user files the switch didn't create: ${missing[*]}"; bad=1
  else
    echo "ok   every user file a default includes is in the user layer"
  fi
  return "$bad"
}

# test/warm-target and the worktree hook in a throwaway repository: the copy of
# main's target/, the timestamps cargo judges, the cold fallbacks, and the
# worktree the hook makes and removes.
c_warm_target() {
  as_user bash "$REPO/test/warm-target-test" "$REPO" || { echo "FAIL test/warm-target-test"; return 1; }
}

# kubaos update, the channels and kubaos rollback, through /out/kubaos: a
# bare repo stands in for GitHub as origin, a root-owned clone of it on its
# `release` branch for /usr/share/kubaos (the real one, the wallpaper link,
# is put aside and back by c_rollback), and a user flake on that release like
# the installer's. apt-get is a shim that logs what it was asked, so nothing
# is downloaded or installed, and apt-cache one that has a candidate for every
# package; the container has no snapper. Docker and Chrome are turned off, so
# the package manifest's sync sets up no vendor repo. The three checks
# run in this order, each from where the last left the machine.
UPD_KUBAOS=/usr/share/kubaos
UPD_SAVED=/usr/share/kubaos.harness-saved
UPD_ORIGIN=/var/tmp/kubaos-origin.git
UPD_WORK=/var/tmp/kubaos-work
UPD_READY=/var/tmp/kubaos-update.ready
UPD_SYSTEM_LOG=/var/tmp/kubaos-system-migration.log
UPD_USER_LOG=$USER_HOME/.local/state/kubaos-user-migration.log
UPD_STATE=$USER_HOME/.local/state/kubaos

# work GIT-ARGS...: git in the work clone of origin, as kuba.
work() { as_user git -C "$UPD_WORK" -c user.name=test -c user.email=test@vm "$@"; }
# release_of: the commit /usr/share/kubaos's `release` branch is at.
release_of() { as_user git -C "$UPD_KUBAOS" rev-parse refs/heads/release; }
# record KEY: KEY's value in /var/lib/kubaos/release.
record() { sed -n "s/^$1=//p" /var/lib/kubaos/release 2>/dev/null; }
# lock_rev: the commit the user flake's flake.lock has KubaOS at.
lock_rev() { jq -r .nodes.kubaos.locked.rev "$USER_HOME/.config/kubaos/home/flake.lock"; }
# lines FILE: how many lines FILE has, 0 when it isn't there.
lines() { [[ -f $1 ]] && wc -l <"$1" | tr -d ' ' || echo 0; }

update_setup() {
  local u="$USER_HOME/.config/kubaos/home" dir
  install_kubaos || return 1
  printf '#!/bin/sh\necho "$*" >>%s/apt-get.log\n' "$LOGS" >/usr/local/bin/apt-get
  chmod 0755 /usr/local/bin/apt-get
  ln -sfn /usr/local/bin/apt-get /usr/local/sbin/apt-get
  printf '#!/bin/sh\n[ "$1" = policy ] || exit 0\nshift\nfor p; do printf "%%s:\\n  Candidate: 1.0\\n" "$p"; done\n' \
    >/usr/local/bin/apt-cache
  chmod 0755 /usr/local/bin/apt-cache
  rm -rf "$UPD_ORIGIN" "$UPD_WORK" /var/lib/kubaos /etc/kubaos /usr/share/kubaos-keys /usr/bin/kubaos \
    "$UPD_SYSTEM_LOG" "$UPD_USER_LOG" "$UPD_STATE/migrations" "$UPD_STATE/locks" \
    "$UPD_STATE/update-available" "$UPD_STATE/notified"
  # Root reads kuba's origin and kuba reads root's checkout, as on the machine.
  for dir in "$UPD_KUBAOS" "$UPD_ORIGIN"; do
    git config --system --get-all safe.directory | grep -qxF -- "$dir" ||
      git config --system --add safe.directory "$dir"
  done
  install -d /etc/kubaos /var/lib/kubaos
  printf 'docker=off\nchrome=off\n' >/etc/kubaos/apt-groups
  as_user git clone -q --bare "$REPO" "$UPD_ORIGIN" || { echo "FAIL cloning the repo copy as origin"; return 1; }
  as_user git clone -q "$UPD_ORIGIN" "$UPD_WORK" || { echo "FAIL cloning origin to work in"; return 1; }
  if [[ -e $UPD_KUBAOS && ! -e $UPD_SAVED ]]; then mv "$UPD_KUBAOS" "$UPD_SAVED"; fi
  rm -rf "$UPD_KUBAOS"
  git clone -q "$UPD_ORIGIN" "$UPD_KUBAOS" && git -C "$UPD_KUBAOS" checkout -q -B release origin/main ||
    { echo "FAIL making $UPD_KUBAOS"; return 1; }
  [[ -L $UPD_SAVED/wallpapers ]] && cp -P "$UPD_SAVED/wallpapers" "$UPD_KUBAOS/wallpapers"
  user_flake "$u" "git+file://$UPD_KUBAOS?ref=refs/heads/release" || return 1
  printf '{ pkgs, ... }:\n{\n  home.packages = [ ];\n}\n' | as_user tee "$u/home.nix" >/dev/null
  kz "nix flake lock '$u'" >"$LOGS/update-lock.log" 2>&1 ||
    { echo "FAIL nix flake lock of the user flake"; tail -20 "$LOGS/update-lock.log"; return 1; }
  [[ $(lock_rev) == "$(release_of)" ]] || { echo "FAIL the user flake isn't locked at release"; return 1; }
  touch "$UPD_READY"
  echo "ok   origin, /usr/share/kubaos on release at $(release_of | cut -c1-7), a user flake locked there"
}

c_update_edge() {
  local before head out log
  rm -f "$UPD_READY"
  update_setup || return 1
  before="$(release_of)"

  as_user install -d "$UPD_WORK/migrations/system" "$UPD_WORK/migrations/user"
  printf '#!/usr/bin/env bash\nset -euo pipefail\necho "$(id -un) $KUBAOS_DIR" >>%s\n' "$UPD_SYSTEM_LOG" |
    as_user tee "$UPD_WORK/migrations/system/2000000000-harness-system.sh" >/dev/null
  printf '#!/usr/bin/env bash\nset -euo pipefail\necho "$(id -un) $KUBAOS_USER_FLAKE" >>%s\n' "$UPD_USER_LOG" |
    as_user tee "$UPD_WORK/migrations/user/2000000000-harness-user.sh" >/dev/null
  # A new kubaos, told apart by its version.
  as_user sed -i 's/^version = "0.1.0"$/version = "0.1.0-harness"/' "$UPD_WORK/Cargo.toml"
  # Every crate that takes the workspace's version, read from the manifests,
  # so a new crate needs no change here: the release builds with --locked.
  local crates
  crates="$(grep -l '^version.workspace = true' "$UPD_WORK"/crates/*/Cargo.toml |
    xargs awk 'FNR == 1 { seen = 0 } !seen && /^name = / { gsub(/^name = "|"$/, ""); print; seen = 1 }' |
    paste -sd'|')"
  as_user awk -v v='version = "0.1.0-harness"' -v names="^name = \"($crates)\"$" \
    'prev ~ names && /^version = / { $0 = v } { print; prev = $0 }' \
    "$UPD_WORK/Cargo.lock" | as_user tee "$UPD_WORK/Cargo.lock.new" >/dev/null && as_user mv "$UPD_WORK/Cargo.lock.new" "$UPD_WORK/Cargo.lock"
  [[ $(grep -c '^version = "0.1.0-harness"$' "$UPD_WORK/Cargo.lock") == "$(tr '|' '\n' <<<"$crates" | wc -l)" ]] ||
    { echo "FAIL bumping the workspace's version in Cargo.lock"; return 1; }
  # A package new to the manifest, and the record of a machine where KubaOS
  # installed sway, which no group lists, and zsh, which one does.
  as_user sed -i 's/^packages = \["zsh"\]$/packages = ["zsh", "kubaos-harness-pkg"]/' "$UPD_WORK/system/packages.toml"
  grep -q '"kubaos-harness-pkg"' "$UPD_WORK/system/packages.toml" ||
    { echo "FAIL adding a package to the manifest"; return 1; }
  printf 'sway\tharness\nzsh\tshell\n' >/var/lib/kubaos/apt-installed
  work add migrations system/packages.toml Cargo.toml Cargo.lock &&
    work commit -q -m 'test: a new kubaos, a system and a user migration, a package' &&
    work push -q origin HEAD:main ||
    { echo "FAIL pushing a commit to origin/main"; return 1; }
  head="$(work rev-parse HEAD)"

  out="$(kz 'kubaos update --check' 2>&1)" || { echo "FAIL kubaos update --check"; printf '%s\n' "$out" | tail -20; return 1; }
  grep -q "^update=.*${head:0:7}" "$UPD_STATE/update-available" ||
    { echo "FAIL update-available doesn't name origin/main's head:"; cat "$UPD_STATE/update-available"; return 1; }
  [[ $(release_of) == "$before" ]] || { echo "FAIL the check moved release"; return 1; }
  echo "ok   kubaos update --check: update-available names main's new head, release stays"

  out="$(kz 'kubaos update -y --foreground' 2>&1)" ||
    { echo "FAIL kubaos update"; printf '%s\n' "$out" | tail -30; return 1; }
  printf '%s\n' "$out" >"$LOGS/update-edge.log"
  grep -q 'allowed_signers on this machine' <<<"$out" ||
    { echo "FAIL no warning that origin/main's head is taken unchecked"; return 1; }
  [[ $(release_of) == "$head" ]] || { echo "FAIL release is at $(release_of), not $head"; return 1; }
  [[ $(cat "$UPD_SYSTEM_LOG" 2>/dev/null) == "root $UPD_KUBAOS" ]] ||
    { echo "FAIL the system migration, as root once, logged: $(cat "$UPD_SYSTEM_LOG" 2>&1)"; return 1; }
  [[ $(cat "$UPD_USER_LOG" 2>/dev/null) == "kuba $USER_HOME/.config/kubaos/home" ]] ||
    { echo "FAIL the user migration, as kuba once, logged: $(cat "$UPD_USER_LOG" 2>&1)"; return 1; }
  [[ -f /var/lib/kubaos/migrations/2000000000-harness-system.sh && -f $UPD_STATE/migrations/2000000000-harness-user.sh ]] ||
    { echo "FAIL the migrations' markers are missing"; return 1; }
  [[ $(record commit) == "$head" && $(record channel) == edge ]] ||
    { echo "FAIL /var/lib/kubaos/release:"; cat /var/lib/kubaos/release; return 1; }
  [[ $(lock_rev) == "$head" ]] || { echo "FAIL the user flake's lock has KubaOS at $(lock_rev)"; return 1; }
  [[ $(lines /var/lib/kubaos/update-history) == 1 && ! -s $UPD_STATE/update-available ]] ||
    { echo "FAIL update-history or update-available after the update"; return 1; }
  grep -q '^upgrade --with-new-pkgs -y -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold$' \
    "$LOGS/apt-get.log" || { echo "FAIL apt-get upgrade never ran, or not keeping changed config files"; return 1; }
  echo "ok   kubaos update on edge: release at main's head, both migrations ran once, the record, history and lock follow"
  apt_manifest_followed || return 1

  # Off a terminal: plain lines, and the programs' output in the run's log.
  ! grep -q $'\e' <<<"$out" || { echo "FAIL escape codes in the update's output off a terminal"; return 1; }
  grep -q '^  ✓ Home' <<<"$out" && grep -q '^  Undo  *kubaos rollback$' <<<"$out" ||
    { echo "FAIL the update's output has no Home phase or no summary:"; printf '%s\n' "$out" | tail -20; return 1; }
  log="$(ls "$UPD_STATE"/update/*-update.log 2>/dev/null | tail -1)"
  [[ -n $log ]] && grep -q '2000000000-harness-user.sh' "$log" ||
    { echo "FAIL no update log in $UPD_STATE/update naming the migrations"; return 1; }
  echo "ok   kubaos update's plain output: phases and a summary, the programs' output in ${log#"$USER_HOME"/}"

  out="$(zrun ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh; command -v kubaos; kubaos --version" 2>&1)"
  [[ $out == "$USER_HOME/.nix-profile/bin/kubaos"$'\n'"kubaos 0.1.0-harness" ]] ||
    { echo "FAIL the kubaos on PATH after the update isn't the release's:"; printf '%s\n' "$out"; return 1; }
  out="$(as_user /usr/bin/kubaos --version 2>&1)"
  [[ $out == "kubaos 0.1.0-harness" ]] && cmp -s /usr/bin/kubaos "$UPD_KUBAOS/install/firstboot/kubaos-launcher" ||
    { echo "FAIL /usr/bin/kubaos isn't the launcher, or runs another kubaos: $out"; return 1; }
  echo "ok   the release's kubaos is on PATH after the update, and /usr/bin/kubaos, now the launcher, runs it"

  out="$(kz 'kubaos update -y --foreground --json' 2>/dev/null)" ||
    { echo "FAIL kubaos update --json, again"; printf '%s\n' "$out" | tail -30; return 1; }
  jq -e . >/dev/null <<<"$out" && [[ $(tail -1 <<<"$out" | jq -r .type) == summary ]] ||
    { echo "FAIL kubaos update --json isn't JSON lines ending in a summary:"; printf '%s\n' "$out" | tail -10; return 1; }
  grep -q 'newest release of its channel' <<<"$out" && [[ $(release_of) == "$head" ]] ||
    { echo "FAIL the second update moved release or didn't say it is current"; printf '%s\n' "$out" | tail -20; return 1; }
  [[ $(lines "$UPD_SYSTEM_LOG") == 1 && $(lines "$UPD_USER_LOG") == 1 ]] ||
    { echo "FAIL a migration ran again"; return 1; }
  echo "ok   kubaos update --json again: one JSON event a line, only packages, no migration runs twice"
}

# apt_manifest_followed: after c_update_edge's update, the package added to
# the manifest was installed, with no migration; sway, which KubaOS installed
# and no group lists, was removed, and nothing else; and `kubaos packages
# status` says what dpkg says.
apt_manifest_followed() {
  local out bad
  # The updater's own install, which keeps changed config files; the
  # package-manifest migration's group_install runs too, without them.
  grep -qE '^install -y --no-install-recommends -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold .*kubaos-harness-pkg( |$)' \
    "$LOGS/apt-get.log" || { echo "FAIL kubaos update didn't install the package added to the manifest:"; cat "$LOGS/apt-get.log"; return 1; }
  [[ $(grep -c '^remove ' "$LOGS/apt-get.log") == 1 ]] &&
    grep -qE '^remove -y -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold sway$' "$LOGS/apt-get.log" ||
    { echo "FAIL removing only sway, which KubaOS installed and no group lists:"; grep remove "$LOGS/apt-get.log"; return 1; }
  ! grep -q '^sway' /var/lib/kubaos/apt-installed && grep -q "^zsh"$'\t'"shell$" /var/lib/kubaos/apt-installed ||
    { echo "FAIL /var/lib/kubaos/apt-installed after the update:"; cat /var/lib/kubaos/apt-installed; return 1; }
  echo "ok   kubaos update installs a package added to the manifest and removes one only KubaOS installed and dropped"

  out="$(kz 'kubaos packages status --json' 2>&1)" ||
    { echo "FAIL kubaos packages status --json"; printf '%s\n' "$out" | tail -10; return 1; }
  bad="$(jq -r '.groups[].packages[] | "\(.name) \(.state)"' <<<"$out" | while read -r p state; do
    d="$(dpkg-query -W -f='${db:Status-Status}' "$p" 2>/dev/null || true)"
    held=no
    apt-mark showhold 2>/dev/null | grep -qx "$p" && held=yes
    case "$state:$d:$held" in
      installed:installed:no | held:installed:yes) ;;
      missing:installed:* | unavailable:installed:* | skipped:installed:*) echo "$p $state, dpkg $d" ;;
      missing:* | unavailable:* | skipped:*) ;;
      *) echo "$p $state, dpkg $d, held $held" ;;
    esac
  done)"
  [[ -z $bad ]] || { echo "FAIL kubaos packages status disagrees with dpkg:"; printf '%s\n' "$bad"; return 1; }
  [[ $(jq -r '.groups[] | select(.name == "shell") | .packages[] | select(.name == "kubaos-harness-pkg") | .state' <<<"$out") == missing ]] ||
    { echo "FAIL the package the shim didn't install isn't missing in the status"; return 1; }
  echo "ok   kubaos packages status matches dpkg for every package of the manifest"
}

c_update_stable() {
  local key=/var/tmp/kubaos-release-key openssh tagged out
  [[ -f $UPD_READY ]] || { echo "FAIL update_edge has to run first"; return 1; }
  openssh="$(as_user "$NIX_BIN/nix" build --no-link --print-out-paths --inputs-from "$REPO" 'nixpkgs#openssh^out')" ||
    { echo "FAIL building openssh for ssh-keygen"; return 1; }
  git config --system gpg.ssh.program "$openssh/bin/ssh-keygen"
  rm -f "$key" "$key.pub"
  as_user "$openssh/bin/ssh-keygen" -q -t ed25519 -N '' -C release -f "$key" || { echo "FAIL ssh-keygen"; return 1; }
  install -d -m 0755 /usr/share/kubaos-keys
  echo "release@kubaos.test namespaces=\"git\" $(cat "$key.pub")" >/usr/share/kubaos-keys/allowed_signers

  out="$(kz 'kubaos channel stable' 2>&1)" && [[ $(cat /etc/kubaos/channel) == stable ]] ||
    { echo "FAIL kubaos channel stable"; printf '%s\n' "$out"; return 1; }
  echo 0.1.1 | as_user tee "$UPD_WORK/VERSION" >/dev/null
  work add VERSION && work commit -q -m 'release: 0.1.1' &&
    work -c gpg.format=ssh -c user.signingkey="$key" -c user.email=release@kubaos.test tag -s v0.1.1 -m 'KubaOS 0.1.1' ||
    { echo "FAIL signing tag v0.1.1"; return 1; }
  tagged="$(work rev-parse HEAD)"
  echo 0.1.2 | as_user tee "$UPD_WORK/VERSION" >/dev/null
  work add VERSION && work commit -q -m 'release: 0.1.2, unsigned' && work tag -a v0.1.2 -m 'KubaOS 0.1.2' &&
    work push -q origin HEAD:main v0.1.1 v0.1.2 || { echo "FAIL pushing the tags"; return 1; }

  out="$(kz 'kubaos update -y --foreground' 2>&1)" ||
    { echo "FAIL kubaos update on stable"; printf '%s\n' "$out" | tail -30; return 1; }
  printf '%s\n' "$out" >"$LOGS/update-stable.log"
  [[ $(release_of) == "$tagged" && $(record version) == 0.1.1 && $(record channel) == stable ]] ||
    { echo "FAIL release isn't at the signed v0.1.1:"; cat /var/lib/kubaos/release; return 1; }
  grep -q 'v0.1.2' <<<"$out" || { echo "FAIL nothing said the unsigned v0.1.2 was skipped"; return 1; }
  [[ $(lock_rev) == "$tagged" ]] || { echo "FAIL the user flake's lock has KubaOS at $(lock_rev)"; return 1; }
  echo "ok   kubaos update on stable: release at the signed v0.1.1, the unsigned v0.1.2 skipped"

  out="$(kz 'kubaos channel edge && kubaos update -y --foreground' 2>&1)" &&
    { echo "FAIL edge took main's unsigned head with allowed_signers there"; return 1; }
  grep -q 'is not signed by a KubaOS key' <<<"$out" && [[ $(release_of) == "$tagged" ]] ||
    { echo "FAIL edge's refusal of an unsigned head:"; printf '%s\n' "$out" | tail -10; return 1; }
  kz 'kubaos channel stable' >/dev/null
  echo "ok   with allowed_signers, edge refuses main's unsigned head and leaves release"
}

c_rollback() {
  local edge out
  [[ -f $UPD_READY ]] || { echo "FAIL update_edge and update_stable have to run first"; return 1; }
  trap update_restore EXIT
  # The commit the edge update moved to, where the stable one started.
  edge="$(as_user git -C "$UPD_KUBAOS" log -1 --format=%H -- migrations)"
  out="$(kz 'kubaos rollback -y --foreground' 2>&1)" ||
    { echo "FAIL kubaos rollback"; printf '%s\n' "$out" | tail -30; return 1; }
  printf '%s\n' "$out" >"$LOGS/rollback.log"
  [[ $(release_of) == "$edge" && $(record commit) == "$edge" && $(record version) == 0.1.0 ]] ||
    { echo "FAIL release isn't back at the edge update's $edge:"; cat /var/lib/kubaos/release; return 1; }
  [[ $(lock_rev) == "$edge" ]] || { echo "FAIL the saved flake.lock isn't back: KubaOS at $(lock_rev)"; return 1; }
  [[ $(lines /var/lib/kubaos/update-history) == 2 ]] ||
    { echo "FAIL update-history keeps $(lines /var/lib/kubaos/update-history) lines, not 2"; return 1; }
  [[ $(lines "$UPD_SYSTEM_LOG") == 1 && -f /var/lib/kubaos/migrations/2000000000-harness-system.sh ]] ||
    { echo "FAIL rollback ran or dropped a migration"; return 1; }
  echo "ok   kubaos rollback: release, the record and flake.lock back at the edge update, migration markers kept"
}

# Shutdown and boot: every way out stops the session's units before Hyprland
# goes, they have short stop timeouts, and the fast-shutdown-boot migration
# leaves nothing waiting for the network at boot. Real shutdown and boot
# times are measured on the laptop.
c_shutdown() {
  as_user lua5.5 "$REPO/test/lua/session_test.lua" "$REPO" || { echo "FAIL test/lua/session_test.lua"; return 1; }
  leave_order || return 1
  session_unit_timeouts || return 1
  fast_boot_migration || return 1
}

# The inotify limits: system/20-base.sh, which built the image, installed the
# sysctl drop-in, and the inotify-limits migration puts it back on a machine
# without it, and changes nothing on a second run. The container's
# /proc/sys is the host's and read-only, so applying it is the laptop's.
c_inotify() {
  local mig out conf=/etc/sysctl.d/60-kubaos-inotify.conf
  cmp -s "$REPO/system/files/sysctl.d/60-kubaos-inotify.conf" "$conf" ||
    { echo "FAIL system/20-base.sh installed no $conf"; return 1; }
  echo "ok   system/20-base.sh installed $conf"
  mig="$(ls "$REPO"/migrations/system/*-inotify-limits.sh)"
  rm -f "$conf"
  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig:"; echo "$out" | tail; return 1; }
  cmp -s "$REPO/system/files/sysctl.d/60-kubaos-inotify.conf" "$conf" ||
    { echo "FAIL the inotify-limits migration installed no $conf"; return 1; }
  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig, run again:"; echo "$out" | tail; return 1; }
  if grep -E 'creating|replacing' <<<"$out"; then
    echo "FAIL the migration changed something on its second run (above)"
    return 1
  fi
  echo "ok   the inotify-limits migration installs the drop-in, and a second run changes nothing"
}

# `dot-session leave`, with a systemctl and a loginctl that log their
# arguments: the stop of graphical-session.target comes before the action.
leave_order() {
  local dir action want cmd rc=0
  dir="$(mktemp -d)"
  for cmd in systemctl loginctl; do
    printf '#!/bin/sh\necho "%s $*" >>"%s/calls"\n' "$cmd" "$dir" >"$dir/$cmd"
    chmod 0755 "$dir/$cmd"
  done
  chown -R "$USER_NAME:" "$dir"
  for action in reboot poweroff logout; do
    rm -f "$dir/calls"
    as_user env PATH="$dir:/usr/bin:/bin" XDG_SESSION_ID=7 bash "$REPO/scripts/dot-session" leave "$action" ||
      { echo "FAIL dot-session leave $action exited $?"; rm -rf "$dir"; return 1; }
    case "$action" in
      logout) want=$'systemctl --user stop graphical-session.target\nloginctl terminate-session 7' ;;
      *) want=$'systemctl --user stop graphical-session.target\n'"systemctl $action" ;;
    esac
    [[ $(cat "$dir/calls") == "$want" ]] ||
      { echo "FAIL dot-session leave $action ran:"; cat "$dir/calls"; rm -rf "$dir"; return 1; }
  done
  as_user env PATH="$dir:/usr/bin:/bin" bash "$REPO/scripts/dot-session" leave suspend >/dev/null 2>&1 || rc=$?
  rm -rf "$dir"
  ((rc == 2)) || { echo "FAIL dot-session leave suspend exited $rc, not 2"; return 1; }
  echo "ok   dot-session leave stops graphical-session.target, then logs out, reboots or powers off"
}

# The switch's drop-ins for the two Wayland clients that held the shutdown.
session_unit_timeouts() {
  local unit f
  for unit in xdg-desktop-portal-hyprland hyprpolkitagent; do
    f="$USER_HOME/.config/systemd/user/$unit.service.d/kubaos-stop.conf"
    grep -qx 'TimeoutStopSec=5s' "$f" 2>/dev/null || { echo "FAIL no TimeoutStopSec=5s in $f"; return 1; }
  done
  echo "ok   the portal and the polkit agent stop within 5 s (user drop-ins)"
}

# The fast-shutdown-boot system migration on a machine as an older install
# left it: no timeouts, NetworkManager-wait-online on, stand-ins for Docker's
# and setvtrgb's units, enabled the way their packages enable them, and the
# splash theme without its shutdown half.
# Then a second run, which changes nothing.
fast_boot_migration() {
  local mig out bad=0 u
  mig="$(ls "$REPO"/migrations/system/*-fast-shutdown-boot.sh)"
  rm -f /etc/systemd/system.conf.d/kubaos-timeouts.conf /etc/systemd/user.conf.d/kubaos-timeouts.conf \
    /etc/systemd/system/docker.service.d/kubaos-stop.conf
  systemctl enable NetworkManager-wait-online.service >/dev/null 2>&1
  stand_in_unit docker.service Service ExecStart=/bin/true multi-user.target
  stand_in_unit containerd.service Service ExecStart=/bin/true multi-user.target
  stand_in_unit setvtrgb.service Service ExecStart=/bin/true sysinit.target
  stand_in_unit docker.socket Socket ListenStream=/run/stand-in.sock sockets.target
  systemctl enable docker.service containerd.service setvtrgb.service >/dev/null 2>&1
  install -d /usr/share/plymouth/themes/kubaos
  echo '# the boot-only theme of an older release' >/usr/share/plymouth/themes/kubaos/kubaos.script

  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig:"; echo "$out" | tail; bad=1; }
  ((bad)) || fast_boot_state || bad=1
  ((bad)) || cmp -s "$REPO/system/files/plymouth/kubaos/kubaos.script" /usr/share/plymouth/themes/kubaos/kubaos.script ||
    { echo "FAIL the migration left the older splash theme"; bad=1; }
  if ((!bad)); then
    out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig, run again:"; echo "$out" | tail; bad=1; }
    if grep -E '^\s+\+ |creating|replacing' <<<"$out"; then
      echo "FAIL the migration changed something on its second run (above)"
      bad=1
    fi
  fi
  ((bad)) || echo "ok   the fast-shutdown-boot migration, and a second run changes nothing"

  for u in docker.service containerd.service docker.socket setvtrgb.service; do
    systemctl unmask "$u" >/dev/null 2>&1
    systemctl disable "$u" >/dev/null 2>&1
    rm -f "/usr/lib/systemd/system/$u"
  done
  rm -rf /etc/systemd/system/docker.service.d
  return "$bad"
}

# stand_in_unit UNIT SECTION SETTING WANTED_BY: a unit file in the package
# directory, installed into WANTED_BY.
stand_in_unit() {
  printf '[Unit]\nDescription=Stand-in for %s\n\n[%s]\n%s\n\n[Install]\nWantedBy=%s\n' "$1" "$2" "$3" "$4" \
    >"/usr/lib/systemd/system/$1"
}

# fast_boot_state: what install_fast_shutdown_boot (system/lib.sh) leaves.
fast_boot_state() {
  local f state
  for f in system.conf.d user.conf.d; do
    cmp -s "$REPO/system/files/systemd/$f/kubaos-timeouts.conf" "/etc/systemd/$f/kubaos-timeouts.conf" &&
      grep -qx 'DefaultTimeoutStopSec=10s' "/etc/systemd/$f/kubaos-timeouts.conf" ||
      { echo "FAIL /etc/systemd/$f/kubaos-timeouts.conf doesn't set DefaultTimeoutStopSec=10s"; return 1; }
  done
  echo "ok   the system and user managers stop a unit after 10 s"
  grep -qx 'TimeoutStopSec=20s' /etc/systemd/system/docker.service.d/kubaos-stop.conf 2>/dev/null ||
    { echo "FAIL Docker has no 20 s stop timeout of its own"; return 1; }
  for f in docker.socket:enabled docker.service:disabled containerd.service:disabled \
    NetworkManager-wait-online.service:disabled setvtrgb.service:masked; do
    state="$(systemctl is-enabled "${f%%:*}" 2>/dev/null || true)"
    [[ $state == "${f#*:}" ]] || { echo "FAIL ${f%%:*} is $state, not ${f#*:}"; return 1; }
  done
  [[ ! -e /etc/systemd/system/multi-user.target.wants/docker.service &&
    ! -e /etc/systemd/system/multi-user.target.wants/containerd.service ]] ||
    { echo "FAIL multi-user.target still wants Docker"; return 1; }
  [[ ! -e /etc/systemd/system/network-online.target.wants/NetworkManager-wait-online.service ]] ||
    { echo "FAIL network-online.target still wants NetworkManager-wait-online"; return 1; }
  echo "ok   Docker starts from its socket, nothing waits for the network, setvtrgb is masked"
}

# Ubuntu's updaters off: the retire-ubuntu-updaters system migration on a machine
# as an older install left it (the apt-daily timers enabled, apt's periodic jobs
# on) masks them and sets the jobs to 0, and a second run changes nothing.
c_updaters() {
  local mig out unit state
  mig="$(ls "$REPO"/migrations/system/*-retire-ubuntu-updaters.sh)"
  rm -f /etc/apt/apt.conf.d/99kubaos-no-periodic
  for unit in apt-daily.timer apt-daily-upgrade.timer; do
    systemctl unmask "$unit" >/dev/null 2>&1
    systemctl enable "$unit" >/dev/null 2>&1 || { echo "FAIL could not enable $unit first"; return 1; }
  done
  printf 'APT::Periodic::Update-Package-Lists "1";\nAPT::Periodic::Unattended-Upgrade "1";\n' >/etc/apt/apt.conf.d/20auto-upgrades

  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig:"; echo "$out" | tail; return 1; }
  for unit in apt-daily.timer apt-daily-upgrade.timer apt-daily.service apt-daily-upgrade.service \
    unattended-upgrades.service; do
    systemctl cat "$unit" >/dev/null 2>&1 || [[ $unit != unattended-upgrades.service ]] || continue
    state="$(systemctl is-enabled "$unit" 2>/dev/null || true)"
    [[ $state == masked ]] || { echo "FAIL $unit is $state, not masked"; return 1; }
  done
  cmp -s "$REPO/system/files/apt/apt.conf.d/99kubaos-no-periodic" /etc/apt/apt.conf.d/99kubaos-no-periodic ||
    { echo "FAIL the migration didn't install 99kubaos-no-periodic"; return 1; }
  [[ $(apt-config shell v APT::Periodic::Update-Package-Lists) == "v='0'" &&
    $(apt-config shell v APT::Periodic::Unattended-Upgrade) == "v='0'" ]] ||
    { echo "FAIL apt still has a periodic job on"; return 1; }

  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig, run again:"; echo "$out" | tail; return 1; }
  if grep -E '^\s+\+ |creating|replacing' <<<"$out"; then
    echo "FAIL the migration changed something on its second run (above)"
    return 1
  fi
  echo "ok   the retire-ubuntu-updaters migration: the apt-daily timers and unattended-upgrades masked, apt's periodic jobs 0, a second run changes nothing"
}

# The boot splash under Xvfb (Plymouth's X11 renderer), in each mode the
# machine runs it in: on boot the sign comes on and stays lit; on shutdown
# and reboot (the modes plymouth-poweroff and plymouth-reboot use) it starts
# lit and goes to black within a few seconds. A script Plymouth can't run
# draws nothing, so this fails on that too.
c_splash() {
  local theme=/usr/share/plymouth/themes/kubaos unit mode f
  for unit in poweroff:shutdown reboot:reboot; do
    f="/usr/lib/systemd/system/plymouth-${unit%%:*}.service"
    grep -q -- "--mode=${unit#*:}" "$f" && grep -qx 'ConditionKernelCommandLine=splash' "$f" &&
      [[ -e /usr/lib/systemd/system/${unit%%:*}.target.wants/plymouth-${unit%%:*}.service ]] ||
      { echo "FAIL plymouth-${unit%%:*}.service isn't wanted by ${unit%%:*}.target in --mode=${unit#*:}"; return 1; }
  done
  grep -qE '^GRUB_CMDLINE_LINUX_DEFAULT=".*\bsplash\b' "$REPO/system/files/grub/kubaos.cfg" ||
    { echo "FAIL the kernel command line in system/files/grub/kubaos.cfg has no splash"; return 1; }
  echo "ok   poweroff and reboot start Plymouth in its shutdown and reboot modes, and the kernel line has splash"
  shutdown_splash_migration || return 1
  boot_theme_migration || return 1

  install -d "$theme"
  install -m 0644 "$REPO"/system/files/plymouth/kubaos/* "$theme/"
  install -m 0644 "$REPO/system/files/plymouth/conf/plymouthd.conf" /etc/plymouth/plymouthd.conf
  Xvfb :93 -screen 0 1280x800x24 >"$LOGS/xvfb.log" 2>&1 &
  local xvfb=$! bad=0
  for _ in $(seq 50); do [[ -S /tmp/.X11-unix/X93 ]] && break; sleep 0.1; done
  splash_mode boot lit-by 6 || bad=1
  for mode in shutdown reboot; do
    splash_mode "$mode" lit-at 0.3 lit || bad=1
    splash_mode "$mode" black-by 6 || bad=1
  done
  kill "$xvfb" 2>/dev/null
  return "$bad"
}

# The shutdown-splash system migration on a machine without its units, then
# a second run, which changes nothing. Plymouth's shutdown units wait for
# user.slice, the waiter is wanted by every target that ends the machine, and
# with no plymouthd running its condition fails, so nothing waits.
shutdown_splash_migration() {
  local mig out unit
  mig="$(ls "$REPO"/migrations/system/*-shutdown-splash.sh)"
  systemctl disable kubaos-shutdown-splash.service >/dev/null 2>&1 || true
  rm -rf /etc/systemd/system/plymouth-{poweroff,reboot,halt,kexec}.service.d \
    /etc/systemd/system/kubaos-shutdown-splash.service

  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig:"; echo "$out" | tail; return 1; }

  for unit in poweroff reboot halt kexec; do
    grep -qx 'After=user.slice' "/etc/systemd/system/plymouth-$unit.service.d/kubaos-after-session.conf" 2>/dev/null ||
      { echo "FAIL plymouth-$unit.service isn't ordered after user.slice"; return 1; }
    [[ -L /etc/systemd/system/$unit.target.wants/kubaos-shutdown-splash.service ]] ||
      { echo "FAIL $unit.target doesn't want kubaos-shutdown-splash.service"; return 1; }
  done
  out="$(systemd-analyze verify /etc/systemd/system/kubaos-shutdown-splash.service 2>&1)" ||
    { echo "FAIL systemd-analyze verify kubaos-shutdown-splash.service:"; echo "$out"; return 1; }
  ! plymouth --ping 2>/dev/null || { echo "FAIL plymouth --ping answers with no plymouthd"; return 1; }
  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig, run again:"; echo "$out" | tail; return 1; }
  if grep -E '^\s+\+ |creating|replacing' <<<"$out"; then
    echo "FAIL the shutdown-splash migration changed something on its second run (above)"
    return 1
  fi
  echo "ok   the shutdown-splash migration: Plymouth after user.slice, the waiter wanted and skipped with no plymouthd"
}

# The boot-theme-tokens system migration on a machine whose themes an older
# release installed: the Plymouth theme with JetBrains Mono and no Geist Mono,
# GRUB's theme with its old fonts. It installs both themes, Geist Mono and the
# .pf2 fonts theme.txt names, removes the fonts neither uses, and the hook
# links the monospace font the theme has; in a container it leaves the
# initramfs and grub.cfg alone. Then a second run, which changes nothing.
boot_theme_migration() {
  local mig out ply=/usr/share/plymouth/themes/kubaos grub=/boot/grub/themes/kubaos spec ttf name dest
  mig="$(ls "$REPO"/migrations/system/*-boot-theme-tokens.sh)"
  rm -rf "$ply" "$grub"
  install -d "$ply" "$grub"
  echo '# the theme of an older release' >"$ply/kubaos.script"
  echo 'stand-in' >"$ply/JetBrainsMono-Regular.ttf"
  echo 'desktop-color: "#000000"' >"$grub/theme.txt"
  echo 'stand-in' >"$grub/jetbrainsmono-regular-24.pf2"

  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig:"; echo "$out" | tail; return 1; }

  cmp -s "$REPO/system/files/plymouth/kubaos/kubaos.script" "$ply/kubaos.script" ||
    { echo "FAIL the migration left the older splash script"; return 1; }
  cmp -s "$REPO/system/files/fonts/GeistMono-Regular.ttf" "$ply/GeistMono-Regular.ttf" ||
    { echo "FAIL the migration didn't install Geist Mono into the splash theme"; return 1; }
  cmp -s "$REPO/system/files/grub/kubaos/theme.txt" "$grub/theme.txt" ||
    { echo "FAIL the migration left the older GRUB theme"; return 1; }
  grep -q 'GeistMono-Regular.ttf' /etc/initramfs-tools/hooks/kubaos-plymouth ||
    { echo "FAIL the initramfs hook doesn't link Geist Mono"; return 1; }
  [[ ! -e $ply/JetBrainsMono-Regular.ttf ]] ||
    { echo "FAIL the older release's JetBrains Mono is still in the splash theme"; return 1; }
  while IFS= read -r spec; do
    IFS='|' read -r ttf _ _ name <<<"$spec"
    dest="$grub/$name.pf2"
    [[ -s $dest ]] || { echo "FAIL no $dest for $ttf, which theme.txt names"; return 1; }
  done < <(bash -c 'source "$1/system/lib.sh"; grub_theme_fonts "$1/system/files/grub/kubaos/theme.txt"' _ "$REPO")
  [[ ! -e $grub/jetbrainsmono-regular-24.pf2 ]] || { echo "FAIL an older release's GRUB font is left"; return 1; }
  grep -q 'initramfs and grub.cfg stay as they are' <<<"$out" ||
    { echo "FAIL in a container the migration went on to the initramfs:"; echo "$out" | tail -3; return 1; }
  out="$(KUBAOS_DIR="$REPO" bash -euo pipefail "$mig" 2>&1)" || { echo "FAIL $mig, run again:"; echo "$out" | tail; return 1; }
  if grep -E '^\s+\+ |creating|replacing' <<<"$out"; then
    echo "FAIL the boot-theme-tokens migration changed something on its second run (above)"
    return 1
  fi
  echo "ok   the boot-theme-tokens migration: both themes, Geist Mono and GRUB's fonts, nothing on a second run"
  plymouth_hook_fonts || return 1
}

# plymouth_hook_fonts: the installed kubaos-plymouth hook, run as update-initramfs runs it, over
# an initramfs tree whose theme has Geist Mono, then only an older release's JetBrains Mono, then
# no monospace font. Plymouth-monospace.ttf links to the best of them each time and never
# dangles: plymouthd crashes at the passphrase prompt without it.
plymouth_hook_fonts() {
  local dest theme=/usr/share/plymouth/themes/kubaos case want got
  for case in GeistMono-Regular.ttf JetBrainsMono-Regular.ttf Geist-Regular.ttf; do
    dest="$(mktemp -d)"
    install -d "$dest$theme"
    install -m 0644 "$REPO/system/files/fonts/Geist-Regular.ttf" "$dest$theme/"
    case $case in
      GeistMono-Regular.ttf) install -m 0644 "$REPO/system/files/fonts/GeistMono-Regular.ttf" "$dest$theme/" ;;
      JetBrainsMono-Regular.ttf) echo 'stand-in' >"$dest$theme/JetBrainsMono-Regular.ttf" ;;
    esac
    # mkinitramfs exports DESTDIR and verbose to every hook, and hook-functions runs under set -u.
    DESTDIR="$dest" verbose=n sh /etc/initramfs-tools/hooks/kubaos-plymouth >/dev/null 2>&1 ||
      { echo "FAIL the kubaos-plymouth hook failed with only $case"; rm -rf "$dest"; return 1; }
    want="$theme/$case"
    got="$(readlink "$dest/usr/share/fonts/Plymouth-monospace.ttf" || true)"
    if [[ $got != "$want" || ! -f $dest$got ]]; then
      echo "FAIL with $case the hook linked Plymouth-monospace.ttf to '$got', not $want"
      rm -rf "$dest"
      return 1
    fi
    rm -rf "$dest"
  done
  echo "ok   the initramfs hook links Plymouth-monospace.ttf to Geist Mono, else JetBrains Mono, else Geist"
}

# splash_mode MODE lit-at SECONDS lit | lit-by SECONDS | black-by SECONDS:
# plymouthd in MODE on Xvfb's :93, the splash shown, and the screen grabbed
# at SECONDS, or every 0.25 s until SECONDS. Lit is a sign bright enough to
# see; black is the whole screen at 0.
splash_mode() {
  local mode="$1" what="$2" secs="$3" want="${4:-}" t0 t shot="$LOGS/splash.png" stats max ok=0
  pkill -x plymouthd 2>/dev/null; sleep 0.2
  DISPLAY=:93 python3 -c '
import os, pty, subprocess, sys
master, slave = pty.openpty()
subprocess.Popen(["plymouthd", "--no-daemon", "--mode=" + sys.argv[1], "--kernel-command-line=quiet splash",
  "--graphical-boot", "--no-boot-log", "--tty=" + os.ttyname(slave)],
  stdin=slave, stdout=slave, stderr=slave, start_new_session=True).wait()
' "$mode" >/dev/null 2>&1 &
  for _ in $(seq 50); do plymouth --ping 2>/dev/null && break; sleep 0.1; done
  plymouth show-splash
  t0="$(now)"
  while :; do
    t="$(since "$t0")"
    if [[ $what == lit-at ]]; then sleep "$(awk -v a="$secs" -v b="$t" 'BEGIN { print (a > b ? a - b : 0) }')"; fi
    DISPLAY=:93 import -window root "$shot"
    stats="$(convert "$shot" -colorspace Gray -format '%[fx:maxima]' info:)"
    max="$(awk -v m="$stats" 'BEGIN { print (m > 0.5) ? "lit" : (m == 0) ? "black" : "dark" }')"
    case "$what" in
      lit-at) [[ $max == "$want" ]] && ok=1; break ;;
      lit-by) [[ $max == lit ]] && { ok=1; break; } ;;
      black-by) [[ $max == black ]] && { ok=1; break; } ;;
    esac
    awk -v a="$(since "$t0")" -v b="$secs" 'BEGIN { exit !(a >= b) }' && break
    sleep 0.25
  done
  t="$(since "$t0")"
  pkill -x plymouthd 2>/dev/null
  if ((ok)); then
    echo "ok   $mode: ${want:-${what%-by}} at ${t} s (brightest pixel $stats)"
  else
    cp "$shot" "$LOGS/splash-$mode-$what.png"
    echo "FAIL $mode: not ${want:-${what%-by}} by ${secs} s (brightest pixel $stats; $LOGS/splash-$mode-$what.png)"
    return 1
  fi
}

# update_restore: the machine as before the update checks.
update_restore() {
  rm -f /usr/local/bin/apt-get /usr/local/sbin/apt-get /usr/local/bin/apt-cache /usr/bin/kubaos "$UPD_READY"
  git config --system --unset gpg.ssh.program || true
  if [[ -e $UPD_SAVED ]]; then rm -rf "$UPD_KUBAOS" && mv "$UPD_SAVED" "$UPD_KUBAOS"; fi
}

e2e_main() {
  local t0 only="${1:-}"
  t0="$(now)"
  say "checks"
  # shellcheck disable=SC2034 # cache_lock (harness.sh) tells the console here of a wait.
  exec {CONSOLE}>&1
  RESULTS+=("switch|PASS|$SWITCH_SECONDS|home-manager switch via system/60-nix.sh")
  printf '%sPASS%s  %-14s %6ss\n' "$C_OK" "$C_OFF" switch "$SWITCH_SECONDS"
  local c
  for c in screenshot login_shell zsh_start greeting path tools links theme cursor tmux nvim ghostty tms pam lua workspaces power keyboard shutdown inotify splash updaters shellcheck flake_check quickshell clock sysmon notifications menu menu_kubaos menu_files files automount bluetooth phone recorder session_reload wallpaper_sets wallpaper_video packages preview suites pins dot_config warm_target user_layer update_edge update_stable rollback; do
    [[ -n $only && ",$only," != *",$c,"* ]] && continue
    check "$c" "c_$c"
  done
  echo
  say "results ($(since "$t0") s for the checks)"
  printf '%-14s %-5s %8s  %s\n' CHECK RESULT SECONDS DETAIL
  local r
  for r in "${RESULTS[@]}"; do
    IFS='|' read -r n s secs d <<<"$r"
    printf '%-14s %-5s %8s  %s\n' "$n" "$s" "$secs" "$d"
  done
  if ((FAILED)); then
    printf '\n%s%d check(s) failed%s; logs in %s (kept with --keep)\n' "$C_ERR" "$FAILED" "$C_OFF" "$LOGS"
    return 1
  fi
  printf '\n%sall %d checks passed%s\n' "$C_OK" "${#RESULTS[@]}" "$C_OFF"
}
