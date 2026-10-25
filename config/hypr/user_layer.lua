-- Where each app's settings live in the user layer, ~/.config/kubaos, grouped
-- by area (docs/CONTRACTS.md, "Defaults and the user layer"). The modules
-- read the user's files through `file()`, never by a path of their own.
--
-- The same table in the other languages: crates/kubaos-user-layer (Rust),
-- home/user-layer.nix (Nix and dot-config). test/lua/user_layer_test.lua
-- holds this one to the seeds in user-layer/.

local M = {}

-- Each app's folder relative to the user layer.
M.apps = {
  hypr = "desktop/hypr",
  quickshell = "desktop/quickshell",
  cursor = "desktop/cursor",
  clock = "desktop/clock",
  keyboard = "desktop/keyboard",
  sysmon = "desktop/sysmon",
  files = "desktop/files",
  wallpapers = "desktop/wallpapers",
  ghostty = "terminal/ghostty",
  foot = "terminal/foot",
  tmux = "terminal/tmux",
  zsh = "terminal/zsh",
  fastfetch = "terminal/fastfetch",
  nvim = "editor/nvim",
  git = "tools/git",
  lazygit = "tools/lazygit",
  yazi = "tools/yazi",
  bat = "tools/bat",
  bacon = "tools/bacon",
  fuzzel = "tools/fuzzel",
  kotodama = "menu/kotodama",
}

-- The user layer: $XDG_CONFIG_HOME/kubaos, or ~/.config/kubaos.
function M.root()
  local config_home = os.getenv("XDG_CONFIG_HOME")
  if not config_home or config_home == "" then
    config_home = (os.getenv("HOME") or "") .. "/.config"
  end
  return config_home .. "/kubaos"
end

-- file("keyboard", "keyboard.toml"): that file in the app's folder.
function M.file(app, name)
  return M.root() .. "/" .. assert(M.apps[app], "no app " .. tostring(app) .. " in the user layer") .. "/" .. name
end

return M
