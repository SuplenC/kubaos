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
  jq -e '.mode == "dark" and (.wallpaper | startswith("/") or startswith("ukiyo:"))' "$st/colors.json" >/dev/null ||
    { echo "FAIL colors.json: mode/wallpaper"; return 1; }
  echo "ok   ${#THEME_FILES[@]} theme files, ${#M3_KEYS[@]} color keys"
}
c_theme() {
  local st="$USER_HOME/.local/state/theme" before after wall out
  expect_ok "dot-theme apply --force" zrun 'dot-theme apply --force' || return 1
  theme_files_ok || return 1
  before="$(jq -r .primary "$st/colors.json")"
  wall="$(zrun 'ls ~/Pictures/wallpapers/japanese/*.jpg | head -1')"
  [[ -n $wall ]] || { echo "FAIL no wallpaper under ~/Pictures/wallpapers/japanese"; return 1; }
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
  grep -q 'search_dirs' "$USER_HOME/.config/tms/config.toml" && echo "ok   ~/.config/tms/config.toml is the repo's"
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
  # Wallpapers: real folders (the user's own images go there too), each of
  # KubaOS's files a link into the store.
  local n_walls=0
  for d in "$REPO"/wallpapers/*/; do
    n="$(basename "$d")"
    [[ -d $USER_HOME/Pictures/wallpapers/$n && ! -L $USER_HOME/Pictures/wallpapers/$n ]] ||
      { echo "FAIL ~/Pictures/wallpapers/$n isn't a directory"; bad=1; }
    for f in "$d"*; do
      target="$(readlink -f "$USER_HOME/Pictures/wallpapers/$n/$(basename "$f")" || true)"
      [[ $target == /nix/store/* ]] && cmp -s "$target" "$f" ||
        { echo "FAIL ~/Pictures/wallpapers/$n/$(basename "$f") -> $target"; bad=1; }
      n_walls=$((n_walls + 1))
    done
  done
  [[ -f $USER_HOME/.config/tmux-nix/plugins.conf ]] || { echo "FAIL no ~/.config/tmux-nix/plugins.conf"; bad=1; }
  [[ -f $USER_HOME/.config/direnv/lib/bwenv.sh ]] || { echo "FAIL no direnv bwenv.sh"; bad=1; }
  ((bad == 0)) && echo "ok   $n_files default files, $(ls "$REPO"/scripts | wc -l) scripts and $n_walls wallpapers from the store"
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
  done < <(as_user git -C "$REPO" ls-files scripts system test config | grep -vE '\.(md|nix|lua|qml|toml|json|jpg|png|webp)$')
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
  
  for t in launcher session powermenu wallpaper notifications controlcenter recorder; do
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

# Moving wallpapers. dot-theme takes a video's colors from its poster and
# names the video as the wallpaper; next/prev step over stills and videos
# alike. Quickshell plays it (software decode here, llvmpipe) and stops when
# motion is set to never. Prints Quickshell's CPU with the video playing and
# with it stopped.
c_wallpaper_video() {
  local st="$USER_HOME/.local/state/theme" vid poster a b out seen n i bad=0 pid
  vid="$(zrun 'ls ~/Pictures/wallpapers/*/*.mp4 2>/dev/null | head -1')"
  # The repo ships no video wallpapers any more (the live scene replaced them).
  [[ -n $vid ]] || { echo "skip no video wallpaper under ~/Pictures/wallpapers"; return 0; }
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
  # A full lap of `next` visits stills and videos (and ukiyo's scenes, listed
  # first in shinjuku/) and comes back.
  n="$(zrun 'ls ~/Pictures/wallpapers/shinjuku' | grep -cvE '\.poster\.')"
  n=$((n + $(zrun 'ukiyo scenes --json 2>/dev/null | jq length' 2>/dev/null || echo 0)))
  seen=""
  for ((i = 0; i < n; i++)); do
    zrun 'dot-theme next' >/dev/null 2>&1 || { echo "FAIL dot-theme next"; bad=1; break; }
    seen+="$(zrun 'dot-theme current')"$'\n'
  done
  grep -q '\.mp4$' <<<"$seen" && grep -q '\.jpg$' <<<"$seen" && [[ "$(zrun 'dot-theme current')" == "$vid" ]] &&
    ! grep -q '\.poster\.' <<<"$seen" &&
    echo "ok   next: $n wallpapers in shinjuku/, stills and videos, no posters, back to the start" ||
    { echo "FAIL next didn't lap shinjuku/ over stills and videos:"; printf '%s' "$seen"; bad=1; }

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

# ukiyo, the live wallpaper (docs/UKIYO.md): the package on PATH, the
# declarative defaults parse and are what the schema reports, the zsh hook is
# in the interactive shell, dot-theme takes ukiyo:<id> with colors from the
# scene's poster. Then in a sway session: the daemon draws, Quickshell steps
# aside, locking makes it still with a lock frame on disk, unlocking resumes,
# and a still wallpaper stops the daemon. Prints ukiyo's CPU while drawing.
c_ukiyo() {
  local st="$USER_HOME/.local/state/theme" out poster a b bad=0 pid
  out="$(zrun 'command -v ukiyo')"
  [[ $out == "$USER_HOME/.nix-profile/bin/ukiyo" ]] && echo "ok   ukiyo on PATH: $out" || { echo "FAIL ukiyo -> $out"; return 1; }
  out="$(zrun 'ukiyo --version' 2>&1)" && [[ $out == ukiyo* ]] && echo "ok   $out" || { echo "FAIL ukiyo --version: $out"; bad=1; }
  [[ -s $USER_HOME/.config/ukiyo/defaults.toml ]] && echo "ok   ~/.config/ukiyo/defaults.toml" || { echo "FAIL no defaults.toml"; bad=1; }
  out="$(zrun 'ukiyo config schema --json' 2>&1)"
  if jq -e 'map(select(.key == "performance.idle_timeout" and .value == 600 and .source == "defaults")) | length == 1' <<<"$out" >/dev/null 2>&1 &&
    jq -e 'map(select(.key == "signals.typing.source" and .value == "idle")) | length == 1' <<<"$out" >/dev/null &&
    jq -e 'map(select(.key | startswith("scenes.kabukicho-gate."))) | length > 0' <<<"$out" >/dev/null; then
    echo "ok   the schema reads defaults.toml ($(jq length <<<"$out") options, the gate's included)"
  else
    echo "FAIL ukiyo config schema doesn't show the defaults: ${out:0:300}"; bad=1
  fi
  out="$(as_user script -qec "zsh -l -i -c 'whence -w _ukiyo_precmd; print -l \$precmd_functions'" /dev/null | tr -d '\r')"
  grep -q '_ukiyo_precmd: function' <<<"$out" && grep -qx '_ukiyo_precmd' <<<"$out" &&
    echo "ok   the zsh hook is sourced (_ukiyo_precmd in precmd_functions)" || { echo "FAIL zsh hook: $out"; bad=1; }
  grep -q 'ukiyo-hook.zsh' "$USER_HOME/.zshrc" || { echo "FAIL ~/.zshrc doesn't source the generated hook"; bad=1; }

  poster="$(zrun 'ukiyo scenes --json' | jq -r '.[] | select(.id == "kabukicho-gate") | .poster')"
  [[ -f $poster ]] || { echo "FAIL no poster for kabukicho-gate: '$poster'"; return 1; }
  expect_ok "dot-theme set <the scene's poster>" zrun "dot-theme set '$poster'" || return 1
  a="$(jq -S 'del(.wallpaper)' "$st/colors.json")"
  expect_ok "dot-theme set ukiyo:kabukicho-gate" zrun 'dot-theme set ukiyo:kabukicho-gate' || return 1
  b="$(jq -S 'del(.wallpaper)' "$st/colors.json")"
  [[ $a == "$b" ]] && echo "ok   colors come from the scene's poster" || { echo "FAIL colors differ from the poster's"; bad=1; }
  [[ "$(zrun 'dot-theme current')" == ukiyo:kabukicho-gate ]] && jq -e '.wallpaper == "ukiyo:kabukicho-gate"' "$st/colors.json" >/dev/null &&
    echo "ok   the wallpaper is ukiyo:kabukicho-gate" || { echo "FAIL dot-theme current / colors.json wallpaper"; bad=1; }
  out="$(zrun 'dot-theme apply' 2>&1)"
  [[ -z $out ]] && echo "ok   dot-theme apply on a scene is quiet with nothing changed" || { echo "FAIL apply: $out"; bad=1; }

  session_start sway 1280x800 1
  in_session 'dot-theme set ukiyo:kabukicho-gate' >"$LOGS/ukiyo-set.log" 2>&1 || { echo "FAIL dot-theme set in the session:"; cat "$LOGS/ukiyo-set.log"; bad=1; }
  if ! qs_start; then
    echo "FAIL Quickshell didn't load"; session_stop; return 1
  fi
  for _ in $(seq 60); do
    out="$(in_session 'qs ipc call wallpaper status' 2>/dev/null)"
    jq -e '.ukiyo == true' <<<"$out" >/dev/null 2>&1 && break
    sleep 0.5
  done
  jq -e '.scene == "kabukicho-gate" and .ukiyo == true' <<<"$out" >/dev/null 2>&1 &&
    echo "ok   ukiyo draws the scene, Quickshell's layer stepped aside" || { echo "FAIL not drawing: $out"; in_session 'ukiyo status' 2>&1 | head; bad=1; }
  pid="$(pgrep -u "$USER_NAME" -n -x ukiyo || true)"
  [[ -n $pid ]] && echo "     ukiyo CPU (100% = one core, llvmpipe), drawing: $(cpu_pct "$pid" 5)%"
  in_session 'qs ipc call session lock' >/dev/null 2>&1
  for _ in $(seq 40); do
    out="$(in_session 'ukiyo status --json' 2>/dev/null)"
    jq -e '.state == "still" and (.still_frames | length > 0)' <<<"$out" >/dev/null 2>&1 && break
    sleep 0.25
  done
  jq -e '.state == "still" and (.still_frames | length > 0)' <<<"$out" >/dev/null 2>&1 &&
    echo "ok   locked: ukiyo is still, lock frame $(jq -r '.still_frames[0]' <<<"$out")" || { echo "FAIL lock: $(jq -c '{state, still, reasons, still_frames, outputs}' <<<"$out" 2>&1)"; bad=1; }
  [[ -n $pid ]] && echo "     ukiyo CPU, still: $(cpu_pct "$pid" 3)%"
  [[ -z "$(qs_problems)" ]] || { echo "FAIL QML errors:"; qs_problems | head; bad=1; }
  in_session 'dot-theme set ~/Pictures/wallpapers/japanese/tokyo-tower-blue-hour.jpg' >"$LOGS/ukiyo-stop.log" 2>&1
  for _ in $(seq 20); do pgrep -u "$USER_NAME" -x ukiyo >/dev/null || break; sleep 0.5; done
  pgrep -u "$USER_NAME" -x ukiyo >/dev/null && {
    echo "FAIL ukiyo still runs with a still as the wallpaper:"; cat "$LOGS/ukiyo-stop.log"
    pgrep -a -u "$USER_NAME" -x ukiyo; tail -n 5 "$st/ukiyo.log"; bad=1
  } ||
    echo "ok   a still as the wallpaper stops the daemon"
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
  for c in login_shell path tools links theme tmux nvim ghostty tms pam lua shellcheck flake_check quickshell recorder wallpaper_video ukiyo user_layer; do
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
