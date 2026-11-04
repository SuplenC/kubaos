-- Your settings for the Go Suite. KubaOS created this file when the Suite was
-- installed and never writes to it again. It loads after the Suite's own
-- settings (the Defaults: LazyVim's Go extra), so what you set here wins.
return {
  -- gopls's settings: gofmt's layout instead of gofumpt's, and no parameter-name hints.
  -- {
  --   "neovim/nvim-lspconfig",
  --   opts = { servers = { gopls = { settings = { gopls = { gofumpt = false, hints = { parameterNames = false } } } } } },
  -- },
  --
  -- Every setting: https://go.dev/gopls/settings
}
