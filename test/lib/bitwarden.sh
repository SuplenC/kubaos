# shellcheck shell=bash disable=SC2015,SC2016
# test/run bitwarden: dot-bw-setup and `use bwenv` against a real Vaultwarden
# (compose service vaultwarden, https://vw on the private network). Sourced
# by test/lib/entry after harness.sh; prepare() has run.
#
# Stand-ins: OpenSSH's ssh-agent on ~/.bitwarden-ssh-agent.sock for the
# Bitwarden desktop agent (no arm64 build), gnome-keyring on a session bus
# unlocked with a fixed password for the login keyring.

# shellcheck source=test/lib/e2e.sh
. "$LIB/e2e.sh"

VW_URL=https://vw
VW_CA=/vw-certs/ca.pem
BW_EMAIL=kuba@example.test
BW_PASSWORD='Correct-Horse-Battery-9'
# Authenticator-app two-step login on the account (base32 secret).
TOTP_SECRET=JBSWY3DPEHPK3PXPDOTFILESTESTAB23
KEYRING_PW=keyring-test-pw
SEED_DIR=/tmp/bw-seed
DEMO=$USER_HOME/Projects/demo
# Values that must arrive intact and never land on disk in plaintext.
TOKEN_VALUE='tok-7c1f'\''"$(touch /tmp/pwned)`id`;|&'
PEM_VALUE=$'-----BEGIN TEST KEY-----\nMIIBmarker9d2e\nline-two\n-----END TEST KEY-----'

# bw_env CMD...: as kuba, login-like env plus the session bus and the CA.
bw_env() {
  runuser -u "$USER_NAME" -- env -i "${JENV[@]}" NODE_EXTRA_CA_CERTS="$VW_CA" "$@"
}
bzsh() { bw_env zsh -l -c "$1"; }

# seed_bw ARGS: bw as root with its own data dir (the user's bw stays logged out).
seed_bw() { BITWARDENCLI_APPDATA_DIR=$SEED_DIR NODE_EXTRA_CA_CERTS=$VW_CA BW_SESSION="${SEED_SESSION:-}" bw "$@"; }
create() { # create TYPE JSON -> id
  seed_bw create "$1" "$(printf '%s' "$2" | seed_bw encode)" | jq -r .id
}

c_bw_server() {
  [[ -s $VW_CA ]] || { echo "FAIL no CA at $VW_CA (vw-certs service didn't run?)"; return 1; }
  install -m 0644 "$VW_CA" /usr/local/share/ca-certificates/dotfiles-test-vw.crt
  update-ca-certificates >/dev/null 2>&1
  curl -fsS --max-time 5 "$VW_URL/alive" >/dev/null || { echo "FAIL $VW_URL/alive not reachable"; return 1; }
  echo "ok   $VW_URL up, CA trusted"
  local r
  r="$(REQUESTS_CA_BUNDLE=/etc/ssl/certs/ca-certificates.crt python3 -I "$LIB/vw-register.py" "$VW_URL" "$BW_EMAIL" "$BW_PASSWORD" "$TOTP_SECRET")" ||
    { echo "FAIL registration: $r"; return 1; }
  echo "ok   account $BW_EMAIL: $r"
}

c_bw_seed() {
  local keys=$SEED_DIR/keys dotfiles env_demo item
  rm -rf "$SEED_DIR"
  mkdir -p "$keys"
  seed_bw config server "$VW_URL" >/dev/null
  SEED_SESSION="$(BW_PASSWORD="$BW_PASSWORD" seed_bw login "$BW_EMAIL" --passwordenv BW_PASSWORD \
    --method 0 --code "$(python3 -I "$LIB/vw-register.py" --totp "$TOTP_SECRET")" --raw)" ||
    { echo "FAIL seed login"; return 1; }
  export SEED_SESSION
  dotfiles="$(create folder '{"name":"dotfiles"}')"
  env_demo="$(create folder '{"name":"env/demo"}')"
  ssh-keygen -q -t ed25519 -N '' -C github -f "$keys/github"
  ssh-keygen -q -t ed25519 -N '' -C homelab -f "$keys/homelab"
  ssh-keygen -q -t ed25519 -N '' -C evil -f "$keys/evil"
  # ssh_item NAME KEYFILE FIELDS_JSON
  ssh_item() {
    jq -n --arg name "$1" --arg folder "$dotfiles" --rawfile priv "$keys/$2" \
      --arg pub "$(cut -d' ' -f1,2 "$keys/$2.pub")" \
      --arg fp "$(ssh-keygen -lf "$keys/$2.pub" -E sha256 | awk '{print $2}')" --argjson fields "$3" \
      '{type: 5, name: $name, folderId: $folder, notes: null, fields: $fields,
        sshKey: {privateKey: $priv, publicKey: $pub, keyFingerprint: $fp}}'
  }
  item="$(ssh_item GitHub github '[{"name":"hosts","value":"github.com","type":0},{"name":"user","value":"git","type":0},{"name":"signing","value":"true","type":2}]')"
  create item "$item" >/dev/null
  item="$(ssh_item 'Homelab server' homelab '[{"name":"hosts","value":"homelab nas.lan","type":0},{"name":"hostname","value":"10.0.0.5","type":0},{"name":"user","value":"kuba","type":0},{"name":"port","value":"2222","type":0}]')"
  create item "$item" >/dev/null
  # An item trying to smuggle a directive into ssh_config: no Host block.
  item="$(ssh_item Evil evil "$(jq -n '[{name: "hosts", value: "evil\n  ProxyCommand touch /tmp/pwned-ssh", type: 0}]')")"
  create item "$item" >/dev/null
  item="$(jq -n --arg folder "$env_demo" --arg tok "$TOKEN_VALUE" --arg pem "$PEM_VALUE" \
    '{type: 2, name: "demo env", folderId: $folder, secureNote: {type: 0}, notes: null,
      fields: [{name: "DATABASE_URL", value: "postgres://demo:pw@db:5432/demo", type: 1},
               {name: "API_TOKEN", value: $tok, type: 1},
               {name: "TLS_KEY", value: $pem, type: 1},
               {name: "bad-name", value: "x", type: 0}]}')"
  create item "$item" >/dev/null
  item="$(jq -n '{type: 2, name: "shared web", secureNote: {type: 0}, notes: null,
      fields: [{name: "bwenv", value: "demo", type: 0}, {name: "EXTRA_VAR", value: "extra-value", type: 1}]}')"
  create item "$item" >/dev/null
  seed_bw sync >/dev/null
  echo "ok   vault: 2 folders, 3 SSH keys (one hostile), env/demo + a bwenv item ($(seed_bw list items | jq length) items)"
}

c_bw_session() {
  # Session bus + unlocked login keyring (gnome-keyring), the Bitwarden agent
  # socket served by OpenSSH's agent with the two real keys.
  printf '%s' "$KEYRING_PW" | bw_env gnome-keyring-daemon --unlock --components=secrets >/dev/null
  rm -f "$USER_HOME/.bitwarden-ssh-agent.sock"
  as_user ssh-agent -a "$USER_HOME/.bitwarden-ssh-agent.sock" >/dev/null
  # Like Bitwarden's agent: every SSH key in the vault.
  install -m 0600 -o "$USER_NAME" "$SEED_DIR/keys/github" "$SEED_DIR/keys/homelab" "$SEED_DIR/keys/evil" "$RUNTIME_DIR/"
  bw_env env SSH_AUTH_SOCK="$USER_HOME/.bitwarden-ssh-agent.sock" ssh-add -q "$RUNTIME_DIR/github" "$RUNTIME_DIR/homelab" "$RUNTIME_DIR/evil"
  rm -f "$RUNTIME_DIR/github" "$RUNTIME_DIR/homelab" "$RUNTIME_DIR/evil"
  bw_env secret-tool store --label probe service dotfiles-test-probe <<<"probe" ||
    { echo "FAIL keyring not writable"; return 1; }
  echo "ok   session bus, unlocked keyring, agent with the vault's 3 keys on ~/.bitwarden-ssh-agent.sock"
}

c_bw_setup() {
  local log=$LOGS/dot-bw-setup.out rc=0
  cat >"$LOGS/bw-setup.exp" <<'EOF'
set timeout 120
log_user 1
spawn zsh -l -c "dot-bw-setup"
# After spawn: expect_after binds to the current spawn id.
expect_after timeout { puts "\nexpect: timed out"; exit 98 }
expect -ex "Choice \[1-3\]:" { send "3\r" }
expect -ex "Server URL" { send "https://vw\r" }
expect -ex "Choice \[1-3\]:" { send "1\r" }
expect -ex "Email:" { send "$env(BW_EMAIL)\r" }
# bw's prompts (inquirer) drop input typed before they switch the tty to raw
# mode: wait for the whole prompt, then a moment. (No (?i): expect's glob
# pre-filter ignores it.)
expect -re {[Mm]aster password.*hidden} { sleep 0.5; send "$env(BW_PASSWORD)\r" }
# dot-bw-setup asks bw for the authenticator-app code (--method 0).
# The prompt redraws on every keystroke, so answer it once.
set code_sent 0
expect {
  -re {[Tt]wo-step login code:} {
    if {!$code_sent} {
      set code_sent 1
      sleep 0.5
      send "[exec python3 -I $env(VW_REGISTER) --totp $env(TOTP_SECRET)]\r"
    }
    exp_continue
  }
  eof
}
catch wait result
exit [lindex $result 3]
EOF
  chmod 0644 "$LOGS/bw-setup.exp"
  bw_env BW_EMAIL="$BW_EMAIL" BW_PASSWORD="$BW_PASSWORD" TOTP_SECRET="$TOTP_SECRET" \
    VW_REGISTER="$LIB/vw-register.py" expect -f "$LOGS/bw-setup.exp" >"$log" 2>&1 || rc=$?
  tr -d '\r' <"$log" | sed -n '/==> Summary/,$p'
  ((rc == 0)) || { echo "FAIL dot-bw-setup exited $rc"; tail -30 "$log"; return 1; }
  grep -q 'vault unlocked for this run' "$log" || { echo "FAIL no unlock in the output"; return 1; }
  grep -q 'agent  running, serves every SSH key' "$log" || { echo "FAIL summary: agent doesn't serve every vault key"; return 1; }
  echo "ok   dot-bw-setup: server prompt, password + authenticator code, unlock, all parts (exit 0)"
}

c_bw_files() {
  local h=$USER_HOME bad=0 out
  for f in .ssh/github.pub .ssh/homelab-server.pub .ssh/config.d/bitwarden \
    .config/git-bitwarden/signing.gitconfig .config/git-bitwarden/allowed_signers .config/dot-bw/config; do
    [[ -f $h/$f ]] && echo "ok   ~/$f" || { echo "FAIL missing ~/$f"; bad=1; }
  done
  [[ ! -e $h/.ssh/github ]] || { echo "FAIL a private key file appeared"; bad=1; }
  grep -q ProxyCommand "$h/.ssh/config.d/bitwarden" && { echo "FAIL hostile field made it into ssh_config"; bad=1; }
  [[ "$(grep -c '^Host ' "$h/.ssh/config.d/bitwarden")" == 2 ]] || { echo "FAIL expected 2 Host blocks"; bad=1; }
  out="$(bzsh 'ssh -G github.com')"
  grep -qx "identityfile $h/.ssh/github.pub" <<<"$out" && grep -qx 'user git' <<<"$out" &&
    grep -qx "identityagent $h/.bitwarden-ssh-agent.sock" <<<"$out" &&
    echo "ok   ssh -G github.com: user git, IdentityFile github.pub, Bitwarden agent" ||
    { echo "FAIL ssh -G github.com"; bad=1; }
  out="$(bzsh 'ssh -G nas.lan')"
  grep -qx 'hostname 10.0.0.5' <<<"$out" && grep -qx 'port 2222' <<<"$out" &&
    echo "ok   ssh -G nas.lan: 10.0.0.5:2222" || { echo "FAIL ssh -G nas.lan"; bad=1; }
  # Signed commit, verified through allowed_signers.
  out="$(bzsh 'rm -rf /tmp/signtest && git init -q /tmp/signtest && cd /tmp/signtest &&
    git commit -q --allow-empty -m signed && git log --show-signature -1' 2>&1)"
  grep -q 'Good "git" signature' <<<"$out" && echo "ok   git commit is signed: Good \"git\" signature" ||
    { echo "FAIL commit signature:"; printf '%s\n' "$out" | head; bad=1; }
  # Nothing secret in plaintext under $HOME (bw's own data.json is encrypted).
  # (The repo copy is skipped: this very file holds the markers.)
  if grep -rIlF --exclude-dir=dotfiles -e 'MIIBmarker9d2e' -e 'tok-7c1f' -e 'BEGIN OPENSSH PRIVATE KEY' "$h" 2>/dev/null; then
    echo "FAIL a secret is on disk in plaintext (files above)"; bad=1
  else
    echo "ok   no secret in plaintext under ~"
  fi
  [[ ! -e /tmp/pwned && ! -e /tmp/pwned-ssh ]] || { echo "FAIL something ran a vault value"; bad=1; }
  out="$(bzsh 'dot-bw-setup --check' 2>&1)"
  grep -q 'agent answers with 3 key' <<<"$out" && echo "ok   dot-bw-setup --check: agent answers" ||
    { echo "FAIL dot-bw-setup --check"; printf '%s\n' "$out" | tail -8; bad=1; }
  return "$bad"
}

c_bw_direnv() {
  local out t0 bad=0
  out="$(bzsh 'dot-bwenv list')"
  printf '%s\n' "$out"
  grep -q '^demo' <<<"$out" || { echo "FAIL keyring has no project demo"; return 1; }
  for v in DATABASE_URL API_TOKEN TLS_KEY EXTRA_VAR; do
    grep -q "$v" <<<"$out" || { echo "FAIL demo lacks $v"; bad=1; }
  done
  grep -q 'bad-name' <<<"$out" && { echo "FAIL a non-shell field name was imported"; bad=1; }
  install -d -o "$USER_NAME" -g "$USER_NAME" "$DEMO"
  echo 'use bwenv demo' | as_user tee "$DEMO/.envrc" >/dev/null
  bzsh "cd $DEMO && direnv allow" >/dev/null 2>&1
  # cd in an interactive zsh: direnv's chpwd hook exports, leaving unloads.
  out="$(bw_env script -qec "zsh -l -i -c 'cd $DEMO; print -r -- \"DB=\$DATABASE_URL\"; print -r -- \"TOK=\$API_TOKEN\"; print -r -- \"\$TLS_KEY\" | wc -l; cd /; print -r -- \"AFTER=\${DATABASE_URL:-unset}\"'" /dev/null 2>/dev/null | tr -d '\r')"
  grep -qx 'DB=postgres://demo:pw@db:5432/demo' <<<"$out" || { echo "FAIL DATABASE_URL on cd"; printf '%s\n' "$out"; bad=1; }
  grep -qxF "TOK=$TOKEN_VALUE" <<<"$out" || { echo "FAIL API_TOKEN not intact"; bad=1; }
  grep -qx '4' <<<"$out" || { echo "FAIL multi-line TLS_KEY not intact"; bad=1; }
  grep -qx 'AFTER=unset' <<<"$out" || { echo "FAIL variables stay after leaving the dir"; bad=1; }
  t0="$(now)"
  bzsh "cd $DEMO && for i in 1 2 3 4 5; do direnv export zsh >/dev/null 2>&1; done"
  echo "     direnv export: $(awk -v s="$(since "$t0")" 'BEGIN { printf "%.0f", s * 1000 / 5 }') ms per run"
  [[ ! -e /tmp/pwned ]] || { echo "FAIL a vault value was executed"; bad=1; }
  ((bad == 0)) && echo "ok   cd exports 4 variables intact (metacharacters, multi-line), leaving unloads them"
  return "$bad"
}

bitwarden_main() {
  local c t0
  t0="$(now)"
  say "bitwarden checks against Vaultwarden at $VW_URL"
  for c in bw_server bw_seed bw_session bw_setup bw_files bw_direnv; do
    check "$c" "c_$c"
    # Each step needs the one before it.
    ((FAILED)) && break
  done
  echo
  say "results ($(since "$t0") s)"
  local r n s secs d
  for r in "${RESULTS[@]}"; do
    IFS='|' read -r n s secs d <<<"$r"
    printf '%-12s %-5s %8s  %s\n' "$n" "$s" "$secs" "$d"
  done
  ((FAILED == 0)) || return 1
  printf '%sall bitwarden checks passed%s\n' "$C_OK" "$C_OFF"
}
