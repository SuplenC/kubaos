# 8. The Apps catalog: a few apps KubaOS installs well, from pinned sources

Status: accepted (2026-11-16). Kuba's decisions in #93; the engine is #425.

## Context

A new machine needs a handful of apps beyond the base system: a browser, a password manager, Docker, the coding agents. Each one comes from somewhere else (a vendor's apt repository, Ubuntu's archive, nixpkgs, a vendor's release page), and the vendors' own instructions mostly end in `curl | sh` or in a repository added with a key nobody checked. KubaOS installs a few apps itself today (Chrome and Docker in the package manifest's optional groups, Bitwarden from its own script), each a different way, and a user can't add or remove one from one place.

## Decision

**A catalog of essential apps, one manifest each.** `apps/<id>.toml` in the repo describes one app: its name, a line on what it is, and the one source it comes from. The catalog is small on purpose; everything else the user installs themselves. Adding an app is a small PR: a manifest, and for an apt vendor repository its `[repo.<name>]` and sources file.

**Sources, in this order:** apt (a vendor repository with its key's fingerprint pinned, then Ubuntu's archive), then nixpkgs (the User flake, as `kubaos package add` does), then a `.deb` from a release URL pinned to its sha256. An AppImage, pinned the same way, is a planned source, not built until an app needs one. No Flatpak and no unpinned `curl | sh`. An app takes the first source in this order that has it current; the PR adding it says which and why.

**A repository is trusted one way only.** An apt app's vendor repository is a `[repo.<name>]` of `system/packages.toml`, set up by the same `repo_setup` as the package manifest's groups, never a second copy of the key or the sources line. A source of the same repository the vendor's own instructions set up is turned off for it, as apt refuses one repository with two keys.

**The user's picks are the source of truth.** `~/.config/kubaos/apps.toml` lists the apps the user picked. `kubaos app add` and `remove` change it and the machine together; `kubaos update` installs a picked app the machine lacks, and does nothing when nothing changed. Whether an app is installed is read from where it is (dpkg, the User flake's `packages.toml`), never kept in a second record. The picks only note which nixpkgs names an app added to `packages.toml`, so removing it takes those and never a name the user added with `kubaos package add`. An app on the machine that isn't picked is left alone: KubaOS removes an app only when the user asks.

**The command line:** `kubaos app list | add <id>... | remove <id>... | status`, each with `--json` for the Command menu.

**Updates.** apt and nixpkgs apps move with `kubaos update`. A pinned download moves when a KubaOS release bumps its pin; an app that updates itself (`self_updates` in its manifest) is never put back to the pin.

**Root goes through one small helper.** A root helper behind one polkit prompt per run, which pam_u2f can answer, built like first boot's `setup-helper`. It takes only app ids, reads each app's manifest from the root-owned catalog KubaOS ships (`/usr/share/kubaos/apps/`), and never takes a package name, URL or path from the caller.

**Profiles stay out of the manifest.** Which apps the first-boot guided setup proposes is that setup's concern (#123), not a field of the manifest.

## Consequences

- Chrome, Bitwarden and Docker leave the base image and become catalog apps (#426); a machine that has them keeps them, recorded as picked.
- The Command menu offers an Install or a Remove Item per app, whichever applies (#427), from `kubaos app status --json`.
- A pinned download's version moves only with KubaOS releases, so its manifest is bumped by a PR when the vendor ships.
- The manifest keeps to the TOML the system scripts' reader understands, as the package manifest does, so the root helper reads it with the same reader.
