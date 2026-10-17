return {
  -- Builds highlights for the wallpaper theme (lua/config/theme.lua).
  { "nvim-mini/mini.base16", lazy = true },

  -- Statusline in the pill style of tmux and the Quickshell bar: rounded caps
  -- (Nerd Font U+E0B4 / U+E0B6), the mode in the accent, the rest on the
  -- glass. Colors: lua/lualine/themes/matugen.lua.
  {
    "nvim-lualine/lualine.nvim",
    opts = function(_, opts)
      opts.options = opts.options or {}
      opts.options.theme = "matugen"
      opts.options.section_separators = { left = "\238\130\180", right = "\238\130\182" }
      opts.options.component_separators = { left = "", right = "" }
      return opts
    end,
  },

  -- Completion menu, docs and signature help as rounded cards, like every
  -- other float (vim.o.winborder in config/options.lua).
  {
    "saghen/blink.cmp",
    opts = {
      completion = {
        menu = { border = "rounded" },
        documentation = { window = { border = "rounded" } },
      },
      signature = { window = { border = "rounded" } },
    },
  },

  -- noice draws LSP hover and signature help itself; give them the same border.
  {
    "folke/noice.nvim",
    opts = { presets = { lsp_doc_border = true } },
  },
}
