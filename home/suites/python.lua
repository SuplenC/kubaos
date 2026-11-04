-- The Python Suite's Neovim settings, part of the Defaults while the Suite is
-- installed (home/suites/python.nix): LazyVim's Python extra, run on the
-- Suite's basedpyright, ruff and debugpy.
local tools = require("config.suite_tools")

-- The extra reads this when it is imported, after the user layer's
-- options.lua, so `vim.g.lazyvim_python_lsp = "pyright"` there still wins.
vim.g.lazyvim_python_lsp = vim.g.lazyvim_python_lsp or "basedpyright"

return {
  { import = "lazyvim.plugins.extras.lang.python" },
  tools.from_nix({ servers = { "basedpyright", "ruff" } }),
}
