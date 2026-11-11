-- Session environment. hl.env() sets the variable in Hyprland's own
-- environment, so every process it spawns inherits it. Hyprland also exports
-- these to systemd --user and D-Bus activation (see autostart.lua).

local user_layer = require("user_layer")

local home = os.getenv("HOME") or ""

-- PATH: dot-* scripts and Nix tools must be reachable from binds even if
-- Hyprland was started from a context that skipped the login shell. mise's
-- shims come before Nix's tools, as in a login shell (home/shell.nix), so
-- what the launcher starts runs a Suite's tools. With the proto Suite
-- (PROTO_HOME set, home/suites/proto.nix), proto's shims come before mise's,
-- so a project's .prototools pins win in it.
do
  -- A variable that is unset or empty is nil, as the shell's `${VAR:-…}` reads it.
  local function set(name)
    local value = os.getenv(name)
    if value ~= "" then
      return value
    end
  end
  local mise_data = set("MISE_DATA_DIR") or ((set("XDG_DATA_HOME") or (home .. "/.local/share")) .. "/mise")
  local wanted = { home .. "/.local/bin" }
  local proto_home = set("PROTO_HOME")
  if proto_home then
    table.insert(wanted, proto_home .. "/shims")
  end
  table.insert(wanted, mise_data .. "/shims")
  table.insert(wanted, home .. "/.nix-profile/bin")
  table.insert(wanted, "/nix/var/nix/profiles/default/bin")
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
-- ~/.config/kubaos/desktop/cursor/cursor.toml, which the Command menu sets. The
-- classic style is the same name: its theme inherits Bibata-Modern-Classic.
do
  local size = "24"
  local file = io.open(user_layer.file("cursor", "cursor.toml"), "r")
  if file then
    for line in file:lines() do
      -- The range `kubaos cursor` takes, and an off-grid size taken to the
      -- nearest multiple of 4 as it takes it (crates/kubaos-cursor/src/settings.rs).
      local value = tonumber(line:match("^%s*size%s*=%s*(%d+)%s*$"))
      if value and value >= 16 and value <= 96 then
        size = tostring((value + 2) // 4 * 4)
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
