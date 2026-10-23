# KubaOS

A personal Ubuntu + Nix + Hyprland desktop OS: Ubuntu owns the system, home-manager owns `$HOME`, Quickshell is the desktop shell.

## Configuration layers

**Defaults**:
The configuration KubaOS ships for an app, read-only on the machine and replaced whole by every update.
_Avoid_: Shipped config, base config

**User layer**:
The user's own per-app overrides, loaded after the **Defaults** and never written by an update.
_Avoid_: Local config, overrides

**User flake**:
The user's own home-manager flake, which takes KubaOS as an input and adds the user's packages.
_Avoid_: Home config, dotfiles

**Curated set**:
The packages KubaOS builds and pins itself, which move only with a KubaOS release.
_Avoid_: KubaOS packages, pinned packages

**Secret store**:
The user's password manager as KubaOS sees it: one way to fetch a named secret, whichever password manager is behind it.
_Avoid_: Vault, Bitwarden (as the general term), keyring

**KubaOS CLI**:
The `kubaos` command, which does the actual work of changing a KubaOS machine: updates, rollbacks, **Suites**.
_Avoid_: Kōbō, manager, helper scripts

**Kōbō**:
The graphical front end to the **KubaOS CLI** (工房, "workshop"): it walks the user through putting KubaOS on a machine and, afterwards, through tasks like adding **Suites** and setting up projects. The work itself is always done by the engine behind the CLI, which Kōbō runs in its own process.
_Avoid_: Installer, setup app

## Command menu

**Command menu**:
The single searchable menu, opened with `SUPER+Space`, through which the user finds and runs anything: apps, system controls, KubaOS actions, packages and **Suites**.
_Avoid_: Launcher, spotlight, palette

**Kotodama**:
The standalone, KubaOS-independent program KubaOS ships as its **Command menu**; it knows nothing about KubaOS, which reaches it only through **Extensions**.
_Avoid_: Calling KubaOS's extensions part of Kotodama

**Extension**:
A separately running program that contributes items, **Actions** and **Views** to the **Command menu**; KubaOS's own features arrive as extensions like anyone else's.
_Avoid_: Plugin, module, provider

**Action**:
Something the user can do to a result in the **Command menu**; every result has one primary action and may have more.
_Avoid_: Command, handler

**View**:
A screen the **Command menu** pushes on top of the search, such as a list, a form, a detail pane or a progress display.
_Avoid_: Page, panel, mode

## Suites

**Suite**:
A complete, KubaOS-managed setup for one programming language or platform, covering toolchain, editor integration, environment and scaffolding, that the user can install, configure and later remove whole.
_Avoid_: Toolchain, language pack, dev environment

**Project pin**:
A per-folder choice of a **Suite**'s version that applies to that folder and everything below it, overriding the globally installed version.
_Avoid_: Local version, override, `.tool-versions`

## Workspace

**Projects root**:
The folder (`~/Projects` by default) where the user's projects live, new projects are scaffolded, and **Project pins** are typically set.
_Avoid_: Workspace, code folder

**Scripts root**:
The folder (`~/Scripts` by default) of the user's own executable scripts, each of which the **Command menu** offers to run.
_Avoid_: Bin folder, snippets

## Screens

**Workspace group**:
One screen's workspaces 1-10, a block of Hyprland ids owned by that physical screen; while the screen is unplugged its group waits on the primary screen under the same numbers, shown apart in the bar.
_Avoid_: Group (Hyprland's tabbed windows), merged workspaces, guests
