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
  primary = "#ffb3ae",
  on_primary = "#68000c",
  primary_container = "#db2b34",
  secondary = "#ffb3ae",
  tertiary = "#78d3ed",
  error = "#ffb4ab",
  background = "#1e0f0e",
  surface = "#1e0f0e",
  surface_container = "#2c1b1a",
  surface_container_high = "#372524",
  surface_container_highest = "#43302f",
  on_surface = "#fadbd9",
  on_surface_variant = "#e5bdba",
  outline = "#ac8886",
  outline_variant = "#5c403e",
  shadow = "#000000",
  inverse_primary = "#bd0e22",
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
