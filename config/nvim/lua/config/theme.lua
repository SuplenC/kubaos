-- Wallpaper theme for Neovim.
--
-- dot-theme (matugen) writes ~/.local/state/theme/nvim.lua, a Lua file that
-- returns the design tokens' colour roles by name ({ primary = "#rrggbb",
-- surface = ..., cold = ..., warning = ... }), the terminal's hues, its dim
-- text, and the mode dot-theme ran in (generated from design/tokens.toml by
-- kubaos-tokens). The roles map onto a base16 palette, applied with
-- mini.base16 as the "matugen" colorscheme (colors/matugen.lua), in that mode. The file is watched and re-applied
-- on change; `:ThemeReload` does it by hand. Without the file the colorscheme is
-- M.fallback, LazyVim's own tokyonight. A plugin spec in the user layer can set
-- `require("config.theme").fallback` to another installed colorscheme.

local M = {}

M.fallback = "tokyonight"

local state_home = vim.env.XDG_STATE_HOME
if not state_home or state_home == "" then
  state_home = vim.fn.expand("~/.local/state")
end
M.dir = state_home .. "/theme"
M.file = M.dir .. "/nvim.lua"

local required = {
  "surface",
  "on_surface",
  "on_surface_variant",
  "surface_container",
  "surface_container_high",
  "outline",
  "primary",
  "secondary",
  "tertiary",
  "error",
  "on_primary_container",
  "on_secondary_container",
  "on_tertiary_container",
  "on_error_container",
}

--- Read the generated palette. Returns the table, or nil and a reason.
function M.read()
  if not (vim.uv or vim.loop).fs_stat(M.file) then
    return nil, "missing " .. M.file
  end
  local chunk, err = loadfile(M.file)
  if not chunk then
    return nil, err
  end
  setfenv(chunk, {}) -- the file is data only
  local ok, c = pcall(chunk)
  if not ok or type(c) ~= "table" then
    return nil, "invalid " .. M.file
  end
  for _, k in ipairs(required) do
    if type(c[k]) ~= "string" or not c[k]:match("^#%x%x%x%x%x%x$") then
      return nil, ("bad or missing color %q in %s"):format(k, M.file)
    end
  end
  return c
end

-- Syntax hues from the generated file (harmonized custom colors), falling
-- back to Material roles for older files: M.hues below.
-- Blend `fg` into `bg` by `a` (0..1), both "#rrggbb". Used for tinted
-- backgrounds (diffs, headings) that must stay dark enough for text.
function M.blend(fg, bg, a)
  local function ch(hex, i)
    return tonumber(hex:sub(i, i + 1), 16)
  end
  local out = {}
  for _, i in ipairs({ 2, 4, 6 }) do
    out[#out + 1] = math.floor(ch(fg, i) * a + ch(bg, i) * (1 - a) + 0.5)
  end
  return ("#%02x%02x%02x"):format(out[1], out[2], out[3])
end

M.hues = function(c)
  return {
    red = c.red or c.error,
    green = c.green or c.secondary,
    yellow = c.yellow or c.on_tertiary_container,
    blue = c.blue or c.primary,
    magenta = c.magenta or c.tertiary,
    cyan = c.cyan or c.on_primary_container,
  }
end

local function base16(c)
  local h = M.hues(c)
  return {
    base00 = c.surface, -- default background
    base01 = c.surface_container, -- lighter background (status, folds)
    base02 = c.surface_container_highest or c.surface_container_high, -- selection
    base03 = c.dim or c.outline, -- comments, invisibles
    base04 = c.on_surface_variant, -- dark foreground
    base05 = c.on_surface, -- default foreground
    base06 = c.on_surface,
    base07 = c.on_background or c.on_surface,
    base08 = h.red, -- errors, deletions, tags
    base09 = h.yellow, -- numbers, constants
    base0A = h.cyan, -- types
    base0B = h.green, -- strings
    base0C = c.tertiary, -- escapes, regex, support
    base0D = h.blue, -- functions
    base0E = h.magenta, -- keywords
    base0F = c.on_error_container, -- deprecated
  }
end

--- Apply the palette. Called from colors/matugen.lua. Errors if no palette.
function M.apply()
  local c, err = M.read()
  if not c then
    error(err)
  end
  local h = M.hues(c)
  -- Plugins pick their own colours by 'background'. Only set when it differs:
  -- setting it reloads the colorscheme.
  local background = c.mode == "light" and "light" or "dark"
  if vim.o.background ~= background then
    vim.o.background = background
  end
  require("mini.base16").setup({ palette = base16(c), use_cterm = nil })

  local hl = function(name, val)
    vim.api.nvim_set_hl(0, name, val)
  end
  -- base16 uses its "red" slot for identifiers/statements too; keep red for
  -- errors and let plain identifiers read as text.
  hl("Identifier", { fg = c.on_surface })
  hl("@variable", { fg = c.on_surface })
  hl("@variable.member", { fg = c.on_surface_variant })
  hl("@property", { fg = c.on_surface_variant })
  hl("@variable.parameter", { fg = c.on_surface, italic = true })
  hl("Statement", { fg = h.magenta })
  hl("@keyword.return", { fg = h.magenta, italic = true })
  hl("@keyword.function", { fg = h.magenta })
  hl("@lsp.type.lifetime", { fg = h.yellow, italic = true })
  hl("@attribute", { fg = c.tertiary })
  hl("Character", { fg = h.green })
  hl("@module", { fg = c.on_surface_variant })
  hl("@punctuation", { fg = c.outline })
  hl("Delimiter", { fg = c.outline })
  hl("Operator", { fg = h.cyan })
  -- The terminal's dim text, so a comment reads as faded as bat's; outline,
  -- which older palettes fall back to, is a graphic's colour.
  hl("Comment", { fg = c.dim or c.outline, italic = true })
  -- Functions take the wallpaper's accent, so code and chrome share a hue
  -- (amber on Shinjuku, blue at Tokyo Tower); keywords keep the magenta.
  hl("Function", { fg = c.primary })
  hl("@function", { fg = c.primary })
  hl("@function.call", { fg = c.primary })
  hl("@function.method", { fg = c.primary })
  hl("@function.method.call", { fg = c.primary })
  hl("@lsp.type.method", { fg = c.primary })
  hl("@lsp.type.function", { fg = c.primary })
  hl("Macro", { fg = c.tertiary })
  hl("@function.macro", { fg = c.tertiary })
  hl("@lsp.type.macro", { fg = c.tertiary })

  -- Transparent editor: Ghostty draws the background at 85% opacity and
  -- Hyprland blurs what is behind it. Anything Neovim paints is opaque, so
  -- the main surfaces stay unpainted and only floats, selections and diffs
  -- get a background.
  local none = "NONE"
  for _, name in ipairs({
    "Normal",
    "NormalNC",
    "NormalSB",
    "SignColumn",
    "FoldColumn",
    "EndOfBuffer",
    "LineNr",
    "LineNrAbove",
    "LineNrBelow",
    "MsgArea",
    "TabLineFill",
    "WinBar",
    "WinBarNC",
    "SnacksDashboardNormal",
  }) do
    local cur = vim.api.nvim_get_hl(0, { name = name, link = false })
    cur.bg = nil
    cur.ctermbg = nil
    vim.api.nvim_set_hl(0, name, vim.tbl_extend("force", cur, { bg = none }))
  end
  hl("EndOfBuffer", { fg = c.surface, bg = none })

  -- Editor chrome: the accent where the cursor is, quiet elsewhere.
  local low = c.surface_container_low or c.surface_container
  local high = c.surface_container_high
  local highest = c.surface_container_highest or high
  hl("CursorLine", { bg = low })
  hl("CursorLineNr", { fg = c.primary, bold = true })
  hl("CursorLineSign", { bg = none })
  hl("CursorLineFold", { bg = none })
  hl("LineNr", { fg = c.outline_variant or c.outline, bg = none })
  -- A selection is a selected row's tinted bar under the text, as in the terminal.
  hl("Visual", { bg = c.secondary_container, fg = c.on_secondary_container })
  hl("Search", { bg = c.tertiary_container or high, fg = c.on_tertiary_container })
  hl("IncSearch", { bg = c.tertiary, fg = c.on_tertiary or c.surface })
  hl("CurSearch", { bg = c.tertiary, fg = c.on_tertiary or c.surface })
  hl("MatchParen", { fg = c.primary, bold = true, underline = true })
  hl("WarningMsg", { fg = h.yellow })
  hl("WinSeparator", { fg = highest, bg = none })
  hl("StatusLine", { fg = c.on_surface_variant, bg = none })
  hl("StatusLineNC", { fg = c.outline, bg = none })
  hl("Folded", { fg = c.on_surface_variant, bg = low })
  hl("NonText", { fg = c.outline_variant or c.outline })
  hl("Whitespace", { fg = c.outline_variant or c.outline })
  hl("LspInlayHint", { fg = c.outline, bg = none, italic = true })

  -- Floats are cards on the glass: opaque, one step above the surface, with a
  -- rounded border (vim.o.winborder in options.lua) in the outline color.
  local float = c.surface_container
  hl("NormalFloat", { fg = c.on_surface, bg = float })
  hl("FloatBorder", { fg = c.outline_variant or c.outline, bg = float })
  hl("FloatTitle", { fg = c.primary, bg = float, bold = true })
  hl("FloatFooter", { fg = c.outline, bg = float })
  hl("Pmenu", { fg = c.on_surface, bg = float })
  hl("PmenuSel", { fg = c.on_secondary_container, bg = c.secondary_container, bold = true })
  hl("PmenuSbar", { bg = high })
  hl("PmenuThumb", { bg = c.outline_variant or c.outline })
  hl("BlinkCmpMenuBorder", { link = "FloatBorder" })
  hl("BlinkCmpDocBorder", { link = "FloatBorder" })
  hl("BlinkCmpSignatureHelpBorder", { link = "FloatBorder" })
  hl("BlinkCmpLabelMatch", { fg = c.primary, bold = true })

  -- Diagnostics: one hue per severity everywhere (text, sign, undercurl,
  -- virtual text, statusline counter). Harmonized red, not M3 `error`, so
  -- errors and warnings don't both read as "warm" on amber wallpapers.
  local sev = { Error = h.red, Warn = h.yellow, Info = h.blue, Hint = h.cyan, Ok = h.green }
  for name, col in pairs(sev) do
    hl("Diagnostic" .. name, { fg = col })
    hl("DiagnosticSign" .. name, { fg = col, bg = none })
    hl("DiagnosticFloating" .. name, { fg = col })
    hl("DiagnosticUnderline" .. name, { sp = col, undercurl = true })
    hl("DiagnosticVirtualText" .. name, { fg = col, bg = M.blend(col, c.surface, 0.1), italic = true })
    hl("DiagnosticVirtualLines" .. name, { fg = col })
  end
  hl("DiagnosticUnnecessary", { fg = c.outline, italic = true })

  -- Diffs (fugitive, diffview, gitsigns previews): tinted line backgrounds,
  -- strong enough to see at a glance, dark enough to keep syntax colors.
  local s = c.surface
  hl("DiffAdd", { bg = M.blend(h.green, s, 0.16) })
  hl("DiffDelete", { fg = M.blend(h.red, s, 0.55), bg = M.blend(h.red, s, 0.13) })
  hl("DiffChange", { bg = M.blend(h.blue, s, 0.15) })
  hl("DiffText", { bg = M.blend(h.blue, s, 0.35), bold = true })
  hl("Added", { fg = h.green })
  hl("Removed", { fg = h.red })
  hl("Changed", { fg = h.blue })
  hl("diffAdded", { fg = h.green })
  hl("diffRemoved", { fg = h.red })
  hl("diffChanged", { fg = h.blue })
  hl("GitSignsAdd", { fg = h.green, bg = none })
  hl("GitSignsChange", { fg = h.blue, bg = none })
  hl("GitSignsDelete", { fg = h.red, bg = none })
  hl("DiffviewDiffAddAsDelete", { bg = M.blend(h.red, s, 0.13) })
  hl("DiffviewDiffDeleteDim", { fg = c.outline_variant or c.outline })

  -- Plugins that would otherwise pick the info blue or their own accent.
  hl("SnacksDashboardHeader", { fg = c.primary })
  hl("SnacksDashboardIcon", { fg = c.primary })
  hl("SnacksDashboardKey", { fg = c.tertiary })
  hl("SnacksDashboardDesc", { fg = c.on_surface })
  hl("SnacksDashboardFooter", { fg = c.outline })
  hl("NoiceCmdlinePopup", { link = "NormalFloat" })
  hl("NoiceCmdlinePopupBorder", { fg = c.primary, bg = float })
  hl("NoiceCmdlinePopupTitle", { fg = c.primary, bg = float, bold = true })
  hl("NoiceCmdlineIcon", { fg = c.primary })
  hl("NoiceCmdlinePopupBorderSearch", { fg = c.tertiary, bg = float })
  hl("NoiceCmdlineIconSearch", { fg = c.tertiary })
  hl("WhichKeyNormal", { link = "NormalFloat" })
  hl("WhichKeyBorder", { link = "FloatBorder" })
  hl("WhichKeyTitle", { link = "FloatTitle" })
  hl("WhichKey", { fg = c.primary })
  hl("WhichKeyGroup", { fg = c.tertiary })
  hl("WhichKeyDesc", { fg = c.on_surface })
  hl("WhichKeySeparator", { fg = c.outline })
  hl("SnacksPickerBorder", { link = "FloatBorder" })
  hl("SnacksPickerTitle", { link = "FloatTitle" })
  hl("SnacksPickerListCursorLine", { bg = highest })
  hl("SnacksPickerMatch", { fg = c.primary, bold = true })
  hl("SnacksIndent", { fg = c.surface_container_high })
  hl("SnacksIndentScope", { fg = c.outline_variant or c.outline })

  -- Markdown: headings in the accent roles, a faint band behind each.
  local heads = { c.primary, c.tertiary, c.secondary, h.green, h.blue, h.magenta }
  for i, col in ipairs(heads) do
    hl("@markup.heading." .. i, { fg = col, bold = true })
    hl("@markup.heading." .. i .. ".markdown", { fg = col, bold = true })
    hl("RenderMarkdownH" .. i, { fg = col, bold = true })
    hl("RenderMarkdownH" .. i .. "Bg", { fg = col, bg = M.blend(col, s, i <= 2 and 0.16 or 0.10), bold = true })
  end
  hl("RenderMarkdownCode", { bg = c.surface_container_low or c.surface_container })
  hl("RenderMarkdownCodeInline", { fg = c.tertiary, bg = c.surface_container })
  hl("@markup.raw", { fg = c.tertiary })
  hl("@markup.link", { fg = c.tertiary, underline = true })
end

local function apply_or_fallback()
  if M.read() then
    vim.cmd.colorscheme("matugen")
  elseif vim.g.colors_name ~= M.fallback then
    if not pcall(vim.cmd.colorscheme, M.fallback) then
      vim.cmd.colorscheme("habamax")
    end
  end
end

function M.reload()
  local _, err = M.read()
  if err then
    vim.notify("Theme not reloaded: " .. err, vim.log.levels.WARN)
    return
  end
  vim.cmd.colorscheme("matugen")
end

local watcher, timer
local function watch()
  local uv = vim.uv or vim.loop
  if watcher then
    return
  end
  -- Watch the directory: generators often replace the file (new inode).
  vim.fn.mkdir(M.dir, "p")
  watcher = uv.new_fs_event()
  timer = uv.new_timer()
  if not watcher or not timer then
    return
  end
  local ok = watcher:start(M.dir, {}, function(err, fname)
    if err or (fname and fname ~= "nvim.lua") then
      return
    end
    timer:stop()
    timer:start(150, 0, function()
      vim.schedule(function()
        if M.read() then
          vim.cmd.colorscheme("matugen")
        end
      end)
    end)
  end)
  if not ok then
    watcher:close()
    watcher = nil
  end
end

--- Entry point (LazyVim `colorscheme` option).
function M.load()
  if not M.loaded then
    M.loaded = true
    vim.api.nvim_create_user_command("ThemeReload", M.reload, { desc = "Reload wallpaper theme" })
    watch()
  end
  apply_or_fallback()
end

return M
