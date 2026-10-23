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
  for want in "$h/.local/bin" "$h/.cargo/bin" "$h/.proto/shims" "$h/.proto/bin" "$h/.nix-profile/bin" /usr/bin; do
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
# exists on arm64, resolved in a login shell, with its owner checked.
NIX_TOOLS=(qs ghostty nvim tmux tms lazygit yazi fzf zoxide atuin direnv eza bat fd rg delta dust dysk
  btop jq yq xh tldr just glow gh fastfetch lazydocker bacon cargo-nextest hyperfine sccache wild
  oh-my-posh matugen home-manager non-nixos-gpu-setup gpu-screen-recorder satty)
APT_TOOLS=(foot fuzzel wl-copy grim slurp playerctl socat qalc notify-send brightnessctl clang mold
  java unzip git ssh secret-tool pamtester pamu2fcfg nmcli wpctl snapper fprintd-verify boltctl
  powerprofilesctl ddcutil xdg-open hyprlock hypridle)
# Hyprland comes from the PPA on amd64 and from nixpkgs on arm64, where
# Ubuntu's is too old for the Lua config (home/desktop.nix).
if [[ "$(dpkg --print-architecture)" == arm64 ]]; then NIX_TOOLS+=(Hyprland start-hyprland); else APT_TOOLS+=(Hyprland); fi
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
CUR_SETTINGS=$USER_HOME/.config/kubaos/cursor/cursor.toml
CUR_PALETTE=$USER_HOME/.local/state/theme/cursor.toml
# cursor_sizes SIZES: waits up to 60 s for the background render to leave an
# XCursor theme at SIZES ("24 36 48").
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

  cursor_sizes "24 36 48" || return 1
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
if sizes != {24, 36, 48}:
    sys.exit(f"FAIL cursors/left_ptr has sizes {sorted(sizes)}, not 24 36 48")
if not opaque:
    sys.exit(f"FAIL cursors/left_ptr has no pixel of the fill #{fill:06x}")
print(f"ok   XCursor theme at 24 36 48, its aliases linked, left_ptr filled with {opaque} #{fill:06x} pixels")
PY

  expect_ok "kubaos cursor style accent" zrun 'kubaos cursor style accent' || return 1
  grep -qx 'style = "accent"' "$CUR_SETTINGS" || { echo "FAIL the settings don't say accent:"; cat "$CUR_SETTINGS"; return 1; }
  cursor_svg_has "$primary" || { echo "FAIL left_ptr.svg isn't filled $primary after style accent"; return 1; }
  cursor_svg_has "$surface" && { echo "FAIL left_ptr.svg still has the dark style's fill $surface"; return 1; }
  echo "ok   style accent: left_ptr filled $primary"
  expect_ok "kubaos cursor size 32" zrun 'kubaos cursor size 32' || return 1
  cursor_sizes "32 48 64" || return 1
  [[ $(zrun 'kubaos cursor show') == "style=accent size=32" ]] || { echo "FAIL kubaos cursor show: $(zrun 'kubaos cursor show')"; return 1; }
  rc=0; out="$(zrun 'kubaos cursor size 7' 2>&1)" || rc=$?
  ((rc == 2)) || { echo "FAIL kubaos cursor size 7 exited $rc, not 2: $out"; return 1; }
  echo "ok   size 32: XCursor at 32 48 64; size 7 is refused"
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
    tmux -L dotfiles-test show -gv @thm_primary; tmux -L dotfiles-test show-messages; \
    tmux -L dotfiles-test kill-server' 2>&1)" || { echo "FAIL tmux: $out"; return 1; }
  grep -qE '^#[0-9a-fA-F]{6}$' <<<"$out" || { echo "FAIL @thm_primary unset; tmux said:"; printf '%s\n' "$out"; return 1; }
  # show-messages logs every command; anything else in it is a complaint.
  if grep -v -E ' command: |^#[0-9a-fA-F]{6}$' <<<"$out" | grep -qiE 'error|unknown|invalid|no such|bad '; then echo "FAIL tmux messages:"; printf '%s\n' "$out"; return 1; fi
  echo "ok   tmux.conf + theme + plugins load, @thm_primary=$(head -1 <<<"$out")"
}

c_nvim() {
  local out
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

c_ghostty() {
  expect_ok "ghostty +validate-config" zrun 'ghostty +validate-config' || return 1
  expect_ok "foot --check-config" zrun 'foot --check-config' || return 1
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
  layer="$(awk '{print $2}' "$USER_HOME/.local/share/kubaos/layer")"
  while IFS= read -r f; do
    f="${f#config/}"
    grep -qxF -- "$f" <<<"$layer" && continue
    target="$(readlink -f "$USER_HOME/.config/$f" || true)"
    [[ $target == /nix/store/* ]] && cmp -s "$target" "$REPO/config/$f" ||
      { echo "FAIL ~/.config/$f -> $target"; bad=1; }
    n_files=$((n_files + 1))
  done < <(as_user git -C "$REPO" ls-files config)
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
  done < <(as_user git -C "$REPO" ls-files scripts system migrations test config | grep -vE '\.(md|nix|lua|qml|toml|json|jpg|png|webp)$')
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
  return "$bad"
}

# Per-screen workspaces (config/hypr/workspaces.lua): the block bookkeeping,
# and the module itself against a stand-in for Hyprland's Lua API, under the
# Lua 5.5 that Hyprland runs.
c_workspaces() {
  as_user lua5.5 "$REPO/test/lua/workspaces_test.lua" "$REPO" || { echo "FAIL test/lua/workspaces_test.lua"; return 1; }
  echo "ok   per-screen workspaces: blocks, slots, joining on unplug and returning on replug"
}

# Keyboard layouts: config/hypr/input.lua hands Hyprland every layout in
# /etc/default/keyboard, and `kubaos keyboard show` reads the same file.
c_keyboard() {
  as_user lua5.5 "$REPO/test/lua/input_test.lua" "$REPO" || { echo "FAIL test/lua/input_test.lua"; return 1; }
  echo "ok   input.lua: two layouts as XKB's lists, one as a name, US without the file"
  local file out
  install_kubaos || return 1
  file="$(mktemp)"
  printf 'XKBMODEL="pc105"\nXKBLAYOUT="us,it"\nXKBVARIANT=",intl"\nXKBOPTIONS=""\n' >"$file"
  chmod 644 "$file"
  out="$(kz "KUBAOS_KEYBOARD_FILE='$file' kubaos keyboard show" 2>&1)" || { echo "FAIL kubaos keyboard show: $out"; rm -f "$file"; return 1; }
  rm -f "$file"
  [[ $out == $'us\nit(intl)' ]] || { echo "FAIL kubaos keyboard show printed: $out"; return 1; }
  echo "ok   kubaos keyboard show lists the layouts, the default first"
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
  out="$(qs_problems)"
  local ipc
  ipc="$(in_session 'qs ipc show' 2>&1 || true)"
  session_stop
  if [[ -n $out ]]; then
    echo "FAIL QML errors/warnings ($QS_LOG):"
    printf '%s\n' "$out" | head -30
    return 1
  fi
  echo "ok   Quickshell loaded with no QML errors"
  echo "     environment warnings (services a container doesn't have):"
  qs_env_warnings | cut -c1-120 | sed 's/^/       /'
  
  for t in session powermenu wallpaper notifications controlcenter recorder; do
    grep -qE "target $t\b" <<<"$ipc" || { echo "FAIL qs ipc show lacks target $t"; printf '%s\n' "$ipc"; return 1; }
  done
  echo "ok   qs ipc show lists every contract target"
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

# menu_stop PID: the Daemon (PID is its in_session job), its Extensions and the
# session. Waiting on the job keeps the shell's "Terminated" out of the log.
menu_stop() {
  pkill -u "$USER_NAME" -x kotodamad 2>/dev/null || true
  pkill -u "$USER_NAME" -f '/kotodama-[a-z-]+$' 2>/dev/null || true
  wait "$1" 2>/dev/null || true
  session_stop
}

# menu_wtype: wtype, the menu's keyboard, from the flake's nixpkgs: the root
# warm made, or built now.
menu_wtype() {
  local root="$CACHE/roots/wtype"
  [[ -x $root/bin/wtype ]] ||
    as_user "$NIX_BIN/nix" build --out-link "$root" --inputs-from "$REPO" nixpkgs#wtype >/dev/null 2>&1
  [[ -x $root/bin/wtype ]] && echo "$root/bin/wtype"
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

# menu_up SIZE: a session at SIZE, the Daemon (its job in MENU_DAEMON) and
# Quickshell with the menu's probe, which logs `menu probe: ...` lines
# (config/quickshell/modules/menu/Probe.qml).
menu_up() {
  session_start sway "$1" 1
  in_session "exec kotodamad >>$LOGS/kotodamad.log 2>&1" &
  MENU_DAEMON=$!
  if ! qs_start KUBAOS_MENU_PROBE=1; then
    echo "FAIL Quickshell didn't load:"
    tail -30 "$QS_LOG"
    return 1
  fi
  sleep 3
}

# menu_step DESC PATTERN CMD [SECONDS]: CMD in the session, then wait up to
# SECONDS (3) for the probe to log a line matching PATTERN (ERE).
menu_step() {
  local desc="$1" pattern="$2" mark
  mark=$(($(wc -l <"$QS_LOG") + 1))
  in_session "$3" >/dev/null 2>&1
  for _ in $(seq $((${4:-3} * 10))); do
    if tail -n +"$mark" "$QS_LOG" | grep -q -E -- "menu probe: $pattern"; then
      echo "ok   $desc"
      return 0
    fi
    sleep 0.1
  done
  echo "FAIL $desc: the probe logged no /$pattern/; it logged:"
  tail -n +"$mark" "$QS_LOG" | grep -o 'menu probe: .*' | tail -5 | sed 's/^/     /'
  return 1
}

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
# would, and KubaOS's own menu inside Quickshell, driven by wtype. `kotodama
# toggle` succeeds only when a menu answers it. KubaOS's settings reach the
# Daemon, which finds nothing wrong with them. The container has no logind and
# no Hyprland, so what an Extension reports (`extension <name>: ...`) is
# listed, not fatal, and the Shortcuts View is test/vm/menu-check's.
#
# At 1280x800 the menu has no detail pane and more rows than Kotodama's own
# menu (seven); at 1920x1080 it has the pane, and every View is walked through
# with Kotodama's example shell Extension. A new virtual keyboard loses its
# first key while the compositor takes its keymap, so each wtype waits 200 ms.
c_menu() {
  local out bad=0 log="$LOGS/kotodamad.log" key
  : >"$log"
  chown "$USER_NAME:" "$log"
  key="$(menu_wtype)" || { echo "FAIL no wtype from the flake's nixpkgs"; return 1; }
  key="$key -s 200"
  menu_example || { echo "FAIL Kotodama's example Extension isn't in flake.lock's Kotodama"; return 1; }

  menu_up 1280x800 || { menu_stop "$MENU_DAEMON"; return 1; }
  menu_step "kotodama toggle opens the menu in Quickshell" 'visible true' 'kotodama toggle' || bad=1
  menu_layout false 8 || bad=1
  menu_step "kotodama toggle closes it" 'visible false' 'kotodama toggle' || bad=1
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

  menu_step "kotodama toggle opens the menu" 'visible true' 'kotodama toggle' || bad=1
  menu_layout true 8 || bad=1
  menu_step "typing searches, and the Results reach the screen" 'frame query="tim" ' "$key -d 120 tim" || bad=1
  menu_step "Ctrl+K opens the Action panel on Timer" 'actions [2-9]' "$key -M ctrl -k k -m ctrl" || bad=1
  menu_step "Esc closes only the Action panel" 'actions 0' "$key -k Escape" || bad=1
  menu_step "Enter on Timer pushes its form" 'view kind=form title="Start timer" depth=1' "$key -k Return" || bad=1
  menu_step "the form's Enter starts a job, shown in a progress View" 'view kind=progress title="2 second timer" .*job=running' \
    "sleep 0.5 && $key -d 50 2 && $key -k Return" || bad=1
  menu_step "the job runs to the end" 'view kind=progress .*job=succeeded' ':' 8 || bad=1
  menu_step "Esc pops the View" 'view kind= title="" depth=0' "$key -k Escape" || bad=1
  menu_step "kotodama open --view opens a list View" 'view kind=list title="Bookmarks" depth=1' \
    'kotodama open --view shell-example/bookmarks' || bad=1
  menu_step "the list View's filter narrows it" 'listed 1$' "$key -d 50 quick" || bad=1
  menu_step "Esc clears the filter first" 'listed 3$' "$key -k Escape" || bad=1
  menu_step "Esc again leaves the list View" 'view kind= title="" depth=0' "$key -k Escape" || bad=1
  menu_step "kotodama open --app enters an app, with its intro card" 'intro ".+"' 'kotodama open --app shell' || bad=1
  menu_step "Backspace with nothing typed leaves the app" 'scope ""' "$key -k BackSpace" || bad=1
  menu_step "Esc clears the query" 'frame query="" ' "$key -k Escape" || bad=1
  menu_step "Enter on Power Off asks first" 'confirming "Power Off\? ' \
    "$key -d 50 'power off' && sleep 1 && $key -k Return" || bad=1
  menu_step "Esc cancels the confirmation" 'confirming ""' "$key -k Escape" || bad=1
  menu_step "Forget on Timer leaves a Toast" 'toast ".+"' \
    "$key -k Escape && $key -d 50 timer && sleep 1 && $key -M ctrl -M shift -k BackSpace -m shift -m ctrl" 5 || bad=1
  menu_step "Ctrl+Z undoes it" 'toast ""' "$key -M ctrl -k z -m ctrl" 5 || bad=1
  menu_step "Esc clears, and Esc again closes the menu" 'visible false' "$key -k Escape && sleep 0.5 && $key -k Escape" || bad=1
  menu_latency || bad=1

  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  if ((bad)); then tail -20 "$log"; fi
  menu_stop "$MENU_DAEMON"
  menu_example --remove
  ((bad)) || echo "ok   the menu answers kotodama toggle, takes KubaOS's settings and shows every View"
  return "$bad"
}

# The KubaOS Extension (issue #29, crates/kotodama-kubaos): it runs under the
# Daemon and its Items are found by typing, with the user layer's action shown
# and the ones it hides gone. test/lib/menu-query.py is the Front end, since
# the `kotodama` command can't query. The Extension's Kotodama crates must be
# the Daemon's commit (Cargo.lock against flake.lock).
c_menu_kubaos() {
  local out bad=0 log="$LOGS/kotodamad-kubaos.log" daemon
  local layer="$USER_HOME/.config/kubaos/kotodama/user.toml" saved="$LOGS/user.toml.saved"
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

  out="$(in_session "$query 'Keyboard: add a layout' 'Keyboard: set layouts' Keyboard 'next layout'" 2>&1)" ||
    { echo "FAIL querying the menu: $out"; bad=1; }
  for want in 'Keyboard: add a layout:keyboard-add' \
    'Keyboard: set layouts:keyboard-set' 'Keyboard:keyboard:us' 'Keyboard:keyboard:it'; do
    grep -E -- "^${want%%:keyboard*}[[:blank:]]" <<<"$out" | grep -qE -- "kubaos/keyboard${want#*:keyboard}( |$)" ||
      { echo "FAIL typing '${want%%:keyboard*}' doesn't find kubaos/keyboard${want#*:keyboard}"; bad=1; }
  done
  ((bad)) || echo "ok   the keyboard Items are found: add, set, and one a layout"
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

  # localed writes the file through a temporary one and a rename; the Items follow it.
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
  return "$bad"
}

# Wallpaper sets (issue #16): every set read-only in /usr/share/kubaos/wallpapers,
# the picker's folders are the chosen ones (~/.config/kubaos/wallpapers/sets,
# all of them without the file) plus ~/Pictures/wallpapers, a real folder.
c_wallpaper_sets() {
  local sys=/usr/share/kubaos/wallpapers sets="$USER_HOME/.config/kubaos/wallpapers/sets"
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

# The state directory and switch jobs (issue #20), through the kubaos binary
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
}

# The Suite framework (issue #21), through the test Suite (home/suites/test.nix,
# in a kubaos built with the test-suite feature) and a user flake under git,
# so every file kubaos writes into kubaos.d/ must reach git's index for Nix to
# see it. Add, configure, scaffold and remove it, with a user override in
# ~/.config/kubaos/nvim/ that must win throughout, and nothing of the Suite
# left in the generation afterwards. Switches run --foreground, as in packages.
c_suites() {
  local u="$USER_HOME/.config/kubaos/home" layer="$USER_HOME/.config/kubaos" out gen
  local mine="$USER_HOME/.config/kubaos/nvim/plugins/test-suite.lua"
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
  [[ -f $mine ]] && grep -q 'created ~/.config/kubaos/nvim/plugins/test-suite.lua' <<<"$out" ||
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
  cmp -s "$mine" "$LOGS/suites-mine.lua" && grep -q 'kept ~/.config/kubaos/nvim/plugins/test-suite.lua' <<<"$out" ||
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
  grep -qx '  ~/.config/kubaos/nvim/plugins/test-suite.lua' <<<"$out" ||
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

# The user layer (issue #9): test/user-layer-check, which the VM bench also
# runs on an installed system. Commits to the checkout, so it runs after the
# other checks but the update ones, which start from a clone of it.
c_user_layer() {
  zrun "'$REPO/test/user-layer-check' '$REPO'"
}

# kubaos update, the channels and kubaos rollback, through /out/kubaos: a
# bare repo stands in for GitHub as origin, a root-owned clone of it on its
# `release` branch for /usr/share/kubaos (the real one, the wallpaper link,
# is put aside and back by c_rollback), and a user flake on that release like
# the installer's. apt-get is a shim that logs what it was asked, so nothing
# is downloaded or installed; the container has no snapper. The three checks
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
  rm -rf "$UPD_ORIGIN" "$UPD_WORK" /var/lib/kubaos /etc/kubaos /usr/share/kubaos-keys /usr/bin/kubaos \
    "$UPD_SYSTEM_LOG" "$UPD_USER_LOG" "$UPD_STATE/migrations" "$UPD_STATE/locks" \
    "$UPD_STATE/update-available" "$UPD_STATE/notified"
  # Root reads kuba's origin and kuba reads root's checkout, as on the machine.
  for dir in "$UPD_KUBAOS" "$UPD_ORIGIN"; do
    git config --system --get-all safe.directory | grep -qxF -- "$dir" ||
      git config --system --add safe.directory "$dir"
  done
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
  local before head out
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
  as_user awk -v v='version = "0.1.0-harness"' \
    'prev ~ /^name = "(kubaos-core|kubaos-cli|kubaos-cursor|kubaos-kobo|kotodama-kubaos)"$/ && /^version = / { $0 = v } { print; prev = $0 }' \
    "$UPD_WORK/Cargo.lock" | as_user tee "$UPD_WORK/Cargo.lock.new" >/dev/null && as_user mv "$UPD_WORK/Cargo.lock.new" "$UPD_WORK/Cargo.lock"
  [[ $(grep -c '^version = "0.1.0-harness"$' "$UPD_WORK/Cargo.lock") == 5 ]] ||
    { echo "FAIL bumping the workspace's version in Cargo.lock"; return 1; }
  work add migrations Cargo.toml Cargo.lock && work commit -q -m 'test: a new kubaos, a system and a user migration' &&
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
  grep -q '^upgrade --with-new-pkgs -y$' "$LOGS/apt-get.log" || { echo "FAIL apt-get upgrade never ran"; return 1; }
  echo "ok   kubaos update on edge: release at main's head, both migrations ran once, the record, history and lock follow"

  out="$(zrun ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh; command -v kubaos; kubaos --version" 2>&1)"
  [[ $out == "$USER_HOME/.nix-profile/bin/kubaos"$'\n'"kubaos 0.1.0-harness" ]] ||
    { echo "FAIL the kubaos on PATH after the update isn't the release's:"; printf '%s\n' "$out"; return 1; }
  out="$(as_user /usr/bin/kubaos --version 2>&1)"
  [[ $out == "kubaos 0.1.0-harness" ]] && cmp -s /usr/bin/kubaos "$UPD_KUBAOS/install/firstboot/kubaos-launcher" ||
    { echo "FAIL /usr/bin/kubaos isn't the launcher, or runs another kubaos: $out"; return 1; }
  echo "ok   the release's kubaos is on PATH after the update, and /usr/bin/kubaos, now the launcher, runs it"

  out="$(kz 'kubaos update -y --foreground' 2>&1)" ||
    { echo "FAIL kubaos update, again"; printf '%s\n' "$out" | tail -30; return 1; }
  grep -q 'newest release of its channel' <<<"$out" && [[ $(release_of) == "$head" ]] ||
    { echo "FAIL the second update moved release or didn't say it is current"; printf '%s\n' "$out" | tail -20; return 1; }
  [[ $(lines "$UPD_SYSTEM_LOG") == 1 && $(lines "$UPD_USER_LOG") == 1 ]] ||
    { echo "FAIL a migration ran again"; return 1; }
  echo "ok   kubaos update again: only packages, no migration runs twice"
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

# update_restore: the machine as before the update checks.
update_restore() {
  rm -f /usr/local/bin/apt-get /usr/local/sbin/apt-get /usr/bin/kubaos "$UPD_READY"
  git config --system --unset gpg.ssh.program || true
  if [[ -e $UPD_SAVED ]]; then rm -rf "$UPD_KUBAOS" && mv "$UPD_SAVED" "$UPD_KUBAOS"; fi
}

e2e_main() {
  local t0 only="${1:-}"
  t0="$(now)"
  say "checks"
  RESULTS+=("switch|PASS|$SWITCH_SECONDS|home-manager switch via system/60-nix.sh")
  printf '%sPASS%s  %-14s %6ss\n' "$C_OK" "$C_OFF" switch "$SWITCH_SECONDS"
  local c
  for c in login_shell path tools links theme cursor tmux nvim ghostty tms pam lua workspaces keyboard shellcheck flake_check quickshell menu menu_kubaos recorder wallpaper_sets wallpaper_video packages suites user_layer update_edge update_stable rollback; do
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
