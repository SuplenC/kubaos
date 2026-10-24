-- A small stand-in for Hyprland's Lua API (`hl`), enough for
-- `config/hypr/workspaces.lua`: screens, workspaces, windows, focus, the
-- dispatchers that module uses and the events it listens to, and workspace
-- rules as far as `persistent` and `monitor` go. Workspaces that are empty,
-- shown nowhere and not persistent go away, as Hyprland's do. Objects read the
-- world live, as Hyprland's do, so a workspace renumbered keeps its object.

local F = {}
F.__index = F

function F.new()
  local world = setmetatable({
    monitors = {},
    workspaces = {},
    windows = {},
    rules = {},
    handlers = {},
    focused = nil,
    next_monitor_id = 0,
    next_window = 1,
    renumbered = 0,
  }, F)
  world.hl = world:api()
  return world
end

function F:monitor_record(name)
  for _, record in ipairs(self.monitors) do
    if record.name == name then
      return record
    end
  end
  return nil
end

function F:workspace_record(id)
  for _, record in ipairs(self.workspaces) do
    if record.id == id then
      return record
    end
  end
  return nil
end

function F:fire(event, ...)
  for _, handler in ipairs(self.handlers[event] or {}) do
    handler(...)
  end
end

function F:window_count(record)
  local count = 0
  for _, window in ipairs(self.windows) do
    if window.workspace == record then
      count = count + 1
    end
  end
  return count
end

function F:shown(record)
  for _, monitor in ipairs(self.monitors) do
    if monitor.active == record then
      return true
    end
  end
  return false
end

-- Empty workspaces shown nowhere go away, unless persistent.
function F:cleanup()
  local kept = {}
  for _, record in ipairs(self.workspaces) do
    if self:window_count(record) > 0 or self:shown(record) or record.persistent then
      kept[#kept + 1] = record
    end
  end
  self.workspaces = kept
end

function F:create(id, monitor)
  local record = { id = id, monitor = monitor }
  self.workspaces[#self.workspaces + 1] = record
  self:fire("workspace.created", self:workspace_object(record))
  return record
end

-- Hyprland's refresh after a workspace rule changes: each enabled persistent
-- rule whose screen is plugged in has its workspace there, created or moved,
-- and every other workspace loses its persistence. Hyprland runs it at the end
-- of the current event, once a screen being removed is gone; here it runs at
-- once and passes over that screen.
function F:refresh()
  local persistent = {}
  for _, rule in ipairs(self.rules) do
    local monitor = rule.enabled and rule.persistent and self:monitor_record(rule.monitor)
    if monitor and not monitor.removing then
      local id = tonumber(rule.workspace)
      local record = self:workspace_record(id) or self:create(id, monitor)
      if record.monitor ~= monitor then
        local from = record.monitor
        record.monitor = monitor
        if from and from.active == record then
          self:refill(from)
        end
      end
      persistent[record] = true
    end
  end
  for _, record in ipairs(self.workspaces) do
    record.persistent = persistent[record] or nil
  end
  self:cleanup()
end

-- `hl.workspace_rule`: merged into the first enabled rule for the same
-- workspace, as Hyprland's `replaceOrAdd` does, else added.
function F:workspace_rule(options)
  local rule = nil
  for _, candidate in ipairs(self.rules) do
    if candidate.enabled and candidate.workspace == options.workspace then
      rule = candidate
      break
    end
  end
  if not rule then
    rule = { workspace = options.workspace, enabled = true }
    self.rules[#self.rules + 1] = rule
  end
  if options.monitor then
    rule.monitor = options.monitor
  end
  if options.persistent ~= nil then
    rule.persistent = options.persistent
  end
  self:refresh()
  local world = self
  return {
    set_enabled = function(_, enabled)
      rule.enabled = enabled
      world:refresh()
    end,
    is_enabled = function() return rule.enabled end,
  }
end

function F:lowest_free()
  local id = 1
  while self:workspace_record(id) do
    id = id + 1
  end
  return id
end

function F:monitor_object(record)
  if not record then
    return nil
  end
  local world = self
  return setmetatable({}, {
    __index = function(_, key)
      if key == "name" then return record.name end
      if key == "description" then return record.description end
      if key == "id" then return record.id end
      if key == "x" then return record.x end
      if key == "is_mirror" then return false end
      if key == "focused" then return world.focused == record end
      if key == "active_workspace" then return world:workspace_object(record.active) end
      if key == "set_workspace" then
        return function(_, options)
          local target = world:workspace_record(tonumber(options.workspace))
          if target then
            target.monitor = record
            record.active = target
            world:cleanup()
          end
        end
      end
      return nil
    end,
  })
end

function F:workspace_object(record)
  if not record then
    return nil
  end
  local world = self
  return setmetatable({}, {
    __index = function(_, key)
      if key == "id" then return record.id end
      if key == "name" then return tostring(record.id) end
      if key == "special" then return false end
      if key == "monitor" then return world:monitor_object(record.monitor) end
      if key == "windows" then return world:window_count(record) end
      return nil
    end,
  })
end

-- A monitor selector: a connector, or l/r/u/d by the screens' order left to
-- right (they all sit in one row here).
function F:select_monitor(selector)
  local directions = { l = -1, r = 1 }
  local step = directions[selector]
  if not step then
    return self:monitor_record(selector)
  end
  local order = {}
  for _, record in ipairs(self.monitors) do
    order[#order + 1] = record
  end
  table.sort(order, function(a, b) return a.x < b.x end)
  for index, record in ipairs(order) do
    if record == self.focused then
      return order[index + step]
    end
  end
  return nil
end

function F:active_window()
  local active = self.focused and self.focused.active
  for index = #self.windows, 1, -1 do
    if self.windows[index].workspace == active then
      return self.windows[index]
    end
  end
  return nil
end

function F:focus_workspace(id)
  local record = self:workspace_record(id)
  if record then
    self.focused = record.monitor
  else
    record = self:create(id, self.focused)
  end
  record.monitor.active = record
  self:cleanup()
end

-- Shows another workspace on a screen whose shown one moved away: any other on
-- it, or a new one under the lowest free number.
function F:refill(monitor)
  for _, record in ipairs(self.workspaces) do
    if record.monitor == monitor and record ~= monitor.active then
      monitor.active = record
      return
    end
  end
  monitor.active = self:create(self:lowest_free(), monitor)
end

function F:apply(action)
  local kind, args = action.kind, action.args
  if kind == "focus" then
    if args.monitor then
      self.focused = self:select_monitor(args.monitor) or self.focused
    elseif args.workspace then
      self:focus_workspace(tonumber(args.workspace))
    elseif args.window then
      local address = args.window:gsub("^address:", "")
      for _, window in ipairs(self.windows) do
        if window.address == address then
          self.focused = window.workspace.monitor
          window.workspace.monitor.active = window.workspace
        end
      end
    end
  elseif kind == "workspace.move" then
    local record = args.workspace and self:workspace_record(tonumber(args.workspace)) or self.focused.active
    local target = self:select_monitor(args.monitor)
    local from = record.monitor
    record.monitor = target
    target.active = record
    if from.active == record then
      self:refill(from)
    end
    self:cleanup()
  elseif kind == "workspace.change_id" then
    local record = self:workspace_record(tonumber(args.workspace))
    assert(record and not self:workspace_record(args.id), "change_id onto a taken id")
    record.id = args.id
    self.renumbered = self.renumbered + 1
  elseif kind == "window.move" then
    local address = args.window:gsub("^address:", "")
    local id = tonumber(args.workspace)
    local record = self:workspace_record(id) or self:create(id, self.focused)
    for _, window in ipairs(self.windows) do
      if window.address == address then
        window.workspace = record
      end
    end
    if args.follow then
      self:focus_workspace(id)
    end
    self:cleanup()
    self:fire("window.move_to_workspace")
  else
    error("the fake has no dispatcher " .. kind)
  end
end

function F:api()
  local world = self
  local function dsp(kind)
    return function(args)
      return { kind = kind, args = args or {} }
    end
  end
  return {
    dsp = {
      focus = dsp("focus"),
      workspace = { move = dsp("workspace.move"), change_id = dsp("workspace.change_id") },
      window = { move = dsp("window.move") },
    },
    workspace_rule = function(options) return world:workspace_rule(options) end,
    dispatch = function(action)
      if type(action) == "function" then
        action()
      else
        world:apply(action)
      end
    end,
    on = function(event, handler)
      world.handlers[event] = world.handlers[event] or {}
      table.insert(world.handlers[event], handler)
    end,
    get_monitors = function()
      local objects = {}
      for _, record in ipairs(world.monitors) do
        objects[#objects + 1] = world:monitor_object(record)
      end
      return objects
    end,
    get_monitor = function(selector) return world:monitor_object(world:select_monitor(selector)) end,
    get_active_monitor = function() return world:monitor_object(world.focused) end,
    get_workspaces = function()
      local objects = {}
      for _, record in ipairs(world.workspaces) do
        objects[#objects + 1] = world:workspace_object(record)
      end
      return objects
    end,
    get_workspace = function(id) return world:workspace_object(world:workspace_record(id)) end,
    get_active_workspace = function() return world:workspace_object(world.focused and world.focused.active) end,
    get_active_window = function()
      local window = world:active_window()
      return window and { address = window.address } or nil
    end,
    get_window = function(selector)
      local address = selector:gsub("^address:", "")
      for _, window in ipairs(world.windows) do
        if window.address == address then
          return { address = address }
        end
      end
      return nil
    end,
  }
end

-- What the tests drive: plugging screens in and out, and opening windows.

-- A screen arrives on a workspace under the lowest free number, as Hyprland
-- puts it, then `monitor.added` fires.
function F:plug(name, description, x)
  local record = { name = name, description = description, id = self.next_monitor_id, x = x or 0 }
  self.next_monitor_id = self.next_monitor_id + 1
  self.monitors[#self.monitors + 1] = record
  record.active = self:create(self:lowest_free(), record)
  self.focused = self.focused or record
  self:fire("monitor.added", self:monitor_object(record))
end

-- Hyprland moves the screen's workspaces to the first other screen in its
-- list (`backup` names another), then fires `monitor.removed` while it still
-- lists the screen.
function F:unplug(name, backup_name)
  local record = self:monitor_record(name)
  local backup = backup_name and self:monitor_record(backup_name)
  for _, other in ipairs(self.monitors) do
    if other ~= record then
      backup = backup or other
    end
  end
  for _, workspace in ipairs(self.workspaces) do
    if workspace.monitor == record then
      workspace.monitor = backup
    end
  end
  record.active = nil
  record.removing = true
  if self.focused == record then
    self.focused = backup
  end
  self:cleanup()
  self:fire("monitor.removed", self:monitor_object(record))
  local kept = {}
  for _, other in ipairs(self.monitors) do
    if other ~= record then
      kept[#kept + 1] = other
    end
  end
  self.monitors = kept
end

-- A window on the focused screen's shown workspace, focused.
function F:open_window()
  local window = { address = string.format("0x%x", self.next_window), workspace = self.focused.active }
  self.next_window = self.next_window + 1
  self.windows[#self.windows + 1] = window
  self:fire("window.open")
  return window
end

function F:close_window(window)
  local kept = {}
  for _, other in ipairs(self.windows) do
    if other ~= window then
      kept[#kept + 1] = other
    end
  end
  self.windows = kept
  self:cleanup()
  self:fire("window.destroy")
end

function F:focus_monitor(name)
  self.focused = self:monitor_record(name)
end

-- The ids on a screen, in order.
function F:ids_on(name)
  local ids = {}
  for _, record in ipairs(self.workspaces) do
    if record.monitor and record.monitor.name == name then
      ids[#ids + 1] = record.id
    end
  end
  table.sort(ids)
  return ids
end

function F:shown_on(name)
  local record = self:monitor_record(name)
  return record and record.active and record.active.id
end

-- The enabled persistent rules, as "id@connector" in id order.
function F:kept()
  local rows = {}
  for _, rule in ipairs(self.rules) do
    if rule.enabled and rule.persistent then
      rows[#rows + 1] = { id = tonumber(rule.workspace), text = rule.workspace .. "@" .. rule.monitor }
    end
  end
  table.sort(rows, function(left, right) return left.id < right.id end)
  local result = {}
  for _, row in ipairs(rows) do
    result[#result + 1] = row.text
  end
  return result
end

-- How many rules, enabled or not, name `workspace`.
function F:rules_for(workspace)
  local count = 0
  for _, rule in ipairs(self.rules) do
    if rule.workspace == workspace then
      count = count + 1
    end
  end
  return count
end

return F
