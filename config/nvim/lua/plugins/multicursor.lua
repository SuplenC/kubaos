return {
  -- Lazy-loaded on its main key (lazy.lua sets defaults.lazy = true, so a
  -- spec without a trigger never loads). VM maps the rest of its keys on load.
  {
    "mg979/vim-visual-multi",
    keys = { { "<C-n>", mode = { "n", "x" }, desc = "Multi-cursor: select word" } },
  },
}
