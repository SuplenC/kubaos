-- Session environment. hl.env() sets the variable in Hyprland's own
-- environment, so every process it spawns inherits it. Hyprland also exports
-- these to systemd --user and D-Bus activation (see autostart.lua).

local home = os.getenv("HOME") or ""

-- PATH: dot-* scripts and Nix tools must be reachable from binds even if
-- Hyprland was started from a context that skipped the login shell.
do
  local wanted = {
    home .. "/.local/bin",
    home .. "/.nix-profile/bin",
    "/nix/var/nix/profiles/default/bin",
  }
  local current = os.getenv("PATH") or "/usr/local/bin:/usr/bin:/bin"
  local seen, parts = {}, {}
  for _, dir in ipairs(wanted) do
    if not seen[dir] then
      seen[dir] = true
      table.insert(parts, dir)
    end
  end
  for dir in current:gmatch("[^:]+") do
    if not seen[dir] then
      seen[dir] = true
      table.insert(parts, dir)
    end
  end
  hl.env("PATH", table.concat(parts, ":"))
end

-- XDG / session identity (portals, screen sharing).
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
hl.env("XDG_SESSION_DESKTOP", "Hyprland")
hl.env("XDG_SESSION_TYPE", "wayland")

-- Toolkit backends: Wayland first, X11 as fallback.
hl.env("GDK_BACKEND", "wayland,x11,*")
hl.env("QT_QPA_PLATFORM", "wayland;xcb")
hl.env("SDL_VIDEODRIVER", "wayland,x11")
hl.env("CLUTTER_BACKEND", "wayland")
hl.env("MOZ_ENABLE_WAYLAND", "1")

-- Qt: no client-side decorations, scale from the compositor, colors from
-- qt6ct (theme pipeline writes ~/.local/state/theme/qt6ct.colors).
hl.env("QT_WAYLAND_DISABLE_WINDOWDECORATION", "1")
hl.env("QT_AUTO_SCREEN_SCALE_FACTOR", "1")
hl.env("QT_QPA_PLATFORMTHEME", "qt6ct")

-- Electron and Chromium: native Wayland when available. Chrome 140+ picks
-- Wayland on its own; Electron apps read the hint. NIXOS_OZONE_WL makes the
-- nixpkgs wrappers of Electron apps add the Wayland flags.
hl.env("ELECTRON_OZONE_PLATFORM_HINT", "auto")
hl.env("NIXOS_OZONE_WL", "1")

-- Java (IntelliJ and friends) on tiling WMs.
hl.env("_JAVA_AWT_WM_NONREPARENTING", "1")

-- Cursor: KubaOS-Bibata, the recoloured Bibata `kubaos cursor` writes (and
-- home/theme.nix names for every other client), at the size in the user's
-- ~/.config/kubaos/cursor/cursor.toml, which the Command menu sets. The
-- classic style is the same name: its theme inherits Bibata-Modern-Classic.
do
  local size = "24"
  local config = os.getenv("XDG_CONFIG_HOME")
  if not config or config == "" then
    config = home .. "/.config"
  end
  local file = io.open(config .. "/kubaos/cursor/cursor.toml", "r")
  if file then
    for line in file:lines() do
      -- The range `kubaos cursor` takes (crates/kubaos-cursor/src/settings.rs).
      local value = line:match("^%s*size%s*=%s*(%d+)%s*$")
      if value and tonumber(value) >= 16 and tonumber(value) <= 96 then
        size = value
        break
      end
    end
    file:close()
  end
  hl.env("XCURSOR_THEME", "KubaOS-Bibata")
  hl.env("HYPRCURSOR_THEME", "KubaOS-Bibata")
  hl.env("XCURSOR_SIZE", size)
  hl.env("HYPRCURSOR_SIZE", size)
end
