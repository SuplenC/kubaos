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
  primary = "#ffb779",
  on_primary = "#4c2700",
  primary_container = "#d37b1b",
  secondary = "#f2bc8e",
  tertiary = "#85cfff",
  error = "#ffb4ab",
  background = "#19120b",
  surface = "#19120b",
  surface_container = "#261e17",
  surface_container_high = "#312821",
  surface_container_highest = "#3d332b",
  on_surface = "#f0dfd4",
  on_surface_variant = "#dac2b1",
  outline = "#a28d7d",
  outline_variant = "#544436",
  shadow = "#000000",
  inverse_primary = "#8e4e00",
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
