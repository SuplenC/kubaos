-- The bookkeeping behind per-screen workspaces (`workspaces.lua`): where each
-- screen stands on the desk and which block of ids that position owns, where a
-- workspace joins another screen, the groups a screen shows and how next and
-- previous walk them, which empty workspaces Hyprland keeps, and the screens
-- file. It never calls Hyprland, so `test/lua/workspaces_test.lua` runs it
-- under plain Lua.
--
-- Adapted from Omarchy's default/hypr/monitor-workspaces.lua (basecamp/omarchy,
-- MIT) at 5d8cc3313b2c60786d4e5019a68906873aed36bd.

local M = { SLOTS = 10 }

-- Block 0 holds ids 1-10, block 1 holds 11-20, and so on.
function M.block_of(id)
  return (id - 1) // M.SLOTS
end

function M.slot_of(id)
  return (id - 1) % M.SLOTS + 1
end

function M.id_of(block, slot)
  return block * M.SLOTS + slot
end

local function internal(name)
  return name:match("^eDP") or name:match("^LVDS") or name:match("^DSI")
end

-- Screens in the order they claim blocks: the laptop panel first, so it keeps
-- 1-10 whatever is plugged in, then by Hyprland's monitor id.
function M.sort_screens(monitors)
  local sorted = {}
  for _, monitor in ipairs(monitors) do
    sorted[#sorted + 1] = monitor
  end
  table.sort(sorted, function(left, right)
    local l, r = internal(left.name) ~= nil, internal(right.name) ~= nil
    if l ~= r then
      return l
    end
    return left.id < right.id
  end)
  return sorted
end

-- Connector -> position: "panel" for the laptop panel, and "1", "2" and on for
-- the externals, counted left to right by where Hyprland's layout puts them
-- (top to bottom at the same x). Where the panel sits doesn't count.
function M.positions(monitors)
  local positions, externals = {}, {}
  for _, monitor in ipairs(monitors) do
    if internal(monitor.name) then
      positions[monitor.name] = "panel"
    else
      externals[#externals + 1] = monitor
    end
  end
  table.sort(externals, function(left, right)
    if left.x ~= right.x then
      return left.x < right.x
    end
    if left.y ~= right.y then
      return left.y < right.y
    end
    return left.id < right.id
  end)
  for index, monitor in ipairs(externals) do
    positions[monitor.name] = tostring(index)
  end
  return positions
end

-- A screen as people know it: its description, which is its make, model and
-- serial, or its connector when it reports none.
local function described(monitor)
  local description = monitor.description or ""
  return description ~= "" and description or monitor.name
end

-- Connector -> identity: the screen's description. Two screens that report the
-- same, such as a pair without serials, get " #2" in connector order, and each
-- keeps its identity (from `current`, connector -> identity) while it stays
-- plugged in. Only `migrate()` uses it, to find the rows of a screens file
-- keyed by screen.
function M.identities(monitors, current)
  local identities, taken = {}, {}

  for _, monitor in ipairs(monitors) do
    local identity = current[monitor.name]
    if identity and not taken[identity] and identity:gsub(" #%d+$", "") == described(monitor) then
      identities[monitor.name] = identity
      taken[identity] = true
    end
  end

  local ordered = {}
  for _, monitor in ipairs(monitors) do
    ordered[#ordered + 1] = monitor
  end
  table.sort(ordered, function(left, right) return left.name < right.name end)

  for _, monitor in ipairs(ordered) do
    if not identities[monitor.name] then
      local identity, count = described(monitor), 1
      while taken[identity] do
        count = count + 1
        identity = described(monitor) .. " #" .. count
      end
      identities[monitor.name] = identity
      taken[identity] = true
    end
  end

  return identities
end

-- The lowest block no position in `blocks` (position -> block) owns.
function M.free_block(blocks)
  local taken = {}
  for _, block in pairs(blocks) do
    taken[block] = true
  end
  local block = 0
  while taken[block] do
    block = block + 1
  end
  return block
end

-- Connector -> block for `monitors` (in `sort_screens` order), giving a free
-- block to each position seen for the first time and adding it to `blocks`
-- (position -> block). `positions` is connector -> position; `seen`, position
-- -> screen, takes the screen now at each.
function M.assign(blocks, seen, monitors, positions)
  local by_connector = {}
  for _, monitor in ipairs(monitors) do
    local position = positions[monitor.name]
    if not blocks[position] then
      blocks[position] = M.free_block(blocks)
    end
    seen[position] = described(monitor)
    by_connector[monitor.name] = blocks[position]
  end
  return by_connector
end

-- Where a screen's workspaces go after its own: the id after the highest in
-- use in `block` (`taken` is the set of ids in use), or the first free one
-- when slot 10 is taken. Nil when all ten are.
function M.append_slot(block, taken)
  local first, last = M.id_of(block, 1), M.id_of(block, M.SLOTS)
  local highest = first - 1
  for id in pairs(taken) do
    if id >= first and id <= last and id > highest then
      highest = id
    end
  end
  if highest < last then
    return highest + 1
  end
  for id = first, last do
    if not taken[id] then
      return id
    end
  end
  return nil
end

-- The id a workspace takes when it joins `block`: its own slot there when that
-- is free, else the block's first free slot. Nil when all ten are taken.
function M.adopt_id(block, id, taken)
  local wanted = M.id_of(block, M.slot_of(id))
  if not taken[wanted] then
    return wanted
  end
  for slot = 1, M.SLOTS do
    local candidate = M.id_of(block, slot)
    if not taken[candidate] then
      return candidate
    end
  end
  return nil
end

-- A screen's groups, in the order its bar shows them and next and previous
-- walk them: block order, which is id order, the order Hyprland's swipe walks
-- a screen's workspaces in. Its own block is one, with its first workspace
-- whether it exists or not and its others wherever they are, as SUPER + 1-0
-- reach them. Each other block with workspaces on this screen is one, those of
-- positions no screen holds waiting here. Only a screen whose own block isn't the
-- lowest, the external holding the laptop's group while the lid is closed,
-- sees a waiting group before its own. `workspaces` lists { id, monitor } with
-- the connector it is on.
function M.groups(own, connector, workspaces)
  local sets, order = { [own] = { [M.id_of(own, 1)] = true } }, { own }
  for _, workspace in ipairs(workspaces) do
    local block = M.block_of(workspace.id)
    if block == own or workspace.monitor == connector then
      if not sets[block] then
        sets[block] = {}
        order[#order + 1] = block
      end
      sets[block][workspace.id] = true
    end
  end
  table.sort(order)

  local groups = {}
  for _, block in ipairs(order) do
    local ids = {}
    for id in pairs(sets[block]) do
      ids[#ids + 1] = id
    end
    table.sort(ids)
    groups[#groups + 1] = { block = block, ids = ids }
  end
  return groups
end

-- The group and position of `id` in `groups`, or nil.
local function find(groups, id)
  for g, group in ipairs(groups) do
    for i, candidate in ipairs(group.ids) do
      if candidate == id then
        return g, i
      end
    end
  end
  return nil
end

-- Where next (`direction` 1) or previous (-1) goes from `current`, nil to
-- stay: the workspace beside it in `groups`, across from one group into the
-- next. It walks what Hyprland's swipe walks, the empty workspaces `kept()`
-- keeps included, and like the swipe it never creates one.
function M.step(groups, current, direction)
  local g, i = find(groups, current)
  if not g then
    return nil
  end
  local ids = groups[g].ids
  if ids[i + direction] then
    return ids[i + direction]
  end
  local beside = groups[g + direction]
  if not beside then
    return nil
  end
  return direction > 0 and beside.ids[1] or beside.ids[#beside.ids]
end

-- The first workspace of the group beside the one holding `current`, in
-- `direction`, or nil.
function M.group_step(groups, current, direction)
  local g = find(groups, current)
  local beside = g and groups[g + direction]
  return beside and beside.ids[1]
end

-- The empty workspaces a block keeps on `connector`, into `result`: its first,
-- so the swipe reaches it, and the one after its last with windows, where the
-- swipe opens a new workspace. Nothing after a window on slot 10.
-- `occupied` is the set of the block's ids with windows.
local function keep_block(result, block, connector, occupied)
  local first, last = M.id_of(block, 1), M.id_of(block, M.SLOTS)
  result[first] = connector
  local highest = nil
  for id in pairs(occupied) do
    if not highest or id > highest then
      highest = id
    end
  end
  if highest and highest < last then
    result[highest + 1] = connector
  end
end

-- Id -> connector: the workspaces Hyprland keeps even when empty, and the
-- screen each stays on. Every plugged-in screen keeps its own block's; a block
-- of a position no screen holds keeps its own on the screen holding its windows,
-- while it has any. `screens` lists { name, block } for the screens plugged
-- in, `workspaces` { id, monitor, windows } with the connector each is on.
function M.kept(screens, workspaces)
  local owned, occupied, holder = {}, {}, {}
  for _, screen in ipairs(screens) do
    owned[screen.block] = screen.name
  end
  for _, workspace in ipairs(workspaces) do
    if (workspace.windows or 0) > 0 then
      local block = M.block_of(workspace.id)
      occupied[block] = occupied[block] or {}
      occupied[block][workspace.id] = true
      if not holder[block] or workspace.id < holder[block].id then
        holder[block] = workspace
      end
    end
  end

  local result = {}
  for block, connector in pairs(owned) do
    keep_block(result, block, connector, occupied[block] or {})
  end
  for block, ids in pairs(occupied) do
    if not owned[block] and holder[block].monitor then
      keep_block(result, block, holder[block].monitor, ids)
    end
  end
  return result
end

local function split(line)
  local fields, start = {}, 1
  while true do
    local tab = line:find("\t", start, true)
    if not tab then
      fields[#fields + 1] = line:sub(start)
      return fields
    end
    fields[#fields + 1] = line:sub(start, tab - 1)
    start = tab + 1
  end
end

local function is_position(field)
  return field == "panel" or field:match("^%d+$") ~= nil
end

-- The screens file: one line per position ever seen, "block<TAB>connector<TAB>
-- position<TAB>screen": the connector of the screen at the position now, empty
-- while there is none, and the screen there now or last. The bar
-- (`config/quickshell/services/Hypr.qml`) maps its connector to a block with
-- the first two columns.
--
-- Returns position -> block and position -> screen. A file keyed by screen,
-- "block<TAB>connector<TAB>identity" with an optional fourth column where the
-- screen last sat, comes back as its rows, { block, connector, identity, x },
-- for `migrate()`.
function M.parse_places(text)
  local places, seen, legacy = {}, {}, {}
  for line in text:gmatch("[^\n]+") do
    local fields = split(line)
    local block = fields[1]:match("^%d+$") and tonumber(fields[1])
    local third = fields[3] or ""
    if block and is_position(third) then
      places[third] = block
      seen[third] = fields[4] or ""
    elseif block and third ~= "" then
      legacy[#legacy + 1] = { block = block, connector = fields[2], identity = third, x = tonumber(fields[4] or "") }
    end
  end
  if next(places) or #legacy == 0 then
    legacy = nil
  end
  return places, seen, legacy
end

-- The rows of a screens file keyed by screen as positions, moving no
-- workspace: the panel keeps its block, each external plugged in keeps its own
-- at the position it stands at now, and the blocks of the screens not plugged
-- in take the positions after those, in the order they last sat, else in block
-- order. With the lid closed, the panel's row is the one last seen on an
-- internal connector. Returns position -> block and position -> screen.
function M.migrate(rows, monitors)
  local current, by_identity = {}, {}
  for _, row in ipairs(rows) do
    by_identity[row.identity] = row
    if row.connector ~= "" then
      current[row.connector] = row.identity
    end
  end
  local identities = M.identities(monitors, current)
  local positions = M.positions(monitors)
  local places, seen, claimed = {}, {}, {}

  local function take(position, row)
    if row and not claimed[row] then
      places[position] = row.block
      seen[position] = row.identity
      claimed[row] = true
    end
  end

  local panel_plugged, last = false, 0
  for _, monitor in ipairs(monitors) do
    local position = positions[monitor.name]
    take(position, by_identity[identities[monitor.name]])
    if position == "panel" then
      panel_plugged = true
    else
      last = math.max(last, tonumber(position))
    end
  end
  if not panel_plugged then
    for _, row in ipairs(rows) do
      if internal(row.connector) then
        take("panel", row)
      end
    end
  end

  local rest = {}
  for _, row in ipairs(rows) do
    if not claimed[row] then
      rest[#rest + 1] = row
    end
  end
  table.sort(rest, function(left, right)
    local l, r = left.x or math.huge, right.x or math.huge
    if l ~= r then
      return l < r
    end
    return left.block < right.block
  end)
  for index, row in ipairs(rest) do
    take(tostring(last + index), row)
  end
  return places, seen
end

-- `positions` is connector -> position for the screens plugged in now.
function M.format_places(places, seen, positions)
  local connector = {}
  for name, position in pairs(positions) do
    connector[position] = name
  end
  local rows = {}
  for position, block in pairs(places) do
    local line = block .. "\t" .. (connector[position] or "") .. "\t" .. position .. "\t" .. (seen[position] or "")
    rows[#rows + 1] = { block = block, line = line }
  end
  table.sort(rows, function(left, right) return left.block < right.block end)
  local lines = {}
  for _, row in ipairs(rows) do
    lines[#lines + 1] = row.line .. "\n"
  end
  return table.concat(lines)
end

return M
