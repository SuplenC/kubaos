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

**Installer**:
The program that puts KubaOS onto a machine and runs its first boot.
_Avoid_: Using it for adding software or **Suites** to a running system

## Command menu

**Command menu**:
The single searchable menu, opened with `SUPER+Space`, through which the user finds and runs anything: apps, system controls, KubaOS actions, packages and **Suites**.
_Avoid_: Launcher, spotlight, palette

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
