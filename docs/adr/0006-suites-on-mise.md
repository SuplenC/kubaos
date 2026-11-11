# 6. Suites on mise: current upstream toolchains, updated and upgraded per tool

Status: accepted (2026-11-11). Amends ADR 0002: supersedes its rejection of mise, its rule that a Suite's toolchain comes from Nix by default, its `.envrc` Project pins, and its line that rollback restores Suites with everything else.

## Context

ADR 0002 builds a Suite's toolchain from the user flake's nixpkgs. That nixpkgs is a stable branch, and its dev tools run five to six months behind upstream (proto 0.56 against 0.63, uv 0.11 against 0.13, gopls 0.22 against 0.24); updating the lock doesn't help, because a stable branch never takes them. ADR 0002 rejected mise because imperative installs drift from the flake and can't be rolled back.

mise has since been measured against what a Suite needs (mise v2026.11.6, its own docs at `jdx/mise` `docs/`, run with every home and `MISE_*` directory moved into a scratch folder). Its requests live in files: a fragment in `~/.config/mise/conf.d/` installs its tools, `mise upgrade` moves each tool within its request and `mise upgrade --bump` past it, rewriting the fragment (`docs/cli/upgrade.md`), and deleting a fragment makes its versions prunable (`docs/cli/prune.md`). A folder's `mise.toml` pins its own versions, and parents merge. Each install takes one to two seconds.

Already decided with Kuba, and not reopened here: `kubaos suite update [NAME…]` makes patch-level updates and `kubaos suite upgrade [NAME…]` minor and major ones, each with a pick list without a name; for a 0.x tool a minor bump is an upgrade; versions are tracked per tool, not per Suite; `kubaos update` runs `kubaos suite update --all`, then lists the upgrades tool by tool and points to `kubaos suite upgrade`; and declarative stays the rule: a Suite's state is a file, deleting it uninstalls, and versions removed or replaced are pruned.

## Decision

**A Suite's toolchain comes from mise.** Each toolchain Suite is one file, `~/.config/mise/conf.d/kubaos-<name>.toml`, which KubaOS writes whole and deletes on removal, beside its record in `kubaos.d/suites/<name>.toml` (options and extras), which keeps driving the Suite's Nix half. Rustup stays the Rust toolchain's source and proto stays moon's; mise brings rustup and proto themselves. Suites that are not toolchains, such as wallpaper depth, keep their Nix module and get no mise file.

**The file is `kubaos-<name>.toml`.** The prefix names who writes it, in a folder the user may also fill, and keeps KubaOS's fragments together in mise's alphabetical load order. Measured with `kubaos-python.toml`, it behaves as the `suite-` names tried before: `mise install` installs it, `mise outdated` names it as the source, and `mise upgrade --bump` rewrites it. mise's `--global` reads only `config.toml` (`docs/cli/upgrade.md`), so KubaOS runs mise without it, from a folder outside any project. The user's own `~/.config/mise/config.toml` loads after the fragments and wins (`docs/configuration.md`, "conf.d"; measured), as the user layer does everywhere else.

**A Suite wires the base and leaves the rest to the user.** It installs the toolchain and its tools, connects them to the editor and the shell, and ships sane defaults for them. Whatever a tool lets its user configure stays the user's: KubaOS writes no setting the user can't override from their own files, and a default of KubaOS's may differ from the language's own where that serves the setup, never as a lock.

**mise owns the shell's directory hook, and direnv leaves KubaOS's shell.** mise's docs call running the two together unsupported, because their hooks disagree about `PATH` (`docs/direnv.md`). The shell runs `mise activate zsh`, with mise's entries ahead of `~/.nix-profile/bin`. `use suite` pins and nix-direnv go: a Project pin is the folder's `mise.toml`, which `kubaos pin` writes, and a migration rewrites existing `use suite` lines. `use bwenv` becomes a `_.source` script in the project's `mise.toml` (`docs/environments/index.md`). A project with `use flake` loads its devshell with `nix develop`; this repository moves its everyday tools to a `mise.toml` and keeps its devshell for `nix develop` and CI. A user who wants direnv back can enable it in `home.nix`, unsupported.

**mise comes from KubaOS's curated set, at a version each release pins.** It is built from the release's tagged source without the `self_update` feature, so `mise self-update` refuses (`docs/packaging.md`), and served from Cachix with the rest of the curated set. Only a KubaOS release moves it. The `kubaos` binary reads mise's JSON and passes its flags, so the two move together, and `kubaos rollback` of a release brings back the mise its `kubaos` was built against. mise ships several releases a week and removes flags on announced dates (`upgrade --local` in 2027.7.0, `use mise` in 2027.10.4), which a release catches before a user does.

**Language servers, formatters, linters and debug adapters go to mise, in the Suite's file**, each its own tool with its own version: gopls, delve, vtsls, basedpyright, ruff, debugpy. They are the most stale part of the Suites today, and gopls built against an older Go misreads code for a newer one. A tool mise can't install as a release (a VS Code extension run as a debug adapter, such as PHP's) stays in the Suite's Nix module until it can. The Nix module keeps the rest: editor settings through `kubaos.nvim.suiteSpecs`, which take servers from `PATH` instead of Mason; session variables; linked Defaults files; activation steps; and the scaffolders, which take the toolchain from `PATH`.

**PHP comes from mise too, built from source.** mise's `php` has no upstream binary to fetch: it builds through a vfox or asdf plugin (`registry/php.toml`), which needs compilers and development headers. Those build dependencies join KubaOS's package manifest, so apt installs them once, as root, with the system, and every PHP install and patch update compiles. In return PHP is as current as the other Suites, `kubaos suite update` and `upgrade` cover it, and a PHP Project pin is a plain `php = "8.3"` in the folder's `mise.toml`, like any other. Composer and Xdebug are installed against that PHP by the Suite, not taken from Nix, whose extensions are built for Nix's PHP.

**A request is written at major.minor precision**, for every tool: `node = "24.22"`, `go = "1.27"`, `uv = "0.13"`. Then the range a plain `mise upgrade` stays within is exactly the patch releases, for 0.x and 1.x tools alike, and `--bump` keeps that precision (measured: `uv = "0.12"` updates to 0.12.24 and bumps to `0.13`). `kubaos suite update` reads `mise outdated --json` and runs `mise upgrade <tool…>`; `kubaos suite upgrade` reads `mise outdated --bump --json` and runs `mise upgrade --bump <tool>@<major.minor>`, naming the target. Every tool upgrades to its newest release, a Suite's main toolchain included: `kubaos suite upgrade node` offers Node 24 → 26 in its list like any other bump, and when it is picked the Suite's record takes the new line with the fragment. `kubaos suite configure --version` still picks a line by hand. Both commands prune at once.

**Update and upgrade move the Suites, never a project.** They run from outside any project and name the Suites' own tools, so a folder's `mise.toml` is not read, rewritten or bumped by either, and the versions it pins survive the prune while the file exists (tracked configs, `docs/cli/prune.md`; measured). A project stays on the version it pinned until `kubaos pin` or the user changes its file.

**A failed install records nothing.** Under the state lock, `kubaos suite add` and `configure` first run `mise install <tool>@<request>…` with nothing written. If it fails, KubaOS uninstalls the versions that run added (a failed run keeps the tools that did install, measured) and exits non-zero. Only after it succeeds are the fragment, the lockfile and the record written, then the switch runs. A failed switch leaves the generation as it was, so KubaOS takes the record and the fragment back out and prunes.

**Each Suite write ends with `mise lock --global`**, which locks the fragments' tools in `~/.config/mise/mise.lock` (`docs/dev-tools/mise-lock.md`, "Global lockfiles"; measured with a `conf.d` fragment). From then on `mise upgrade` keeps the lockfile current.

**`kubaos rollback` undoes the versions an update moved, not the files.** Before its Suite step, `kubaos update` saves the Suite tools' lockfile entries, as it saves `flake.lock`. A rollback puts those entries back, runs `mise install --locked` and prunes. Measured: with the old entry back, mise selects the old version again and marks the newer one prunable. An update never changes a fragment, only `suite upgrade` and `configure` do, so rollback leaves the fragments and the records alone, as it leaves `kubaos.d/` alone today. A version pruned since is downloaded again, checked against the lockfile's checksum.

## Consequences

- Suites install current upstream releases the day they ship, and stop growing the Nix store. Their versions leave home-manager generations: rollback covers them through the lockfile, not a generation.
- KubaOS has three package sources: apt, Nix and mise.
- The package manifest carries PHP's build dependencies on every machine, and a PHP install or update takes a compile, not a download.
- `home/pins.nix`, `config/direnv/lib/suite.sh`, `kubaos.suiteToolchains` and nix-direnv go.
- `docs/CONTRACTS.md` ("Suites", "Project pins"), `docs/USING.md` and `GLOSSARY.md`'s Project pin are rewritten as each piece lands.

## Considered options

- **Keep the Suites on nixpkgs, or move them to nixpkgs unstable.** Rejected: stable never takes the new tools, and unstable moves the whole home's package set to get them.
- **Keep direnv for `use flake` projects beside mise.** Rejected: unsupported, and both hooks rewrite `PATH`.
- **Keep direnv as the only hook and reach mise through its shims.** Supported, and `use flake` keeps working, but a shim runs mise on every call (measured at about 45 ms a call on top of the 6 to 11 ms the tool itself takes) and doesn't set `[env]`.
- **mise from its own PPA through the package manifest, or `mise self-update`.** Rejected: a mise release would reach users before KubaOS was tested against it, and wouldn't roll back with a release.
- **Language servers left in Nix.** Rejected: they are the most stale tools, and their versions are what `suite update` is for.
- **The main toolchain kept on the line the user chose, upgraded only by `suite configure --version`.** Rejected: an upgrade is asked for by name and picked from a list, so it moves the Suite's own version; what must not move is a project, and a pin already holds that.
- **PHP left in Nix, pinned through mise as a `path:` to a Nix-built toolchain.** It works (measured: mise runs the path and `prune` leaves it alone) and needs no compile, but PHP would stay months behind and outside `suite update`, the one Suite the change didn't reach.
- **Writing the fragment before the install, and deleting it on failure.** Rejected: the fragment is live config for every shell the moment it exists, and a crash between the two leaves a half-installed Suite.
- **Rollback putting back the fragments as well.** Rejected: it would undo a `suite add`, `configure` or `upgrade` the user ran by name after the update, and leave their records behind.
