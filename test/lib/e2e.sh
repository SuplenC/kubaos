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
  delta.gitconfig fzf atuin.toml eza.yml glow.json)
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

# The Command menu: Kotodama's Daemon, started by hand as its socket unit
# would, and the menu inside KubaOS's Quickshell. `kotodama toggle` succeeds
# only when a menu answers it. KubaOS's settings reach the Daemon, which finds
# nothing wrong with them. The container has no logind and no Hyprland, so
# what an Extension reports (`extension <name>: ...`) is listed, not fatal.
c_menu() {
  local out bad=0 log="$LOGS/kotodamad.log" daemon
  session_start sway 1280x720 1
  : >"$log"
  chown "$USER_NAME:" "$log"
  in_session "exec kotodamad >$log 2>&1" &
  daemon=$!
  if ! qs_start; then
    echo "FAIL Quickshell didn't load:"
    tail -30 "$QS_LOG"
    menu_stop "$daemon"
    return 1
  fi
  sleep 3

  expect_ok "kotodama toggle opens the menu in Quickshell" in_session 'kotodama toggle' || bad=1
  expect_ok "kotodama toggle closes it" in_session 'kotodama toggle' || bad=1

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

  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  if ((bad)); then tail -20 "$log"; fi
  menu_stop "$daemon"
  ((bad)) || echo "ok   the menu answers kotodama toggle and takes KubaOS's settings"
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

  wall="$sys/shinjuku/kabukicho-red-gate.jpg"
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
kz() { zrun ". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh; $1"; }

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

# user_flake DIR: a fresh user flake in DIR, rendered from user-flake/ with
# the repo copy as KubaOS and KubaOS's own locked nixpkgs as the user's, and
# no home.nix yet.
user_flake() {
  local u="$1" nixpkgs
  nixpkgs="$(as_user "$NIX_BIN/nix" flake archive --dry-run --json "$REPO" | jq -r .inputs.nixpkgs.path)"
  [[ $nixpkgs == /nix/store/* ]] || { echo "FAIL KubaOS's nixpkgs: $nixpkgs"; return 1; }
  rm -rf "$u"
  as_user install -d "$u"
  sed -e 's/@USER@/kuba/g; s/@HOST@/vm/g' -e "s/@SYSTEM@/$(uname -m)-linux/" \
    -e "s#github:NixOS/nixpkgs/nixos-26.05#path:$nixpkgs#" \
    -e "s#git+file:///usr/share/kubaos?ref=refs/heads/release#git+file://$REPO#" \
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
# runs on an installed system. Commits to the checkout, so it runs last.
c_user_layer() {
  zrun "'$REPO/test/user-layer-check' '$REPO'"
}

e2e_main() {
  local t0 only="${1:-}"
  t0="$(now)"
  say "checks"
  RESULTS+=("switch|PASS|$SWITCH_SECONDS|home-manager switch via system/60-nix.sh")
  printf '%sPASS%s  %-14s %6ss\n' "$C_OK" "$C_OFF" switch "$SWITCH_SECONDS"
  local c
  for c in login_shell path tools links theme tmux nvim ghostty tms pam lua workspaces shellcheck flake_check quickshell menu menu_kubaos recorder wallpaper_sets wallpaper_video packages suites user_layer; do
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
