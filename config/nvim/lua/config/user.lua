-- The user layer's Neovim folder, ~/.config/kubaos/editor/nvim
-- (docs/CONTRACTS.md, "Defaults and the user layer"). This config is
-- read-only defaults; the user's options, keymaps, autocmds and plugin specs
-- live there and run after ours. So do the files lazy.nvim and LazyVim write:
-- lazy-lock.json and lazyvim.json.
local M = {}

-- Neovim's folder in the user layer, grouped as crates/kubaos-user-layer and
-- home/user-layer.nix put it; this is Neovim's only copy of the path.
local FOLDER = "editor/nvim"

local config_home = vim.env.XDG_CONFIG_HOME
if not config_home or config_home == "" then
  config_home = vim.env.HOME .. "/.config"
end
M.dir = config_home .. "/kubaos/" .. FOLDER

local function load(file)
  local ok, result = pcall(dofile, file)
  if ok then
    return result
  end
  vim.schedule(function()
    vim.notify("Error in " .. file .. ":\n" .. tostring(result), vim.log.levels.ERROR)
  end)
end

-- run("options"): the user's options.lua, keymaps.lua or autocmds.lua, if any.
function M.run(name)
  local file = M.dir .. "/" .. name .. ".lua"
  if vim.uv.fs_stat(file) then
    load(file)
  end
end

-- Specs from every plugins/*.lua, for lazy.nvim, after ours.
function M.plugins()
  local specs = {}
  for _, file in ipairs(vim.fn.globpath(M.dir .. "/plugins", "*.lua", false, true)) do
    table.insert(specs, load(file))
  end
  return specs
end

return M
