-- Look and feel: gaps, borders, rounding, blur, shadows, animations, layout.
-- Colors come from colors.lua (matugen output with fallbacks). The active
-- window's border and shadow are the neon focus tube and its glow, which
-- focus.lua owns.
--
-- Sizes and motion follow the design tokens (design/tokens.toml): the shell
-- draws an 8 px frame with 24 px inner corners, the surface radius, and
-- reserves it as exclusive zones. Windows sit gaps_out (8) inside it, measured to the outside of the
-- 2 px border, and Hyprland rounds that outer edge with rounding +
-- border_size. Concentric with the frame means 24 - 8 = 16 for the outer
-- edge, so rounding = 14
-- (OUTERROUND in Hyprland 0.56.2 src/render/decorations/CHyprBorderDecoration.cpp).

local colors = require("colors")
local focus = require("focus")
local tokens = require("tokens")

hl.config({
  general = {
    layout = "dwindle",
    gaps_in = 4,  -- 8 px between windows
    gaps_out = 8, -- inside the shell's frame
    gaps_workspaces = 40,
    border_size = 2,
    resize_on_border = true,
    extend_border_grab_area = 8,
    hover_icon_on_border = true,
    allow_tearing = false,
  },

  decoration = {
    rounding = 14, -- + border_size 2 = 16 on the outer edge, see the top
    rounding_power = 2,
    dim_inactive = false,
    active_opacity = 1.0,
    inactive_opacity = 1.0,

    -- Frosted glass for the shell's frame and panels (see the qs-frame layer
    -- rule in rules.lua) and the translucent terminals (Ghostty and foot draw
    -- their background at 85% opacity; every window with alpha is blurred):
    -- wide and soft, a little vibrancy so neon survives, slightly darker so
    -- text stays readable over bright signage.
    blur = {
      enabled = true,
      size = 8,
      passes = 3,
      new_optimizations = true,
      ignore_opacity = true,
      xray = false,
      noise = 0.012,
      contrast = 0.9,
      brightness = 0.85,
      vibrancy = 0.2,
      vibrancy_darkness = 0.1,
      popups = true,
      special = false,
    },

    -- One falloff for every window's shadow: the one the active window's glow
    -- is fitted to (focus.lua).
    shadow = {
      enabled = true,
      range = focus.shadow.range,
      render_power = focus.shadow.power,
    },
  },

  dwindle = {
    preserve_split = true, -- needed for togglesplit
    smart_resizing = true,
    force_split = 2,       -- new window goes right / below
  },

  group = {
    groupbar = {
      font_size = 10,
      height = 18,
      gradients = true,
      rounding = 6,
      gaps_in = 3,
      gaps_out = 2,
    },
  },

  misc = {
    disable_hyprland_logo = true,
    disable_splash_rendering = true,
    force_default_wallpaper = 0,
    focus_on_activate = true,
    animate_manual_resizes = false,
    animate_mouse_windowdragging = false,
    key_press_enables_dpms = true,
    mouse_move_enables_dpms = true,
    -- Lets a restarted Quickshell take over a session lock its crashed
    -- predecessor held, instead of leaving a red "locked" screen.
    allow_session_lock_restore = true,
  },

  -- XWayland apps stay upscaled by the compositor (slightly soft on the 1.5x
  -- panel) rather than tiny; Chrome, Electron, Qt and GTK run native Wayland.
  xwayland = {
    force_zero_scaling = false,
  },

  ecosystem = {
    no_update_news = true,
  },

  animations = {
    enabled = true,
  },
})

-- Animations, on Material 3 curves: things that appear decelerate in with a
-- slight overshoot ("spatial"), things that leave accelerate out, quicker.
-- speed is in deciseconds (4 = 400 ms). The shell's own layers have no_anim
-- (rules.lua): Quickshell animates them itself.
hl.curve("spatial", { type = "bezier", points = { { 0.38, 1.21 }, { 0.22, 1 } } })
hl.curve("emphasizedDecel", { type = "bezier", points = { { 0.05, 0.7 }, { 0.1, 1 } } })
hl.curve("emphasizedAccel", { type = "bezier", points = { { 0.3, 0 }, { 0.8, 0.15 } } })
local standard = tokens.easing.standard
hl.curve("standard", { type = "bezier", points = { { standard[1], standard[2] }, { standard[3], standard[4] } } })
hl.curve("effects", { type = "bezier", points = { { 0.34, 0.8 }, { 0.34, 1 } } })

hl.animation({ leaf = "windowsIn", enabled = true, speed = 4, bezier = "spatial", style = "popin 88%" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 2, bezier = "emphasizedAccel", style = "popin 92%" })
hl.animation({ leaf = "windowsMove", enabled = true, speed = 4, bezier = "emphasizedDecel" })
hl.animation({ leaf = "layersIn", enabled = true, speed = 3.5, bezier = "emphasizedDecel", style = "slide" })
hl.animation({ leaf = "layersOut", enabled = true, speed = 2, bezier = "emphasizedAccel", style = "slide" })
hl.animation({ leaf = "fadeLayers", enabled = true, speed = 3, bezier = "effects" })
hl.animation({ leaf = "fade", enabled = true, speed = 3, bezier = "effects" })
hl.animation({ leaf = "fadeDim", enabled = true, speed = 3, bezier = "effects" })
-- The border's and the shadow's fades: focus.lua turns them off while the
-- active window's tube strikes and back on after.
focus.fade({ leaf = "border", enabled = true, speed = tokens.duration.medium / 100, bezier = "standard" })
focus.fade({ leaf = "fadeShadow", enabled = true, speed = tokens.duration.medium / 100, bezier = "effects" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 4, bezier = "emphasizedDecel", style = "slide" })
hl.animation({ leaf = "specialWorkspace", enabled = true, speed = 4, bezier = "emphasizedDecel", style = "slidefadevert 15%" })

-- Colors. Applied at load and again on demand: dot-theme can run
--   hyprctl eval 'dot.reload_colors()'
-- after regenerating ~/.local/state/theme/hypr.lua (a plain `hyprctl reload`
-- works too, it just re-runs the whole config).
local function apply_colors(c)
  local inactive = focus.inactive(c)

  hl.config({
    general = {
      col = {
        inactive_border = inactive.border,
      },
    },
    group = {
      col = {
        border_inactive = inactive.border,
        border_locked_active = colors.rgba(c.error, "ee"),
        border_locked_inactive = inactive.border,
      },
      groupbar = {
        text_color = colors.rgba(c.on_surface),
        col = {
          active = colors.rgba(c.primary_container, "cc"),
          inactive = colors.rgba(c.surface_container_high, "99"),
        },
      },
    },
    decoration = {
      shadow = {
        color_inactive = inactive.shadow,
      },
    },
    misc = {
      background_color = colors.rgba(c.surface),
    },
  })
  focus.use(c)
end

apply_colors(colors.load())

function dot.reload_colors()
  apply_colors(colors.load())
end
