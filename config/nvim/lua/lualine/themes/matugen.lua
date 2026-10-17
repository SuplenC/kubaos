-- lualine theme from the wallpaper palette (lua/config/theme.lua), so the
-- mode pill matches the tmux session pill and the Quickshell bar: the mode in
-- a filled accent pill, the branch section on a raised surface, the rest on
-- the editor's (transparent) background. lualine reloads its theme on
-- ColorScheme, so `dot-theme` changes apply live. Falls back to "auto"
-- without a generated palette.
local c = require("config.theme").read()
if not c then
  return require("lualine.themes.auto")
end

local none = "None"
local high = c.surface_container_high

local function mode(bg, fg)
  return {
    a = { bg = bg, fg = fg, gui = "bold" },
    b = { bg = high, fg = c.on_surface },
    c = { bg = none, fg = c.on_surface_variant },
  }
end

return {
  normal = mode(c.primary, c.on_primary),
  insert = mode(c.tertiary, c.on_tertiary),
  visual = mode(c.secondary, c.on_secondary),
  replace = mode(c.error, c.on_error),
  command = mode(c.primary_container, c.on_primary_container),
  terminal = mode(c.tertiary_container, c.on_tertiary_container),
  inactive = {
    a = { bg = c.surface_container, fg = c.outline, gui = "bold" },
    b = { bg = c.surface_container, fg = c.outline },
    c = { bg = none, fg = c.outline },
  },
}
