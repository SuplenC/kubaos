-- Neovim for the terminals' shots: the tree's theme (config/nvim's colors/matugen.lua,
-- lua/config/theme.lua and lualine's theme) with mini.base16 and lualine from the harness, the
-- options LazyVim sets that show colour, and a selection, a search and a float on screen.
-- LazyVim itself isn't loaded: it fetches its plugins.

local config = vim.env.HOME .. "/.config/nvim"
vim.opt.rtp:prepend(config)
for dir in vim.gsplit(vim.env.KUBAOS_SHOTS_NVIM_PLUGINS, ":", { plain = true }) do
  vim.opt.rtp:prepend(dir)
end

vim.o.termguicolors = true
vim.o.number = true
vim.o.relativenumber = true
vim.o.cursorline = true
vim.o.signcolumn = "yes"
vim.o.laststatus = 3
vim.o.showmode = false
vim.o.winborder = "rounded"
vim.o.hlsearch = true

require("config.theme").load()
require("lualine").setup({ options = { theme = "matugen", globalstatus = true } })

vim.api.nvim_create_autocmd("VimEnter", {
  callback = function()
    vim.treesitter.start()
    vim.fn.setreg("/", "colour")
    local doc = vim.api.nvim_create_buf(false, true)
    vim.api.nvim_buf_set_lines(doc, 0, -1, false, { " M.apply(palette)", " Sets each highlight of the palette. " })
    vim.api.nvim_open_win(doc, false, {
      relative = "editor",
      row = 4,
      col = 44,
      width = 40,
      height = 2,
      title = " theme.lua ",
      style = "minimal",
    })
    vim.api.nvim_feedkeys("19GVjj", "n", false)
  end,
})
