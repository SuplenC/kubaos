-- Per-screen workspaces: every screen has its own workspaces 1-10.
--
-- Adapted from Omarchy's default/hypr/monitor-workspaces.lua (basecamp/omarchy,
-- MIT) at 5d8cc3313b2c60786d4e5019a68906873aed36bd.
--
-- Hyprland numbers workspaces globally, so each screen shows a block of ten
-- ids: the laptop panel 1-10, the next screen 11-20, and so on. SUPER + 3 goes
-- to the third workspace of the focused screen's block, and the bar labels
-- each screen's workspaces 1-10. A workspace appears when it is focused or a
-- window moves to it, and an empty one goes away once left, except the ones
-- each group keeps (below).
--
-- A block belongs to a *position* on the desk, not to a screen: the laptop
-- panel is one, and the externals are positions 1, 2 and on, counted left to
-- right by where Hyprland's layout places them. Whatever screen stands at
-- position 1 shows position 1's block, so the two screens beside the laptop at
-- work and the two at home show the same blocks. The position comes from the
-- layout, so a desk needs its screens placed in the order they stand (the
-- user layer's `hl.monitor` lines); auto placement follows connector order.
-- Position -> block lives in a state file, with the screen each position last
-- had, and the bar reads the same file to know which block is a screen's own.
--
-- The user can pin screens to a group, by description, in the user layer's
-- `desktop/workspaces/groups.toml`: a pinned screen stands at its group's
-- number wherever the layout puts it, the first listed winning when two of a
-- group's screens are plugged in, and the unpinned externals take the numbers
-- left, left to right, never one a group pins. The panel pinned to a group
-- shows that group, and its own 1-10 waits on the primary screen, itself,
-- like any group without a screen; with the lid closed it is not plugged in
-- as far as groups go. The file is read at load and again on
-- `reload_groups()`; one that doesn't read is a toast and pins nothing.
--
-- A position's block is its *group*. When a screen goes, Hyprland moves its
-- workspaces to any other screen; reconciling on every monitor change sends
-- each group to the screen now at its position, and gathers the group of a
-- position no screen holds on the primary screen (the laptop panel, else the
-- external Hyprland numbered first), under its own numbers, until a screen
-- takes that position again. A waiting group with no windows has no
-- workspaces left, so it vanishes. The bar shows a screen's groups in block
-- order.
--
-- Hyprland reports a screen's arrival before it arranges the layout, so the
-- screen still sits at 0,0 and its position can't be read yet; a dock brings
-- two screens a moment apart, and takes them away the same way, and while
-- one arrives the other can be missing from the list. So an arrival, a
-- removal or a screen moving in the layout only starts a short wait,
-- restarted by every layout change, and the reconcile runs once the layout
-- has held still. The mapping it computes, screen -> block, is the only one
-- there is until the next reconcile: a window opening or closing never
-- recomputes positions.
--
-- HAZARD: no workspace is renumbered on unplug or replug. The bar keys its
-- cells by id, so a renumbered workspace would leave and grow back in another
-- place; a group moves whole under its own numbers instead. The one renumber,
-- a workspace sent to another screen (`move_to_monitor`), happens before the
-- move, so the move is the first event naming the new id; the bar's
-- Quickshell keys a workspace by that and never changes it.
--
-- The three-finger swipe is Hyprland's own, which follows the fingers. It
-- walks the workspaces that exist on the screen in id order and never makes
-- one, so each group keeps two empty ones as persistent workspaces
-- (`workspace_blocks.kept`): its first, and one after its last with windows,
-- where a new workspace opens. Once a window lands on that one, the next id
-- is kept in its place and the old rule is released. A pinned window counts
-- nowhere: it follows its screen from workspace to workspace, and Hyprland
-- counts it on each in turn, so every empty one visited would read as holding
-- a window. A workspace holding only pinned windows is empty here, as it is
-- to Hyprland once the view moves on: the pinned windows go along, and the
-- workspace goes away unless it is kept. Next and previous walk
-- the same workspaces in the same order (`workspace_blocks.step`). SUPER + 1-0
-- address the focused group: the screen's own, or the waiting group whose
-- workspace it shows.
--
-- Every action is a function a bind runs, and also an expression the socket
-- runs: `hyprctl dispatch 'dot.workspaces.focus(3)'` does what SUPER + 3 does.
--
-- The `hl.*` API: /usr/share/hypr/stubs/, https://wiki.hypr.land/Configuring/

local blocks_lib = require("workspace_blocks")
local user_layer = require("user_layer")

local SLOTS = blocks_lib.SLOTS
local block_of = blocks_lib.block_of

-- How long the layout has to hold still after a screen arrives or moves
-- before the reconcile runs. Hyprland arranges a screen right after it reports
-- it, and a dock's two screens come within this of each other.
local SETTLE_MS = 250
-- How many more waits a reconcile allows the monitor list to finish changing
-- when it still looks mid-change (a known screen missing, two on one spot).
local SETTLE_RETRIES = 4

local function state_dir()
  local state_home = os.getenv("XDG_STATE_HOME") or ((os.getenv("HOME") or "") .. "/.local/state")
  return state_home .. "/kubaos/hypr"
end

-- Hyprland's own folder for this run of it, which a config reload keeps and
-- the next login doesn't share, so a new login says again what the last one
-- said. Nil without a runtime folder: nothing is remembered then.
local function runtime_dir()
  local runtime, instance = os.getenv("XDG_RUNTIME_DIR"), os.getenv("HYPRLAND_INSTANCE_SIGNATURE")
  return runtime and instance and (runtime .. "/hypr/" .. instance)
end

local SCREENS_FILE = state_dir() .. "/workspace-screens"
local GROUPS_FILE = user_layer.file("workspaces", "groups.toml")
local TOASTS_FILE = runtime_dir() and (runtime_dir() .. "/kubaos-workspace-groups-toasts")

local M = { slots = SLOTS }

-- The file's text; nil when it is missing.
local function read_file(path)
  local file = io.open(path, "r")
  if not file then
    return nil
  end
  local text = file:read("a") or ""
  file:close()
  return text
end

local function read(path)
  return read_file(path) or ""
end

-- Written whole or not at all, its folder made when missing: the bar reads
-- the screens file on every change, and a half-written one would map a screen
-- to no block.
local function write(path, text)
  local temp = path .. ".tmp"
  local file = io.open(temp, "w")
  if not file then
    os.execute("mkdir -p '" .. path:match("^(.*)/"):gsub("'", "'\\''") .. "'")
    file = io.open(temp, "w")
  end
  if not file then
    return false
  end
  file:write(text)
  file:close()
  return os.rename(temp, path) and true or false
end

-- Position -> block and position -> the screen there now or last, kept across
-- sessions. `legacy` holds the rows of a screens file keyed by screen until
-- the first screens are known to migrate it.
local screens_text = read(SCREENS_FILE)
local places, seen, legacy = blocks_lib.parse_places(screens_text)
-- The text the file holds. The module loads afresh on every config reload,
-- and the bar reads the file again on every write, so the same text isn't
-- written twice.
local written = screens_text

-- The workspace groups the user pinned screens to (`workspace_blocks.
-- parse_groups`), and the text they were read from. A file that doesn't
-- read pins nothing, so every screen stands where it would without one.
local groups, groups_text = {}, nil

-- How long, in seconds, a toast about the groups file is tried again until
-- the shell shows it.
local TOAST_TRIES = 60

-- What the toasts about the groups file have said, from the toasts file: the
-- last token handed out, the problem shown, and the one being tried, with its
-- token and the time its tries run out. A problem is kept on one line.
local function read_toasts()
  local toasts = { token = 0 }
  for line in (TOASTS_FILE and read(TOASTS_FILE) or ""):gmatch("[^\n]+") do
    local token = line:match("^token\t(%d+)$")
    local shown = line:match("^shown\t(.*)$")
    local trying, expires, problem = line:match("^trying\t(%d+)\t(%d+)\t(.*)$")
    if token then
      toasts.token = tonumber(token)
    elseif shown then
      toasts.shown = shown
    elseif trying then
      toasts.trying = { token = tonumber(trying), expires = tonumber(expires), problem = problem }
    end
  end
  return toasts
end

local function write_toasts(toasts)
  if not TOASTS_FILE then
    return
  end
  local lines = { "token\t" .. toasts.token }
  if toasts.shown then
    lines[#lines + 1] = "shown\t" .. toasts.shown
  end
  local trying = toasts.trying
  if trying then
    lines[#lines + 1] = ("trying\t%d\t%d\t%s"):format(trying.token, trying.expires, trying.problem)
  end
  write(TOASTS_FILE, table.concat(lines, "\n") .. "\n")
end

-- Says what is wrong with the groups file, in a toast. At login Hyprland
-- loads its config before the shell, which is the notification server, is up,
-- so a shell tries `notify-send` once a second until it shows, then calls
-- `problem_shown()` with the toast's token. Only then is the problem kept as
-- shown. The module loads afresh on every config reload, so what was said
-- lives in the toasts file, and a lid change or `hyprctl reload` doesn't show
-- it again; one still being tried isn't tried twice.
local function warn(problem)
  local toasts = read_toasts()
  if not problem then
    if toasts.shown or toasts.trying then
      toasts.shown, toasts.trying = nil, nil
      write_toasts(toasts)
    end
    return
  end
  problem = problem:gsub("\n", " ")
  local trying = toasts.trying
  if toasts.shown == problem or (trying and trying.problem == problem and os.time() < trying.expires) then
    return
  end
  toasts.token = toasts.token + 1
  toasts.trying = { problem = problem, token = toasts.token, expires = os.time() + TOAST_TRIES }
  write_toasts(toasts)
  local body = ("In groups.toml, %s. Until it's fixed, no screen is pinned: they take their places left to right.")
    :format(problem)
  local toast = "notify-send -a 'Workspace groups' -i video-display -u critical 'Workspace groups' '"
    .. body:gsub("'", "'\\''") .. "'"
  hl.exec_cmd(("for try in $(seq %d); do %s && exec hyprctl dispatch 'dot.workspaces.problem_shown(%d)'; sleep 1; done")
    :format(TOAST_TRIES, toast, toasts.token))
end

-- The shell showed the toast with `token`, so its problem counts as shown.
function M.problem_shown(token)
  return function()
    local toasts = read_toasts()
    local trying = toasts.trying
    if trying and trying.token == token then
      toasts.shown, toasts.trying = trying.problem, nil
      write_toasts(toasts)
    end
  end
end

local function apply_groups(text)
  groups_text = text
  local parsed, problem = blocks_lib.parse_groups(text)
  groups = parsed or {}
  warn(problem)
end

-- Reads the groups file again: true when its text changed, false when not,
-- and nil when it is missing but was there a moment ago. An editor saving by
-- rename (nvim's default) moves the file away and writes a new one in its
-- place, so a read in between finds nothing; taking that as "no pins" would
-- move every pinned group for a moment and back. The caller reads again
-- after the wait instead (`recheck_groups`); a file really deleted pins
-- nothing then.
local function read_groups()
  local text = read_file(GROUPS_FILE)
  if not text and (groups_text or "") ~= "" then
    return nil
  end
  text = text or ""
  if text == groups_text then
    return false
  end
  apply_groups(text)
  return true
end

read_groups()

-- Position -> the name of its group.
local function group_names()
  local names = {}
  for _, group in ipairs(groups) do
    names[tostring(group.number)] = group.name
  end
  return names
end

local function save(positions)
  local text = blocks_lib.format_places(places, seen, positions, group_names())
  if text ~= written and write(SCREENS_FILE, text) then
    written = text
  end
end

-- Connector -> block as the last reconcile mapped the screens. Every action
-- and window event reads it; only a reconcile writes it.
local current = {}

-- The screens as the last reconcile saw them (`layout_of`); nil while none is
-- mapped.
local reconciled_layout = nil

-- Each screen's connector, description and place, one line each: what a
-- reconcile reads from the layout.
local function layout_of(monitors)
  local lines = {}
  for _, monitor in ipairs(monitors) do
    lines[#lines + 1] = table.concat({ monitor.name, monitor.description or "", monitor.x, monitor.y }, "\t")
  end
  return table.concat(lines, "\n")
end

local function regular(workspace)
  return workspace and not workspace.special and workspace.id > 0
end

-- Workspace id -> how many pinned windows it holds, read once per pass over
-- the workspaces.
local function pinned_counts()
  local counts = {}
  for _, window in ipairs(hl.get_windows()) do
    local workspace = window.pinned and window.workspace
    if workspace then
      counts[workspace.id] = (counts[workspace.id] or 0) + 1
    end
  end
  return counts
end

-- How many windows `workspace` holds that are not pinned; `pinned` comes from
-- `pinned_counts()`.
local function windows_of(workspace, pinned)
  return (workspace.windows or 0) - (pinned[workspace.id] or 0)
end

-- The ids of every regular workspace with windows, as a set; `pinned` comes
-- from `pinned_counts()`.
local function occupied_ids(pinned)
  local occupied = {}
  for _, workspace in ipairs(hl.get_workspaces()) do
    if regular(workspace) and windows_of(workspace, pinned) > 0 then
      occupied[workspace.id] = true
    end
  end
  return occupied
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

-- The blocks with a window in them, as a set.
local function busy_blocks()
  local busy = {}
  for id in pairs(occupied_ids(pinned_counts())) do
    busy[block_of(id)] = true
  end
  return busy
end

-- The output Hyprland adds while no real screen is plugged in (the lid closed
-- and the dock pulled), which holds no position.
local FALLBACK = "FALLBACK"

-- The screens that show workspaces of their own, in block order.
local function connected()
  local monitors = {}
  for _, monitor in ipairs(hl.get_monitors()) do
    if not monitor.is_mirror and monitor.name ~= FALLBACK then
      monitors[#monitors + 1] = monitor
    end
  end
  return blocks_lib.sort_screens(monitors)
end

-- The screens plugged in now and the block each was last mapped to; a screen
-- that arrived since the last reconcile has none yet.
local function mapping()
  local monitors = connected()
  local by_connector = {}
  for _, monitor in ipairs(monitors) do
    by_connector[monitor.name] = current[monitor.name]
  end
  return monitors, by_connector
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

-- The block of the group a screen shows: a waiting group when it shows one of
-- its workspaces, else its own; the block of what it shows until it is mapped.
local function focused_block(monitor, monitors, by_connector)
  local active = monitor.active_workspace
  if regular(active) and not owner(active.id, monitors, by_connector) then
    return block_of(active.id)
  end
  if by_connector[monitor.name] then
    return by_connector[monitor.name]
  end
  return regular(active) and block_of(active.id) or 0
end

-- The id of a slot in the group a screen shows.
function M.workspace_id(monitor, slot)
  local monitors, by_connector = mapping()
  return blocks_lib.id_of(focused_block(monitor, monitors, by_connector), slot)
end

-- A screen's groups, as the bar shows them.
local function groups_on(monitor)
  local monitors, by_connector = mapping()
  local workspaces = {}
  for _, workspace in ipairs(hl.get_workspaces()) do
    if regular(workspace) then
      workspaces[#workspaces + 1] = { id = workspace.id, monitor = workspace.monitor and workspace.monitor.name }
    end
  end
  return blocks_lib.groups(focused_block(monitor, monitors, by_connector), monitor.name, workspaces)
end

-- Id -> connector -> the rule keeping that workspace on that screen, and id ->
-- the connector it is kept on, nil while released. A workspace has one rule,
-- carried from screen to screen by merging into it, so rules never pile up;
-- a second one appears only for a released rule still naming a screen that
-- is plugged in, since enabling that would move the workspace there first.
-- The workspace kept on that screen again enables that rule.
local rules, kept = {}, {}

local function release(id)
  -- Hyprland returns no rule when it rejects one, and reports why on screen.
  local rule = rules[id] and rules[id][kept[id]]
  if rule then
    rule:set_enabled(false)
  end
  kept[id] = nil
end

-- Keep workspace `id` on `connector`; `present` is the set of connectors
-- plugged in. Hyprland merges `hl.workspace_rule` into the enabled rule for
-- the workspace, if there is one. It applies the rules on the next turn of
-- its event loop, after the handler that changed them, so only the rules as
-- they stand at the end of a handler count; it then creates or moves each
-- persistent workspace to the screen its rule names, and passes over a rule
-- naming a screen it can't find.
local function keep(id, connector, present)
  local held = rules[id] or {}
  rules[id] = held
  local from = kept[id]
  if from and from ~= connector and held[connector] then
    -- Merging would leave the released rule for `connector` in Hyprland
    -- with no handle, so the two swap instead.
    if held[from] then
      held[from]:set_enabled(false)
    end
    held[connector]:set_enabled(true)
  elseif from and from ~= connector then
    held[connector] = hl.workspace_rule({ workspace = tostring(id), persistent = true, monitor = connector })
    held[from] = nil
  elseif held[connector] then
    -- HAZARD: a released rule is only enabled again for the screen it names.
    -- Enabling one naming another screen would move the workspace there.
    held[connector]:set_enabled(true)
  else
    -- A released rule naming a screen that is gone can be enabled and merged
    -- into, so the workspace keeps one rule.
    local stale = nil
    for name in pairs(held) do
      if not present[name] then
        stale = name
        break
      end
    end
    if stale then
      held[stale]:set_enabled(true)
      held[stale] = nil
    end
    held[connector] = hl.workspace_rule({ workspace = tostring(id), persistent = true, monitor = connector })
  end
  kept[id] = connector
end

local function present_set(monitors)
  local present = {}
  for _, monitor in ipairs(monitors) do
    present[monitor.name] = true
  end
  return present
end

-- Bring Hyprland's persistent workspaces in line with `workspace_blocks.kept`
-- for `monitors` mapped by `by_connector`. With no screen plugged in, no
-- workspace would be wanted, so the rules stay as they are until one comes.
-- HAZARD: release before keeping. Once the handler ends Hyprland moves every
-- persistent workspace to the screen its rule names, so a rule left naming a
-- screen a workspace is leaving would pull it back there.
local function sync_kept_on(monitors, by_connector)
  if #monitors == 0 then
    return
  end
  local screens = {}
  for _, monitor in ipairs(monitors) do
    if by_connector[monitor.name] then
      screens[#screens + 1] = { name = monitor.name, block = by_connector[monitor.name] }
    end
  end
  local pinned = pinned_counts()
  local workspaces = {}
  for _, workspace in ipairs(hl.get_workspaces()) do
    if regular(workspace) then
      workspaces[#workspaces + 1] = {
        id = workspace.id,
        monitor = workspace.monitor and workspace.monitor.name,
        windows = windows_of(workspace, pinned),
      }
    end
  end

  local wanted = blocks_lib.kept(screens, workspaces)
  for id in pairs(kept) do
    if not wanted[id] then
      release(id)
    end
  end
  local present = present_set(monitors)
  for id, connector in pairs(wanted) do
    if kept[id] ~= connector then
      keep(id, connector, present)
    end
  end
end

-- The kept workspaces of the screens as last mapped: what a window opening or
-- closing brings in line.
local function sync_kept()
  sync_kept_on(mapping())
end

-- Point the rule of each kept workspace whose group is about to change screens
-- at the screen it is going to: the one now at its position, or the primary
-- screen to wait there, so that when Hyprland applies the rules after this
-- handler, each names the screen the workspace is on by then.
-- HAZARD: every kept workspace that moves needs its rule pointed at where it
-- goes, in the same handler as the move. A rule still naming the old screen
-- would pull the workspace back there once the handler ends.
local function repoint_moving(monitors, by_connector)
  local connector_of = {}
  for connector, block in pairs(by_connector) do
    connector_of[block] = connector
  end
  local present = present_set(monitors)
  for id, connector in pairs(kept) do
    local destination = connector_of[block_of(id)] or monitors[1].name
    if destination ~= connector then
      keep(id, destination, present)
    end
  end
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
  settle(away, mapping())
end

local function target_monitor(name)
  if name then
    return hl.get_monitor(name)
  end
  return hl.get_active_monitor()
end

-- Connector -> the workspace of a waiting group the user last went to on that
-- screen, which the reconcile leaves in view, empty or not, until the screen
-- shows another. A workspace of a group that has a screen is never held here:
-- when the screen's group changes, its view goes to the new group's.
local went = {}

-- Show workspace `id` on a screen and focus it, bringing it back from another
-- screen, or creating it here.
local function go(monitor, id)
  if not monitor.focused then
    hl.dispatch(hl.dsp.focus({ monitor = monitor.name }))
  end
  reclaim(monitor, id)
  went[monitor.name] = not owner(id, mapping()) and id or nil
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

-- Go to the group of a desk position, "panel" or "1", "2" and on: focus the
-- screen standing there, else show the lowest workspace of its group where it
-- waits, or its first on the primary screen when it has none. The Command
-- menu's desk Items run it.
function M.go_to_position(position)
  return function()
    local monitors = connected()
    local positions = blocks_lib.positions(monitors, groups)
    for _, monitor in ipairs(monitors) do
      if positions[monitor.name] == position then
        hl.dispatch(hl.dsp.focus({ monitor = monitor.name }))
        return
      end
    end
    local block = places[position]
    if not block or #monitors == 0 then
      return
    end
    local lowest = nil
    for _, workspace in ipairs(hl.get_workspaces()) do
      if regular(workspace) and block_of(workspace.id) == block and (not lowest or workspace.id < lowest.id) then
        lowest = workspace
      end
    end
    if lowest then
      if lowest.monitor then
        went[lowest.monitor.name] = lowest.id
      end
      hl.dispatch(hl.dsp.focus({ workspace = tostring(lowest.id) }))
    else
      go(monitors[1], blocks_lib.id_of(block, 1))
    end
  end
end

-- Next or previous workspace through a screen's groups (`direction` 1 or -1).
local function scroll(direction, monitor_name)
  return on_monitor(monitor_name, function(monitor)
    local active = monitor.active_workspace
    if not regular(active) then
      return
    end
    local id = blocks_lib.step(groups_on(monitor), active.id, direction)
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

-- Move the active window to the workspace after the last with windows of the
-- focused group, the empty one kept there, and follow it. Does nothing when
-- all ten have windows, or when the window is alone on its workspace and no
-- other comes after it.
function M.move_window_to_new()
  return function()
    local monitor = hl.get_active_monitor()
    if not monitor then
      return
    end
    local monitors, by_connector = mapping()
    local pinned = pinned_counts()
    local occupied = occupied_ids(pinned)
    -- A window alone on its workspace leaves it empty by moving, so it
    -- doesn't count as the last with windows.
    local window = hl.get_active_window()
    local from = window and window.workspace
    if regular(from) and windows_of(from, pinned) == 1 then
      occupied[from.id] = nil
    end
    local id = blocks_lib.append_slot(focused_block(monitor, monitors, by_connector), occupied)
    if id and id ~= (regular(from) and from.id) then
      move_active_window(id, true)
    end
  end
end

-- Move the active workspace to the screen in a direction, where it joins that
-- screen's block, under the same number when it is free, else the block's
-- first free one. With all ten taken, or no block for that screen yet, it
-- does nothing.
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
    -- A screen that arrived within the wait has no block yet; by the time a
    -- key reaches it, Hyprland has arranged it, so the wait ends here. Should
    -- the layout still look mid-change, it has none after the wait either.
    if not current[target.name] then
      M.flush()
    end
    local monitors, by_connector = mapping()
    local block = by_connector[target.name]
    if not block then
      return
    end
    -- Renumbered before it moves, so the move names the new id: the bar's
    -- Quickshell keys a workspace by the first id an event names for it, and
    -- a move naming the old id would leave it there for good.
    if block_of(id) ~= block then
      local wanted = blocks_lib.adopt_id(block, id, taken_ids())
      if not wanted then
        return
      end
      renumber(workspace, wanted)
      id = wanted
    end
    hl.dispatch(hl.dsp.workspace.move({ workspace = tostring(id), monitor = target.name }))

    local moved = hl.get_workspace(id)
    if not moved or not moved.monitor or (from and moved.monitor.name == from.name) then
      return
    end
    if from then
      settle(from, monitors, by_connector)
    end
    -- Settling the screen it left can take focus; it belongs with the
    -- workspace that moved.
    hl.dispatch(hl.dsp.focus({ workspace = tostring(moved.id) }))
    sync_kept()
  end
end

-- Put every workspace whose screen is plugged in back on it, and every screen
-- on a workspace of its own. `added` is the set of screens that just arrived,
-- which leave whatever Hyprland put them on. With no screen plugged in no
-- screen is mapped, so the next screen on a connector one had isn't taken for
-- it, and no screen is waited for.
function M.reconcile(added)
  added = added or {}
  local monitors = connected()
  if #monitors == 0 then
    current, reconciled_layout = {}, nil
    return
  end
  reconciled_layout = layout_of(monitors)

  if legacy then
    places, seen = blocks_lib.migrate(legacy, monitors, groups)
    legacy = nil
  end
  local positions = blocks_lib.positions(monitors, groups)
  local busy = busy_blocks()
  local by_connector = blocks_lib.assign(places, seen, monitors, positions, busy)
  local held = {}
  for _, position in pairs(positions) do
    held[position] = true
  end
  blocks_lib.heal(places, seen, held, busy)
  current = by_connector

  repoint_moving(monitors, by_connector)
  local focused_window = hl.get_active_window()
  local focused_monitor = hl.get_active_monitor()
  local focused_name = focused_monitor and focused_monitor.name
  local changed = false
  local shown = {}
  for _, monitor in ipairs(monitors) do
    local active = monitor.active_workspace
    shown[monitor.name] = regular(active) and active.id
  end

  -- Every screen on a workspace of its own: the one it showed, else the
  -- lowest of its own on it. A move shows the moved workspace on the screen
  -- it lands on, so a screen that received its group ends on the group's
  -- lowest, not on whichever moved last; a screen that just arrived ends
  -- there too, not on the number Hyprland parked it on.
  local function own_views()
    local pinned = pinned_counts()
    for _, monitor in ipairs(monitors) do
      local block = by_connector[monitor.name]
      local active = monitor.active_workspace
      local own = regular(active) and block_of(active.id) == block and active.id == shown[monitor.name]
        and not added[monitor.name]
      -- A workspace of a position no screen holds stays in view where it was
      -- put, when it was in view before this reconcile and the screen hasn't
      -- only just arrived on it; one a move of this reconcile brought into
      -- view gives way. An empty one is only the number Hyprland starts a
      -- screen on, unless the user went to it.
      local guest = regular(active) and not owner(active.id, monitors, by_connector) and not added[monitor.name]
        and active.id == shown[monitor.name] and (windows_of(active, pinned) > 0 or went[monitor.name] == active.id)
      if not (regular(active) and went[monitor.name] == active.id) then
        went[monitor.name] = nil
      end
      if not own and not guest then
        local id = home(monitor, block)
        if id ~= (regular(active) and active.id) then
          show(monitor, id)
          changed = true
        end
        shown[monitor.name] = id
      end
    end
  end

  -- A workspace on a screen other than the plugged-in one that owns it goes
  -- to that screen: a screen came to its position, or the screens shifted
  -- position as one came or went. It moves rather than joining the block of
  -- the screen it is on, which would renumber it (the HAZARD above). An empty
  -- one is only the number Hyprland parked a screen on, and goes away once the
  -- screen shows its own.
  local function place()
    local pinned = pinned_counts()
    for _, workspace in ipairs(hl.get_workspaces()) do
      if regular(workspace) and workspace.monitor and windows_of(workspace, pinned) > 0 then
        local home_monitor = owner(workspace.id, monitors, by_connector)
        if home_monitor and home_monitor.name ~= workspace.monitor.name then
          hl.dispatch(hl.dsp.workspace.move({ workspace = tostring(workspace.id), monitor = home_monitor.name }))
          changed = true
        end
      end
    end
  end

  -- A workspace of a position no screen holds waits on the primary screen, the
  -- first `connected()` lists, with the rest of its group. An empty one is left
  -- to go away.
  local function gather()
    local primary = monitors[1]
    local pinned = pinned_counts()
    for _, workspace in ipairs(hl.get_workspaces()) do
      if regular(workspace) and workspace.monitor and windows_of(workspace, pinned) > 0
        and not owner(workspace.id, monitors, by_connector) and workspace.monitor.name ~= primary.name then
        hl.dispatch(hl.dsp.workspace.move({ workspace = tostring(workspace.id), monitor = primary.name }))
        changed = true
      end
    end
  end

  -- Views first for a new screen, so the number Hyprland gave it is left
  -- empty and goes away instead of being carried to the screen that owns it.
  if next(added) then
    own_views()
  end
  place()
  gather()
  -- Moving a visible workspace home leaves a gap Hyprland fills with anything.
  own_views()

  if changed then
    if focused_window and hl.get_window("address:" .. focused_window.address) then
      hl.dispatch(hl.dsp.focus({ window = "address:" .. focused_window.address }))
    elseif focused_name and hl.get_monitor(focused_name) then
      hl.dispatch(hl.dsp.focus({ monitor = focused_name }))
    end
  end
  sync_kept_on(monitors, by_connector)
  -- Last, so the bar never maps a screen to a block before the block is there.
  save(positions)
end

-- The wait for the layout to settle: the screens that arrived meanwhile, and
-- how many extra waits it has had. A later wait supersedes an earlier one
-- through `generation`, since a timer can't be re-armed.
local pending = nil
local generation = 0

-- Whether the monitor list still looks mid-change: a screen mapped by the
-- last reconcile is missing without having been reported removed, or two
-- externals sit on one spot, the place Hyprland gives a screen before
-- arranging it.
local function unsettled(monitors)
  local present = present_set(monitors)
  for connector in pairs(current) do
    if not present[connector] and not pending.removed[connector] then
      return true
    end
  end
  local spots = {}
  for _, monitor in ipairs(monitors) do
    local spot = monitor.x .. "," .. monitor.y
    if spots[spot] then
      return true
    end
    spots[spot] = true
  end
  return false
end

local settle_later

local function settled()
  local wait = pending
  local monitors = connected()
  if wait.retries < SETTLE_RETRIES and unsettled(monitors) then
    wait.retries = wait.retries + 1
    settle_later()
    return
  end
  pending = nil
  M.reconcile(wait.added)
end

-- End a pending wait now.
function M.flush()
  if pending then
    generation = generation + 1
    settled()
  end
end

-- A read of the groups file that found it missing a moment after it was
-- there is done again once `SETTLE_MS` have passed; a later one supersedes
-- it through `recheck`.
local recheck = 0

local function recheck_groups()
  recheck = recheck + 1
  local mine = recheck
  hl.timer(function()
    if mine ~= recheck then
      return
    end
    local text = read(GROUPS_FILE)
    if text ~= groups_text then
      apply_groups(text)
      if not pending then
        M.reconcile()
      end
    end
  end, { timeout = SETTLE_MS, type = "oneshot" })
end

-- Read the workspace groups file again and put each group on the screen it
-- pins now: the bar runs it when the file changes
-- (`config/quickshell/services/Hypr.qml`). A file that reads the same moves
-- nothing, and a wait for the layout in progress reconciles with the new
-- groups when it ends.
function M.reload_groups()
  return function()
    local changed = read_groups()
    if changed == nil then
      recheck_groups()
    elseif changed and not pending then
      M.reconcile()
    end
  end
end

-- Reconcile once the layout has held still for `SETTLE_MS`; `added` names a
-- screen that just arrived, `removed` one that just went.
settle_later = function(added, removed)
  pending = pending or { added = {}, removed = {}, retries = 0 }
  if added then
    pending.added[added] = true
  end
  if removed then
    pending.added[removed] = nil
    pending.removed[removed] = true
  end
  generation = generation + 1
  local mine = generation
  hl.timer(function()
    if mine == generation then
      settled()
    end
  end, { timeout = SETTLE_MS, type = "oneshot" })
end

-- The screen focus left as it was being removed, from the `monitor.focused`
-- Hyprland sends for the screen it backs up to, before `monitor.removed`:
-- only then does it show what the removed screen showed on the backup. A
-- screen being removed is still listed but has no workspace in view any
-- more, which tells that focus move from one the user made. The name is
-- read at load, since the module starts afresh on every config reload.
local focused_name, left_on_removal = nil, nil

local function read_focused()
  local monitor = hl.get_active_monitor()
  focused_name = monitor and monitor.name
end

read_focused()

hl.on("monitor.focused", function(monitor)
  local previous = focused_name
  focused_name = monitor and monitor.name
  local record = previous and previous ~= focused_name and hl.get_monitor(previous)
  left_on_removal = record and not record.active_workspace and previous or nil
end)

hl.on("hyprland.start", function()
  M.reconcile()
end)

hl.on("config.reloaded", function()
  read_focused()
  M.reconcile()
end)

-- Point the rule of each kept workspace Hyprland moved to another plugged-in
-- screen at the screen it is on now.
-- HAZARD: run it in `monitor.added`. Hyprland has moved the workspaces that
-- last lived on the screen's connector back to it by then, and right after
-- the handler puts every persistent workspace on the screen its rule names:
-- a rule still naming the screen a workspace waited on would pull it back
-- there until the reconcile.
local function keep_where_they_are()
  local present = present_set(connected())
  for id, connector in pairs(kept) do
    local workspace = hl.get_workspace(id)
    local now = workspace and workspace.monitor and workspace.monitor.name
    if now and now ~= connector and present[now] then
      keep(id, now, present)
    end
  end
end

hl.on("monitor.added", function(monitor)
  keep_where_they_are()
  settle_later(monitor and monitor.name)
end)

-- Hyprland reports a layout change twice for every pass over its monitor
-- rules, and for a new mode alone (the refresh rate a power profile sets).
-- One that leaves every screen where the last reconcile saw it, with no
-- screen arriving or going, changes no position.
hl.on("monitor.layout_changed", function()
  if not pending and layout_of(connected()) == reconciled_layout then
    return
  end
  settle_later()
end)

-- Hyprland has moved the removed screen's workspaces to `backup` by now and,
-- as the removed screen had focus, shows there the workspace it showed, or
-- an empty one it parked it on. The backup goes back to one of its own at
-- once, before anything is drawn, rather than switching twice in view.
local function leave_foreign(backup)
  local _, by_connector = mapping()
  local monitor = hl.get_monitor(backup)
  local own = monitor and by_connector[backup]
  local active = own and monitor.active_workspace
  if not regular(active) then
    return
  end
  local foreign = block_of(active.id) ~= own
  local parked = windows_of(active, pinned_counts()) <= 0 and not kept[active.id]
  if foreign or parked then
    show(monitor, home(monitor, own))
  end
end

-- A dock's two screens go one at a time, like they come. Reconciling on the
-- first removal would shift the other screen's position for a moment and move
-- every group twice, so a removal waits like an arrival; by the time the
-- reconcile runs the screen is off the list.
hl.on("monitor.removed", function(monitor)
  local name = monitor and monitor.name
  if name and left_on_removal == name then
    leave_foreign(focused_name)
  end
  left_on_removal = nil
  if name then
    went[name] = nil
  end
  settle_later(nil, name)
end)

-- Whether a `sync_kept` is waiting to run.
local sync_queued = false

-- A window arriving on a workspace or leaving it, or pinned or unpinned there,
-- moves where its group ends. Hyprland reports a move
-- (`window.move_to_workspace`, also for each pinned window following a
-- switch) and an unmap (`window.close`) while the window still counts on the
-- workspace it leaves, so the sync runs on the next turn of its event loop,
-- once for every event of the turn. An X11 app closing to the tray only
-- unmaps.
-- HAZARD: the timeout stays above 0, which `hl.timer` refuses as an error.
for _, event in ipairs({ "window.open", "window.close", "window.destroy", "window.move_to_workspace", "window.pin" }) do
  hl.on(event, function()
    if sync_queued then
      return
    end
    sync_queued = true
    hl.timer(function()
      sync_queued = false
      sync_kept()
    end, { timeout = 1, type = "oneshot" })
  end)
end

dot.workspaces = M

-- No wider gaps for a lone window: corners are concentric with the shell's
-- frame only at gaps_out 8 (looks.lua), so one window keeps the same 8 px.

return M
