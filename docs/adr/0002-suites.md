# 2. Suites, the command menu, and one engine behind the CLI

Status: accepted (2026-10-18)

## Context

ADR 0001 made `SUPER+Space` the single menu for apps and KubaOS actions, and gave KubaOS a `kubaos` CLI. Two things were still missing: a menu good enough to carry that, and a way to install and later remove a whole programming setup (toolchain, editor integration, environment, scaffolding) without touching the user's own config. Today the Installer's crates do the install work themselves, and toolchains other than Rust's cargo tools are installed by hand (`docs/INSTALL.md`). The terms used here are defined in `GLOSSARY.md`.

## Decision

**The menu is Kotodama, a separate product.** Kotodama lives in its own repo (`SuplenC/kotodama`) and knows nothing about KubaOS. Its own architecture ADR covers the daemon, the Quickshell front end and the Extension protocol. KubaOS ships it in the curated set and adds its own features through one Extension, `kotodama-kubaos`, built in this repo against Kotodama's protocol crate. That Extension controls the system directly (NetworkManager and BlueZ over D-Bus, `wpctl`, brightnessctl), so the controls still work when Quickshell is down. Quickshell keeps IPC only for what it owns itself: panels, the wallpaper picker, DND.

**One engine, two front ends.** The install engine moves out of `installer/` into a top-level `crates/` Cargo workspace: `kubaos-core` (the engine: install, first boot, updates, Suites, Project pins), `kubaos-cli` (the `kubaos` binary), `kubaos-kobo` (the iced GUI, formerly the Installer, renamed Kōbō) and `kotodama-kubaos`. Kōbō calls `kubaos-core` directly instead of running the CLI as a subprocess, so progress and errors stay typed. The work itself is always done by the engine.

**Suite state lives in files the tool owns.** The user flake's `home.nix` stays the user's. Everything the CLI, the menu or Kōbō installs is declared in `~/.config/kubaos/home/kubaos.d/`: `packages.toml` plus `suites/<name>.toml` (the Suite's options and version). KubaOS's home module reads the directory with `builtins.readDir` and `builtins.fromTOML` and enables the matching Suite modules. Installing a Suite writes its file and removing it deletes the file, so the tool never edits Nix code or rewrites a file in place. `home.nix` gains one import line, added by a migration that only adds. Rollback (ADR 0001) restores Suites along with everything else.

**Suites write only to the Defaults.** A Suite's editor configuration, such as a LazyVim language extra or KubaOS's settings for it, goes into the Defaults. The user layer loads last, so the user's settings always win. A Suite may seed a missing user-layer file with a commented example, as `dot-config apply` does, and never writes to one that exists.

**Removing a Suite** removes everything it declared. Caches and downloaded dependencies (`~/go`, the composer cache, and so on) are deleted only if the user ticks a box that shows their size. Scaffolded projects and the user's own files are never touched. The remove view lists user-layer files that mention the Suite.

**Each Suite picks its own toolchain source.** Nix is the default and rustup is used for Rust; the Suite hides which one is behind it. A **Project pin** is a `.envrc` line, `use suite <name> <version>`, served by a direnv function KubaOS ships (direnv and nix-direnv are already enabled). direnv loads the nearest `.envrc` above the current folder, so a pin on a folder applies to everything below it. `kubaos pin` and a menu Action write it.

**Long jobs run as transient systemd user units**, so closing the menu doesn't kill them. One lock keeps `home-manager switch` from running twice at once.

**v1 Suites are home-only**: Rust, Go, Node, Python, PHP, proto+moon. Suites that need root changes (Flutter's KVM and udev rules) wait for a small set of privileged steps behind one polkit prompt, which the Suite declares and removal reverts.

## Considered options

- **Editing `home.nix` from the tool.** Rejected: it breaks the contract that the user flake is the user's, and editing Nix code programmatically is fragile.
- **Imperative installs (`nix profile`, mise).** Rejected as the source of truth: they drift from the flake and can't be rolled back. "Try it" stays available as `nix shell`.
- **One `kubaos.toml`.** Rejected for a directory: one file per Suite makes install and remove atomic file writes and deletes, and leaves the user's comments in other files alone.
- **Folding KubaOS features into Kotodama.** Rejected: Kotodama is useful outside KubaOS, and keeping KubaOS behind the same Extension API anyone else uses keeps that API honest.
