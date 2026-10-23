-- Per-screen workspaces: every screen has its own workspaces 1-10.
--
-- Adapted from Omarchy's default/hypr/monitor-workspaces.lua (basecamp/omarchy,
-- MIT) at 5d8cc3313b2c60786d4e5019a68906873aed36bd.
--
-- Hyprland numbers workspaces globally, so each screen owns a block of ten
-- ids: the laptop panel 1-10, the next screen 11-20, and so on. SUPER + 3 goes
-- to the third workspace of the focused screen's block, and the bar labels
-- each screen's workspaces 1-10. No workspace is persistent: one appears when
-- it is focused or a window moves to it, and an empty one goes away once left.
--
-- A block belongs to the physical screen, named by its make, model and
-- serial. The connector and Hyprland's monitor id would both be wrong: a dock
-- renames its connectors between plugs, and Hyprland hands a free id to
-- whichever display turns up next. The assignment lives in a state file, so a
-- screen gets the same block every time it comes back, and the bar reads the
-- same file to know which block is its own.
--
-- A screen's block is its *group*. On unplug Hyprland moves the screen's
-- workspaces to any other screen; reconciling on every monitor change gathers
-- the group on the primary screen (the laptop panel, else the external
-- Hyprland numbered first), under its own numbers, and its screen takes it
-- back when it returns. A group with no windows has no workspaces left, so it vanishes.
-- The bar shows a screen's own group, then each one waiting there.
--
-- HAZARD: no workspace is renumbered on unplug or replug. The bar's
-- Quickshell (v0.3.2) keeps the id a workspace had when it first saw it, so a
-- renumbered one shows under its old number and a click on it goes nowhere.
--
-- Next and previous walk the focused screen's groups in the bar's order
-- (`workspace_blocks.step`). SUPER + 1-0 address the focused group: the
-- screen's own, or the waiting group whose workspace it shows.
--
-- Every action is a function a bind runs, and also an expression the socket
-- runs: `hyprctl dispatch 'dot.workspaces.focus(3)'` does what SUPER + 3 does.
--
-- The `hl.*` API: /usr/share/hypr/stubs/, https://wiki.hypr.land/Configuring/

local blocks_lib = require("workspace_blocks")

local SLOTS = blocks_lib.SLOTS
local block_of = blocks_lib.block_of

local function state_dir()
  local state_home = os.getenv("XDG_STATE_HOME") or ((os.getenv("HOME") or "") .. "/.local/state")
  return state_home .. "/kubaos/hypr"
end

local SCREENS_FILE = state_dir() .. "/workspace-screens"

local M = { slots = SLOTS }

local function read(path)
  local file = io.open(path, "r")
  if not file then
    return ""
  end
  local text = file:read("a") or ""
  file:close()
  return text
end

local function write(path, text)
  local file = io.open(path, "w")
  if not file then
    os.execute("mkdir -p '" .. state_dir():gsub("'", "'\\''") .. "'")
    file = io.open(path, "w")
  end
  if not file then
    return false
  end
  file:write(text)
  file:close()
  return true
end

-- Identity -> block and identity -> x where the screen last sat, kept across
-- sessions, and connector -> identity for the screens plugged in now.
local blocks, current, positions = blocks_lib.parse_blocks(read(SCREENS_FILE))
local written = nil

local function save(identities)
  local text = blocks_lib.format_blocks(blocks, identities, positions)
  if text ~= written and write(SCREENS_FILE, text) then
    written = text
  end
end

local function regular(workspace)
  return workspace and not workspace.special and workspace.id > 0
end

-- The ids of every regular workspace, as a set.
local function taken_ids()
  local taken = {}
  for _, workspace in ipairs(hl.get_workspaces()) do
    if regular(workspace) then
      taken[workspace.id] = true
    end
  end
  return taken
end

-- The screens that show workspaces of their own, in block order. `gone` names
-- one Hyprland still lists while it reports the screen's removal.
local function connected(gone)
  local monitors = {}
  for _, monitor in ipairs(hl.get_monitors()) do
    if not monitor.is_mirror and monitor.name ~= gone then
      monitors[#monitors + 1] = monitor
    end
  end
  return blocks_lib.sort_screens(monitors)
end

-- Connector -> block for the screens plugged in now, noting where each sits.
local function assign(monitors)
  local identities = blocks_lib.identities(monitors, current)
  current = identities
  local by_connector = blocks_lib.assign(blocks, monitors, identities)
  for _, monitor in ipairs(monitors) do
    positions[identities[monitor.name]] = monitor.x
  end
  save(identities)
  return by_connector
end

local function renumber(workspace, id)
  hl.dispatch(hl.dsp.workspace.change_id({ workspace = tostring(workspace.id), id = id }))
end

-- The plugged-in screen whose block holds `id`.
local function owner(id, monitors, by_connector)
  if id < 1 then
    return nil
  end
  for _, monitor in ipairs(monitors) do
    if by_connector[monitor.name] == block_of(id) then
      return monitor
    end
  end
  return nil
end

-- The block of the group a screen shows: the waiting group of an unplugged
-- screen when it shows one of its workspaces, else its own.
local function focused_block(monitor, monitors, by_connector)
  local active = monitor.active_workspace
  if regular(active) and not owner(active.id, monitors, by_connector) then
    return block_of(active.id)
  end
  return by_connector[monitor.name] or 0
end

-- The id of a slot in the group a screen shows.
function M.workspace_id(monitor, slot)
  local monitors = connected()
  return blocks_lib.id_of(focused_block(monitor, monitors, assign(monitors)), slot)
end

-- A screen's groups, as the bar shows them.
local function groups_on(monitor)
  local by_connector = assign(connected())
  local workspaces = {}
  for _, workspace in ipairs(hl.get_workspaces()) do
    if regular(workspace) then
      workspaces[#workspaces + 1] = { id = workspace.id, monitor = workspace.monitor and workspace.monitor.name }
    end
  end
  return blocks_lib.groups(by_connector[monitor.name] or 0, monitor.name, workspaces,
    blocks_lib.x_of_block(blocks, positions))
end

-- The workspace a screen falls back to when what it shows isn't its own: the
-- lowest of its own already on it, or its first.
local function home(monitor, block)
  local best = nil
  for _, workspace in ipairs(hl.get_workspaces()) do
    if regular(workspace) and block_of(workspace.id) == block and workspace.monitor
      and workspace.monitor.name == monitor.name and (not best or workspace.id < best) then
      best = workspace.id
    end
  end
  return best or blocks_lib.id_of(block, 1)
end

-- Show a workspace on a screen, bringing it over from another or creating it
-- there. Bringing it over is a move rather than a focus on this screen, which
-- would swap the two screens' workspaces when it is visible over there.
local function show(monitor, id)
  local workspace = hl.get_workspace(id)
  if workspace and workspace.monitor and workspace.monitor.name ~= monitor.name then
    hl.dispatch(hl.dsp.workspace.move({ workspace = tostring(id), monitor = monitor.name }))
    workspace = hl.get_workspace(id)
  end
  if workspace then
    monitor:set_workspace({ workspace = workspace.name })
  else
    hl.dispatch(hl.dsp.focus({ monitor = monitor.name }))
    hl.dispatch(hl.dsp.focus({ workspace = tostring(id) }))
  end
end

-- Moving a screen's visible workspace away leaves Hyprland to fill the gap
-- with any workspace it finds, or a new one with the lowest free number, so
-- put the screen back on one of its own.
local function settle(monitor, monitors, by_connector)
  local block = by_connector[monitor.name]
  local active = monitor.active_workspace
  if block and regular(active) and block_of(active.id) ~= block and owner(active.id, monitors, by_connector) then
    show(monitor, home(monitor, block))
  end
end

-- Bring a workspace of this screen's block back from wherever an unplug left
-- it, before focusing it here.
local function reclaim(monitor, id)
  local workspace = hl.get_workspace(id)
  if not workspace or not workspace.monitor or workspace.monitor.name == monitor.name then
    return
  end
  local away = workspace.monitor
  hl.dispatch(hl.dsp.workspace.move({ workspace = tostring(id), monitor = monitor.name }))
  local monitors = connected()
  settle(away, monitors, assign(monitors))
end

-- Renumber a workspace into a block. False when all ten slots are taken.
local function adopt(workspace, block)
  local id = blocks_lib.adopt_id(block, workspace.id, taken_ids())
  if not id then
    return false
  end
  renumber(workspace, id)
  return true
end

local function target_monitor(name)
  if name then
    return hl.get_monitor(name)
  end
  return hl.get_active_monitor()
end

-- Show workspace `id` on a screen and focus it, bringing it back from another
-- screen, or creating it here.
local function go(monitor, id)
  if not monitor.focused then
    hl.dispatch(hl.dsp.focus({ monitor = monitor.name }))
  end
  reclaim(monitor, id)
  hl.dispatch(hl.dsp.focus({ workspace = tostring(id) }))
end

-- An action that runs `fn(monitor)` on a screen, the focused one unless named.
local function on_monitor(monitor_name, fn)
  return function()
    local monitor = target_monitor(monitor_name)
    if monitor then
      fn(monitor)
    end
  end
end

-- Go to a slot of the group a screen shows, the focused screen unless named.
function M.focus(slot, monitor_name)
  return on_monitor(monitor_name, function(monitor)
    go(monitor, M.workspace_id(monitor, slot))
  end)
end

-- Go to workspace `id` on a screen, whichever group it is in: the bar's click.
function M.go_to(id, monitor_name)
  return on_monitor(monitor_name, function(monitor)
    go(monitor, id)
  end)
end

-- Next or previous workspace through a screen's groups (`direction` 1 or -1).
local function scroll(direction, monitor_name)
  return on_monitor(monitor_name, function(monitor)
    local active = monitor.active_workspace
    if not regular(active) then
      return
    end
    local id = blocks_lib.step(groups_on(monitor), active.id, direction, (active.windows or 0) > 0, taken_ids())
    if id then
      go(monitor, id)
    end
  end)
end

-- The first workspace of the next or previous group on a screen.
local function scroll_group(direction, monitor_name)
  return on_monitor(monitor_name, function(monitor)
    local active = monitor.active_workspace
    local id = regular(active) and blocks_lib.group_step(groups_on(monitor), active.id, direction)
    if id then
      go(monitor, id)
    end
  end)
end

function M.next(monitor_name)
  return scroll(1, monitor_name)
end

function M.previous(monitor_name)
  return scroll(-1, monitor_name)
end

function M.next_group(monitor_name)
  return scroll_group(1, monitor_name)
end

function M.previous_group(monitor_name)
  return scroll_group(-1, monitor_name)
end

local function move_active_window(id, follow)
  local window = hl.get_active_window()
  if not window then
    return
  end
  hl.dispatch(hl.dsp.window.move({ workspace = tostring(id), follow = follow ~= false, window = "address:" .. window.address }))
end

-- Move the active window to a slot on the focused screen, following it unless
-- `follow` is false.
function M.move_window(slot, follow)
  return function()
    local monitor = hl.get_active_monitor()
    if not monitor then
      return
    end
    local id = M.workspace_id(monitor, slot)
    reclaim(monitor, id)
    move_active_window(id, follow)
  end
end

-- Move the active window to a new workspace after the last of the focused
-- group, and follow it. Does nothing when all ten are taken.
function M.move_window_to_new()
  return function()
    local monitor = hl.get_active_monitor()
    if not monitor then
      return
    end
    local monitors = connected()
    local id = blocks_lib.append_slot(focused_block(monitor, monitors, assign(monitors)), taken_ids())
    if id then
      move_active_window(id, true)
    end
  end
end

-- Move the active workspace to the screen in a direction, where it joins that
-- screen's block, under the same number when it is free. With all ten taken it
-- keeps its number and stays a guest until its own screen wants it back.
function M.move_to_monitor(direction)
  return function()
    local workspace = hl.get_active_workspace()
    if not regular(workspace) then
      return
    end
    -- Hyprland reports a move toward no screen as an error on screen.
    local target = hl.get_monitor(direction)
    if not target then
      return
    end

    local id = workspace.id
    local from = hl.get_active_monitor()
    hl.dispatch(hl.dsp.workspace.move({ monitor = target.name }))

    local moved = hl.get_workspace(id)
    if not moved or not moved.monitor or (from and moved.monitor.name == from.name) then
      return
    end

    local monitors = connected()
    local by_connector = assign(monitors)
    local block = by_connector[moved.monitor.name]
    if block and block_of(id) ~= block then
      adopt(moved, block)
    end
    if from then
      settle(from, monitors, by_connector)
    end
    -- Settling the screen it left can take focus; it belongs with the
    -- workspace that moved.
    hl.dispatch(hl.dsp.focus({ workspace = tostring(moved.id) }))
  end
end

-- Put every workspace whose screen is plugged in back on it, and every screen
-- on a workspace of its own. `added` names a screen that just arrived, which
-- leaves whatever Hyprland put it on; `gone` one being removed.
function M.reconcile(added, gone)
  local monitors = connected(gone)
  if #monitors == 0 then
    return
  end

  local by_connector = assign(monitors)
  local focused_window = hl.get_active_window()
  local focused_monitor = hl.get_active_monitor()
  local focused_name = focused_monitor and focused_monitor.name
  local changed = false

  local function own_views()
    for _, monitor in ipairs(monitors) do
      local block = by_connector[monitor.name]
      local active = monitor.active_workspace
      local own = regular(active) and block_of(active.id) == block
      -- A workspace whose screen is unplugged stays in view where it was put,
      -- unless this screen has only just arrived on it. An empty one is only
      -- the number Hyprland starts a screen on.
      local guest = regular(active) and not owner(active.id, monitors, by_connector) and monitor.name ~= added
        and (active.windows or 0) > 0
      if not own and not guest then
        show(monitor, home(monitor, block))
        changed = true
      end
    end
  end

  -- A workspace on a screen other than the plugged-in one that owns it goes
  -- home when a screen arrives: that screen is returning to the workspaces an
  -- unplug left elsewhere. Otherwise it got there some other way, such as a
  -- window rule naming a number, and joins the block of the screen it is on,
  -- so its windows don't jump screens. An empty one is only the number
  -- Hyprland parked a screen on, and goes away once the screen shows its own.
  local function place()
    for _, workspace in ipairs(hl.get_workspaces()) do
      if regular(workspace) and workspace.monitor and (workspace.windows or 0) > 0 then
        local home_monitor = owner(workspace.id, monitors, by_connector)
        if home_monitor and home_monitor.name ~= workspace.monitor.name then
          local here = by_connector[workspace.monitor.name]
          if added or not here or not adopt(workspace, here) then
            hl.dispatch(hl.dsp.workspace.move({ workspace = tostring(workspace.id), monitor = home_monitor.name }))
          end
          changed = true
        end
      end
    end
  end

  -- A workspace whose screen is unplugged waits on the primary screen, the
  -- first `connected()` lists, with the rest of its group. An empty one is left
  -- to go away.
  local function gather()
    local primary = monitors[1]
    for _, workspace in ipairs(hl.get_workspaces()) do
      if regular(workspace) and workspace.monitor and (workspace.windows or 0) > 0
        and not owner(workspace.id, monitors, by_connector) and workspace.monitor.name ~= primary.name then
        hl.dispatch(hl.dsp.workspace.move({ workspace = tostring(workspace.id), monitor = primary.name }))
        changed = true
      end
    end
  end

  -- Views first for a new screen, so the number Hyprland gave it is left
  -- empty and goes away instead of being carried to the screen that owns it.
  if added then
    own_views()
  end
  place()
  gather()
  -- Moving a visible workspace home leaves a gap Hyprland fills with anything.
  own_views()

  if changed then
    if focused_window and hl.get_window("address:" .. focused_window.address) then
      hl.dispatch(hl.dsp.focus({ window = "address:" .. focused_window.address }))
    elseif focused_name and focused_name ~= gone and hl.get_monitor(focused_name) then
      hl.dispatch(hl.dsp.focus({ monitor = focused_name }))
    end
  end
end

hl.on("hyprland.start", function()
  M.reconcile()
end)

hl.on("config.reloaded", function()
  M.reconcile()
end)

hl.on("monitor.added", function(monitor)
  M.reconcile(monitor and monitor.name)
end)

hl.on("monitor.removed", function(monitor)
  M.reconcile(nil, monitor and monitor.name)
end)

dot.workspaces = M

-- No wider gaps for a lone window: corners are concentric with the shell's
-- frame only at gaps_out 8 (looks.lua), so one window keeps the same 8 px.

return M
