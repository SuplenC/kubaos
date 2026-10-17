return {
  {
    "neovim/nvim-lspconfig",
    opts = {
      servers = {
        jinja_lsp = {
          filetypes = { "jinja", "jinja2", "html" },
        },
      },

      setup = {},
    },
  },
  {
    "stevearc/conform.nvim",
    opts = {
      formatters = {
        djlint = {},
      },

      formatters_by_ft = {
        -- html = { "djlint" },
        htmldjango = { "djlint" },
        jinja = { "djlint" },
        ["jinja.html"] = { "djlint" }, -- Use quotes for filetypes with dots

        -- You can add other filetypes djlint supports if desired
        -- css = { "djlint" }, -- If you want djlint to format CSS
        -- javascript = { "djlint" }, -- If you want djlint to format JS

        -- Ensure you keep other formatters for these filetypes if needed
        -- Example: Prettier might also be used for HTML/CSS/JS
        -- html = { "prettier", "djlint" }, -- Runs prettier first, then djlint

        lua = { "stylua" },
        python = { "isort", "black" },
      },
    },
  },
}
