-- The bookkeeping behind per-screen workspaces (`workspaces.lua`): which block
-- of ids a screen owns, where a workspace joins another screen, the groups a
-- screen shows and how next and previous walk them, which empty workspaces
-- Hyprland keeps, and the screens file. It never calls Hyprland, so
-- `test/lua/workspaces_test.lua` runs it under plain Lua.
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

local function described(monitor)
  local description = monitor.description or ""
  return description ~= "" and description or monitor.name
end

-- Connector -> identity: the screen's description, which is its make, model
-- and serial. Two screens that report the same, such as a pair without
-- serials, get " #2" in connector order when they first appear, and each keeps
-- its identity (from `current`, connector -> identity) while it stays plugged
-- in, so unplugging one of the pair doesn't hand its block to the other.
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

-- The lowest block no identity in `blocks` (identity -> block) owns.
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
-- block to each identity seen for the first time. Adds those to `blocks`.
function M.assign(blocks, monitors, identities)
  local by_connector = {}
  for _, monitor in ipairs(monitors) do
    local identity = identities[monitor.name]
    if not blocks[identity] then
      blocks[identity] = M.free_block(blocks)
    end
    by_connector[monitor.name] = blocks[identity]
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
-- unplugged screens waiting here. Only a screen whose own block isn't the
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
-- of an unplugged screen keeps its own on the screen holding its windows,
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

-- The screens file: one line per screen ever seen, "block<TAB>connector<TAB>
-- identity", the connector empty while it is unplugged. The bar
-- (`config/quickshell/services/Hypr.qml`) maps its connector to a block with
-- it. A fourth column, where the screen last sat, is read past.
function M.parse_blocks(text)
  local blocks, current = {}, {}
  for line in text:gmatch("[^\n]+") do
    local block, connector, identity = line:match("^(%d+)\t([^\t]*)\t([^\t]+)")
    if block then
      blocks[identity] = tonumber(block)
      if connector ~= "" then
        current[connector] = identity
      end
    end
  end
  return blocks, current
end

-- `identities` is connector -> identity for the screens plugged in now.
function M.format_blocks(blocks, identities)
  local connector = {}
  for name, identity in pairs(identities) do
    connector[identity] = name
  end
  local rows = {}
  for identity, block in pairs(blocks) do
    rows[#rows + 1] = { block = block, line = block .. "\t" .. (connector[identity] or "") .. "\t" .. identity }
  end
  table.sort(rows, function(left, right) return left.block < right.block end)
  local lines = {}
  for _, row in ipairs(rows) do
    lines[#lines + 1] = row.line .. "\n"
  end
  return table.concat(lines)
end

return M
