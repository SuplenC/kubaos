return {
  -- Filetype, syntax and ftdetect for .pest grammars (lazy.nvim sources the
  -- ftdetect of `ft` plugins at startup).
  { "pest-parser/pest.vim", ft = "pest" },
  {
    "neovim/nvim-lspconfig",
    opts = {
      servers = {
        -- LazyVim sets it up (vim.lsp.config/enable) and mason installs it.
        pest_ls = {},
      },
    },
  },
}
