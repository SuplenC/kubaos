# Research: coordinating ISO releases and in-system updates

Question (2026-10-18): the ISOs can't go on GitHub Releases as they are (issue #7). Beyond that, how should KubaOS coordinate the ISO releases with updates that run on an installed machine?

Sources are linked inline. Repo claims cite `path:line` at commit `bb6ddd9`. Omarchy claims cite its source at `basecamp/omarchy@4e059ac` (2026-10-18). Nix claims cite the manual sources in [NixOS/nix](https://github.com/NixOS/nix/tree/2.35.3) at tag `2.35.3`, the version `system/60-nix.sh:24` installs. home-manager claims cite [`release-26.05` at `ab3823b`](https://github.com/nix-community/home-manager/tree/ab3823b4b24c0c3ca984c1f3ee577fd6d667424c). Cachix CLI claims cite [`cachix/cachix@3349ce7`](https://github.com/cachix/cachix/tree/3349ce74ba77f1382814cf611c6f42484956dc41).

**Built on #9.** [#9](https://github.com/SuplenC/kubaos/issues/9) is implemented (`7bb2d5f`). The checkout's `config/` is now read-only defaults, user changes live in `~/.config/kubaos/<app>/`, and no update writes there. "What #9 built" below has the details. The design here relies on it.

## TL;DR

Two kinds of change reach an installed machine, and they move at different speeds:

- **Ordinary packages** (neovim, ripgrep, Ubuntu's apt packages, security fixes) update whenever you run `kubaos update`. No KubaOS release gates them.
- **KubaOS-curated things** (Hyprland, Quickshell, ukiyo, home-manager, the installer and first-boot bits, the `system/` scripts, the defaults in `config/`) move only with a KubaOS release, as a signed git tag.

Concretely:

1. **One version, one tag.** A `VERSION` file replaces the three hardcoded `0.1.0`s. A release is a signed tag `vX.Y.Z`. Patches (`0.1.1`) keep the codename. A minor or major bump gets the next Tokyo neighbourhood.
2. **Split the Nix pin in two.** KubaOS's flake keeps its own `flake.lock`, and that lock pins nixpkgs for the curated packages. The installer writes a small user flake to `~/.config/kubaos/home/`. It has its own `nixpkgs` input, which you update freely, and it takes KubaOS as an input from a root-owned, verified checkout at the installed release. Ordinary packages come from your nixpkgs. Curated ones come from KubaOS's. This also removes the installer's username commit, and `~/Projects/dotfiles` stops being needed on an installed machine.
3. **Split apt the same way.** Ubuntu updates flow as usual. The Hyprland stack is held, and the release carries an exact `Pin: version` file for it. Only a KubaOS patch moves that pin.
4. **`kubaos update` and `kubaos upgrade`.** `update` always updates packages (apt, plus `nix flake update` of your flake), and it also applies a newer *patch* release of your current codename if there is one. `upgrade` is the deliberate step to the next codename, like Ubuntu's `do-release-upgrade` versus `apt upgrade`.
5. **Notification.** A daily user timer runs `git ls-remote --tags`. A new patch gets one quiet notification. A new codename gets one notification that points to the release notes and `kubaos upgrade`. Package updates never notify.
6. **Binary cache now, from the Mac.** A public Cachix cache (free tier, 5 GB) signed with our own key. `just cache-push <arch>` runs after `just payload <arch>` in the same Docker store volume. Its public key goes into `/etc/nix/nix.conf` in the payload. The cache only works because of point 2. Curated packages build against KubaOS's pinned nixpkgs, so every machine asks for the same store paths.
7. **ISOs** go to Cloudflare R2 when hosting happens. The GitHub Release stays the index. Not urgent while the audience is you and a few others.

## Decisions (2026-10-18)

These replace the earlier open questions.

1. **Notifications.** Updates will be rare. A patch release (bug fixes, security) gets a quiet notification. A big release (new codename, minor or major bump) is a separate, deliberate upgrade path: `kubaos update` for patches within the current codename, `kubaos upgrade` for the next one. Patches keep the codename, and the name changes only with bigger releases (decided earlier).
2. **Audience.** You plus a small number of interested people. So: two channels, no `rc` channel, cheap hosting, and migrations that are tested but not built for strangers' machines.
3. **Hyprland.** Each release pins the version it was tested with. Hyprland upgrades ship only in KubaOS patch releases. `update.sh --hyprland` goes away.
4. **CI and ISO hosting.** Planned, not yet. Builds and signing stay on the Mac for now. "When CI comes" lists what changes.
5. **Ordinary packages are not gated by releases.** A new neovim must not wait for a KubaOS release. Only system-level and KubaOS-curated things go through the KubaOS channel. `kubaos update` still runs the normal package updates on demand. This overturns the earlier "the tag's `flake.lock` is the pin for everyone" design. See "Splitting the Nix pin".
6. **Binary cache now, without CI.** See "Binary cache".
7. **Local edits vs upstream** (decided earlier): user changes live in a layer updates never write to. Implemented in #9.
8. **Old series keep getting patches.** After Akihabara ships, Shinjuku still gets patch releases, mostly security fixes. `kubaos upgrade` stays optional. See "Maintenance branches".
9. **No domain on Cloudflare yet.** Until there is one, ISOs go out as split parts on the GitHub Release (or by hand), and the Nix cache stays on Cachix.
10. **Non-security Ubuntu updates wait for `kubaos update`.** unattended-upgrades keeps Ubuntu's security-only origins (`system/10-strip.sh:130-139`), as today.
11. **Wallpapers move to their own repository** and all of them ship in the ISO. The installer lets the user pick which sets to keep. An ISO budget of **8 GB** is fine, so up to about 1 GB of wallpapers is acceptable (x86_64 is 5.3 GB today, issue #6). See "Wallpapers".
12. **Everything is doable from the command menu,** updates included. No command menu exists yet: `SUPER + Space` opens the Quickshell launcher, which lists desktop entries and has a calculator (`config/quickshell/modules/launcher/`). See "Command menu".
13. **The launcher is the menu,** as in Omarchy. `SUPER + Space` opens one list with everything in it, apps and KubaOS actions together, and typing searches all of it. No prefix and no second menu.
14. **Old series have no fixed support window.** They get patches for as long as patching them is worth it to you. No promise is made, so the end-of-life notice is sent whenever you stop.

## Current state in the repo

### Versioning

- The version is hardcoded in three places: `justfile:7` (`ver := "0.1.0"`, never used), `installer/image/build:94` (`VERSION=0.1.0`), and `installer/iso/lib/brand.env:3-6` (`KUBAOS_VERSION`, `KUBAOS_RELEASE_ID=shinjuku`). The codename comes in separately through `installer/image/build:125` (`OS_CODENAME=shinjuku`).
- The repo has no git tags, no GitHub Releases, and no `.github/` workflows. `README.md:7`: "Nothing is released yet." The repo and ukiyo are public on GitHub (`gh repo view`).
- The payload records `version`, `codename`, and `commit` in `manifest.json` (`installer/crates/core/src/probe.rs:59-81`), and the build id `<date>-<commit12>` in `/usr/share/dotinstall/image.json` (`installer/image/README.md:17,35`).
- `/usr/lib/os-release` gets `PRETTY_NAME="KubaOS Shinjuku"` and `VERSION="0.1 (Shinjuku)"` at payload build time (`installer/image/lib/assemble.sh:202-203`). Nothing rewrites it after install, so a new codename (Akihabara is next, `docs/BRAND.md:9`) needs an in-system step.

### The GitHub limit

- Issue #7: "GitHub Releases caps files at 2 GiB and our ISOs are 4–5 GB." GitHub's docs confirm it: "Each file included in a release must be under 2 GiB", up to 1000 assets, and "no limit on the total size of a release, nor bandwidth usage" ([about releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)).
- Measured: x86_64 ISO 5.3 GB (issue #6), aarch64-vm ISO 3.8 GB. The payload squashfs alone is about 3.26 GiB, so shrinking the ISO (issue #6) won't get it under 2 GiB on its own.

### What #9 built (`7bb2d5f`)

- `config/<app>/` is copied into the store once (`kubaos-defaults`) and each file is linked read-only into a real `~/.config/<app>/` (`home/links.nix:27-30,78-90`). An update replaces the defaults whole. Editing `config/` now takes a `home-manager switch`.
- The user layer is `~/.config/kubaos/<app>/`, which every app loads last (`docs/CONTRACTS.md:25-32`, per-app table at `:34-52`). `dot-config apply` runs on every switch and creates missing user files from `user-layer/` (`home/links.nix:119-123`, `scripts/dot-config:1-13`). Files that apps rewrite (btop, tms, lazy.nvim's lock) are seeded once, and three files with no include mechanism are merged (`home/links.nix:32-49`).
- The contract for migrations already exists: a migration may change a user file only if it adds to it, is safe to re-run, backs the file up first, and skips any setting the user changed (`docs/CONTRACTS.md:32`).
- `test/user-layer-check` proves the promise. It fills every user file, commits newer defaults to the checkout, switches, and checks the user's files are byte-identical (`test/user-layer-check:1-17`, `just vm-user-layer`, `justfile:73-75`).
- Still live-linked into `~/Projects/dotfiles`: every script (`home/links.nix:99-101`, `docs/CONTRACTS.md:62`), the wallpapers (`home/theme.nix:130-131`), and the tty1 Hyprland autostart that `~/.zprofile` sources (`home/shell.nix:138-142`). `dotfilesDir` is hardcoded to that path (`home/default.nix:21`).
- Not in the user layer yet: Quickshell settings, matugen templates, oh-my-posh (`docs/CONTRACTS.md:56`).

### How an installed machine is put together

- apt owns the system: kernel, drivers, Hyprland (held), PAM, Docker, Chrome. Standalone home-manager owns `$HOME` (`docs/PLAN.md:9`). system-manager is not used.
- One flake, one lock. `flake.nix` has nixpkgs `nixos-26.05`, home-manager `release-26.05`, Quickshell `v0.3.2`, and ukiyo tracking `main`. home-manager, Quickshell, and ukiyo all `follows = "nixpkgs"` (`flake.nix:4-25`). So ordinary packages and curated ones share one nixpkgs revision.
- The username lives in `home/username`, read by `flake.nix:37`. For a non-default name, the engine writes that file in the installed checkout and commits it as `dotinstall` (`installer/crates/core/src/engine/install.rs:1254-1270`, `docs/INSTALLER.md:115-123`). That local commit would block a clean `git checkout` of a release tag.
- `~/Projects/dotfiles` is a full clone of the payload's commit, "no remote" (`installer/image/README.md:36,141`). A root-owned copy stays at `/usr/share/dotinstall/dotfiles`, and first-boot `setup-helper` runs `system/30-hardware.sh` from it, never from the user's copy (`docs/INSTALLER.md:199`). Nothing updates it after install.
- The payload ships `nix flake archive` of the inputs, so the flake evaluates offline (`installer/image/README.md:26`).
- Subvolumes are flat: `@ @home @snapshots @log @cache @nix …` (`docs/INSTALLER.md:98`). A snapper snapshot of `@` covers `/usr/share/…` and `/etc`, but not `$HOME` or `/nix`.

### What exists for updating

- `system/update.sh` does: snapper snapshot, `apt-get upgrade --with-new-pkgs`, optional Hyprland unhold/upgrade/re-hold, optional `nix flake update` + `home-manager switch` + PAM relinks + recorder helper, and a Bitwarden release check (`system/update.sh:1-8,61-175`). Documented in `docs/USING.md:27-43` and `docs/PLAN.md:199-209`.
- `--nix` runs `nix flake update` on the machine (`system/update.sh:143`), so every machine resolves its own versions. ukiyo tracks a branch (`flake.nix:21-24`).
- The Hyprland stack is installed from `ppa:cppiber/hyprland` and held with `apt-mark hold`, together with every installed `hypr*`/`libaquamarine*` library (`system/50-hyprland.sh:16,78-96`, `system/lib.sh:334-340`). There is no version pin. The only apt pin is `no-snapd` (`system/files/apt/preferences.d/no-snapd`).
- unattended-upgrades is on, with Ubuntu's security-only origins (`system/10-strip.sh:130-139`).
- No update checker and no timer, on purpose (`docs/INSTALLER.md:208`, `docs/USING.md:29`).
- Rollback: snapper snapshots of `@` bootable from GRUB with a manual `@` swap, home-manager generations, and the foot/fuzzel fallback (`docs/USING.md:38-43`).
- No migration mechanism. The `system/*.sh` scripts say they're "safe to re-run" (`system/10-strip.sh:15`, `system/20-base.sh:16`, `system/25-boot.sh:5`).

### How builds run today

All on the Mac, in Docker. `installer/image/build` runs `assemble.sh` in a `linux/arm64` or `linux/amd64` container, and amd64 runs under Rosetta (`installer/image/build:110-127`; the Rosetta seccomp workaround is `installer/image/lib/assemble.sh:49-55`). The Nix store is a Docker volume per arch, `dotinstall-image-nix-<arch>-<nix version>` (`installer/image/build:62-64`). The test harness has its own volume, `dotfiles-nix` (`test/compose.yaml:34,91`). The Mac has no Nix of its own. Docker is already the Linux builder, so nixpkgs' `darwin.linux-builder` ([manual](https://nixos.org/manual/nixpkgs/stable/#sec-darwin-builder)) adds nothing here.

## Getting around the 2 GiB limit

None of this matters to in-system updates, which never download the image. The updater and the notifier read the release's metadata (version, checksums, signature, notes).

| Option | Limits and cost | Notes for KubaOS |
|---|---|---|
| Split into <2 GiB parts as release assets | 2 GiB per asset, 1000 assets, no bandwidth limit ([GitHub](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)) | No new accounts. The user has to `cat` the parts together before `dd`. Fine for a handful of people who can follow a README. |
| Cloudflare R2 | $0.015/GB-month, 10 GB-month free, 1M Class A and 10M Class B requests free, egress free ([pricing](https://developers.cloudflare.com/r2/pricing/)); 5 TiB max object ([limits](https://developers.cloudflare.com/r2/platform/limits/)). Public `r2.dev` URLs are "rate-limited and should only be used for development purposes"; production needs a custom domain added as a zone in the same Cloudflare account ([public buckets](https://developers.cloudflare.com/r2/buckets/public-buckets/)) | About 20 GB per release for four ISOs, so roughly $0.15/month per release kept beyond the free tier. Needs a domain on Cloudflare. The plan once hosting happens. |
| Backblaze B2 | $6.95/TB-month, first 10 GB free, free egress up to 3x stored ([pricing](https://www.backblaze.com/cloud-storage/pricing)) | Fine too. Metered egress beyond 3x. |
| GHCR (`oras push`) | "10 GB size limit for each layer", "10 minute timeout limit for uploads" ([container registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)) | 5 GB in 10 minutes needs about 8.5 MB/s sustained. Risky from home. Downloaders need `oras`. |
| Git LFS | 2 GB max file on Free and Pro ([GitHub](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-git-large-file-storage)) | Same wall. Ruled out. |

Whichever host serves the bytes, the GitHub Release stays the index. `GET /repos/{owner}/{repo}/releases/latest` returns "the most recent non-prerelease, non-draft release" ([REST](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)). A small `release.json` asset (version, codename, commit, ISO URLs, sha256s, Hyprland versions) gives the README, the notifier, and the updater one stable URL. Small assets like the Hyprland `.deb`s (see "Hyprland") fit on the Release itself.

## In-system update mechanisms

### Nix side

- **`nix flake update` locally (today).** "By default, all inputs are updated" (`src/nix/flake-update.md:60`). Fine for a dev box. As a release mechanism it moves the curated packages too, because they `follows` the same nixpkgs (`flake.nix:15,23`).
- **Follow a release's lock file.** That's what NixOS's `system.autoUpgrade` does with a flake. It honours the lock unless you add `--update-input` flags ([auto-upgrade.nix](https://github.com/NixOS/nixpkgs/blob/nixos-26.05/nixos/modules/tasks/auto-upgrade.nix), lines 42-92, 218-227). It also has the timer knobs `dates`, `randomizedDelaySec`, `persistent` (same file, 93-185). Right for curated packages, wrong for ordinary ones after decision 5.
- **How lock files compose.** "Lock files transitively lock direct as well as indirect dependencies. That is, if a lock file exists and is up to date, Nix will not look at the lock files of dependencies. However, lock file generation itself *does* use the lock files of dependencies by default" (`src/nix/flake.md:760-764`). So when a user flake takes KubaOS as an input without `follows`, a fresh lock copies KubaOS's own nixpkgs revision from KubaOS's `flake.lock`. Re-running `nix flake update` copies it again. It can't drift while the KubaOS input stays put.
- **`follows`.** "The value of the `follows` attribute is a `/`-separated sequence of input names denoting the path of inputs to be followed from the root flake" (`src/nix/flake.md:571-576`). `inputs.kubaos.inputs.nixpkgs.follows = "nixpkgs"` would rebuild every curated package against the user's nixpkgs. That means different store paths on every machine, no cache hits, and a Quickshell build nobody tested. The manual notes that eliminating a transitive nixpkgs is "generally not useful", because most flakes compose into the top-level nixpkgs through overlays or modules (`src/nix/flake.md:599-610`). KubaOS is the exception on purpose: its `packages` output is built against its own nixpkgs.
- **home-manager and two nixpkgs.** `homeManagerConfiguration` takes `pkgs` from the caller, and `lib` defaults to `pkgs.lib` (`lib/default.nix:5-11`). home-manager's own nixpkgs input plays no part in a standalone config. It warns when `home.version.release`, `lib.trivial.release`, and `pkgs.lib.trivial.release` differ (`modules/home-environment.nix:583-615`, option at `:549-562`). It compares release strings like `26.05`, not revisions. A user nixpkgs that tracks `nixos-26.05` at any revision raises no warning against home-manager `release-26.05`. Pointing it at `nixos-unstable` does, and should.
- **home-manager rollback.** `home-manager switch --rollback` "will turn the previous configuration into the current configuration" ([manual](https://nix-community.github.io/home-manager/usage/rollbacks.html)).
- **system-manager (numtide)** could replace parts of `system/*.sh` with Nix ([README](https://github.com/numtide/system-manager)). That goes against PLAN rule 3 (`docs/PLAN.md:9`). Not now.

### Ubuntu side

- unattended-upgrades picks origins through `Unattended-Upgrade::Allowed-Origins` and skips packages in `Package-Blacklist` ([README](https://github.com/mvo5/unattended-upgrades/blob/master/README.md), lines 28-46, 149-166). KubaOS keeps Ubuntu's security-only default (`system/10-strip.sh:130-137`). Everything else in Ubuntu's archive arrives when you run `kubaos update`. No KubaOS release is involved.
- **apt_preferences(5).** A record with `Pin: version <v>` assigns a priority to those versions of the named packages. `P >= 1000` "causes a version to be installed even if this constitutes a downgrade of the package", and `990 <= P < 1000` installs it "unless the installed version is more recent" ([apt_preferences.5.xml](https://salsa.debian.org/apt-team/apt/-/blob/main/doc/apt_preferences.5.xml), the "specific form" and "How APT Interprets Priorities" sections; [Debian manpage](https://manpages.debian.org/unstable/apt/apt_preferences.5.en.html)). One record can name several packages, but each package in the Hyprland stack has its own version string, so it's one record per package.
- **A pin alone isn't enough with a PPA.** Launchpad keeps a published package "until … a later upload supersedes it" ([Launchpad PPA reference](https://ubuntu.com/docs/launchpad/user/reference/packaging/ppas/ppa/)). Once the PPA moves on, a `Pin: version` record matches nothing. apt then falls back to the newest version at the default priority 500 and would upgrade. The `apt-mark hold` stays as the guard, and the release keeps its own copy of the tested `.deb`s (see "Hyprland").
- An own apt repo (a `kubaos-*` meta-package) is how Omarchy ships itself now (below). It needs repo hosting and signing, and it duplicates what the git checkout and migrations already do. Not worth it at this audience size.

### Image-based (bootc, ABRoot, ChromeOS)

These replace the whole root at once. bootc queues an image and applies it at shutdown, and `--check` fetches only "typically kilobyte-sized metadata" ([bootc-upgrade(8)](https://bootc.dev/bootc/man/bootc-upgrade.8.html)). ABRoot alternates two roots ([ABRoot](https://github.com/Vanilla-OS/ABRoot)). ChromeOS writes the inactive partition and falls back after failed boots ([design doc](https://www.chromium.org/chromium-os/chromiumos-design-docs/filesystem-autoupdate/)). KubaOS is a mutable Ubuntu with DKMS drivers signed by a per-machine MOK and a 3.3 GiB payload per architecture. Not recommended. The idea worth taking is the small, separate "is there an update" check.

## Prior art

### Omarchy

Read from source (`basecamp/omarchy@4e059ac`):

- **Distribution.** A pacman package installed to `/usr/share/omarchy`. `stable` and `rc` install `omarchy`, `edge` and `dev` install `omarchy-dev` (`bin/omarchy-channel-set:110-132`). `dev` runs a git checkout with `git pull --ff-only` (`bin/omarchy-channel-set:142-161`, `bin/omarchy-update-dev:14-27`).
- **Update order** (`bin/omarchy-update:99-177`): free-space check, confirm, one sudo prompt kept alive, snapshot, system packages, **then** migrations, then service restarts and hooks, and a reboot offer. Migrations come after packages because they "ship with the packages installed here and are written against them" (124-126).
- **Migrations** (`bin/omarchy-migrate`): `migrations/<unix-timestamp>.sh` in filename order, a marker file per success in `~/.local/state/omarchy/migrations/` (31-33, 88-97), system markers under `/var/lib/omarchy/migrations/` (`migrations/1786380259.sh:3`), `bash -euo pipefail`, first failure stops the run. A fresh install marks every shipped migration done (`install/user/first-run/audio-tuning.sh:5-7`).
- **Notification.** `omarchy-migrate-notify` notifies about pending migrations after login. `omarchy-update-available` fetches with a 10 s timeout and counts commits behind (9-21).

Its migrations are the part to copy almost verbatim.

### Universal Blue and Bazzite

Channels are image tags (`stable`, `testing`, `unstable`), with dated tags kept 90 days and `rebase` to move between them ([rebase guide](https://docs.bazzite.gg/Installing_and_Managing_Software/Updates_Rollbacks_and_Rebasing/rebase_guide/)). `uupd` runs system, Flatpak, distrobox, and brew updates from one timer ([uupd](https://github.com/ublue-os/uupd)). Two lessons. The ISO is only a bootstrap, and the machine then follows a channel. And the OS image and the user's apps (Flatpak, brew) update on separate tracks. Decision 5 asks for that second split.

### Ubuntu's `apt upgrade` vs `do-release-upgrade`

`apt upgrade` moves packages within a release. `do-release-upgrade` is "Upgrade the operating system to the latest release from the command-line" ([do-release-upgrade(8)](https://manpages.ubuntu.com/manpages/noble/man8/do-release-upgrade.8.html)). Its config `Prompt=normal` checks for a new release and upgrades only to "the supported release that immediately succeeds the currently-running release" ([release-upgrades](https://git.launchpad.net/ubuntu-release-upgrader/plain/data/release-upgrades)). The release list is a small text file, one stanza per release with `Dist`, `Version`, `Supported`, and a signed upgrade tool ([meta-release-lts](https://changelogs.ubuntu.com/meta-release-lts)). `kubaos update` and `kubaos upgrade` copy that split, one codename at a time.

## Proposed design

### What moves with what

| Moves whenever you run `kubaos update` | Moves only with a KubaOS release |
|---|---|
| Ubuntu packages and kernels from Ubuntu's archive (`apt upgrade`), security updates daily (unattended-upgrades) | The Hyprland stack (exact versions, held) |
| Your nixpkgs: every ordinary package in `home/*.nix` (neovim, ripgrep, ghostty, fonts …) and anything you add in your own `home.nix` | Quickshell, ukiyo, the curated Nix packages listed below, home-manager itself |
| | `config/` defaults, `scripts/`, `system/` scripts, migrations, the installer and first-boot bits, kernel flavour and boot choices (`system/25-boot.sh`), `/etc/nix/nix.conf` lines |

**The curated Nix set** is whatever is built from source outside nixpkgs, or loaded into a curated process:

- Quickshell with its Qt modules (`home/desktop.nix:10-17`). Its Qt has to match every Qt plugin it loads, so the `qt` module's platform theme packages (qt6ct, `home/theme.nix:99-104`) come from the same set.
- ukiyo and its zsh hook (`home/ukiyo.nix:21`, `home/shell.nix:27-32`).
- nixpkgs' Hyprland on the aarch64 VM (`home/desktop.nix:50`).
- The PAM modules the Quickshell lock screen loads, which come from nixpkgs because Quickshell links Nix's libpam (`system/50-hyprland.sh:145-147`, built at `system/lib.sh:308`).
- gpu-screen-recorder, whose KMS helper gets file capabilities (`system/update.sh:157-162`).
- The GPU driver bundle (`targets.genericLinux.gpu.packages`, default `pkgs`, `modules/targets/generic-linux/gpu/default.nix:27-32`). Taking Mesa from the curated set is the safe direction. A user app built against a newer glibc can load an older Mesa, but not the other way round, and the user's nixpkgs is never older than KubaOS's except right after a rollback.
- The cyan Papirus override (`home/theme.nix:20`). It's an override, so cache.nixos.org doesn't have it, and at 139 MiB it's the biggest local build in the generation (measured below).

### Splitting the Nix pin

Three ways to do it:

| Option | How | Problem |
|---|---|---|
| A. One flake, user runs `nix flake update nixpkgs` | as today, minus the other inputs | The lock lives in KubaOS's tree. Every release checkout has to merge the user's lock. `follows` (`flake.nix:15,23`) drags Quickshell and ukiyo along, so they're untested and miss the cache. The username commit stays. |
| B. One flake, a second `nixpkgs-user` input and an overlay ([overlays](https://nixos.org/manual/nixpkgs/stable/#chap-overlays)) | curated packages from `nixpkgs`, the rest from `nixpkgs-user` | Technically fine. Updating `nixpkgs-user` still edits KubaOS's `flake.lock` in KubaOS's checkout, so the ownership problem stays. |
| **C. A user flake that imports KubaOS** | `~/.config/kubaos/home/flake.nix` has its own `nixpkgs` and takes KubaOS as an input, with no `follows` | Two nixpkgs evaluations and, once they diverge, some duplicate closure (measured below). One line has to move on `kubaos upgrade`. |

**Recommended: C.** The lock files then match who owns what. KubaOS's `flake.lock` pins the curated set. The user's `flake.lock` pins the user's nixpkgs and records which KubaOS release is installed.

What the user gets (the installer writes it once, then it's theirs):

```nix
# ~/.config/kubaos/home/flake.nix
{
  inputs = {
    # Your packages. `kubaos update` runs `nix flake update` here.
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    # KubaOS: the root-owned, verified checkout, at the installed release.
    kubaos.url = "git+file:///usr/share/kubaos?ref=refs/heads/release";
  };
  outputs = { nixpkgs, kubaos, ... }: {
    homeConfigurations."alice@tuxedo" = kubaos.lib.mkHome {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      username = "alice";
      modules = [ ./home.nix ];   # your own packages and options
    };
  };
}
```

What KubaOS's flake exports:

```nix
lib.mkHome = { pkgs, username, modules ? [ ] }:
  home-manager.lib.homeManagerConfiguration {
    inherit pkgs;                      # the user's nixpkgs: ordinary packages
    extraSpecialArgs = {
      inherit username;
      kubaosPkgs = self.packages.${pkgs.stdenv.hostPlatform.system};  # the curated set
    };
    modules = [ self.homeModules.default ] ++ modules;
  };
packages.<system> = { quickshell, ukiyo, papirus-cyan, pam-fprintd, … };  # built from KubaOS's nixpkgs
homeModules.default = ./home;
```

How it behaves:

- **KubaOS's input points at a local branch, not a tag.** `release` is a local branch in the root-owned checkout. The updater moves it only after verifying a tag and running the system migrations. The user's `flake.nix` never changes for a patch. `nix flake update` (all inputs, by hand or from `kubaos update`) moves your nixpkgs to the branch head and re-reads `release`, which is the installed release. A plain `nix flake update` therefore can't pull in a release that skipped its migrations. Nix's git fetcher refuses a repo owned by another user unless it's in `safe.directory` (`installer/image/lib/assemble.sh:69-74`), so the payload adds `/usr/share/kubaos` to `/etc/gitconfig`.
- **Curated packages can't drift.** With no `follows`, KubaOS's transitive nixpkgs comes from KubaOS's own lock every time the user's lock is generated (`src/nix/flake.md:760-764`). Store paths are identical on every machine on the same release, so the binary cache hits.
- **home-manager is curated.** The KubaOS module is written against home-manager `release-26.05` options, so `mkHome` uses KubaOS's pinned home-manager. As long as the user's nixpkgs tracks `nixos-26.05`, the release check stays quiet (`modules/home-environment.nix:585-588`). `kubaos upgrade` moves both together (below).
- **Your own packages, for free.** `home.nix` next to the flake is the user layer for Nix. Add `home.packages = [ pkgs.foo ];` there. Nothing could do this before #9's layer existed.
- **Evaluation cost.** Measured in the test container on the aarch64 Docker volume, at `bb6ddd9`, `nix eval` of `homeConfigurations."kuba@vm".activationPackage.drvPath` with `NIX_SHOW_STATS`. With one nixpkgs instance: 2.2 s CPU, 496 MiB heap, 11.23M values. With home-manager's `pkgs` from a second `import nixpkgs` and the curated packages from `legacyPackages`: 2.2 s, 544 MiB, 11.59M values. Both instances were at the same revision and produced the same `.drv`. Nix evaluates lazily, so the second instance only evaluates what the curated packages reach. Expect about +50 MiB of heap and no noticeable time.
- **Closure cost.** Once your nixpkgs moves past KubaOS's, the curated set keeps its own glibc, Qt, and Mesa. On the aarch64 payload the closures are 1.4 GiB for `quickshell-wrapped`, 104 MiB for `ukiyo`, and 4.8 GiB for the whole generation (`nix path-info -S`), and much of that is shared. Worst case is about 1.5 GiB extra until the next KubaOS release re-aligns. `nix-collect-garbage` after old generations expire drops it.
- **The username commit goes away.** The username is a literal in the user's `flake.nix`, which the engine writes. KubaOS's checkout has no `home/username` (`flake.nix:37`, `docs/CONTRACTS.md:10`). The engine's "Another username" step writes the file from a template instead of committing (`install.rs:1254-1270`). It still builds the same ~50 user-specific derivations offline (`docs/INSTALLER.md:123`). The template's `flake.lock` is generated at payload build with the user's `nixpkgs` node set to KubaOS's locked revision and original `nixos-26.05`. On day one there's one nixpkgs. They only diverge when the user updates.
- **`~/Projects/dotfiles` is no longer needed on an installed machine.** Scripts link from the KubaOS source in the store, like the defaults already do (`home/links.nix:27-30`). The same goes for the wallpapers (29 MB) and the tty1 autostart snippet. `system/` scripts and migrations run from the root checkout. `--inputs-from` for the PAM modules points there too (`system/lib.sh:308`). Hacking on KubaOS becomes a dev-machine thing: clone anywhere and run `home-manager switch --flake ~/.config/kubaos/home --override-input kubaos git+file://$HOME/Projects/kubaos`. Two things need a new home: `~/Pictures/wallpapers` becomes a real directory with KubaOS's folders linked from the store, so your own images go in a folder of your own next to them. And the Quickshell README's "run the checkout" tip (`config/quickshell/README.md:38`) becomes a dev-clone tip.
- **Flakes and git.** `~/.config/kubaos/home` is a plain directory, so Nix copies all of it as a `path:` flake. If you put `~/.config/kubaos` under git, new files there need `git add`, the same rule as today (`docs/CONTRACTS.md:12`).

**Contract change.** "Updates never write to a file in the user layer" (`docs/CONTRACTS.md:30`) stays true for patches. `nix flake update` writes the user's `flake.lock` when the user runs `kubaos update`. That's Nix acting on the user's command, the same as running it by hand. `kubaos upgrade` runs one user migration that edits the `nixpkgs.url` line under the existing migration rule: back up first, change only that line, and skip it if the user changed the branch.

### Version and release identity

- `VERSION` at the repo root holds `0.1.0`, plus a `CODENAME` file with `shinjuku`. `justfile`, `installer/image/build`, and `installer/iso/lib/brand.env` read them.
- Semantics: `X.Y.Z`. Z bumps are **patches**: same codename, same nixpkgs and home-manager release branch, same Ubuntu base. Y or X bumps are **big releases**: next codename in `docs/BRAND.md:9`. They may move the nixpkgs and home-manager branch or, rarely, the Ubuntu LTS.
- A release is a **signed annotated tag** `vX.Y.Z`, on `main` for a new `X.Y` and on the series' maintenance branch for its patches, signed with the Bitwarden SSH key you already use for commits (`config/git/config:6-11`, `scripts/dot-bw-setup:364-366`). Git checks SSH signatures against `gpg.ssh.allowedSignersFile` ([git-config gpg.ssh](https://github.com/git/git/blob/master/Documentation/config/gpg.adoc)).
- `just release` refuses to build unless `HEAD` is exactly `v$(cat VERSION)`, the tag verifies, and the tree is clean. It writes `SHA256SUMS` and signs it with `ssh-keygen -Y sign -n kubaos-release` ([ssh-keygen(1)](https://man.openbsd.org/ssh-keygen)).
- Pin ukiyo to a tag or rev in `flake.nix` before the first release.
- Pre-releases are `vX.Y.Z-rc.N`, published as GitHub pre-releases. The stable channel ignores them.

### Maintenance branches

Decision 8 means more than one series can be live at once.

- `vX.Y.0` is tagged on `main`. At the same commit, a branch `release/X.Y` starts (Conventional Branch allows dots in `release/` names, `CLAUDE.md`). Every later `vX.Y.Z` is tagged on that branch.
- A fix lands on `main` first, then is cherry-picked to every supported `release/X.Y` (`git cherry-pick -x`, so the commit records where it came from). Fixes only `main` needs stay there.
- Patch releases on an old series move only within that series: same nixpkgs and home-manager branch, same Ubuntu base, Hyprland only if a security fix requires it. KubaOS's own `flake.lock` on `release/X.Y` gets `nix flake update nixpkgs home-manager` within the same branch, which is how Nix security fixes reach the curated packages.
- `update`'s target rule already fits: the highest verified `vX.Y.Z'` with the installed `X.Y` (step 2 below). The notifier keeps sending patch notices for the installed series. The upgrade notice still fires once per new codename.
- **No fixed support window** (decision 14). A series gets patches while it's worth it to you, and never past the upstream limit below.
- **The upstream limit.** A series can only get Nix fixes while its nixpkgs and home-manager release branch does, and Ubuntu fixes while its Ubuntu release does. Each NixOS release is maintained for a limited time after the next one ships; check the exact window on [nixos.org](https://nixos.org/manual/nixos/stable/release-notes) before announcing one. When the branch goes end-of-life, the last patch of the series says so and the notifier switches to "this series no longer gets fixes: kubaos upgrade".
- `just vm-upgrade` runs per supported series, so each `release/X.Y` gets tested on its own.

### Wallpapers

- A separate public repo (for example `SuplenC/kubaos-wallpapers`), one folder per set (today's `wallpapers/cyberpunk`, `japanese`, `shinjuku`), each with its credits. KubaOS takes it as a flake input pinned in KubaOS's own lock, so new wallpapers arrive with a release like any other curated thing.
- The payload ships every set read-only, for example under `/usr/share/kubaos/wallpapers/<set>/`, so the ISO works offline.
- The installer gets a wallpaper step: a grid of sets with a preview each, all selected by default. The engine records the choice in the user layer (`~/.config/kubaos/wallpapers/sets`). The picker and `dot-theme`'s folder cycle show the chosen sets plus `~/Pictures/wallpapers/`, which is a real, user-owned directory for the user's own images. That settles where your own wallpapers go.
- Updates follow the user-layer rule: a release can add a new set to `/usr/share`, but it only shows up once it's chosen in settings (or a migration appends it to `sets`, if you want new sets to appear by default). Removing a set from the repo needs care, because someone may have it as their current wallpaper.
- Unselected sets stay on disk in `/usr/share`. Deleting them would save at most about 1 GB, and they'd come back with the next update anyway.

### Command menu

Decision 12 means every action in this design needs an entry in a command menu, not only a terminal command:

- `Update KubaOS` → `kubaos update` in a terminal window, so prompts and apt output stay visible. Shows the pending patch version when the notifier has found one.
- `Upgrade to <codename>` → `kubaos upgrade`, only listed when one is available.
- `Roll back last update` → `kubaos rollback`.
- `Switch channel` → `kubaos channel stable|edge`.
- `Choose wallpaper sets`.

The menu is the Quickshell launcher (decision 13). With an empty query it lists the KubaOS actions and the apps; typing searches both with the existing fuzzy matcher (`config/quickshell/modules/launcher/fuzzy.js`). Commands should come from one list (for example `.desktop` files with a `KubaOS` category, or a small JSON registry under the defaults with user-layer additions) so new features add an entry instead of editing QML. The notifier's bar marker opens the same entry.

### Distribution

- The repo on GitHub is the update source. Git fetches it, and it's small.
- The GitHub Release `vX.Y.Z` holds the notes, `SHA256SUMS`, `SHA256SUMS.sig`, `release.json`, and the Hyprland `.deb`s for x86_64.
- The ISOs go to R2 once hosting happens (`kubaos-releases/vX.Y.Z/…`). Until then, split parts on the Release or a direct hand-off work for a few people.
- The Nix cache is on Cachix for now (see "Binary cache").

### On the machine

- `/usr/share/kubaos`: a root-owned git clone with `origin` set to `https://github.com/SuplenC/kubaos.git`. It replaces `/usr/share/dotinstall/dotfiles`. `setup-helper` runs `system/30-hardware.sh` from it (`docs/INSTALLER.md:199`). Local branch `release` points at the installed tag.
- `/usr/share/kubaos-keys/allowed_signers`, root-owned, shipped in the payload. Not the user's `~/.config/git-bitwarden/allowed_signers`, which anything running as the user can edit.
- `/var/lib/kubaos/release` (version, codename, commit, channel, date), `/var/lib/kubaos/update-history`, `/var/lib/kubaos/migrations/`.
- `~/.config/kubaos/home/{flake.nix,flake.lock,home.nix}`, `~/.local/state/kubaos/{migrations/,locks/,notified}`.

### Channels

| Channel | `release` branch follows | For |
|---|---|---|
| `stable` (default) | Highest verified `v*` tag that isn't a pre-release, within the rules of `update`/`upgrade` below | Everyone |
| `edge` | `origin/main`, signed-commit check only | Your VM, testing a release before tagging |

Stored in `/etc/kubaos/channel`. `kubaos channel edge|stable`. Going back from `edge` to `stable` waits for the next tag at or above the current commit. No `rc` channel at this audience size. To try a candidate, run `kubaos update --to v0.2.0-rc.1`.

### What `kubaos update` does

`kubaos` lives in `scripts/` and replaces `system/update.sh`. `kubaos update` always updates packages. It also applies a release if a newer **patch** of the installed `X.Y` exists. It never changes the codename.

1. **Lock and checks.** `flock`, free-space check, network check. Load the Nix environment like `system/update.sh:136-141`.
2. **Look for a patch.** As root: `git -C /usr/share/kubaos fetch --tags origin`. The target is the highest `vX.Y.Z'` with the installed `X.Y` and a higher `Z'` that passes `git verify-tag` against the root-owned `allowed_signers`. If a higher `X.Y` exists, print one line, "KubaOS Akihabara (0.2.0) is available: kubaos upgrade", and go on.
3. **Show and confirm.** The patch's tag message and `git log --oneline` since the installed tag (if there is a patch), the number of upgradable apt packages, and whether the Hyprland pin changes. `-y` skips the prompt. `--packages-only` skips steps 6, 7, and 10.
4. **Snapshot.** `snapper create --description "before kubaos update <date> (vA → vB)"`, as `system/update.sh:61-66` does. Copy `~/.config/kubaos/home/flake.lock` to `~/.local/state/kubaos/locks/<date>.lock`, because `@home` isn't in the snapshot. Append the current release and lock path to `/var/lib/kubaos/update-history`.
5. **apt.** `apt-get update && apt-get upgrade --with-new-pkgs`, as now (`system/update.sh:69-79`). The Hyprland stack stays held.
6. **Move the release** (patch only). In `/usr/share/kubaos`: `git checkout --detach vX.Y.Z'`, `git branch -f release vX.Y.Z'`. No merge and no conflict. Nobody edits this tree.
7. **System changes** (patch only). Apply the release's Hyprland pin if it changed (see "Hyprland"). Run `migrations/system/*.sh` from the root checkout that have no marker in `/var/lib/kubaos/migrations/`. The first failure stops the run.
8. **Nix.** `nix flake update --flake ~/.config/kubaos/home`. That moves your nixpkgs and re-reads `release`. Then `home-manager switch -b hm-backup --flake ~/.config/kubaos/home`, the PAM relinks, and the recorder helper (`system/update.sh:147-162`), with `--inputs-from /usr/share/kubaos`.
9. **Bitwarden check**, as `system/update.sh:173-175`.
10. **User migrations** (patch only): `migrations/user/*.sh`, markers in `~/.local/state/kubaos/migrations/`, under the rules in `docs/CONTRACTS.md:32`.
11. **Record and finish.** Write `/var/lib/kubaos/release`, report `reboot-required` (`system/update.sh:179-181`), and clear the notifier state for the installed version.

Ordering follows Omarchy: packages first, then migrations, which are written against the new packages.

### What `kubaos upgrade` does

The deliberate step to the next codename, one at a time, like `Prompt=normal`.

1. Refuse unless the installed release is the newest patch of its `X.Y`. The message says to run `kubaos update` first. This keeps the migration chain linear.
2. The target is the lowest verified `X.Y` above the installed one, and its newest patch.
3. Show the release notes and a summary from the target's `release.json`: new codename, nixpkgs and home-manager branch, Hyprland version, Ubuntu base, and the list of defaults that changed. Always confirm, even with `-y`.
4. Steps 4–11 of `update`, plus:
   - a user migration that moves the user flake's `nixpkgs.url` to the release's branch, for example `nixos-26.05` → `nixos-26.11`. It skips the change and warns if the user picked another branch. home-manager's release check (`modules/home-environment.nix:583-615`) will then say so on every switch.
   - a system migration that rewrites `/usr/lib/os-release` from the template in `assemble.sh:195-210` and runs `update-grub`, so the boot menu shows the new name.
   - only when the release moves the Ubuntu LTS: `do-release-upgrade` as its own confirmed step, before the KubaOS migrations. Rare, every two years at most.

### Migrations

- One script per change, `<unix timestamp>-<slug>.sh`, under `migrations/system/` and `migrations/user/`. `just migration system|user <slug>` creates one.
- Each runs under `bash -euo pipefail` and must be safe to run twice. A system migration is often "re-run `system/20-base.sh --only X`".
- **Fresh installs.** The payload build lists the migrations at the tag, and the engine writes markers for all of them. Otherwise the first update would replay every migration ever written. Omarchy does the same.
- **Testing.** `just vm-upgrade <from-tag>` installs the old tag's bench ISO, fills every user-layer file, adds a package to `home.nix`, runs `kubaos update` (patch) or `kubaos upgrade` to `HEAD` on the `edge` channel, then runs the `vm-install` checks plus `test/user-layer-check`. That test now takes "newer defaults" from a scratch KubaOS clone and `--override-input`, instead of a commit into the installed checkout (`test/user-layer-check:9-11`).

### Rollback

From smallest to largest:

1. **User environment.** `home-manager switch --rollback`. That covers a bad package update too.
2. **Release.** `kubaos rollback` reads the last `update-history` entry. As root it runs `git branch -f release <previous>` and the checkout. It restores the saved `flake.lock` and switches. The user layer isn't touched. Migration markers stay, because migrations go forward only.
3. **System.** Boot the "before kubaos update …" snapshot from GRUB and swap `@` by hand (`docs/USING.md:42`). The snapshot includes `/usr/share/kubaos`, `/etc`, and the Hyprland pin. `/nix` and `$HOME` aren't in it, so pair it with step 2's lock restore.

### Notification

PLAN rule 1 (`docs/PLAN.md:7`) stays true. Nothing installs by surprise. A notification only tells you.

- A systemd user timer, daily, `Persistent=true`, with a random delay. It runs `git ls-remote --tags https://github.com/SuplenC/kubaos 'v*'`. That needs no root, no REST quota, and no clone.
- It compares the tags with `/var/lib/kubaos/release` and skips pre-releases and unsigned tags. Signatures are checked at update time, so a check here would only help the display. A newer patch of the installed `X.Y` counts as a **patch**. A higher `X.Y` counts as an **upgrade**. For the display name it fetches `release.json` of the candidate only.
- **Patch:** one low-urgency notification, "KubaOS 0.1.2 is ready: kubaos update". **Upgrade:** one notification, "KubaOS Akihabara is out. Release notes · kubaos upgrade". Each fires once per version, recorded in `~/.local/state/kubaos/notified`. The bar shows a small marker until it's applied. Clicking it opens a terminal with the right command.
- Package updates (apt, nixpkgs) never notify. They're continuous, security fixes install themselves, and the rest waits for your next `kubaos update`.
- `docs/INSTALLER.md:208` ("no update checker or timer") and `docs/USING.md:29` need rewording to "a daily check that only tells you".

### Hyprland

- **At release.** `just release` writes `system/files/apt/preferences.d/kubaos-hyprland` from the payload's `packages.txt` (`installer/image/lib/check-inside.sh:225`). It has one record per package matching `HYPR_HOLD_RE` (`system/lib.sh:334`):

  ```
  Package: hyprland
  Pin: version 0.56.1-1~resolute1
  Pin-Priority: 1001
  ```

  1001 makes apt pick that exact version, even as a downgrade (apt_preferences(5), "P >= 1000"). It also attaches those `.deb`s (x86_64; the PPA has no arm64 builds, `system/50-hyprland.sh:56-59`) and their sha256s in `SHA256SUMS` to the GitHub Release.
- **The hold stays.** The pin says which version. The hold stops apt when the pinned version has left the PPA (see "A pin alone isn't enough"). Unattended-upgrades won't touch held packages (`system/10-strip.sh:139`).
- **On `kubaos update`** (step 7), when the release's pin file differs from `/etc/apt/preferences.d/kubaos-hyprland`: install it, `apt-mark unhold`, then `apt-get install --no-install-recommends hyprland=<v> …` with every version spelled out. If the PPA no longer has a version, download that `.deb` from the Release, check it against the verified `SHA256SUMS`, and `apt-get install ./…deb`. Then re-hold, including newly pulled `hypr*` libraries. Keep the trap and `Hyprland --verify-config` from `system/update.sh:107-127`.
- **aarch64 VM.** Hyprland comes from KubaOS's nixpkgs in the curated set (`home/desktop.nix:44-50`), so it moves with the release lock the same way.
- `--hyprland` and PLAN's manual "unhold, upgrade, test, re-hold" (`docs/PLAN.md:205`) go away for users. On your dev machine, you test a new Hyprland by bumping the pin file and running `system/50-hyprland.sh`.

### Binary cache

Today a nixpkgs bump compiles ukiyo (Rust) and Quickshell (C++) on the laptop, and recolours Papirus. What a cache would have to serve, measured on the aarch64 payload's generation: 869 store paths, of which 85 aren't on cache.nixos.org (`HEAD /<hash>.narinfo` per path). Their NAR sizes add up to 188 MiB uncompressed: Papirus cyan 139 MiB, ukiyo 23 MiB, `quickshell-wrapped` 12 MiB, cpptrace 0.7 MiB, and the rest is home-manager's own small files.

**Why it depends on decision 5.** A cache serves exact store paths. Today every `nix flake update` on the laptop gives Quickshell and ukiyo new paths, because they follow its nixpkgs (`flake.nix:15,23`). With the split, they build against KubaOS's locked nixpkgs, so every machine on a release asks for the paths you pushed. Ordinary packages come from the user's nixpkgs and are on cache.nixos.org already. home-manager's per-user files are tiny and build locally either way.

| Option | Cost and limits | Fit |
|---|---|---|
| **Cachix** | Free for open source: 5 GB. LRU entries evicted at the limit. "Entries from cache.nixos.org are not stored by default" ([pricing](https://www.cachix.org/pricing)). Cachix signs by default, or you opt in to your own key with `cachix generate-keypair` ([getting started](https://docs.cachix.org/getting-started)). `cachix push` computes the closure and asks the server which paths are missing (`cachix/src/Cachix/Client/Push.hs`, `getMissingPathsForClosure`). Auth and key come from `$CACHIX_AUTH_TOKEN` and `$CACHIX_SIGNING_KEY` (`Command/Push.hs:195,215`) | 188 MiB per arch per release fits about 13 arch-releases before LRU eviction starts. No domain, no server. **Use this now.** |
| R2 + `nix copy` | Same R2 pricing as above. `nix copy --to 's3://bucket?…&endpoint=…'` (`src/libstore/s3-binary-cache-store.md:98-103`), signing with the store's `secret-key` setting (`binary-cache-store.hh:50`). Public reads should go through the HTTPS URL as an HTTP cache (`s3-binary-cache-store.md:11-15`) | `nix copy` uploads the whole closure, about 5 GB per arch, unless you filter out cache.nixos.org's paths yourself. It also needs a custom domain, because `r2.dev` is development-only. Revisit when the ISOs move to R2 and the domain exists. |
| Attic | Self-hosted server on S3-compatible storage. Signs on the server, LRU garbage collection, and calls itself "still an early prototype" ([docs](https://docs.attic.rs/)) | A server to run for a few users. No. |

**How a multi-user Ubuntu install trusts it.** Put both lines in `/etc/nix/nix.conf`, which root owns and the daemon reads:

```
extra-substituters = https://kubaos.cachix.org
extra-trusted-public-keys = kubaos.cachix.org-1:<public key>
```

The `extra-` prefix appends to the default list (`doc/manual/source/command-ref/conf-file-prefix.md:48-54`). Nix only accepts a substituted path signed by a key in `trusted-public-keys` while `require-sigs` is on (the default) (`src/libstore/include/nix/store/globals.hh:243-288`). A substituter a *client* passes is used only if it's in `trusted-substituters` or the user is in `trusted-users` (`worker-settings.hh:287-305`). Adding a user to `trusted-users` "is essentially equivalent to giving that user root access" (`src/nix/unix/daemon.cc:62-72`). So: daemon config, no `trusted-users`, no `cachix use` as the user, and no flake `nixConfig`.

**Do this now, from the Mac:**

1. Create a public Cachix cache `kubaos` with your own signing key. Run `cachix generate-keypair kubaos` in a throwaway container, for example `docker run --rm -it -e CACHIX_AUTH_TOKEN nixos/nix nix run nixpkgs#cachix -- generate-keypair kubaos`. Store the secret key and a write-only per-cache token in Bitwarden. Commit the public key to `system/files/nix/kubaos-cache.conf`.
2. Add `just cache-push <arch>`. After `just payload <arch>`, start a container on the same store volume (`dotinstall-image-nix-<arch>-<nix version>`, `installer/image/build:62-64`, `--platform linux/<darch>`) with `CACHIX_AUTH_TOKEN` and `CACHIX_SIGNING_KEY` taken from `bw get` on the Mac. Push what `nix build --print-out-paths` prints for the curated package outputs. Until the split lands, push the generation's `quickshell-wrapped`, `ukiyo`, and `papirus-icon-theme` paths. Cachix skips what cache.nixos.org has.
3. Ship the key. `system/60-nix.sh` appends the two lines next to its flakes step (`system/60-nix.sh:110-123`) and restarts `nix-daemon`. Mirror that in `installer/image/Dockerfile:105` (the payload) and `test/Dockerfile:69`. For the laptop, run `system/60-nix.sh --no-switch` once. Later, a system migration does it.
4. Until the split lands, the cache only helps if the laptop stops resolving its own lock. Bump inputs on the Mac, `just payload`, `just cache-push`, commit `flake.lock`, then on the laptop `git pull` and switch, without `--nix`'s `nix flake update`. That needs a `system/update.sh --nix --no-flake-update` (or `--switch-only`) flag.

Rotation: name keys `kubaos.cachix.org-N` and keep the old public key in `extra-trusted-public-keys` for one release (`src/nix/key-generate-secret.md`: "a suffix denoting the number of the key (to be incremented every time you need to revoke a key)").

### When CI comes

Short version. A GitHub Actions workflow builds the payloads and ISOs from a pushed tag, runs `cachix push` with the token as a repo secret, and uploads the ISOs to R2. The tag signature, the `allowed_signers` check, and `SHA256SUMS.sig` stay on the Mac. You sign the sums CI produced, or a separate minisign key lives in CI. The Cachix signing key moves to a CI secret. Nothing on installed machines changes, because they only see the tag, the cache, and the Release.

### Changes this implies, in order

1. **Cache now.** Cachix cache, `just cache-push`, the `nix.conf` lines in `60-nix.sh` and both Dockerfiles, and `update.sh --switch-only`.
2. `VERSION` and `CODENAME` files, read in the three places. Pin ukiyo.
3. **The flake split.** `lib.mkHome`, `homeModules.default`, `packages.<system>` for the curated set, `kubaosPkgs` in `home/desktop.nix`, `home/ukiyo.nix`, `home/shell.nix`, `home/theme.nix`, the GPU bundle, the PAM `--inputs-from`. A user flake template. The engine writes it instead of committing `home/username` (`install.rs:1254-1270`). Scripts, wallpapers, and the autostart snippet come from the store, and `dotfilesDir` goes away. `/usr/share/kubaos` with `origin`, a `release` branch, and `safe.directory`. Update `docs/CONTRACTS.md:9-12,62`, `docs/INSTALLER.md:108,115-123,199`, `docs/USING.md:27-73`, and PLAN rules 1 and 4 (`docs/PLAN.md:7,10`).
4. `migrations/` and the runner, plus engine markers at install.
5. `kubaos update`, `upgrade`, `rollback`, and `channel`, replacing `system/update.sh`.
6. Signed tags, `allowed_signers` in the payload, `just release` gating on the tag, `release.json`, and the Hyprland pin file plus `.deb` assets.
7. The notifier timer and bar marker.
8. `just vm-upgrade`.
9. Maintenance branches: `release/X.Y`, cherry-pick routine, end-of-life notice. Needed before the first `0.2.0`, not before `0.1.0`.
10. ISO hosting: split parts on GitHub now, R2 once there's a domain. Then close #7.
11. Command menu, with entries for update, upgrade, rollback and channel as soon as those commands exist.
12. Wallpaper repo, the payload's `/usr/share/kubaos/wallpapers`, and the installer's wallpaper step.

## Open questions

None right now.
