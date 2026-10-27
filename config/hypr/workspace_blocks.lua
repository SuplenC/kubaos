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

-- A screen as people know it: its description, which is its make, model and
-- serial, or its connector when it reports none.
local function described(monitor)
  local description = monitor.description or ""
  return description ~= "" and description or monitor.name
end

-- Left to right by where Hyprland's layout puts them, top to bottom at the
-- same x.
local function by_layout(left, right)
  if left.x ~= right.x then
    return left.x < right.x
  end
  if left.y ~= right.y then
    return left.y < right.y
  end
  return left.id < right.id
end

-- Connector -> position: "panel" for the laptop panel, and "1", "2" and on for
-- the externals. A screen pinned to a group (`groups`, as `parse_groups`
-- returns them) stands at the group's number; when two of a group's screens
-- are plugged in, the first listed wins and the other counts as unpinned, the
-- panel included, whose unpinned place is "panel". The unpinned externals
-- take the positions left, counted left to right by where Hyprland's layout
-- puts them (top to bottom at the same x), and never the number of a group
-- that pins a screen, plugged in or not. Where the panel sits doesn't count.
function M.positions(monitors, groups)
  local ordered = {}
  for _, monitor in ipairs(monitors) do
    ordered[#ordered + 1] = monitor
  end
  table.sort(ordered, by_layout)

  local positions, pinned = {}, {}
  for _, group in ipairs(groups or {}) do
    local position = tostring(group.number)
    if #group.screens > 0 then
      pinned[position] = true
    end
    local winner = nil
    for _, screen in ipairs(group.screens) do
      for _, monitor in ipairs(ordered) do
        if not positions[monitor.name] and described(monitor) == screen then
          winner = monitor
          break
        end
      end
      if winner then
        break
      end
    end
    if winner then
      positions[winner.name] = position
    end
  end

  local count = 0
  for _, monitor in ipairs(ordered) do
    if not positions[monitor.name] and internal(monitor.name) then
      positions[monitor.name] = "panel"
    elseif not positions[monitor.name] then
      repeat
        count = count + 1
      until not pinned[tostring(count)]
      positions[monitor.name] = tostring(count)
    end
  end
  return positions
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

-- The lowest block not in the set `taken`.
local function lowest_free(taken)
  local block = 0
  while taken[block] do
    block = block + 1
  end
  return block
end

-- The lowest block no position in `blocks` (position -> block) owns.
function M.free_block(blocks)
  local taken = {}
  for _, block in pairs(blocks) do
    taken[block] = true
  end
  return lowest_free(taken)
end

-- Connector -> block for `monitors` (in `sort_screens` order), giving a free
-- block to each position seen for the first time and adding it to `blocks`
-- (position -> block). `positions` is connector -> position; `seen`, position
-- -> screen, takes the screen now at each. A block in `busy` (a set) still
-- holds a window, so a new position passes over it: a block set free by
-- `heal()` may keep windows waiting until the user moves them. The panel
-- standing at a group's position leaves the panel's position with no screen;
-- it still gets its block first, 1-10 when free, so the panel's own group is
-- there to wait and to come back to.
function M.assign(blocks, seen, monitors, positions, busy)
  for _, monitor in ipairs(monitors) do
    if internal(monitor.name) and positions[monitor.name] ~= "panel" and not blocks.panel then
      local taken = {}
      for _, block in pairs(blocks) do
        taken[block] = true
      end
      blocks.panel = taken[0] and lowest_free(taken) or 0
      seen.panel = seen.panel or described(monitor)
    end
  end

  local by_connector = {}
  local ordered = {}
  for _, monitor in ipairs(monitors) do
    ordered[#ordered + 1] = monitor
  end
  -- Fresh blocks go out in position order, so the leftmost gets the lowest.
  table.sort(ordered, function(left, right)
    local l, r = positions[left.name], positions[right.name]
    if (l == "panel") ~= (r == "panel") then
      return l == "panel"
    end
    return (tonumber(l) or 0) < (tonumber(r) or 0)
  end)
  for _, monitor in ipairs(ordered) do
    local position = positions[monitor.name]
    if not blocks[position] then
      local taken = {}
      for _, block in pairs(blocks) do
        taken[block] = true
      end
      for block in pairs(busy or {}) do
        taken[block] = true
      end
      blocks[position] = lowest_free(taken)
    end
    seen[position] = described(monitor)
    by_connector[monitor.name] = blocks[position]
  end
  return by_connector
end

-- Drops from `places` and `seen` every ghost: a position no screen holds
-- (`held` is the set of positions with a screen) whose last screen stands at a
-- held position now, once its block has no windows (`busy` is the set of
-- blocks with one). A screen owns the block of the position it stands at,
-- never a second one waiting elsewhere; a row the old migration invented for
-- it goes, and so does the row of the position it shifted from. While the
-- block still has windows they wait as a group, reachable by the position's
-- Item, and the row stays until they are gone. The panel's position is never
-- a ghost: the panel standing at a group's position comes back to it.
function M.heal(places, seen, held, busy)
  local holding = {}
  for position in pairs(held) do
    if seen[position] then
      holding[seen[position]] = position
    end
  end
  for position, block in pairs(places) do
    if position ~= "panel" and not held[position] and holding[seen[position] or ""] and not busy[block] then
      places[position], seen[position] = nil, nil
    end
  end
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
-- position<TAB>screen", and "<TAB>name" for a group with a name: the
-- connector of the screen at the position now, empty while there is none,
-- and the screen there now or last. The bar
-- (`config/quickshell/services/Hypr.qml`) maps its connector to a block with
-- the first two columns.
--
-- Returns position -> block and position -> screen. A file keyed by screen,
-- "block<TAB>connector<TAB>identity" with an optional fourth column where the
-- screen last sat, comes back as its rows, { block, connector, identity, x },
-- for `migrate()`.
--
-- A damaged file is repaired the same way every time: of two rows for one
-- position the first stands, and of two positions owning one block the one
-- with a screen plugged in, else the first; the position that lost its block
-- is forgotten and gets a fresh one when a screen stands there again.
function M.parse_places(text)
  local rows, legacy = {}, {}
  for line in text:gmatch("[^\n]+") do
    local fields = split(line)
    local block = fields[1]:match("^%d+$") and tonumber(fields[1])
    local third = fields[3] or ""
    if block and is_position(third) then
      rows[#rows + 1] = { block = block, connector = fields[2], position = third, screen = fields[4] or "" }
    elseif block and third ~= "" then
      legacy[#legacy + 1] = { block = block, connector = fields[2], identity = third, x = tonumber(fields[4] or "") }
    end
  end

  local first_of, owner_of = {}, {}
  for _, row in ipairs(rows) do
    if not first_of[row.position] then
      first_of[row.position] = row
      local owner = owner_of[row.block]
      if not owner or (owner.connector == "" and row.connector ~= "") then
        owner_of[row.block] = row
      end
    end
  end
  local places, seen = {}, {}
  for _, row in pairs(owner_of) do
    places[row.position] = row.block
    seen[row.position] = row.screen
  end
  if next(places) or #legacy == 0 then
    legacy = nil
  end
  return places, seen, legacy
end

-- The rows of a screens file keyed by screen as positions, moving no
-- workspace: the panel keeps its block, each external plugged in keeps its own
-- at the position it stands at now, and the blocks of the screens not plugged
-- in take the positions 1, 2 and on, in the order they last sat, else in
-- block order, up to the first position a plugged-in screen holds. A position
-- a screen holds is never given away; the blocks left over wait with no
-- position, as groups the bar shows, since a position invented for them would
-- contradict the one they stand at when they come back. With the lid closed,
-- the panel's row is the one last seen on an internal connector, else the one
-- owning block 0, which the panel always claimed first. `groups` pins screens
-- as `positions()` takes them. Returns position -> block and position ->
-- screen.
function M.migrate(rows, monitors, groups)
  local current, by_identity = {}, {}
  for _, row in ipairs(rows) do
    by_identity[row.identity] = row
    if row.connector ~= "" then
      current[row.connector] = row.identity
    end
  end
  local identities = M.identities(monitors, current)
  local positions = M.positions(monitors, groups)
  local places, seen, claimed, held = {}, {}, {}, {}

  local function take(position, row)
    if row and not claimed[row] then
      places[position] = row.block
      seen[position] = row.identity
      claimed[row] = true
    end
  end

  for _, monitor in ipairs(monitors) do
    local position = positions[monitor.name]
    held[position] = true
    take(position, by_identity[identities[monitor.name]])
  end
  if not held.panel then
    local fallback = nil
    for _, row in ipairs(rows) do
      if internal(row.connector) then
        take("panel", row)
      elseif row.block == 0 and not claimed[row] then
        fallback = row
      end
    end
    if not places.panel then
      take("panel", fallback)
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
    local position = tostring(index)
    if held[position] then
      break
    end
    take(position, row)
  end
  return places, seen
end

-- The workspace groups file, `desktop/workspaces/groups.toml` in the user
-- layer: a table a group, numbered from 1, with an optional `name` and the
-- `screens` pinned to it by description, the first listed first.
--
--   [group.1]
--   name = "Left"
--   screens = ["BNQ ZOWIE XL LCD G4L01663SL0", "Dell Inc. DELL U2724D 5D6E7F8"]
--
-- Returns the groups in number order, { number, name, screens }, or nil and
-- what is wrong, with the line it is on. It reads the TOML such a file needs:
-- comments, `[group.N]` tables, and strings and lists of strings. Anything
-- else, a key it doesn't know, or a screen in two groups is a mistake, so a
-- typo never pins a screen somewhere unexpected.
function M.parse_groups(text)
  local at, line = 1, 1

  local function fail(message)
    error({ problem = "line " .. line .. ": " .. message }, 0)
  end

  local function peek()
    return text:sub(at, at)
  end

  -- Spaces and comments; line ends too with `lines`.
  local function skip(lines)
    while at <= #text do
      local char = peek()
      if char == " " or char == "\t" or char == "\r" then
        at = at + 1
      elseif char == "#" then
        at = text:find("\n", at, true) or #text + 1
      elseif char == "\n" and lines then
        at, line = at + 1, line + 1
      else
        return
      end
    end
  end

  local escapes = { b = "\b", t = "\t", n = "\n", f = "\f", r = "\r", ['"'] = '"', ["\\"] = "\\" }

  local function basic_string()
    if text:sub(at, at + 2) == '"""' then
      fail("a string over several lines; write it on one")
    end
    at = at + 1
    local parts = {}
    while true do
      local stop = text:find('["\\\n]', at)
      if not stop or text:sub(stop, stop) == "\n" then
        fail("a string with no closing quote")
      end
      parts[#parts + 1] = text:sub(at, stop - 1)
      at = stop + 1
      if text:sub(stop, stop) == '"' then
        return table.concat(parts)
      end
      local escape = text:sub(at, at)
      if escapes[escape] then
        parts[#parts + 1] = escapes[escape]
        at = at + 1
      elseif escape == "u" or escape == "U" then
        local digits = escape == "u" and 4 or 8
        local hex = text:sub(at + 1, at + digits)
        local code = #hex == digits and hex:match("^%x+$") and tonumber(hex, 16)
        if not code or code > 0x10FFFF then
          fail("a broken \\" .. escape .. " escape")
        end
        parts[#parts + 1] = utf8.char(code)
        at = at + 1 + digits
      else
        fail("an escape TOML doesn't know, \\" .. escape)
      end
    end
  end

  local function literal_string()
    if text:sub(at, at + 2) == "'''" then
      fail("a string over several lines; write it on one")
    end
    local stop = text:find("['\n]", at + 1)
    if not stop or text:sub(stop, stop) == "\n" then
      fail("a string with no closing quote")
    end
    local value = text:sub(at + 1, stop - 1)
    at = stop + 1
    return value
  end

  local function key()
    local char = peek()
    if char == '"' then
      return basic_string()
    elseif char == "'" then
      return literal_string()
    end
    local bare = text:match("^[%w_-]+", at)
    if not bare then
      fail("a name was expected")
    end
    at = at + #bare
    return bare
  end

  -- `a.b.c` as written, and its parts.
  local function dotted_key()
    local keys = { key() }
    skip(false)
    while peek() == "." do
      at = at + 1
      skip(false)
      keys[#keys + 1] = key()
      skip(false)
    end
    return table.concat(keys, "."), keys
  end

  local function value()
    local char = peek()
    if char == '"' then
      return basic_string()
    elseif char == "'" then
      return literal_string()
    elseif char ~= "[" then
      fail("a value this file doesn't take; names and screens are text in quotes")
    end
    at = at + 1
    local list = {}
    while true do
      skip(true)
      if peek() == "]" then
        at = at + 1
        return list
      end
      local entry = value()
      if type(entry) ~= "string" then
        fail("a list in a list")
      end
      list[#list + 1] = entry
      skip(true)
      if peek() == "," then
        at = at + 1
      elseif peek() ~= "]" then
        fail("a comma or ] was expected in the list")
      end
    end
  end

  local function line_end()
    skip(false)
    if peek() == "\n" then
      at, line = at + 1, line + 1
    elseif peek() ~= "" then
      fail("more on the line after its value")
    end
  end

  local groups, numbers, group = {}, {}, nil

  local function header()
    if text:sub(at, at + 1) == "[[" then
      fail("[[...]] makes a list; a group is [group.1], [group.2] and on")
    end
    at = at + 1
    skip(false)
    local name, keys = dotted_key()
    if peek() ~= "]" then
      fail("[" .. name .. " has no closing ]")
    end
    at = at + 1
    local number = #keys == 2 and keys[1] == "group" and keys[2]:match("^%d+$") and tonumber(keys[2])
    if not number or number < 1 then
      fail("[" .. name .. "] isn't a group; groups are [group.1], [group.2] and on")
    elseif numbers[number] then
      fail("[group." .. number .. "] twice")
    end
    group = { number = number, screens = {}, set = {} }
    numbers[number] = group
    groups[#groups + 1] = group
  end

  local function setting()
    local name = dotted_key()
    if peek() ~= "=" then
      fail("`" .. name .. "` needs = and a value")
    end
    at = at + 1
    skip(false)
    local written = value()
    if not group then
      fail("`" .. name .. "` before any group; settings go under [group.1], [group.2] and on")
    elseif name ~= "name" and name ~= "screens" then
      fail("`" .. name .. "` isn't a setting of a group: name or screens")
    elseif group.set[name] then
      fail("`" .. name .. "` twice in [group." .. group.number .. "]")
    elseif name == "name" and type(written) ~= "string" then
      fail("a group's name is text in quotes")
    elseif name == "screens" and type(written) ~= "table" then
      fail('screens is a list: ["make model serial", ...]')
    end
    group.set[name] = true
    if name == "name" then
      local trimmed = written:match("^%s*(.-)%s*$")
      group.name = trimmed ~= "" and trimmed or nil
    else
      group.screens = written
    end
  end

  local function read()
    while true do
      skip(true)
      if at > #text then
        return
      end
      if peek() == "[" then
        header()
      else
        setting()
      end
      line_end()
    end
  end

  local ok, err = pcall(read)
  if not ok then
    return nil, type(err) == "table" and err.problem or tostring(err)
  end

  table.sort(groups, function(left, right) return left.number < right.number end)
  local pinned_to = {}
  for _, entry in ipairs(groups) do
    entry.set = nil
    for index, screen in ipairs(entry.screens) do
      screen = screen:match("^%s*(.-)%s*$")
      entry.screens[index] = screen
      if screen == "" then
        return nil, "group " .. entry.number .. " lists an empty screen"
      elseif pinned_to[screen] == entry.number then
        return nil, "group " .. entry.number .. " lists " .. screen .. " twice"
      elseif pinned_to[screen] then
        return nil, screen .. " is in group " .. pinned_to[screen] .. " and group " .. entry.number
          .. "; a screen is pinned to one group"
      end
      pinned_to[screen] = entry.number
    end
  end
  return groups
end

-- `positions` is connector -> position for the screens plugged in now, and
-- `names` position -> the name of its group, written as a fifth column on the
-- rows of a group with one.
function M.format_places(places, seen, positions, names)
  local connector = {}
  for name, position in pairs(positions) do
    connector[position] = name
  end
  local rows = {}
  for position, block in pairs(places) do
    local line = block .. "\t" .. (connector[position] or "") .. "\t" .. position .. "\t" .. (seen[position] or "")
    local name = names and names[position]
    if name then
      line = line .. "\t" .. name:gsub("%c", " ")
    end
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
