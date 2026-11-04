# KubaOS's Neovim

KubaOS's default Neovim: [LazyVim](https://github.com/LazyVim/LazyVim) with a base set of
extras, themed from the wallpaper. home-manager links this folder to `~/.config/nvim`, read-only.
Your own settings go in the user layer, `~/.config/kubaos/editor/nvim/`, which loads after this
config, so it wins.

## What ships

- **LazyVim**, with these extras (`lua/config/lazy.lua`):
  - languages: Docker, markdown, git, JSON, TOML, YAML and shell dotfiles, and prettier, which
    formats markdown, JSON, YAML and the web languages;
  - editing: mini-surround, yanky, dial, inc-rename, mini-move, mini-hipatterns;
  - the snacks explorer and picker, aerial, dap and test.
- **The extras' npm-based tools, from Nix** (`home/cli.nix`), so they work without the Node
  Suite: prettier, markdownlint-cli2 and markdown-toc, and the JSON, YAML, Dockerfile, Docker
  Compose and bash language servers. Mason leaves them out; it still installs the tools that
  aren't npm packages (marksman, taplo, hadolint, shellcheck, lua-language-server, stylua, shfmt).
- **Plugins** (`lua/plugins/`): diffview and fugitive for git, vim-tmux-navigator (`C-h/j/k/l`
  across Neovim splits and tmux panes), oil, vim-visual-multi, and the theme.
- **The theme** (`lua/config/theme.lua`, `colors/matugen.lua`): the colours `dot-theme` makes from
  the wallpaper, applied live when it changes. Without them, LazyVim's tokyonight.
- **The installed Suites' extras**, such as a language's LazyVim extra (the Node Suite's
  TypeScript and eslint), from `lua/kubaos/suites.lua`, which `home/suites.nix` writes.

## Make it yours

| To change | Put it in `~/.config/kubaos/editor/nvim/` |
|---|---|
| Options, keymaps, autocmds | `options.lua`, `keymaps.lua`, `autocmds.lua` |
| Add a plugin, change one's options, turn one off | `plugins/*.lua`, lazy.nvim specs |
| Turn on more LazyVim extras | `:LazyExtras` (kept in `lazyvim.json`) |
| Plugin versions | `:Lazy update` (kept in `lazy-lock.json`) |
| The colorscheme when there's no wallpaper theme | `require("config.theme").fallback = "<name>"` in a `plugins/*.lua` |

"Settings: Editor" in the Command menu opens that folder. `user-layer/editor/` holds the files
each new user starts with, and docs/CONTRACTS.md, "Defaults and the user layer", says how each
one loads.
