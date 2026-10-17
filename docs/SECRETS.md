# Secrets: Bitwarden and the security key

KubaOS keeps secrets out of files. One Bitwarden account holds my SSH keys, project env secrets and the GitHub CLI token, and the private keys never touch the disk. A YubiKey C Bio (FIDO2 with a fingerprint sensor) stands in for the password at sudo, polkit and the lock screen. The password keeps working everywhere, so a lost key is a bad day, not a lockout.

| Piece | Where it lives | Set up by |
|---|---|---|
| SSH private keys | Bitwarden vault only, served by the desktop app's SSH agent | you, in the vault |
| SSH public keys, Host blocks | `~/.ssh/<name>.pub`, `~/.ssh/config.d/bitwarden` | `dot-bw-setup` |
| `SSH_AUTH_SOCK` | `~/.bitwarden-ssh-agent.sock` for zsh, Hyprland, systemd --user | `home/secrets.nix` |
| git signing | `~/.config/git-bitwarden/{signing.gitconfig,allowed_signers}`, included by `config/git/config` | `dot-bw-setup` |
| GitHub CLI token | GNOME keyring (gh's own entry) | `dot-bw-setup` |
| Project env secrets | vault, cached in the GNOME keyring | `dot-bw-setup`, `dot-bwenv pull` |
| Security key credentials | `/etc/u2f_mappings`, root:root 0644 | `dot-u2f-enroll` |
| YubiKey SSH keys (optional) | `~/.ssh/<name>_sk` (key handle) + `.pub` | `dot-ssh-sk-new` |
| Server URL, email, folder name | `~/.config/dot-bw/config`, nothing secret | `dot-bw-setup` |

## First-time setup

Installed from the KubaOS stick? Steps 1 and 2 and `system/75-yubikey.sh` are already done, and first-boot setup (`kubaos-setup`, docs/INSTALLER.md) runs step 4 and `dot-u2f-enroll` for you. What's left by hand is the desktop app's SSH agent toggle (step 3) and the key's own PIN and fingerprints.

1. `system/70-bitwarden.sh` (after `60-nix.sh`). It installs the Bitwarden desktop `.deb` and the `bw` CLI, both pinned and sha256-checked, plus `secret-tool`, `pam_u2f` and `pamu2fcfg`. It puts the security key in front of the password for sudo and polkit and installs the lock screen's `qs-lock-u2f` service.
2. `home-manager switch --flake ~/Projects/dotfiles`, then log out and in, so `SSH_AUTH_SOCK` points at Bitwarden everywhere.
3. Open Bitwarden (Hyprland starts it at login), log in, then in Settings:
   - turn on **Enable SSH agent**. It's off by default. The app keeps the switch in its own `data.json` with no documented config-file way to set it, so this stays a click.
   - optional: **Unlock with system authentication**. That's a polkit prompt, and the security key answers it like it answers sudo.
   - optional: **Start to tray icon**, so the login start stays out of the way.
4. `dot-bw-setup`. The first time it asks for the server (bitwarden.com, bitwarden.eu or your own URL) and how to log in. Then bw asks for the master password and the two-step code. After that it writes the SSH, git, gh and env pieces, checks the agent and prints a summary. Re-running it is safe.
5. The YubiKey: `system/75-yubikey.sh`, then the order in "YubiKey C Bio" below (PIN, fingerprints, Bitwarden two-step login, `dot-u2f-enroll` for each key).

What stays manual, and why: the SSH agent toggle (no supported config switch), the master password and two-step code (typed into bw's own prompt, never into these scripts), the touch during enrollment, and the web vault's two-step settings.

## Vault conventions

Folder **`dotfiles`** (rename it with `folder=<name>` in `~/.config/dot-bw/config`):

- **SSH key items** (Bitwarden item type "SSH key"). Optional custom fields:

  | Field | Example | Effect |
  |---|---|---|
  | `hosts` | `github.com` or `homelab nas.lan` | writes a `Host` block for these patterns (space or comma separated) |
  | `hostname` | `10.0.0.5` | `HostName` |
  | `user` | `git` | `User` |
  | `port` | `2222` | `Port` |
  | `signing` | boolean `true` | git signs commits with this key. Exactly one. Text `verify` keeps an old key in `allowed_signers` after a rotation |
  | `emails` | `me@work.com` | extra emails for `allowed_signers` (git's `user.email` is always in) |
  | `file` | `id_work` | name of the `.pub` file, default is the item name in lowercase with dashes |

  The public key goes to `~/.ssh/<file>.pub`. The Host block names that file in `IdentityFile` with `IdentitiesOnly yes`, so ssh asks the agent for that key only. Without `hosts` you get the `.pub` file and no block.
- **An item named `gh-token`**: a login item whose password is a GitHub token (classic with `repo`, `read:org`, or fine-grained), or any item with a hidden field `token`. Optional field `host` for GitHub Enterprise.

Env secrets, one project per folder **`env/<project>`**: every custom field of every item in it becomes a variable. Field name is the variable name, so it must be a shell name (`DATABASE_URL`, not `database-url`). Items elsewhere join a project with a text field `bwenv` = `<project>`.

Example vault:

```text
dotfiles/
  GitHub            SSH key   hosts=github.com  user=git  signing=true
  Homelab server    SSH key   hosts="homelab nas.lan"  hostname=10.0.0.5  user=kuba  port=2222
  gh-token          Login     password=github_pat_...
env/arms/
  arms dev          Secure note  DATABASE_URL=postgres://...  STRIPE_KEY=sk_test_...
Shared/
  web env           Secure note  bwenv=web  API_TOKEN=...
```

This gives `~/.ssh/github.pub`, `~/.ssh/homelab-server.pub`, two Host blocks, signed commits with the GitHub key, gh logged in, and keyring entries for `arms` (2 variables) and `web` (1).

## Daily use

```sh
cd ~/Projects/arms
echo 'use bwenv arms' >> .envrc && direnv allow   # once per project
```

Every `cd` into the project then exports the variables from the keyring. One `secret-tool` call does it, and `direnv export` with 5 variables took 22 ms in the test container. Nothing talks to Bitwarden on `cd`. If the keyring has nothing for the project, direnv prints `run: dot-bwenv pull arms` and exports nothing.

| Command | What |
|---|---|
| `dot-bwenv pull arms` | fetch one project from the vault (asks for the master password, about 12 s) |
| `dot-bwenv refresh` | pull every project the keyring already has |
| `dot-bwenv list` | projects and variable names, never values |
| `dot-bwenv clear arms` / `--all` | forget a project or all of them |
| `dot-bw-setup --refresh` | after any vault change: SSH, git, gh, all projects (about 15 s) |
| `dot-bw-setup --check` | no bw call: is the agent up, which files exist |
| `dot-u2f-enroll --list` / `--remove N` | your enrolled keys |

Each `bw` call takes about 3 s, so the scripts sync once and read the whole vault with one `bw list items`. They unlock once per run and keep `BW_SESSION` in that process and its children. It's never written down. Export `BW_SESSION` yourself and they reuse it without asking.

There's no systemd unit or timer. A re-run needs the master password, a unit can't ask for it, and keeping a session around for one would be exactly the long-lived vault access this setup avoids.

SSH: the desktop app must be running and unlocked once since it started. It asks before each use of a key. Pick how often under Settings, "Ask for authorization when using SSH agent".

The GNOME keyring: tty autologin means nobody types the login password, so nothing can unlock a password-protected login keyring. An install from the KubaOS stick gets a default keyring without a password (docs/INSTALLER.md, "A calm first login"): it's always open, and LUKS protects it at rest. On a machine set up by hand with these scripts, the first `cd` into a project (or `gh`) after boot shows the keyring's unlock prompt. Give that keyring the same password as your account so the prompt is easy to answer, or create the passwordless one the same way the installer does.

## Rotation

- **SSH key**: in Bitwarden, create a new SSH key item with the same fields. Change the old item's `signing` to `verify` or move it out of `dotfiles`. Add the new public key on GitHub and servers, then `dot-bw-setup --refresh`. Delete the old item once nothing uses it. Its `.pub` and Host block disappear on the next refresh.
- **Env secret**: change the field in Bitwarden, then `dot-bwenv pull <project>` and `cd .` (or `direnv reload`).
- **gh token**: put the new token in `gh-token`, `dot-bw-setup --refresh --only gh`.
- **Master password**: change it in the web vault. Nothing here stores it.
- **Security key**: `dot-u2f-enroll` with the new key, `dot-u2f-enroll --remove N` for the old one, and do the same in the web vault.

## YubiKey C Bio

The key does FIDO2 and U2F, and the fingerprint on its sensor counts as user verification (UV), the same thing a PIN proves. The FIDO Edition stops there. The Multi-protocol Edition (sold through YubiKey as a Service) adds PIV, a smart card. Everything here uses FIDO2 only, so both editions behave the same. `dot-u2f-enroll --info` prints which edition and firmware you have, from `ykman info`. ykman names them "YubiKey C Bio - FIDO Edition" and "YubiKey C Bio - Multi-protocol Edition".

Facts from Yubico's Bio documentation that the setup relies on:

- The key holds 5 fingerprints. It needs a FIDO2 PIN before it lets you enroll one.
- After 3 failed scans in a row, FIDO2 wants the PIN. One successful FIDO2 operation with the PIN clears that. The key only needs replugging after 3 wrong PINs in a row. 8 wrong PINs in total block the FIDO2 application, and only a reset brings it back, which deletes every credential and fingerprint on the key.
- On the Multi-protocol Edition, PIV shares the PIN, which must then be 6 to 8 characters.

### First-time setup, in this order

1. `system/75-yubikey.sh` (it installs `fido2-tools`). `home-manager switch` brings `ykman`.
2. Set the FIDO2 PIN: `ykman fido access change-pin`. Use 6 to 8 characters if you might ever use PIV.
3. Enroll 2 or 3 fingers, including one from the other hand: `ykman fido fingerprints add right-index`, `... add left-index`. `ykman fido fingerprints list` and `delete <id>` manage them.
4. Bitwarden, before you turn on two-step login: web vault → Settings → Security → Two-step login → **View recovery code**. Write it down and keep it offline, away from the laptop. It's the way back in if every two-step method is lost, and using it turns two-step login off.
5. Add the key there: **Passkey (FIDO2 WebAuthn)**. FIDO2 two-step login is free on every plan. Add the backup key too.
6. Add **Authenticator app** (TOTP) as a second method. `bw` can't use the key, so the CLI logs in with this code (below). It's also how you get in from a device that has no USB-C port.
7. `dot-u2f-enroll`, then `dot-u2f-enroll --test` and `--test-lock`. Then `sudo -k; sudo true` from a second tty is the real test.
8. With only the backup key plugged in, repeat steps 2, 3 and 7. Buy the second key. With two, losing one costs you nothing. A YubiKey 5 without a fingerprint sensor works as the backup too, through `dot-u2f-enroll --pin`.

`ykman` (Nix's `yubikey-manager`) covers the PIN and the fingerprints, so the Yubico Authenticator app isn't installed. On Linux it ships as a tarball, and Flathub's build isn't Yubico's own. It wants pcscd for its OATH screen, and the Bio has no OATH. For a GUI anyway, run `nix run nixpkgs#yubioath-flutter`.

### What the finger unlocks, and where

| Where | Key plugged in | Finger fails or times out | Key not plugged in |
|---|---|---|---|
| sudo, `sudo -i` | "Touch the fingerprint sensor" | "Please enter the PIN:", then a touch. Enter skips to the password | password, as before |
| polkit dialogs (hyprpolkitagent) | same prompts in the dialog | PIN, then password | password |
| Bitwarden desktop "Unlock with system authentication" | polkit dialog, finger | PIN, then password | password |
| Lock screen (Quickshell) | "Touch the fingerprint sensor" under the field, password works in parallel | "Fingerprint not recognised", then it asks again. After 3 bad scans: "Fingerprint locked" | "Or use your security key", password |
| SSH key made with `dot-ssh-sk-new` | finger on every use | OpenSSH asks for the PIN | unusable |
| Bitwarden login (web, desktop, extension) | Passkey two-step login: finger | the authenticator-app code | authenticator-app code |
| `bw` CLI login | no FIDO2 support | | authenticator-app code, or the API key |
| tty login, hyprlock | not used | | password |

The lock screen never asks for the PIN: a lock screen that took a PIN would also take a guessed one. When the finger is locked out, unlock with the password, then run `sudo true` and enter the PIN at its prompt. That clears the lockout.

Bitwarden desktop's "Unlock with system authentication" is polkit's `com.bitwarden.Bitwarden.unlock` (`auth_self`), so the polkit stack above answers it. The app greys the option out after each start until you've unlocked once with the master password. It installs its polkit policy with `pkexec` the first time you turn the option on (the `.deb`; Flatpak needs it copied by hand). **Needs checking on the hardware:** whether hyprpolkitagent shows pam_u2f's cue text, and the whole finger-to-unlocked round trip. The container tests can't plug in a key.

### How the PAM side is set up

`pam_u2f` 1.4.0 (Ubuntu's in `/etc/pam.d/{sudo,sudo-i,polkit-1}`, nixpkgs' for the lock screen) runs before the password:

```text
auth  sufficient  pam_u2f.so authfile=/etc/u2f_mappings cue nouserok userverification=1 pinverification=0 [cue_prompt=Touch the fingerprint sensor]
auth  sufficient  pam_u2f.so authfile=/etc/u2f_mappings cue nouserok userverification=0 pinverification=1 [cue_prompt=Touch the key to confirm the PIN]
@include common-auth
```

This is pam_u2f(8)'s own example for "built-in UV with a fallback to PIN". Line 1 only talks to keys whose `uv` option is on, meaning a fingerprint is enrolled. A key without one is skipped silently. Line 2 asks for the PIN, then for a touch. `sufficient` means a good key ends the stack. Anything else falls through to the unchanged `@include common-auth`: no key, a wrong finger, a timeout, an empty PIN, or a user with no mapping (`nouserok` returns "ignore", never "success"). With no key plugged in, both lines fail in about 50 ms without printing anything. Enter at the PIN prompt costs no PIN try, because libfido2 refuses a PIN shorter than 4 characters before sending anything to the key. Ubuntu's sudo is sudo-rs, which prints the cue as `[sudo] Touch the fingerprint sensor` and the PIN prompt as `[sudo: authenticate] Please enter the PIN:`.

The lock screen's `/etc/pam.d/qs-lock-u2f` has line 1 only.

What the credential has to look like: `dot-u2f-enroll` runs `pamu2fcfg -V`. That makes a non-resident credential with UV (you scan a finger during enrollment), and the mapping gets `+presence+verification`. It never adds `-N`, because a `+pin` credential makes pam_u2f ask for the PIN on every use, even with `pinverification=0`, and the lock screen can't answer. It refuses a key with no PIN or no fingerprint and tells you the `ykman` command. With several FIDO devices plugged in it stops, since pamu2fcfg would take the first one. Before enrolling it asks the plugged-in key whether it already holds one of your credentials, using a silent `fido2-assert` with `up=false`, the same check pam_u2f does. If the key does, it offers to replace that credential in place (`--replace` skips the question), so re-running is safe. `--pin` enrolls a key without a sensor (`pamu2fcfg -N`): sudo and polkit ask for its PIN, and the lock screen skips it. `--touch` is gone, because both lines demand UV or a PIN.

Why one root-owned `/etc/u2f_mappings` instead of `~/.config/Yubico/u2f_keys`: anything running as you can write a file in your home. Malware could add a credential for a software "key" it controls and then run sudo without your password. Only root can change the central file. It's world-readable because the lock screen runs PAM as you, and it holds credential ids and public keys, nothing secret.

Access to the key: on 26.04, systemd's `60-fido-id.rules` marks FIDO tokens `ID_SECURITY_TOKEN`, and `70-uaccess.rules` gives the active seat user an ACL on the `/dev/hidraw*` node. `libu2f-udev` no longer exists in resolute, and nothing else is needed. polkit 127 runs authentication in a sandboxed `polkit-agent-helper@.service` with `PrivateDevices=yes`, which hides the key. `70-bitwarden.sh` adds a drop-in that gives it `/dev/hidraw*` back, the same drop-in Arch ships (polkit-org/polkit#622).

The lock screen runs `qs-lock-u2f` next to the password (`Settings.u2f`, on by default, active only when you have a mapping). Quickshell links Nix's libpam, and that can't load Ubuntu's `pam_u2f.so`: its loader doesn't search `/usr/lib/<triplet>`, so `libfido2.so.1` isn't found. The service loads nixpkgs' module (also 1.4.0) through the root-owned link `/usr/local/lib/security/pam_u2f.so`, pinned with a GC root, the same way as `pam_fprintd`. With no key plugged in, the conversation fails at once and restarts every 4 s, so plugging the key in later works. After a failed scan it retries in 2.5 s. If it fails twice right after the cue, the hint says "Fingerprint locked" and it retries every 15 s. The old fprintd reader path is still there (`Settings.fingerprint`, now off).

Packages: `fido2-tools` comes from apt, built against the same libfido2 1.16 that pam_u2f and pamu2fcfg use, so what `fido2-token -I` reports is what PAM sees. `ykman` comes from Nix, because apt's `yubikey-manager` depends on pcscd and the FIDO Edition never needs a smart-card daemon. `system/75-yubikey.sh --piv` installs pcscd and pcsc-tools for the Multi-protocol Edition's PIV. Only the socket is enabled, and pcscd exits when idle. Nix's pcsclite (2.4.1) talks to Ubuntu's pcscd (2.4.1).

Untouched: tty login (autologin, nothing to ask) and hyprlock, the fallback locker, which stays password-only.

`system/70-bitwarden.sh --remove-u2f` takes the key out of sudo and polkit again.

## Two-step login: Bitwarden

Bitwarden's help pages (checked 2026-10) list FIDO2 two-step login for the web vault, the browser extensions and the desktop apps on Windows, macOS and Linux. So the desktop app here can take the key directly. The `bw` CLI can't: it does the authenticator app (method 0), email (1) and YubiKey OTP (3), and the docs say FIDO2 and Duo aren't supported.

`dot-bw-setup` therefore logs in like this:

1. **Email + master password + authenticator-app code** (the default, Enter at the menu). It runs `bw login <email> --method 0`, so bw asks for the TOTP code even when the key is the account's preferred method.
2. **Personal API key**, offered when that fails, or chosen with `--login apikey` (web vault → Settings → Security → Keys → View API key). `bw login --apikey` asks for `client_id` and `client_secret` and skips two-step login. The scripts never see or store the pair, but bw keeps a refresh token in `~/.config/Bitwarden CLI/data.json` like any login. The trade-off: the API key is one factor. Anyone with it, plus your master password, can read the vault. Keep it in the vault and rotate it in the web vault if it leaks.
3. **SSO**, if an organization gives you one.

The desktop app's first login can use the key, the TOTP code, or "Log in with device" (approve it from the phone app). After that, day to day, unlock it with system authentication: the polkit prompt, which your finger answers (table above).

## SSH keys on the YubiKey (optional)

`dot-ssh-sk-new <name>` creates an `ed25519-sk` key with `-O verify-required`, so every use needs your finger. OpenSSH 10.2 uses the key's built-in UV and asks for the PIN only when that fails. `--resident` also stores the key on the YubiKey (asks for the PIN), so `ssh-keygen -K` can fetch it on a new machine. The Bio holds 25 such keys on firmware 5.5/5.6 and 100 on 5.7+. `~/.ssh/<name>_sk` is a key handle, useless without the key and your finger, so it has no passphrase. Make one per YubiKey (`<name>` and `<name>-backup`) and authorize both.

When to use which:

- **Bitwarden-held keys** (the default): they sync to every machine, the desktop agent serves them and asks before each use, and losing a YubiKey doesn't touch them. They're as safe as the vault and the running desktop app.
- **`-sk` keys**: the private key can't leave the hardware, and every use needs your finger. Use them for the few places where "my finger, here, now" is worth the friction: production, the homelab's root, maybe GitHub. Bitwarden's agent accepts Ed25519 and RSA only, so these go straight from the file with `IdentityAgent none` in their Host block (the script prints it). On your own servers, prefix the `authorized_keys` line with `verify-required` so the server insists on UV too.

## LUKS unlock with the key (optional, not automated)

Before anything else, check which initramfs builder you have: `dpkg -l dracut initramfs-tools`. Fresh 26.04 installs use dracut, and dracut's systemd-cryptsetup understands FIDO2. initramfs-tools ignores `fido2-device=`. A system still on initramfs-tools would need switching to dracut first (`sudo apt install dracut`, which replaces initramfs-tools).

```sh
sudo systemd-cryptenroll /dev/nvme0n1p3                     # lists keyslots; the passphrase is slot 0
sudo systemd-cryptenroll --fido2-device=auto --fido2-with-user-verification=yes /dev/nvme0n1p3
# asks for the passphrase, then the key's PIN, then a touch; repeat with the backup key
# /etc/crypttab: add fido2-device=auto to the volume's options, e.g.
#   luks-...  UUID=...  none  luks,discard,fido2-device=auto
sudo dracut -f
```

Notes:

- Keep the passphrase keyslot. Never run `--wipe-slot=password`. Without the key, boot asks for the passphrase like today.
- `--fido2-with-user-verification=yes` asks for UV at boot. In the initramfs that's the key's PIN, or a finger if the Bio does its own UV there. **Needs checking on the hardware.** systemd also asks for the PIN by default (`--fido2-with-client-pin=yes`).
- Risks: a wrong crypttab line or a missing FIDO2 module in the initramfs means a boot that stops at the emergency shell. Take a snapper snapshot first, keep the 26.04 USB stick at hand, and test one reboot with the passphrase before relying on the key. Recovery: boot the stick, `cryptsetup open` with the passphrase, chroot, fix crypttab, `dracut -f`.
- A reset of the YubiKey (8 wrong PINs) or a lost key kills its keyslot. The passphrase still works. Remove the dead slot with `systemd-cryptenroll --wipe-slot=<n>`.

## Threat model in plain words

- **Stolen laptop, powered off.** LUKS. Nothing secret sits outside the encrypted disk anyway, and the vault cache in `~/.config/Bitwarden*` is encrypted with your master password.
- **Stolen laptop, locked.** The lock screen wants your password or your finger on your key. The keyring stays unlocked while you're logged in, so a lock-screen bypass would expose the env secrets but not the vault or the SSH private keys.
- **Malware running as you.** It can read env secrets from the unlocked keyring and anything a process you run can read. It can ask the agent to sign, and Bitwarden asks you first unless you set its authorization prompt to Never. It can't copy the private keys: they don't exist on disk and the agent never hands them out. It can't enroll its own key for sudo (root-owned mappings). It can wait for you to type the master password into bw, as with any password manager. `bw` is root-owned in `/usr/local/bin`, but malware can still put its own `bw` earlier in your PATH.
- **Other local users.** Secrets never appear in argv (`/proc/*/cmdline` is world-readable). The scripts pass them on stdin, and `BW_SESSION` travels in the environment, which only you and root can read.
- **Someone who takes the security key.** They need your enrolled finger or the key's PIN (8 tries, then the key wipes its FIDO2 side), and the key only counts on this machine (credentials are bound to `pam://<hostname>`).
- **Vault compromise.** Everything here comes from it. That's the deal with one vault. The security key as two-step login protects the account itself.

## If the key is lost

1. Nothing locks you out. The password still works for sudo, polkit and the lock screen, the backup key works too, and Bitwarden takes the authenticator-app code.
2. Remove the lost key: plug in the backup and run `dot-u2f-enroll --list`. The backup is marked "(plugged in)". Then `dot-u2f-enroll --remove N` for the other one. In the web vault, remove it from two-step login. Delete its `-sk` SSH public keys wherever you authorized them, and its LUKS keyslot if you added one.
3. Enroll a replacement (PIN, fingers, `dot-u2f-enroll`, web vault).

Lost both keys? The password still works on this machine, and Bitwarden takes the TOTP code. Lost the phone too? Use the recovery code you saved offline.

The finger stopped working (a cut, a bandage): the PIN works at sudo and polkit, the password works everywhere. Enroll another finger with `ykman fido fingerprints add`.

## Troubleshooting

- **`ssh -T git@github.com` asks for a password or says "no identities".** Run `dot-bw-setup --check`. Then `echo $SSH_AUTH_SOCK`: it should name `~/.bitwarden-ssh-agent.sock`. If not, log out and in after `home-manager switch`. `systemctl --user show-environment | grep SSH` shows what systemd services get.
- **sudo waits at "Touch the fingerprint sensor".** The key is plugged in and waiting for an enrolled finger. If it doesn't match, or you wait it out, sudo asks for the PIN. Press Enter there to get the password prompt, or pull the key out.
- **The finger never works, the PIN does.** `dot-u2f-enroll --info`: "fingerprints: none yet" means none are enrolled (`ykman fido fingerprints add`). "N scans left" counts down to the PIN lockout.
- **`dot-u2f-enroll`: "no FIDO key found".** `ls -l /dev/hidraw*` should show a `+` (ACL) on the key's node. If not, you're not on the active seat (over ssh, say), or udev didn't tag it: `udevadm info /dev/hidrawN | grep SECURITY_TOKEN`.
- **Lock screen ignores the key.** `ls -l /usr/local/lib/security/pam_u2f.so` must point into `/nix/store` (else `system/70-bitwarden.sh` again). Test without locking: `dot-u2f-enroll --test-lock`. A PIN-only key can't unlock the lock screen.
- **direnv: "can't read the keyring".** gnome-keyring isn't running (Hyprland starts it) or you dismissed its unlock prompt. `secret-tool search --all service dotfiles-env` should list your entries.
- **Keys stopped working after a hostname change.** Credentials are bound to `pam://<hostname>`. Enroll the keys again.
- **Check a PAM file without risking a lockout:** `sudo pamtester sudo $USER authenticate` from a second tty.
