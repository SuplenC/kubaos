-- The Node Suite's Neovim settings, part of the Defaults while the Suite is
-- installed (home/suites/node.nix): LazyVim's TypeScript extra, run on the
-- Suite's vtsls, eslint server, prettier and js-debug.
local tools = require("config.suite_tools")

return {
  { import = "lazyvim.plugins.extras.lang.typescript" },
  tools.from_nix({
    servers = { "vtsls", "eslint" },
    packages = { "js-debug-adapter", "prettier" },
  }),
  {
    "mfussenegger/nvim-dap",
    optional = true,
    opts = function()
      -- The extra launches a TypeScript file through tsx or ts-node, and the
      -- Suite installs neither: Node runs TypeScript itself, stripping the
      -- types, so a missing runner is left out and Node runs the file.
      for _, language in ipairs({ "typescript", "typescriptreact" }) do
        for _, config in ipairs(require("dap").configurations[language] or {}) do
          local runner = config.runtimeExecutable
          if runner and vim.fn.executable(runner) == 0 then
            config.runtimeExecutable = nil
          end
        end
      end
    end,
  },
}
