# editor/

Neovim (LazyVim). Your files run after KubaOS's and LazyVim's, so they win; changes take
effect in a new Neovim.

| Folder | Files | What they change |
|---|---|---|
| `nvim/` | `options.lua` | Options: line numbers, indent, wrap, format on save |
| | `keymaps.lua` | Your keys, and LazyVim's you remove |
| | `autocmds.lua` | Autocommands |
| | `plugins/*.lua` | Plugin specs: add plugins, change their options, turn them off |
| | `lazyvim.json` | LazyVim extras; `:LazyExtras` writes it |
| | `lazy-lock.json` | Plugin versions; `:Lazy update` writes it |

The Command menu's "Settings: Editor" opens this folder in Files.
