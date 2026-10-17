-- Autocmds are automatically loaded on the VeryLazy event
-- Default autocmds that are always set: https://github.com/LazyVim/LazyVim/blob/main/lua/lazyvim/config/autocmds.lua
-- Add any additional autocmds here

-- LazyVim turns spell checking on for markdown; in READMEs full of names
-- (Yamanote, nextest, dotfiles) the red curls are mostly noise. Keep it for
-- commit messages and plain text.
vim.api.nvim_create_autocmd("FileType", {
  group = vim.api.nvim_create_augroup("dot_markdown_nospell", { clear = true }),
  pattern = { "markdown" },
  callback = function()
    vim.opt_local.spell = false
  end,
})
