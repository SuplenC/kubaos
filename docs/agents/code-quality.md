# Code quality

How KubaOS's code is written. The rules come from Kotodama's guide (`SuplenC/kotodama`, `docs/agents/code-quality.md`), adapted to a repo that is mostly shell, Nix, QML and Lua around a few Rust crates. Each Rust rule names its source: Microsoft's Pragmatic Rust Guidelines (`M-…`), the Rust API Guidelines (`C-…`), the std docs, or **kubaos** where the choice is this project's own.

New code follows every rule here. Older code is brought in line where you touch it: edit the lines you change in the new style, without restyling the rest of the file in the same PR. A restyle of a whole file or crate is its own PR.

Tools check some of these rules: `shellcheck` in the shell harness (`test/lib/e2e.sh`), `nix fmt`, clippy, and the design checks (`just design-check`, below). The reviewer checks the rest.

## Comments, in every language

The comment rules hold for Rust, shell, Nix, QML, Lua and config files alike, written with that language's line comment (`//`, `#`, `--`).

Comments are written for a human: the developer reading the code to understand it, and the reviewer of the change. Developers read code first, so the code explains itself. When something is long or strange, the first fix is to split it into well-named functions; a comment is the fallback for logic that stays non-obvious. Obvious code carries no comment at all.

### What gets a comment

- **Nothing that restates the code.** A comment repeating the next line is deleted, and a choice the name already explains needs none.
- **Guard clauses and early returns speak for themselves**; they carry no comment.
- **A non-obvious reason**, in plain prose: the constraint from outside the file, or what the simpler version would break. Performance-driven code says why in a sentence, with the number if one was measured.
- **Invariants** the code relies on and the types or names do not show, as a plain sentence where they are relied on.
- **Hazards** (code a careful developer would plausibly "fix" and break) start with `HAZARD:` and say what breaks: `# HAZARD: take the lock before reading the current wallpaper. Two logins at once theme from different files.`
- **A comment describes the code as it is.** It never narrates a change: no "before this", "now uses", "previously", "no longer", "changed to", "added". What changed and why belongs in the commit message.
- **The why, never an issue or ADR number.** Code comments and rustdoc don't cite `#12` or "ADR 0002"; those belong in the issue and the PR. Docs in `docs/` may cite them. The exceptions are `TODO(#n):` and the regression-test line below.
- **Facts, not hedges.** "Probably", "should work", "I think" become a fact that was checked, or a TODO.

### Pointing elsewhere

- **Other code by backticked path**: `` # `scripts/dot-theme` owns the run lock. ``
- **External sources named and linked**: the claim, then the URL on its own line, as `config/hypr/rules.lua` does with the Hyprland wiki.
- **Workarounds for a dependency**: a `TODO(#n):` saying what it works around and when it can go, plus the upstream link.
- **Wording reads naturally**; common abbreviations (msg, ctx, fd) are fine.

### Form

- **Full sentences, capitalised, ending with a period**, declarative and about the code: "The watcher re-arms on the new target", never "we re-arm".
- **Above the code, never trailing.** No comment after code on the same line, and no labels on closing braces. A `shellcheck disable=` directive may trail, with its reason after `#`.
- **Line comments only**: `//` in Rust and QML (never `/* */`), `#` in shell and Nix, `--` in Lua. Wrap at 100 columns.
- **Markdown is allowed** inside comments: backticks, emphasis, short lists.
- **A comment needing paragraphs means the block wants to be a function**: extract it and put the prose in its doc comment. The header of a script or a config file is the exception (see below).
- **Numbered steps (`# 1. …`) only where a sequence is genuinely non-obvious** and splitting it into functions would not make it clearer.
- **Small text diagrams** (a state machine, a timeline, under about ten lines) are allowed where a picture beats prose.
- **Commented-out code is deleted**; git keeps it, and an intended future list lives in an issue.
- **Section banners** only in long flat files (a Hyprland rules file, `Cargo.toml`, a long table of consts), in the file's existing banner style. Elsewhere a long file splits into modules.

### Headers

- **A script opens with what it does and its usage**, as `scripts/dot-config` does: one summary line, the subcommands with their arguments, and the rules a caller relies on ("It never changes a file in the user layer that exists").
- **A Rust module's `//!`** gives its summary, then `# Owns` and `# Not here` (with where those things live instead).
- **A Nix module, QML file or Lua config file** opens with a line or two on what it owns, when the file name doesn't say it.
- **Config files** (`flake.nix`, the `justfile`, workflows, Nix modules, Hyprland Lua, QML) may explain at more length, for the developer setting things up. They follow the same rule on history: none.

### TODOs

- **`TODO(#77): <the action>; <why it waits>.`** Every TODO has an open issue, so the work is visible; whoever writes the TODO opens the issue first. The sentence still stands alone.
- **The PR that closes an issue removes its TODOs.**
- **`FIXME` and `XXX` are not used.**

### Tests

- **The test's name is a sentence saying what it checks**: `a_hidden_app_never_shows_in_the_menu`, or the shell harness's `c_<what>` with a description that reads as one. No comment.
- **Blank lines separate setup, action and assertion**; a comment only for setup whose purpose is not visible.
- **A regression test** has one line linking the bug: `# Regression: #102.`

## Performance comes before style

New KubaOS components are written for speed and memory first. On a hot path (anything a keystroke, a login or a theme change waits on), performance wins over the style rules here. Three conditions keep it honest:

- **It is measured.** The PR shows the number the faster version wins on; a guess at speed does not override a rule.
- **It says why, once.** A comment above the code names what it optimises and the number, so the next reader does not "clean it up".
- **It stays on the hot path.** Everything around it still follows the guide.

## Rust

### Formatting and layout

- **Imports: three blocks, one item per `use`.** `std`, then external crates, then `crate`/`super`/`self`, a blank line between. A module is imported as `use std::io;`, never `use std::io::{self};`. Source: kubaos (from Kotodama).
- **`std::` paths everywhere**, not `core::`/`alloc::`. Source: kubaos.
- **Line width 120.** Source: kubaos.
- **A module with children is `foo.rs` beside `foo/`**, never `foo/mod.rs`. Clippy `mod_module_files`. Source: kubaos.
- **Test helpers under `tests/` live in `tests/support/<name>.rs`**, included with `#[path = "support/<name>.rs"] mod <name>;`. A file directly in `tests/` is a target of its own. Source: kubaos.
- **No `unsafe`.** A crate opens with `#![forbid(unsafe_code)]`. If an FFI boundary ever needs it, each `unsafe` block carries a `// SAFETY:` comment saying why every requirement of the call holds. Source: M-UNSAFE.

### Lints

- **Clippy is clean under `-D warnings`, with `pedantic` on.** Source: M-STATIC-VERIFICATION.
- **Silence a lint with `#[expect(lint, reason = "…")]`**, at the narrowest item. `#[allow]` is for generated code and macro output only. Source: M-LINT-OVERRIDE-EXPECT.
- Beyond pedantic: `allow_attributes`, `clone_on_ref_ptr`, `dbg_macro`, `manual_let_else`, `mod_module_files`, `return_self_not_must_use`, `unwrap_used` (off in tests). The `[workspace.lints]` of Kotodama's root `Cargo.toml` is the reference set when a KubaOS workspace adopts them.

### Errors

- **A fallible public operation returns a public enum, one variant per cause.** Callers match on the cause; a stringly error cannot be matched. Source: kubaos.
- **An operation that can fail for one cause only returns a struct**, not a one-variant enum: a tuple struct with a private field, derived with `thiserror`, made by a named constructor (`.map_err(ReadError::io)`). When a second cause appears, it becomes an enum. Source: kubaos.
- **`Display` is this layer's message only; the cause is `source()`.** Write `#[error("reading packages.toml")] Read(#[source] io::Error)`, never `#[error("reading packages.toml: {0}")]`. A reporter that walks the chain prints each cause once. Source: `std::error::Error` docs.
- **`thiserror` writes `Display` and `Error`.** Source: kubaos.
- **Convert at the call site with an explicit, named constructor** (`.map_err(Error::read)?`), not a blanket `From`. Source: kubaos.
- **`#[non_exhaustive]` case by case.** On a public enum that will grow and whose callers needn't react; exhaustive when a new case must break the caller's `match`. Source: C-STRUCT-PRIVATE.

### Panics

- **Return an error or restructure so the case cannot arise.** That is always the first choice. A panicking install step leaves a half-installed machine. Source: M-PANIC-IS-STOP.
- **A panic is acceptable at startup**, while the process is being assembled, where the developer sees it at once.
- **At runtime, a panic only for a broken process**, such as a poisoned mutex: `.expect("job queue mutex poisoned")`, naming what broke. No `unwrap()` outside tests. Source: M-PANIC-MESSAGE.

### Control flow

- **`let … else` for early exit**, so what remains in the `if` is short. Clippy `manual_let_else`.
- **Let-chains for the conditions that stay together**: `if let Some(suite) = suite && suite.installed`. A small named helper where a chain still reads long, not one per condition.
- **Iterator chains for pure transforms; a `for` loop once there is an early return, a side effect, or more than two steps.**

### Threads and shared state

- **Shared mutable state is a `std::sync::Mutex` behind a struct with methods**, so no caller holds the lock longer than one method. A thread owning the state behind channels when the state has its own lifecycle (a job queue, a watcher). Source: kubaos.
- **Lock order lives on the field's `///`**, with a one-line pointer at each site that takes more than one lock.

### Signatures and types

- **Generic spelling by shape**: `impl Trait` for a simple, single-use bound; a named parameter when a caller may turbofish it or it repeats; a `where` clause once bounds no longer fit on the line.
- **String arguments: `impl AsRef<str>` when the function only reads**; `impl Into<String>` only when it keeps the string.
- **An on/off tunable is a pair of setters, `x()` and `no_x()`**, never a `bool` argument.
- **A builder method returns `Self` and is `#[must_use]`**, and so is every constructor. Source: C-BUILDER.
- **Every public type implements `Debug`.** A type holding a secret (a password, a LUKS passphrase, a token) writes `Debug` by hand and prints `<redacted>`. Source: M-PUBLIC-DEBUG, C-DEBUG.
- **Clone an `Arc` as `Arc::clone(&x)`.** Clippy `clone_on_ref_ptr`.
- **A default value is a documented `const`**, named `DEFAULT_…`, whose doc says why that value. The `Default` impl names the const. Source: M-DOCUMENTED-MAGIC.

### Docs

- **Rustdoc: a summary sentence, then `# Errors` and `# Panics` on every public item that can.** Source: M-CANONICAL-DOCS.
- **Rustdoc describes the end state.** It never cites ADRs or issue numbers. Source: M-NO-META-DESIGN-DOCUMENTATION.
- **Domain terms are the glossary's** (`GLOSSARY.md`): a Suite, a Project pin, Kōbō, the Defaults and the User layer, never a synonym.

### Comments next to tools

- **`#[expect]`** carries its whole explanation in `reason`.
- **`.expect("…")`** keeps a short message saying what failed; when the reason it cannot fail is not obvious, a comment above explains it.
- **Copied code** opens with one line naming its origin and licence.
- **Private fields and non-trivial private functions that need explaining get `///`**, so the explanation shows on hover.

### Tests

- **Unit tests in a sibling file**: `#[cfg(test)] mod tests;` at the bottom of `install.rs`, the tests in `install/tests.rs`. Behaviour across processes goes in the crate's `tests/` and the end-to-end suites (`install/test/`, `test/`).
- **Tests may `unwrap()`.** A fallible step whose line doesn't say what failed is `.expect("the step")`: `.expect("write the user flake")`.
- **Never weaken a test to make a change pass.** If an expectation must change, the PR says which one and why.

### Logging

- **Log with the `log` (or `tracing`) macros**, one event per fact, with the values in the message.
- **No `println!`, `eprintln!` or `dbg!`** outside a binary's user-facing output (the `kubaos` command, the CLI's progress). Source: M-LOG-NOT-PRINT.

## Shell

- **`#!/usr/bin/env bash` and `set -euo pipefail`**, at the top of every script. A sourced library (`system/lib.sh`, `test/lib/*.sh`) opens with `# shellcheck shell=bash` instead of a shebang.
- **`shellcheck` clean.** A `disable=` directive names the code and says why on the same line.
- **Quote every expansion**, use `[[ ]]` for tests and arrays for argument lists (`flags=(--release)`, `"${flags[@]}"`).
- **Errors go to stderr with a non-zero exit**; usage errors exit 2.
- **A script that changes the system takes `--dry-run` and `--help`**, as `system/*.sh` do through `system/lib.sh`.
- **A script grows into a function once it repeats**; a script past a few hundred lines with real logic is a candidate for the Rust engine.
- **An apt package goes in the package manifest, `system/packages.toml`**, in the group of the feature that needs it (a new group when none fits, `optional` when the user may leave it out), and the script installs it with `group_install <group>`, never with an `apt_install` list of its own. Every `kubaos update` installs what the manifest adds, so a PR that adds a package writes no migration for it; migrations are for config. A vendor repository is a `[repo.<name>]` with its key's fingerprint and a sources file in `system/files/apt/sources.list.d/`. `docs/CONTRACTS.md`, "The package manifest", has the rest.

## Nix

- **`nix fmt`** (nixfmt) formats every `.nix` file.
- **Attribute arguments are listed, one per line**, with `...` last, as `nixfmt` lays them out.
- **Curated packages come from `kubaosPkgs`**, the user's from `pkgs`. A module never takes a curated package from `pkgs`.
- **`let` bindings are named for what they hold**, and carry a comment only for a non-obvious value (a version pinned for a reason, a size chosen for the screens).

## QML

- **Imports first, Qt then Quickshell then `qs.*`**, as in `config/quickshell/services/`.
- **A file opens with a comment on what it owns** when the file name doesn't say it.
- **Bindings over imperative updates.** A `readonly property` derived from others, not a handler that sets it; a handler only for a side effect.
- **A property that needs explaining gets a one-line comment above it**, never trailing.
- **No magic numbers in layout**: colours, sizes, spacing, radii, durations and fonts come from the tokens, as "Design" below says. A named property holding a literal (`property int iconSize: 22`) is still a literal.
- **A search over a list in the shell uses Kotodama's engine, never a hand-rolled filter.** The list's entries are Items of a Kotodama app (an Extension with a word, in `crates/kotodama-kubaos`), and the view embeds a `KotodamaSearch` of its own that enters the app and draws its Results, so typing matches the way the Command menu does, accents, typos and frecency included, and the same list is in the menu. `modules/controlcenter/KeyboardAddView.qml` is the example. A field that parses what is typed (a time, a password) isn't a search and stays as it is.

## Design

Every surface a person sees follows `docs/DESIGN.md`, in every toolkit. Read it before writing or reviewing UI code.

- **Values come from the tokens**, never written out: a colour is a role, a length a `Space`, `Size` or `Radius` token, text a type style, motion a duration and a curve. In Rust that is `kubaos_ui::tokens`; in QML, `KubaOS.Components`' generated `Tokens` singleton (the shell's `Theme` only feeds it matugen's colours from the wallpaper). A value the tokens lack is added to `design/tokens.toml` first, with its reason, and then used. Source: kubaos.
- **A radius is a role, or nested.** A component's corners take the radius its role names (`Radius::Small`, `Control`, `Surface`, `Full`); a shape that fills another less an inset takes `nested(outer, inset)`. Never a radius number, and never `outer - inset` by hand. Source: kubaos.
- **Components come from the libraries**: `kubaos-ui` in iced, `KubaOS.Components` in QML, by their shared names. An app composes them into blocks; it never builds a button out of a toolkit's own. A new kind, size or component changes `docs/DESIGN.md` first. Source: kubaos.
- **Layout goes through the primitives** (`page`, `stack`, `row`, `grid`, `split` in `kubaos-ui`), which keep boxes on device pixels and tell the geometry check what each box is. A new widget in `kubaos-ui` wraps its output in `probe::component` with its shared name, variant and fixed measures. Source: kubaos.
- **The design lint** (`kubaos-design lint`) fails on a literal design value or a toolkit control in QML or Rust UI code. `design/lint-baseline.toml` holds what the older front ends still have, per file and rule, and only shrinks: a file over its count fails, and a file under it fails until `just design-baseline` lowers the count in the same PR. Touching a baselined file, fix the violations in the lines you change, as with every rule here. Source: kubaos.
- **A justified literal** (a value that isn't a design value: a protocol's size, a font in the specimen's comparison sheet) carries a marker on its own line directly above it, naming the rule and the reason; it allows those rules on that one line: `// design-allow(size): the tray icon's size is the StatusNotifierItem spec's.` The reason is required; a marker that allows nothing fails. Source: kubaos.
- **The geometry check** (`kubaos-specimen check`) holds every specimen page to the grid at 1, 1.25, 1.5 and 2, light and dark. A change to `kubaos-ui` or the specimen passes it, and a change to how anything looks commits the regenerated `design/specimen/` shots. `just design-check` runs the lint, the contrast check, the geometry check and parity; `cargo test --workspace` and `nix flake check` run the first three. Source: kubaos.
- **Every colour a front end draws on another is a listed pair** in `[contrast]`, with what it needs (WCAG 2.1 AA) and the opacity it is drawn at; a new pairing adds its line, and `kubaos-tokens` refuses one under its minimum. `outline` is never text. Source: kubaos.
- **Icons take their size from the text beside them** (`Type::icon()`, `Tokens.type.<style>.icon`), every key named on screen is a `Keycap`, every page or panel body is a `Page` or `Container`, and every picture is a `Media`. Source: kubaos.

## Lua (Hyprland, Neovim)

- **Two-space indent**, `local` for everything not exported.
- **A helper for a repeated shape**, as `tag()` in `config/hypr/rules.lua`, not a copied block.
- **Links to the upstream option docs** at the top of a file that configures an external program.
