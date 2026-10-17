return {
  "sindrets/diffview.nvim",
  cmd = {
    "DiffviewOpen",
    "DiffviewClose",
    "DiffviewToggleFiles",
    "DiffviewFocusFiles",
    "DiffviewFileHistory",
  },
  keys = {
    { "<leader>gV", "<cmd>DiffviewOpen<cr>", desc = "Diffview Open" },
    { "<leader>gX", "<cmd>DiffviewClose<cr>", desc = "Diffview Close" },
    { "<leader>gH", "<cmd>DiffviewFileHistory %<cr>", desc = "File History (Diffview)" },
    { "<leader>gR", "<cmd>DiffviewFileHistory<cr>", desc = "Repo History (Diffview)" },
  },
  opts = {
    enhanced_diff_hl = true, -- better highlighting of changed text
    view = {
      default = { layout = "diff2_horizontal" },
      merge_tool = {
        layout = "diff3_mixed", -- OURS | THEIRS on top, MERGED below
        disable_diagnostics = true, -- no LSP noise in conflicted files
        winbar_info = true, -- show which side is which
      },
      file_history = { layout = "diff2_horizontal" },
    },
    file_panel = {
      listing_style = "tree",
      win_config = { position = "left", width = 35 },
    },
    keymaps = {
      view = { { "n", "q", "<cmd>DiffviewClose<cr>", { desc = "Close Diffview" } } },
      file_panel = { { "n", "q", "<cmd>DiffviewClose<cr>", { desc = "Close Diffview" } } },
      file_history_panel = { { "n", "q", "<cmd>DiffviewClose<cr>", { desc = "Close Diffview" } } },
    },
  },
}
