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

-- Cursor size. The theme name comes from home-manager (home.pointerCursor,
-- owned by home/theme.nix), which exports XCURSOR_THEME / HYPRCURSOR_THEME.
hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")
