# KubaOS

Personal Ubuntu + Nix + Hyprland OS. Recipes live in `justfile`; docs in `docs/`.

## Agent skills

- **Issues**: GitHub Issues via `gh`. See `docs/agents/issue-tracker.md`.
- **Triage labels**: defaults (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.
- **Domain docs**: single-context (`GLOSSARY.md` + `docs/adr/` at root). See `docs/agents/domain.md`.

## Writing code

Before writing or reviewing code in this repo, read `docs/agents/code-quality.md`: the comment rules for every language, then the Rust, shell, Nix, QML and Lua rules. New code follows them; older code is brought in line where it is touched. Code comments carry the why, never an issue or ADR number; those belong in the issue and the PR.

## Every feature reaches the Command menu

A new feature ships with its Kotodama integration in the same PR, through the KubaOS Extension (`crates/kotodama-kubaos`, actions in `config/kotodama/kubaos.toml`): Items to run it, or at the very least its settings, so it can be found, run and changed from the menu. A feature with no Item is not done.

## Git

- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/). `<type>(<scope>)?: <description>`, types `feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert`; `!` or a `BREAKING CHANGE:` footer for breaking changes.
- **Branches**: [Conventional Branch](https://conventional-branch.github.io/). `<type>/<description>`, types `feature|bugfix|hotfix|release|chore`; lowercase, hyphens only (dots allowed in `release/` versions), e.g. `feature/issue-12-waybar-theme`.
