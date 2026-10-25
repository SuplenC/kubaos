# terminal/

The terminal and the shell in it.

| Folder | Files | What they change | Applies |
|---|---|---|---|
| `ghostty/` | `config` | Ghostty: font, size, transparency, padding | ctrl+shift+, in a window |
| `foot/` | `foot.ini` | foot, the fallback terminal (SUPER + ALT + Return) | new windows |
| `tmux/` | `tmux.conf` | tmux: prefix, mouse, status bar, keys | `tmux source-file ~/.config/tmux/tmux.conf` |
| `zsh/` | `zshrc`, `no-greeting` | Aliases, variables, PATH; `no-greeting` (from `dot-greeting off`) turns off the greeting | new shells |
| `fastfetch/` | `config.jsonc` | The greeting's fastfetch, merged over KubaOS's | `dot-config apply` |

The Command menu's "Settings: Terminal" opens this folder in Files.
