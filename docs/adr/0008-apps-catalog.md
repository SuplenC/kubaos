# 8. The Apps catalog: a few apps KubaOS installs well, from pinned sources

Status: accepted (2026-11-16). Kuba's decisions in #93; the engine is #425, setups #431.

## Context

A new machine needs a handful of apps beyond the base system: a browser, a password manager, Docker, the coding agents. Each one comes from somewhere else (a vendor's apt repository, Ubuntu's archive, nixpkgs, a vendor's release page), and the vendors' own instructions mostly end in `curl | sh` or in a repository added with a key nobody checked. KubaOS installs a few apps itself today (Chrome and Docker in the package manifest's optional groups, Bitwarden from its own script), each a different way, and a user can't add or remove one from one place.

## Decision

**A catalog of essential apps, one manifest each.** `apps/<id>.toml` in the repo describes one app: its name, a line on what it is, and the one source it comes from. The catalog is small on purpose; everything else the user installs themselves. Adding an app is a small PR: a manifest, and for an apt vendor repository its `[repo.<name>]` and sources file.

**Sources, in this order:** apt (a vendor repository with its key's fingerprint pinned, then Ubuntu's archive), then nixpkgs (the User flake, as `kubaos package add` does), then a `.deb` from a release URL pinned to its sha256. An AppImage, pinned the same way, is a planned source, not built until an app needs one. No Flatpak and no unpinned `curl | sh`. An app takes the first source in this order that has it current; the PR adding it says which and why.

**A repository is trusted one way only.** An apt app's vendor repository is a `[repo.<name>]` of `system/packages.toml`, set up by the same `repo_setup` as the package manifest's groups, never a second copy of the key or the sources line. A source of the same repository the vendor's own instructions set up is turned off for it, as apt refuses one repository with two keys.

**The user's picks are the source of truth.** `~/.config/kubaos/apps.toml` lists the apps the user picked. `kubaos app add` and `remove` change it and the machine together; `kubaos update` installs a picked app the machine lacks, and does nothing when nothing changed. Whether an app is installed is read from where it is (dpkg, the User flake's `packages.toml`), never kept in a second record. Only which nixpkgs names an app added to `packages.toml` is noted, in the state folder beside the machine it describes (`~/.local/state/kubaos/apps-added.toml`), not in the picks: removing the app takes those and never a name the user added with `kubaos package add`, and a `~/.config` copied to another machine carries no record that would take that machine's names. An app on the machine that isn't picked is left alone: KubaOS removes an app only when the user asks.

**The command line:** `kubaos app list | add <id>... | remove <id>... | status`, each with `--json` for the Command menu.

**Updates.** apt and nixpkgs apps move with `kubaos update`. A pinned download moves when a KubaOS release bumps its pin; an app that updates itself (`self_updates` in its manifest) is never put back to the pin.

**Root goes through one small helper.** A root helper behind one polkit prompt per run, which pam_u2f can answer, built like first boot's `setup-helper`. It takes only app ids, reads each app's manifest from the root-owned catalog KubaOS ships (`/usr/share/kubaos/apps/`), and never takes a package name, URL or path from the caller.

**An app may offer setups, one installed at a time.** Some apps can be set up more than one useful way: T3 Code as a desktop app that runs its agents itself, or as a background service of the user's with the desktop app as its client. A manifest may list named setups (`[setup.<id>]`), each with the name the menu and the command line call it by. The app's source is shared by every setup; a setup adds its own parts on top, which are the user's, never root's: a pair of user hooks run as the user from the release's checkout, a download pinned to its sha256 that they install, and a marker file in the home that is there exactly while those parts are. One setup may have no parts of its own, the app as its source installs it. There is no default: the Command menu offers one Install Item per setup, and `kubaos app add` without `--setup` refuses and names them. The picks record the chosen setup (`[setups]` in `apps.toml`); picking another switches, taking away the old setup's own parts and leaving what the two share. A setup is read from the machine like everything else: its marker, or the source alone for the setup without parts, so a setup someone made by hand is found as that setup and never installed over. Removing the app takes every setup's parts that are there. A setup's download is installed once and then kept current by the app itself, as KubaOS can't read its version back from a marker.

**Profiles stay out of the manifest.** Which apps the first-boot guided setup proposes is that setup's concern (#123), not a field of the manifest.

## Consequences

- Chrome, Bitwarden and Docker leave the base image and become catalog apps (#426); a machine that has them keeps them, recorded as picked.
- The Command menu offers an Install or a Remove Item per app, whichever applies (#427), from `kubaos app status --json`.
- A pinned download's version moves only with KubaOS releases, so its manifest is bumped by a PR when the vendor ships.
- T3 Code offers two setups (#431): the desktop app, its pinned `.deb`; and a service, which adds its `t3` runtime from the release's pinned CLI archive, the background service as a systemd user unit on this machine only (no local network, no T3 Connect), and the desktop app set to connect to it. Remote access is the user's to turn on from T3 Code itself.
- The manifest keeps to the TOML the system scripts' reader understands, as the package manifest does, so the root helper reads it with the same reader.
