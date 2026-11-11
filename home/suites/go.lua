-- The Go Suite's Neovim settings, part of the Defaults while the Suite is
-- installed (home/suites/go.nix): LazyVim's Go extra, run on the gopls,
-- gofumpt, goimports, golangci-lint and delve mise installs for the Suite,
-- which Neovim finds on PATH: mise's own entries in a shell, its shims when
-- started from the launcher.
local tools = require("config.suite_tools")

local packages = { "delve", "gofumpt", "goimports", "golangci-lint" }

return {
  { import = "lazyvim.plugins.extras.lang.go" },
  tools.from_nix({ servers = { "gopls" }, packages = packages }),
  -- mason-nvim-dap installs the debug adapters dap is set up with unless told
  -- not to, and delve is the Suite's.
  {
    "jay-babu/mason-nvim-dap.nvim",
    optional = true,
    opts = { automatic_installation = { exclude = packages } },
  },
}
