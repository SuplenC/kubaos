return {
  {
    "mrcjkb/rustaceanvim",
    opts = function(_, opts)
      -- 1. Completely replace the 'ignored' table.
      -- opts.server.default_settings["rust-analyzer"].procMacro.ignored = vim.empty_dict()
      opts.server = opts.server or {}
      opts.server.default_settings = opts.server.default_settings or {}

      local my_settings = {
        ["rust-analyzer"] = {
          check = {
            command = "check",
          },
          cargo = {
            allFeatures = true,
            allTargets = false,
            loadOutDirsFromCheck = true,
            buildScripts = {
              enable = true,
            },
          },
          checkOnSave = false,
          diagnostics = {
            enable = true,
            disabled = vim.empty_dict(),
          },
          procMacro = {
            enable = true,
          },
        },
      }

      opts.server.default_settings = vim.tbl_deep_extend("force", opts.server.default_settings, my_settings)

      -- 2. Set your on_attach function. This will replace the default one.
      opts.server.on_attach = function(_, bufnr)
        vim.keymap.set("n", "<leader>cR", function()
          vim.cmd.RustLsp("codeAction")
        end, { desc = "Code Action", buffer = bufnr })
        vim.keymap.set("n", "<leader>dr", function()
          vim.cmd.RustLsp("debuggables")
        end, { desc = "Rust Debuggables", buffer = bufnr })
        vim.keymap.set("n", "<leader>cb", function()
          vim.cmd.RustLsp("flyCheck")
        end, { desc = "Run Clippy", buffer = bufnr })
        vim.keymap.set("n", "<leader>ce", function()
          vim.cmd.RustLsp("expandMacro")
        end, { desc = "Expand Macro Recursively", buffer = bufnr })
      end
      return opts
    end,
  },
}
