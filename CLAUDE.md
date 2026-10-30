# KubaOS

Personal Ubuntu + Nix + Hyprland OS. Recipes live in `justfile`; docs in `docs/`.

## Agent skills

- **Issues**: GitHub Issues via `gh`. See `docs/agents/issue-tracker.md`.
- **Triage labels**: defaults (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.
- **Domain docs**: single-context (`GLOSSARY.md` + `docs/adr/` at root). See `docs/agents/domain.md`.

## Writing code

Before writing or reviewing code in this repo, read `docs/agents/code-quality.md`: the comment rules for every language, then the Rust, shell, Nix, QML and Lua rules. New code follows them; older code is brought in line where it is touched. Code comments carry the why, never an issue or ADR number; those belong in the issue and the PR.

A change to a file under `system/files/` ships a new `migrations/system/` script that installs it again (it calls the file's `install_*`), or installed machines keep the old copy; `system/files.map` names what installs each file, and `just system-files-check` fails a branch without one (`docs/agents/code-quality.md`, "Shell").

## Design

**Before you plan, brief or build anything a person sees, read `docs/agents/building-ui.md`.** A UI is assembled from library parts that Kuba has already sketched, reviewed and approved: atoms, molecules and templates live in the component library, organisms in the app. A missing or lacking part goes through inventory, a sketch with Kuba and the library first; an app never builds its own. Apps are `kubaos-ui` (Rust, iced) windows; the shell's surfaces are `KubaOS.Components` (QML).

Any UI, in any toolkit, follows `docs/DESIGN.md` and each component's page under `docs/design/components/`: tokens and the shared components only, never a literal colour, size, spacing, radius, duration or font. `just design-check` passes (the rules are in `docs/agents/code-quality.md`, "Design"). Before a UI change reaches Kuba, review its rendered screenshots against `docs/agents/design-review.md`, his recurring review points.

Render, diff and attach a UI change's screenshots with `just shots`, as `docs/agents/design-pr-loop.md` describes.

Generated files aren't committed, so UI PRs don't collide: the token outputs are built where they are read (`just tokens` writes them into a checkout for an editor), and the specimen's pictures are rendered, never checked in. A new component is a page of its own under `docs/design/components/` and a specimen file of its own on each side, plus its lines in the lists (`docs/DESIGN.md`, "Changing the design").

## Docs describe the current state

Every file in the repo (docs, comments, config, agent guides) says how things are now. Git keeps the history: what changed, when, who asked and why go in commit messages, PRs and issues, never in a file as a dated note, a "previously", or an appended changelog. When something changes, rewrite the text to the new state instead of adding to it. The exceptions are records that are dated by design (ADRs in `docs/adr/`, `docs/PLAN.md`, `docs/RESEARCH-*.md`): they stay as written, and the current behaviour lives in `docs/CONTRACTS.md` and `docs/USING.md`, not in a note added to the record.

## Every feature reaches the Command menu

A new feature ships with its Kotodama integration in the same PR, through the KubaOS Extension (`crates/kotodama-kubaos`, actions in `config/kotodama/kubaos.toml`): Items to run it, or at the very least its settings, so it can be found, run and changed from the menu. A feature with no Item is not done.

One result per thing: a feature adds Actions to its subject's Item, each with its verbs and role, never Items of its own about another one (no "Install X" beside X, no "Proxy on" beside "Proxy off"). See "The Command menu" in `docs/agents/code-quality.md`.

In-shell searches over lists use Kotodama's engine, never a hand-rolled filter (see the QML rules in `docs/agents/code-quality.md`).

## Git

- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/). `<type>(<scope>)?: <description>`, types `feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert`; `!` or a `BREAKING CHANGE:` footer for breaking changes.
- **Branches**: [Conventional Branch](https://conventional-branch.github.io/). `<type>/<description>`, types `feature|bugfix|hotfix|release|chore`; lowercase, hyphens only (dots allowed in `release/` versions), e.g. `feature/issue-12-waybar-theme`.
