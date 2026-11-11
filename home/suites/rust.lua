-- The Rust Suite's Neovim settings, part of the Defaults while the Suite is
-- installed (home/suites/rust.nix): LazyVim's Rust extra, run on rustup's
-- rust-analyzer and the codelldb mise installs for the Suite, which Neovim
-- finds on PATH through the Suite's links in ~/.nix-profile/bin.
-- https://github.com/mrcjkb/rustaceanvim#books-usage--features
local tools = require("config.suite_tools")

local packages = { "codelldb" }

-- codelldb with the LLDB its extension carries, two folders up from the
-- adapter the link on PATH points at.
local function codelldb_adapter()
  local codelldb = vim.fn.exepath("codelldb")
  if codelldb == "" then
    return false
  end
  local extension = vim.fs.dirname(vim.fs.dirname(vim.fn.resolve(codelldb)))
  local liblldb = vim.fs.joinpath(extension, "lldb", "lib", "liblldb.so")
  return require("rustaceanvim.config").get_codelldb_adapter(codelldb, liblldb)
end

return {
  { import = "lazyvim.plugins.extras.lang.rust" },
  tools.from_nix({ packages = packages }),
  -- LazyVim's Rust extra points the adapter at Mason's LLDB unless
  -- vim.g.rustaceanvim names one already, which it keeps.
  {
    "mrcjkb/rustaceanvim",
    optional = true,
    init = function()
      vim.g.rustaceanvim = vim.tbl_deep_extend("keep", vim.g.rustaceanvim or {}, {
        dap = { adapter = codelldb_adapter },
      })
    end,
  },
  -- mason-nvim-dap installs the debug adapters dap is set up with unless told
  -- not to, and codelldb is the Suite's.
  {
    "jay-babu/mason-nvim-dap.nvim",
    optional = true,
    opts = { automatic_installation = { exclude = packages } },
  },
}
