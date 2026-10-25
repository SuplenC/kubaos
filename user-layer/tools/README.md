# tools/

The command-line tools and the fallback launcher.

| Folder | Files | What they change | Applies |
|---|---|---|---|
| `git/` | `config` | git settings and aliases (your name and email live in `~/.config/git-identity/`) | the next git command |
| | `ignore` | Global ignore patterns, added to KubaOS's | `dot-config apply` |
| `lazygit/` | `config.yml` | lazygit | its next start |
| `yazi/` | `yazi.toml`, `keymap.toml` | yazi's settings and keys; `keymap.toml` holds KubaOS's drag key (Ctrl+N) | its next start |
| `bat/` | `config` | bat's theme, style and tabs, appended to KubaOS's | `dot-config apply` |
| `bacon/` | `prefs.toml` | bacon | its next start |
| `fuzzel/` | `fuzzel.ini` | fuzzel, the fallback launcher (SUPER + ALT + Space) | its next start |

The Command menu's "Settings: Tools" opens this folder in Files.
