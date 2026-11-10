-- Material 3 colors for Hyprland.
--
-- The theme pipeline (dot-theme + matugen) writes
-- ~/.local/state/theme/hypr.lua, a Lua file returning a table with the keys
-- of docs/CONTRACTS.md ("primary", "outline_variant", ... values "#rrggbb").
-- The file may be missing (first boot, broken pipeline): every key falls
-- back to the defaults below, so Hyprland always gets valid colors.

local M = {}

-- Dark Material 3 fallback palette (the same as
-- config/matugen/default-colors.json: the default wallpaper's colors).
M.defaults = {
  primary = "#ffb86c",
  on_primary = "#492900",
  primary_container = "#c68231",
  secondary = "#e9bf96",
  tertiary = "#79d1fe",
  error = "#ffb4ab",
  background = "#18120d",
  surface = "#18120d",
  surface_container = "#251e18",
  surface_container_high = "#302922",
  surface_container_highest = "#3b332d",
  on_surface = "#ede0d6",
  on_surface_variant = "#d7c3b2",
  outline = "#9f8e7e",
  outline_variant = "#524437",
  shadow = "#000000",
  inverse_primary = "#895100",
}

M.path = (os.getenv("HOME") or "") .. "/.local/state/theme/hypr.lua"

-- Returns the merged palette. Never raises.
function M.load()
  local palette = {}
  for key, value in pairs(M.defaults) do
    palette[key] = value
  end

  local file = io.open(M.path, "r")
  if not file then
    return palette
  end
  file:close()

  local ok, generated = pcall(dofile, M.path)
  if ok and type(generated) == "table" then
    for key, value in pairs(generated) do
      if type(value) == "string" and value:match("^#?%x%x%x%x%x%x$") then
        palette[key] = value
      end
    end
  elseif not ok then
    hl.notification.create({ text = "theme hypr.lua: " .. tostring(generated), timeout = 6000, icon = "warning" })
  end

  return palette
end

-- "#rrggbb" + optional alpha "aa" -> "rgba(rrggbbaa)", the format Hyprland
-- color options accept.
function M.rgba(hex, alpha)
  return "rgba(" .. (hex:gsub("^#", "")) .. (alpha or "ff") .. ")"
end

return M
