-- The Python Suite's Neovim settings, part of the Defaults while the Suite is
-- installed (home/suites/python.nix): LazyVim's Python extra, run on the
-- basedpyright, ruff and debugpy mise installs for the Suite, which Neovim
-- finds on PATH: mise's own entries in a shell, its shims when started from
-- the launcher. nvim-dap-python starts the `debugpy-adapter` on PATH, which
-- runs the program on the project's venv with its own copy of debugpy, so the
-- venv needs none.
local tools = require("config.suite_tools")

local packages = { "debugpy" }

-- The extra reads this when it is imported, after the user layer's
-- options.lua, so `vim.g.lazyvim_python_lsp = "pyright"` there still wins.
vim.g.lazyvim_python_lsp = vim.g.lazyvim_python_lsp or "basedpyright"

return {
  { import = "lazyvim.plugins.extras.lang.python" },
  tools.from_nix({ servers = { "basedpyright", "ruff" }, packages = packages }),
  -- mason-nvim-dap installs the debug adapters dap is set up with unless told
  -- not to, and debugpy is the Suite's.
  {
    "jay-babu/mason-nvim-dap.nvim",
    optional = true,
    opts = { automatic_installation = { exclude = packages } },
  },
}
