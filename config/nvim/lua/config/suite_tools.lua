-- Specs for a Suite whose language servers and tools come from Nix
-- (home/suites/*.lua). LazyVim's language extras ask Mason for them otherwise,
-- and Mason's copies would shadow the Suite's: Mason downloads its own, and
-- puts its bin/ first on Neovim's PATH.
-- https://github.com/mason-org/mason.nvim#configuration
-- https://www.lazyvim.org/plugins/lsp
local M = {}

-- The specs for a Suite's tools: `servers`, the lspconfig names LazyVim starts
-- from PATH instead of installing with Mason, and `packages`, the Mason
-- packages the extras ask for that the Suite installs itself.
---@param tools { servers?: string[], packages?: string[] }
function M.from_nix(tools)
  local servers = {}
  for _, name in ipairs(tools.servers or {}) do
    servers[name] = { mason = false }
  end
  local installed = {}
  for _, name in ipairs(tools.packages or {}) do
    installed[name] = true
  end
  return {
    { "neovim/nvim-lspconfig", opts = { servers = servers } },
    {
      "mason-org/mason.nvim",
      opts = function(_, opts)
        opts.ensure_installed = vim.tbl_filter(function(name)
          return not installed[name]
        end, opts.ensure_installed or {})
        -- A tool Mason installed before the Suite was keeps working, but the
        -- Suite's copy, on PATH, wins over it.
        opts.PATH = "append"
      end,
    },
  }
end

return M
