-- Options are automatically loaded before lazy.nvim startup
-- Default options that are always set: https://github.com/LazyVim/LazyVim/blob/main/lua/lazyvim/config/options.lua
-- Add any additional options here
vim.opt.scrolloff = 10
vim.g.snacks_animate = false
vim.opt.diffopt:append({ "algorithm:histogram", "indent-heuristic" })
-- noice shows 'showcmd' (pending keys and counts) in lualine and never clears
-- it, so the last keys typed ("36G") stay there. which-key shows pending keys.
vim.opt.showcmd = false
-- Rounded borders on every float (hover, signature help, blink.cmp menus,
-- diagnostics), matching the shell's rounded panels. Colors: config/theme.lua.
vim.o.winborder = "rounded"
