# 1. Releases and in-system updates

Status: accepted (2026-10-18)

## Context

KubaOS ships as an ISO, but an installed machine never downloads the ISO again, so the 2 GiB GitHub asset limit (#7) only affects the download. Updates on an installed machine need their own design. Defaults and the user's config are already split (#9, `7bb2d5f`), and no update writes to the user's layer. The research behind this ADR, with sources, is in `docs/RESEARCH-updates.md`.

## Decision

**Two kinds of change, two speeds.**

- Ordinary packages (Ubuntu's apt packages, everything from nixpkgs like Neovim, anything the user adds) update whenever the user runs `kubaos update`. No KubaOS release gates them. Ubuntu security updates keep installing automatically through unattended-upgrades; everything else waits for `kubaos update`.
- KubaOS-curated things (Hyprland, Quickshell, ukiyo, home-manager, the cyan Papirus icons, the lock screen's PAM modules, the GPU driver bundle, the `config/` defaults, the scripts, migrations, installer and first-boot bits) move only with a KubaOS release.

**Versions.** `VERSION` and `CODENAME` files at the repo root. A release is a signed annotated tag `vX.Y.Z`, signed with the SSH key already used for commits. Patch releases (`Z`) keep the codename and the NixOS, home-manager and Ubuntu base. Minor or major releases get the next codename.

**Maintenance branches.** `vX.Y.0` is tagged on `main`, and `release/X.Y` branches from it for that version's patches. Fixes land on `main` and are cherry-picked onto each supported branch. Older versions get patches, mostly security fixes, for as long as patching them is worth it, with no promised window. They can never get Nix fixes past their NixOS release's end of life.

**Two locks.** KubaOS's `flake.lock` pins nixpkgs and home-manager for the curated set. The installer writes a user flake to `~/.config/kubaos/home/`, with its own `nixpkgs` input. It takes KubaOS as an input from a root-owned checkout at `/usr/share/kubaos`, on a local `release` branch that only the updater moves, after verifying the tag. There is no `follows`, so curated store paths are identical on every machine on the same release.

**Hyprland** is held and pinned to the exact version each release was tested with (`Pin-Priority: 1001`). It changes only through KubaOS patch releases, and the release attaches the tested `.deb`s in case the PPA drops that version.

**Commands.** `kubaos update` updates packages and applies a newer patch of the installed version. `kubaos upgrade` is the separate, deliberate step to the next codename, one at a time, like Ubuntu's `do-release-upgrade`. `kubaos rollback` and `kubaos channel stable|edge` complete the set. Numbered migrations run once each. On user files they may only add, back up first, and skip settings the user changed.

**Notification.** A daily user timer runs `git ls-remote --tags`. A new patch gets one quiet notification. A new codename gets one notification pointing to the release notes and `kubaos upgrade`. Package updates never notify.

**Everything is in the launcher.** `SUPER + Space` is the one menu, as in Omarchy. It lists apps and KubaOS actions together, including update, upgrade, rollback, channel and wallpaper sets, and typing searches all of them.

**Binary cache.** A public Cachix cache signed with our own key, pushed from the Mac. Its public key is in `/etc/nix/nix.conf` through `extra-substituters` and `extra-trusted-public-keys`. Users are not added to `trusted-users`.

**Hosting.** ISOs stay under 8 GB. Until there is a domain on Cloudflare, they go out as split parts on the GitHub Release; after that, on Cloudflare R2. The GitHub Release stays the index: notes, `SHA256SUMS` and its signature, `release.json`, and the Hyprland `.deb`s.

**Wallpapers** move to their own repository, pinned as a KubaOS input. Every set ships in the ISO, and the installer lets the user pick which sets to use. `~/Pictures/wallpapers/` is the user's own folder.

## Consequences

- `system/update.sh`, `update.sh --hyprland` and running `nix flake update` on KubaOS's own lock on the laptop all go away. Bumping KubaOS's inputs becomes release work on the dev machine.
- The installer no longer commits `home/username`. `~/Projects/dotfiles` isn't needed on an installed machine, so hacking on KubaOS means a separate clone and `--override-input`.
- Once the user's nixpkgs moves ahead of KubaOS's, the two copies duplicate up to about 1.5 GiB of store until the next release lines them up again.
- Building and signing stay on the Mac until CI exists, so releases need Bitwarden unlocked.
- `docs/PLAN.md` rules 1 and 4, `docs/INSTALLER.md` ("no update checker") and `docs/USING.md` need rewording as each piece lands.
