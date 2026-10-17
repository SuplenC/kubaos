return {
  {
    "nvim-telescope/telescope.nvim",
    dependencies = {
      -- "Myzel394/easytables.nvim",
      -- "Myzel394/telescope-last-positions",
      -- Other dependencies
      -- ..
      "Myzel394/jsonfly.nvim",
    },
    keys = {
      {
        "<leader>fj",
        "<cmd>Telescope jsonfly<cr>",
        desc = "Search in json",
        ft = { "json", "xml", "yaml" },
        mode = "n",
      },
    },
  },
}
