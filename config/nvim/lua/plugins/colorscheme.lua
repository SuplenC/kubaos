return {
  {
    "loctvl842/monokai-pro.nvim",
    lazy = true,
    opts = {
      filter = "pro",
      transparent_background = true,
    },
  },
  {
    "khoido2003/monokai-v2.nvim",
    lazy = true,
    opts = {
      filter = "pro",
      transparent_background = true,
    },
  },
  {
    "rebelot/kanagawa.nvim",
    lazy = true,
  },
  {
    "rose-pine/neovim",
    name = "rose-pine",
    lazy = true,
  },
  {
    "sainnhe/sonokai",
    lazy = true,
    priority = 1000,
    config = function()
      vim.g.sonokai_style = "shusia"
    end,
  },
  {
    "LazyVim/LazyVim",
    opts = {
      -- colorscheme = "sonokai",
      -- colorscheme = "monokai-pro",
      -- colorscheme = "monokai-v2",
      -- Wallpaper theme (~/.local/state/theme/nvim.lua), falls back to monokai-v2.
      -- See lua/config/theme.lua.
      colorscheme = function()
        require("config.theme").load()
      end,
    },
  },
}
