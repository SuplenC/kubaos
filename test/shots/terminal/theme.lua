-- A fixture for the terminals' shots: Lua, which Neovim highlights with its own parser.
local M = {}

M.fallback = "habamax"

--- Blend `fg` into `bg` by `a`, both "#rrggbb".
function M.blend(fg, bg, a)
  local function channel(colour, at)
    return tonumber(colour:sub(at, at + 1), 16)
  end
  local out = {}
  for _, at in ipairs({ 2, 4, 6 }) do
    out[#out + 1] = math.floor(channel(fg, at) * a + channel(bg, at) * (1 - a) + 0.5)
  end
  return ("#%02x%02x%02x"):format(out[1], out[2], out[3])
end

function M.apply(palette)
  if not palette then
    error("no palette")
  end
  for name, colour in pairs(palette) do
    vim.api.nvim_set_hl(0, name, { fg = colour, bold = name == "Title" })
  end
  return true
end

return M
