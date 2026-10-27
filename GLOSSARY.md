# KubaOS

A personal Ubuntu + Nix + Hyprland desktop OS: Ubuntu owns the system, home-manager owns `$HOME`, Quickshell is the desktop shell.

## Configuration layers

**Defaults**:
The configuration KubaOS ships for an app, read-only on the machine and replaced whole by every update.
_Avoid_: Shipped config, base config

**User layer**:
The user's own per-app overrides in `~/.config/kubaos`, grouped by area (desktop, terminal, editor, tools, menu), loaded after the **Defaults** and never written by an update.
_Avoid_: Local config, overrides

**User flake**:
The user's own home-manager flake, which takes KubaOS as an input and adds the user's packages.
_Avoid_: Home config, dotfiles

**Package manifest**:
The one list of KubaOS's apt packages, `system/packages.toml`, which the install and every update read, so the machine's apt packages follow the release as its Nix packages do.
_Avoid_: Package list, base packages

**Package group**:
A named set of the **Package manifest**'s packages for one feature (Docker, the file manager, Hyprland); an optional one the user can turn off or on for good.
_Avoid_: Feature flag, bundle

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

**Desk position**:
Where a screen stands on the desk: the laptop panel, or the externals numbered 1, 2 and on. A screen pinned to a **Workspace group** stands at that group's number; the others are counted from the left by Hyprland's layout, passing over the numbers of groups that pin a screen. A workspace group belongs to a desk position, so any screen standing at position 1 shows position 1's group.
_Avoid_: Slot, monitor index, pairing

**Workspace group**:
A **Desk position**'s workspaces 1-10, a block of Hyprland ids owned by that position whatever screen stands there; while no screen holds the position its group waits on the primary screen under the same numbers, shown apart in the bar. The user can pin screens to a group by description, in `desktop/workspaces/groups.toml` of the **User layer**, and give it a name: a pinned screen shows its group wherever it is plugged in, the first listed winning when two of a group's screens are; the laptop panel can be pinned too, and holds no group while its lid is closed.
_Avoid_: Group (Hyprland's tabbed windows), merged workspaces, guests

## Devices

**Phone link**:
KubaOS's link between the laptop and the user's phone through KDE Connect: the clipboard and files both ways, and ringing the phone. Off until the user sets up a phone; nothing listens for phones until then.
_Avoid_: KDE Connect (as the feature's name), phone sync

## Power

**Power stop**:
One of the five fixed positions of the power slider, Super Saver, Power Saver, Balanced, Performance and Max Performance, each defined across every layer of the machine on top of one of power-profiles-daemon's three profiles. Each power source (battery, plugged in) remembers the stop last picked on it.
_Avoid_: Power profile (power-profiles-daemon's own three), power mode, power plan
