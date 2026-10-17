return {
  {
    "folke/snacks.nvim",
    opts = {
      picker = {
        include = {
          "**/.env",
          "**/.env.*",
          ".env",
          ".env.*",

          "**/.cargo",
          "**/.cargo/*",
          "**/config.toml",
        },
        exclude = {},
      },
      -- lazygit inside Neovim: snacks writes a theme from the highlights and
      -- loads it after ~/.local/state/theme/lazygit.yml. Keep the selected
      -- row the tinted bar the standalone theme uses, not Visual's accent.
      lazygit = {
        theme = {
          selectedLineBgColor = { bg = "PmenuSel" },
          inactiveViewSelectedLineBgColor = { bg = "CursorLine" },
        },
      },
    },
  },
}
