-- Apps open on the workspace they were launched from, not the one focused
-- when their window shows. An app can take a while to show it, and by then
-- the user may be on another screen.
--
-- Hyprland's own `misc:initial_workspace_tracking` follows only programs it
-- starts itself. Launchers have systemd start each app as a service of its
-- own, so the launch watcher (`kubaos-launches`, home/launches.nix) follows
-- systemd instead and calls `started()` the moment an app's service appears;
-- the workspace focused then is the launch's.
--
-- Each launch holds a window rule, `workspace <id> silent` for windows whose
-- class is the launch's app id. Hyprland applies it as the window maps, so
-- the window is first drawn there and the focus stays where the user is.
-- This also catches an app that was already running: Chrome hands each
-- launch to the process it has, and that process opens the window. The rule
-- is on only while the launch's workspace exists and isn't the focused one:
-- a launch whose user stayed put opens and takes the focus as any window
-- does, and a workspace that went away isn't made again on another screen.
--
-- A window whose class isn't its app id (some apps name their windows
-- otherwise) still comes from the launch's service. It is moved as it maps,
-- and the focus, which a moved window takes, is handed back.
--
-- A launch ends at its first window, or after LIFETIME_MS.

local M = {}

-- Long enough for a slow app's first window, short enough that a launch
-- whose window never came claims no later one.
local LIFETIME_MS = 20000

-- Rules are named, and a rule declared under a name already taken replaces
-- it: launches take these names in turn, so the rules never pile up. More
-- than ever wait at once.
local SLOTS = 8

-- { unit, app, workspace, slot, rule, timer }, oldest first.
local launches = {}
local next_slot = 1

-- The window that was moved and the window to give the focus back to.
local handing_back = nil

local function wanted(launch)
  local active = hl.get_active_workspace()
  return active ~= nil and active.id ~= launch.workspace and hl.get_workspace(launch.workspace) ~= nil
end

local function refresh()
  for _, launch in ipairs(launches) do
    launch.rule:set_enabled(wanted(launch))
  end
end

local function finish(launch)
  for i, other in ipairs(launches) do
    if other == launch then
      table.remove(launches, i)
      break
    end
  end
  launch.rule:set_enabled(false)
  launch.timer:set_enabled(false)
end

-- `text` matched literally in a regex.
local function literal(text)
  return (text:gsub("[%^%$%(%)%%%.%[%]%*%+%-%?%{%}|\\]", "\\%0"))
end

-- Notes that `unit` just started `app` from the focused workspace. A special
-- workspace is no launch's home: it hides again.
function M.started(unit, app)
  local workspace = hl.get_active_workspace()
  if not workspace or workspace.special then
    return
  end
  local slot = next_slot
  next_slot = slot % SLOTS + 1
  for _, launch in ipairs(launches) do
    if launch.slot == slot then
      finish(launch)
      break
    end
  end
  local launch = { unit = unit, app = app:lower(), workspace = workspace.id, slot = slot }
  launch.rule = hl.window_rule({
    name = "launch-" .. slot,
    match = { class = "(?i)^" .. literal(app) .. "$" },
    workspace = workspace.id .. " silent",
  })
  launch.timer = hl.timer(function()
    finish(launch)
  end, { timeout = LIFETIME_MS, type = "oneshot" })
  launches[#launches + 1] = launch
  launch.rule:set_enabled(false)
end

-- The unit whose cgroup process `pid` runs in: the last part of its cgroup
-- v2 path.
local function unit_of(pid)
  local file = io.open("/proc/" .. pid .. "/cgroup", "r")
  if not file then
    return nil
  end
  local line = file:read("l")
  file:close()
  return line and line:match("([^/]+)$")
end

-- The launch `window` comes of: by its class, as its rule matched it, else
-- by its process's unit.
local function launch_of(window)
  local class = (window.class or ""):lower()
  for _, launch in ipairs(launches) do
    if launch.app == class then
      return launch
    end
  end
  local unit = window.pid and window.pid > 0 and unit_of(window.pid)
  for _, launch in ipairs(launches) do
    if launch.unit == unit then
      return launch
    end
  end
  return nil
end

local function on_open_early(window)
  if #launches == 0 then
    return
  end
  local launch = launch_of(window)
  if not launch then
    return
  end
  local move = wanted(launch) and window.workspace and window.workspace.id ~= launch.workspace
  finish(launch)
  if not move then
    return
  end
  local previous = hl.get_active_window()
  local monitor = hl.get_active_monitor()
  handing_back = {
    window = window.address,
    previous = previous and previous.address,
    monitor = monitor and monitor.name,
  }
  hl.dispatch(hl.dsp.window.move({
    workspace = tostring(launch.workspace),
    follow = false,
    window = "address:" .. window.address,
  }))
end

local function on_open(window)
  local back = handing_back
  if not back or back.window ~= window.address then
    return
  end
  handing_back = nil
  if back.previous then
    hl.dispatch(hl.dsp.focus({ window = "address:" .. back.previous }))
  elseif back.monitor then
    hl.dispatch(hl.dsp.focus({ monitor = back.monitor }))
  end
end

hl.on("window.open_early", on_open_early)
hl.on("window.open", on_open)
for _, event in ipairs({ "workspace.active", "monitor.focused", "workspace.created", "workspace.removed" }) do
  hl.on(event, refresh)
end

dot = dot or {}
dot.launches = M

return M
