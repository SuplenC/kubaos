-- Plugin specs: every file in ~/.config/kubaos/editor/nvim/plugins/ is loaded after
-- KubaOS's specs, so it can add plugins, change their opts or turn them off, the same as
-- LazyVim's lua/plugins/*.lua. Turn on LazyVim extras with :LazyExtras (they are kept in
-- lazyvim.json beside this folder).
return {
  -- Turn a plugin off.
  -- { "folke/flash.nvim", enabled = false },
  --
  -- Change a plugin's options.
  -- { "nvim-lualine/lualine.nvim", opts = { options = { section_separators = "" } } },
  --
  -- Add a plugin.
  -- { "tpope/vim-surround" },
  --
  -- Full docs: https://lazy.folke.io/spec and https://www.lazyvim.org/configuration/plugins
}
