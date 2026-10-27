-- Per-screen workspaces (`config/hypr/workspaces.lua` and `workspace_blocks.lua`)
-- under plain Lua, against `fake_hyprland.lua`.
--
--   lua5.5 test/lua/workspaces_test.lua REPO
--
-- Prints an ok or FAIL line per test and exits 1 when any fails.

local repo = assert(arg[1], "usage: lua5.5 test/lua/workspaces_test.lua REPO")
package.path = repo .. "/config/hypr/?.lua;" .. repo .. "/test/lua/?.lua;" .. package.path

local Fake = require("fake_hyprland")
local blocks = require("workspace_blocks")

local failed = 0

local function test(name, fn)
  local ok, err = pcall(fn)
  if ok then
    print("ok   " .. name)
  else
    failed = failed + 1
    print("FAIL " .. name .. ": " .. tostring(err))
  end
end

local function show(value)
  if type(value) ~= "table" then
    return tostring(value)
  end
  local parts = {}
  for _, v in ipairs(value) do
    parts[#parts + 1] = show(v)
  end
  return "{" .. table.concat(parts, ", ") .. "}"
end

local function same(actual, expected, what)
  if show(actual) ~= show(expected) then
    error(string.format("%s: got %s, want %s", what, show(actual), show(expected)), 2)
  end
end

local function set(list)
  local result = {}
  for _, id in ipairs(list) do
    result[id] = true
  end
  return result
end

local real_getenv = os.getenv

-- A new fake on a fresh state directory and user layer, without the module.
local function bare()
  local dir = os.tmpname()
  os.remove(dir)
  assert(os.execute("mkdir -p '" .. dir .. "/kubaos/hypr' '" .. dir .. "/config/kubaos/desktop/workspaces'"))
  os.getenv = function(key)
    if key == "XDG_STATE_HOME" then
      return dir
    elseif key == "XDG_CONFIG_HOME" then
      return dir .. "/config"
    end
    return real_getenv(key)
  end
  local world = Fake.new()
  _G.hl = world.hl
  _G.dot = {}
  world.state = dir .. "/kubaos/hypr"
  world.groups = dir .. "/config/kubaos/desktop/workspaces/groups.toml"
  return world
end

-- A fresh state directory and a fresh copy of the module on a new fake.
local function session()
  local world = bare()
  package.loaded.workspaces = nil
  return world, require("workspaces")
end

-- The module again on the same world and state, as a config reload makes it.
local function reload(world)
  package.loaded.workspaces = nil
  world.handlers = {}
  return require("workspaces")
end

local function read(path)
  local file = assert(io.open(path, "r"))
  local text = file:read("a")
  file:close()
  return text
end

local function write(path, text)
  local file = assert(io.open(path, "w"))
  file:write(text)
  file:close()
end

local LG_A = "LG Electronics LG ULTRAGEAR 312NTAB1A234"
local LG_B = "LG Electronics LG ULTRAGEAR 312NTAB1A999"
local LG_C = "LG Electronics LG ULTRAGEAR+ 000NTAB0C000"
local BENQ = "BNQ BenQ GW2480 ET00000000SL0"
local PANEL = "BOE 0x0BCA"

-- The laptop and one external at login.
local function two_screens()
  local world, module = session()
  world:plug("eDP-1", PANEL, 0)
  world:plug("DP-1", LG_A, 1920)
  world:fire("hyprland.start")
  return world, module
end

local function run(action)
  hl.dispatch(action)
end

-- The pure bookkeeping.

test("a workspace id names its block and slot", function()
  same({ blocks.block_of(1), blocks.block_of(10), blocks.block_of(11), blocks.block_of(20) }, { 0, 0, 1, 1 }, "blocks")
  same({ blocks.slot_of(1), blocks.slot_of(10), blocks.slot_of(13) }, { 1, 10, 3 }, "slots")
  same(blocks.id_of(2, 4), 24, "id")
end)

test("the free block is the lowest no screen owns", function()
  same(blocks.free_block({}), 0, "empty")
  same(blocks.free_block({ a = 0, b = 2 }), 1, "gap")
  same(blocks.free_block({ a = 0, b = 1 }), 2, "full")
end)

test("the laptop panel claims a block before any external", function()
  local sorted = blocks.sort_screens({ { name = "DP-1", id = 0 }, { name = "eDP-1", id = 3 }, { name = "HDMI-A-1", id = 1 } })
  same({ sorted[1].name, sorted[2].name, sorted[3].name }, { "eDP-1", "DP-1", "HDMI-A-1" }, "order")
end)

test("two screens reporting the same description get two identities", function()
  local identities = blocks.identities({
    { name = "DP-2", description = "Generic Monitor" },
    { name = "DP-1", description = "Generic Monitor" },
  }, {})
  same({ identities["DP-1"], identities["DP-2"] }, { "Generic Monitor", "Generic Monitor #2" }, "identities")
end)

test("a screen keeps its identity while it stays plugged in", function()
  local identities = blocks.identities({ { name = "DP-1", description = "Generic Monitor" } }, { ["DP-1"] = "Generic Monitor #2" })
  same(identities["DP-1"], "Generic Monitor #2", "kept")
end)

test("a screen without a description is known by its connector", function()
  same(blocks.identities({ { name = "HEADLESS-2", description = "" } }, {})["HEADLESS-2"], "HEADLESS-2", "identity")
end)

test("the next free slot follows the highest in use", function()
  same(blocks.append_slot(0, set({ 1, 2, 3, 12 })), 4, "after 3")
  same(blocks.append_slot(1, set({ 1, 2 })), 11, "empty block")
  same(blocks.append_slot(0, set({ 1, 10 })), 2, "10 taken: first free")
  same(blocks.append_slot(0, set({ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 })), nil, "full")
end)

test("a joining workspace keeps its slot when it is free", function()
  same(blocks.adopt_id(1, 3, set({ 11 })), 13, "slot free")
  same(blocks.adopt_id(1, 3, set({ 11, 13 })), 12, "slot taken")
end)

test("externals count from the leftmost, wherever the panel sits", function()
  local positions = blocks.positions({
    { name = "eDP-1", id = 0, x = 4480, y = 0 },
    { name = "DP-8", id = 1, x = 2560, y = 0 },
    { name = "DP-9", id = 2, x = 640, y = 0 },
  })
  same({ positions["eDP-1"], positions["DP-9"], positions["DP-8"] }, { "panel", "1", "2" }, "positions")
end)

test("externals stacked at the same x count from the top", function()
  local positions = blocks.positions({ { name = "DP-1", id = 0, x = 0, y = 1440 }, { name = "DP-2", id = 1, x = 0, y = 0 } })
  same({ positions["DP-2"], positions["DP-1"] }, { "1", "2" }, "positions")
end)

test("the screens file reads back what it wrote", function()
  local places, seen = { panel = 0, ["1"] = 1, ["2"] = 2 }, { panel = PANEL, ["1"] = LG_A, ["2"] = LG_B }
  local text = blocks.format_places(places, seen, { ["eDP-1"] = "panel", ["DP-3"] = "1" })
  same(text, "0\teDP-1\tpanel\t" .. PANEL .. "\n1\tDP-3\t1\t" .. LG_A .. "\n2\t\t2\t" .. LG_B .. "\n", "text")
  local parsed, parsed_seen, legacy = blocks.parse_places(text)
  same({ parsed.panel, parsed["1"], parsed["2"], parsed_seen["2"], legacy }, { 0, 1, 2, LG_B, nil }, "parsed")
end)

-- The file of a laptop that knew its screens by make and serial: the panel,
-- two home screens unplugged, and the two work screens plugged in, the one on
-- DP-9 to the left.
local SCREENS_BY_SERIAL = "0\teDP-1\t" .. PANEL .. "\n1\t\t" .. LG_C .. "\n2\t\t" .. BENQ
  .. "\n3\tDP-8\t" .. LG_A .. "\n4\tDP-9\t" .. LG_B .. "\n"

test("a screens file keyed by screen is read as one to migrate", function()
  local places, _, legacy = blocks.parse_places(SCREENS_BY_SERIAL)
  same({ next(places), #legacy, legacy[5].block, legacy[5].identity }, { nil, 5, 4, LG_B }, "legacy")
end)

test("migrating, the plugged-in externals take the positions they stand at and the unplugged get no position", function()
  local _, _, legacy = blocks.parse_places(SCREENS_BY_SERIAL)
  local places, seen = blocks.migrate(legacy, {
    { name = "eDP-1", id = 0, x = 4480, y = 0, description = PANEL },
    { name = "DP-8", id = 1, x = 2560, y = 0, description = LG_A },
    { name = "DP-9", id = 2, x = 640, y = 0, description = LG_B },
  })
  same({ places.panel, places["1"], places["2"], places["3"], places["4"] }, { 0, 4, 3, nil, nil }, "places")
  same({ seen["1"], seen["2"] }, { LG_B, LG_A }, "seen")
end)

test("migrating, unplugged screens take the positions before the first a plugged screen holds", function()
  local _, _, legacy = blocks.parse_places("0\teDP-1\t" .. PANEL .. "\n1\t\t" .. LG_C .. "\n2\tDP-1\t" .. LG_A .. "\n3\t\t" .. BENQ .. "\n")
  local places = blocks.migrate(legacy, {
    { name = "eDP-1", id = 0, x = 0, y = 0, description = PANEL },
    { name = "DP-1", id = 1, x = 3840, y = 0, description = LG_A },
  })
  same({ places.panel, places["1"], places["2"], places["3"] }, { 0, 2, nil, nil }, "places")
end)

test("migrating with the lid closed, a panel row written with the lid closed is found by its block", function()
  local _, _, legacy = blocks.parse_places("0\t\t" .. PANEL .. "\n1\tDP-1\t" .. LG_A .. "\n")
  local places, seen = blocks.migrate(legacy, { { name = "DP-1", id = 1, x = 0, y = 0, description = LG_A } })
  same({ places.panel, places["1"], seen.panel }, { 0, 1, PANEL }, "places")
end)

test("a screens file naming a position twice keeps its first row", function()
  local places, seen = blocks.parse_places("3\tDP-1\t1\tA\n5\tDP-2\t1\tB\n")
  same({ places["1"], seen["1"], places["2"] }, { 3, "A", nil }, "places")
end)

test("a screens file giving two positions one block keeps the row of the screen plugged in", function()
  local places = blocks.parse_places("5\t\t1\tA\n5\tDP-2\t2\tB\n5\t\t3\tC\n")
  same({ places["1"], places["2"], places["3"] }, { nil, 5, nil }, "places")
end)

test("a position whose screen stands at another is a ghost, dropped once its block is empty", function()
  local places = { panel = 0, ["1"] = 4, ["2"] = 3, ["3"] = 1, ["4"] = 2 }
  local seen = { panel = PANEL, ["1"] = BENQ, ["2"] = LG_A, ["3"] = LG_A, ["4"] = BENQ }

  blocks.heal(places, seen, { ["1"] = true, ["2"] = true, panel = true }, { [2] = true })

  same({ places.panel, places["1"], places["2"], places["3"], places["4"] }, { 0, 4, 3, nil, 2 }, "places")
  same({ seen["3"], seen["4"] }, { nil, BENQ }, "seen")
end)

test("a position never seen gets a block no position owns and no workspace lives in", function()
  local places, seen = { panel = 0, ["1"] = 3 }, { panel = PANEL, ["1"] = LG_A }
  local monitors = { { name = "eDP-1", id = 0, description = PANEL }, { name = "DP-1", id = 1, description = LG_A }, { name = "DP-2", id = 2, description = LG_B } }

  local by_connector = blocks.assign(places, seen, monitors, { ["eDP-1"] = "panel", ["DP-1"] = "1", ["DP-2"] = "2" }, { [1] = true })

  same(by_connector["DP-2"], 2, "block")
end)

test("migrating, unplugged screens follow in the order they last sat", function()
  local _, _, legacy = blocks.parse_places("0\teDP-1\t" .. PANEL .. "\t0\n1\t\t" .. LG_A .. "\t3840\n2\t\t" .. LG_B .. "\t1920\n")
  local places = blocks.migrate(legacy, { { name = "eDP-1", id = 0, x = 0, y = 0, description = PANEL } })
  same({ places.panel, places["1"], places["2"] }, { 0, 2, 1 }, "places")
end)

test("migrating with the lid closed, the row last on the panel's connector stays the panel's", function()
  local _, _, legacy = blocks.parse_places("0\teDP-1\t" .. PANEL .. "\n1\tDP-1\t" .. LG_A .. "\n")
  local places = blocks.migrate(legacy, { { name = "DP-1", id = 1, x = 0, y = 0, description = LG_A } })
  same({ places.panel, places["1"] }, { 0, 1 }, "places")
end)

-- The workspace groups file and the screens it pins.

-- What `parse_groups` read, as "number name: screen, screen" lines.
local function groups_read(text)
  local groups, problem = blocks.parse_groups(text)
  assert(groups, problem)
  local lines = {}
  for _, group in ipairs(groups) do
    lines[#lines + 1] = group.number .. " " .. tostring(group.name) .. ": " .. table.concat(group.screens, ", ")
  end
  return lines
end

local function problem_of(text)
  local groups, problem = blocks.parse_groups(text)
  assert(not groups, "read as groups")
  return problem
end

test("the groups file reads each group's number, name and screens, first listed first", function()
  local text = table.concat({
    "# Pinned screens.",
    "[group.2]  # the centre",
    'name = "Centre"',
    "screens = [",
    '  "LG Electronics LG ULTRAGEAR+ 510NTCZN7787",  # home',
    "  'Dell Inc. DELL U2724D 1A2B3C4', # work",
    "]",
    "",
    '[group."1"]',
    'name = "  Left\\u00e9 "',
    'screens = ["BNQ ZOWIE XL LCD G4L01663SL0"]',
    "[group.3]",
    'screens = []',
    "[group.4]",
    'name = ""',
  }, "\n")

  same(groups_read(text), {
    "1 Lefté: BNQ ZOWIE XL LCD G4L01663SL0",
    "2 Centre: LG Electronics LG ULTRAGEAR+ 510NTCZN7787, Dell Inc. DELL U2724D 1A2B3C4",
    "3 nil: ",
    "4 nil: ",
  }, "groups")
end)

test("the seed pins nothing, and its example, uncommented, reads", function()
  local seed = read(repo .. "/user-layer/desktop/workspaces/groups.toml")
  local example = seed:gsub("\n# (%[group)", "\n%1"):gsub("\n# (name =)", "\n%1"):gsub("\n# (screens =)", "\n%1")

  same(groups_read(seed), {}, "seed")
  same(groups_read(example), {
    "1 Left: BNQ ZOWIE XL LCD G4L01663SL0, Dell Inc. DELL U2724D 5D6E7F8",
    "2 Centre: LG Electronics LG ULTRAGEAR+ 510NTCZN7787, Dell Inc. DELL U2724D 1A2B3C4",
  }, "example")
end)

-- The groups files in test/fixtures/workspace-groups/`kind`, which the menu's
-- reader (crates/kotodama-kubaos/src/desk/groups/tests.rs) holds to the same
-- verdicts, as { name, text }.
local function group_fixtures(kind)
  local dir = repo .. "/test/fixtures/workspace-groups/" .. kind
  local list = assert(io.popen("ls '" .. dir .. "'"))
  local found = {}
  for name in list:lines() do
    found[#found + 1] = { name = name, text = read(dir .. "/" .. name) }
  end
  list:close()
  assert(#found > 0, "no fixtures in " .. dir)
  return found
end

test("every groups file the menu takes, Hyprland takes", function()
  for _, fixture in ipairs(group_fixtures("valid")) do
    local groups, problem = blocks.parse_groups(fixture.text)
    assert(groups, fixture.name .. ": " .. tostring(problem))
  end
end)

test("every groups file the menu refuses, Hyprland refuses", function()
  for _, fixture in ipairs(group_fixtures("invalid")) do
    assert(not blocks.parse_groups(fixture.text), fixture.name .. " was read")
  end
end)

test("an empty groups file, or one of comments only, pins nothing", function()
  same(groups_read(""), {}, "empty")
  same(groups_read("# [group.1]\n# screens = [\"A\"]\n\n"), {}, "comments")
end)

test("a groups file that doesn't read says what is wrong and on which line", function()
  same(problem_of('[group.1]\nscreen = ["A"]\n'), "line 2: `screen` isn't a setting of a group: name or screens", "unknown key")
  same(problem_of('[groups.1]\n'), "line 1: [groups.1] isn't a group; groups are [group.1], [group.2] and on", "table")
  same(problem_of('[group.0]\n'), "line 1: [group.0] isn't a group; groups are [group.1], [group.2] and on", "zero")
  same(problem_of('[[group]]\n'), "line 1: [[...]] makes a list; a group is [group.1], [group.2] and on", "array of tables")
  same(problem_of('screens = ["A"]\n'), "line 1: `screens` before any group; settings go under [group.1], [group.2] and on", "no group")
  same(problem_of('[group.1]\nname = 3\n'), "line 2: a value this file doesn't take; names and screens are text in quotes", "number")
  same(problem_of('[group.1]\nname = "Left\n'), "line 2: a string with no closing quote", "open string")
  same(problem_of('[group.1]\nscreens = "A"\n'), 'line 2: screens is a list: ["make model serial", ...]', "not a list")
  same(problem_of('[group.1]\nscreens = ["A" "B"]\n'), "line 2: a comma or ] was expected in the list", "no comma")
  same(problem_of('[group.1]\nname = "A" name = "B"\n'), "line 2: more on the line after its value", "two on a line")
  same(problem_of('[group.1]\n[group.1]\n'), "line 2: [group.1] twice", "group twice")
  same(problem_of('[group.1]\nname = "A"\nname = "B"\n'), "line 3: `name` twice in [group.1]", "key twice")
end)

test("a screen pinned to two groups, or twice to one, is a mistake", function()
  same(problem_of('[group.1]\nscreens = ["A"]\n[group.2]\nscreens = ["B", " A "]\n'),
    "A is in group 1 and group 2; a screen is pinned to one group", "two groups")
  same(problem_of('[group.1]\nscreens = ["A", "A"]\n'), "group 1 lists A twice", "twice")
  same(problem_of('[group.1]\nscreens = [" "]\n'), "group 1 lists an empty screen", "empty")
end)

-- The screens of `positions()` in a row: { name, description, x }.
local function row_of(...)
  local monitors = {}
  for index, screen in ipairs({ ... }) do
    monitors[#monitors + 1] = { name = screen[1], description = screen[2], id = index - 1, x = screen[3], y = 0 }
  end
  return monitors
end

local function pinned(...)
  local groups = {}
  for _, spec in ipairs({ ... }) do
    groups[#groups + 1] = { number = spec[1], screens = { table.unpack(spec, 2) } }
  end
  return groups
end

local function positions_of(monitors, groups, ...)
  local positions = blocks.positions(monitors, groups)
  local result = {}
  for _, name in ipairs({ ... }) do
    result[#result + 1] = positions[name]
  end
  return result
end

test("a pinned screen stands at its group's number wherever the layout puts it", function()
  local monitors = row_of({ "DP-8", LG_A, 0 }, { "DP-9", LG_B, 2560 }, { "eDP-1", PANEL, 4480 })

  same(positions_of(monitors, pinned({ 1, LG_B }), "DP-9", "DP-8", "eDP-1"), { "1", "2", "panel" }, "positions")
end)

test("an unpinned screen never takes the number of a group whose screen is absent", function()
  local monitors = row_of({ "DP-1", LG_A, 0 }, { "DP-2", LG_B, 1920 })

  same(positions_of(monitors, pinned({ 1, BENQ }), "DP-1", "DP-2"), { "2", "3" }, "positions")
end)

test("of two screens of one group plugged in, the first listed wins and the other takes a free number", function()
  local monitors = row_of({ "DP-1", LG_A, 0 }, { "DP-2", BENQ, 1920 }, { "DP-3", LG_C, 3840 })

  same(positions_of(monitors, pinned({ 1, BENQ, LG_A }, { 2, LG_C }), "DP-2", "DP-3", "DP-1"), { "1", "2", "3" }, "positions")
end)

test("of two screens reporting one description, the pin goes to the leftmost", function()
  local monitors = row_of({ "DP-1", LG_A, 1920 }, { "DP-2", LG_A, 0 })

  same(positions_of(monitors, pinned({ 2, LG_A }), "DP-2", "DP-1"), { "2", "1" }, "positions")
end)

test("a group with no screens, only a name, pins nothing", function()
  local monitors = row_of({ "DP-1", LG_A, 0 })

  same(positions_of(monitors, { { number = 1, name = "Left", screens = {} } }, "DP-1"), { "1" }, "positions")
end)

test("the panel pinned to a group stands at its number until a screen listed before it is plugged in", function()
  local alone = row_of({ "eDP-1", PANEL, 0 })
  local docked = row_of({ "eDP-1", PANEL, 0 }, { "DP-1", LG_A, 1920 })

  same(positions_of(alone, pinned({ 2, LG_A, PANEL }), "eDP-1"), { "2" }, "alone")
  same(positions_of(docked, pinned({ 2, LG_A, PANEL }), "eDP-1", "DP-1"), { "panel", "2" }, "docked")
  same(positions_of(docked, pinned({ 2, PANEL, LG_A }), "eDP-1", "DP-1"), { "2", "1" }, "listed first")
end)

test("the panel standing at a group's number keeps 1-10 for its own group", function()
  local places, seen = {}, {}
  local monitors = row_of({ "eDP-1", PANEL, 0 })

  local by_connector = blocks.assign(places, seen, monitors, { ["eDP-1"] = "2" }, {})

  same({ places.panel, places["2"], by_connector["eDP-1"], seen.panel }, { 0, 1, 1, PANEL }, "blocks")
end)

test("the panel's position is never dropped as a ghost while the panel stands elsewhere", function()
  local places, seen = { panel = 0, ["2"] = 1 }, { panel = PANEL, ["2"] = PANEL }

  blocks.heal(places, seen, { ["2"] = true }, {})

  same(places.panel, 0, "kept")
end)

test("a group's name goes in a fifth column of its rows", function()
  local text = blocks.format_places({ panel = 0, ["1"] = 1 }, { panel = PANEL, ["1"] = LG_A }, { ["DP-1"] = "1" },
    { ["1"] = "Left\tside" })

  same(text, "0\t\tpanel\t" .. PANEL .. "\n1\tDP-1\t1\t" .. LG_A .. "\tLeft side\n", "text")
  same(blocks.parse_places(text)["1"], 1, "read back")
end)

-- The groups of `groups()` as lists of ids, for comparing.
local function ids_of(groups)
  local result = {}
  for _, group in ipairs(groups) do
    result[#result + 1] = group.ids
  end
  return result
end

local function on(monitor, ...)
  local list = {}
  for _, id in ipairs({ ... }) do
    list[#list + 1] = { id = id, monitor = monitor }
  end
  return list
end

local function concat(...)
  local result = {}
  for _, list in ipairs({ ... }) do
    for _, item in ipairs(list) do
      result[#result + 1] = item
    end
  end
  return result
end

test("a screen's own group always holds its first workspace", function()
  same(ids_of(blocks.groups(0, "eDP-1", on("eDP-1", 2, 3), {})), { { 1, 2, 3 } }, "groups")
end)

test("a screen's own group counts its workspaces left on another screen", function()
  same(ids_of(blocks.groups(0, "eDP-1", concat(on("eDP-1", 1), on("DP-1", 2, 11)), {})), { { 1, 2 } }, "groups")
end)

test("the groups of unplugged screens follow the screen's own in block order", function()
  local workspaces = concat(on("eDP-1", 1, 2), on("eDP-1", 21), on("eDP-1", 11, 12))
  same(ids_of(blocks.groups(0, "eDP-1", workspaces)), { { 1, 2 }, { 11, 12 }, { 21 } }, "groups")
end)

test("a waiting group with lower numbers comes before the screen's own, as the swipe walks ids", function()
  local workspaces = concat(on("DP-1", 11, 12), on("DP-1", 1, 2))
  same(ids_of(blocks.groups(1, "DP-1", workspaces)), { { 1, 2 }, { 11, 12 } }, "groups")
end)

local GROUPS = { { block = 0, ids = { 1, 3, 4 } }, { block = 1, ids = { 11, 12 } }, { block = 2, ids = { 21 } } }

test("next and previous walk a group's workspaces", function()
  same(blocks.step(GROUPS, 1, 1), 3, "next")
  same(blocks.step(GROUPS, 4, -1), 3, "previous")
end)

test("from a group's last, next goes to the first of the group on its right", function()
  same(blocks.step(GROUPS, 4, 1), 11, "to the right")
  same(blocks.step(GROUPS, 21, 1), nil, "nothing on the right: stay")
end)

test("previous from a group's first goes to the last of the group on its left", function()
  same(blocks.step(GROUPS, 11, -1), 4, "to the left")
  same(blocks.step(GROUPS, 1, -1), nil, "nothing on the left: stay")
end)

test("next and previous group go to the first workspace of the group beside", function()
  same(blocks.group_step(GROUPS, 3, 1), 11, "next")
  same(blocks.group_step(GROUPS, 12, -1), 1, "previous")
  same(blocks.group_step(GROUPS, 21, 1), nil, "none")
end)

-- Workspaces for `kept()`: { id, monitor, windows }.
local function holding(monitor, windows, ...)
  local list = {}
  for _, id in ipairs({ ... }) do
    list[#list + 1] = { id = id, monitor = monitor, windows = windows }
  end
  return list
end

-- What `kept()` returns, as a sorted list of "id@connector".
local function kept_list(screens, workspaces)
  local result = {}
  for id, connector in pairs(blocks.kept(screens, workspaces)) do
    result[#result + 1] = id .. "@" .. connector
  end
  table.sort(result, function(left, right) return tonumber(left:match("^%d+")) < tonumber(right:match("^%d+")) end)
  return result
end

local SCREENS = { { name = "eDP-1", block = 0 }, { name = "DP-1", block = 1 } }

test("a screen with no windows keeps only its first workspace", function()
  same(kept_list(SCREENS, {}), { "1@eDP-1", "11@DP-1" }, "kept")
end)

test("a group keeps one empty workspace after its last with windows", function()
  local workspaces = concat(holding("eDP-1", 1, 1, 3), holding("eDP-1", 0, 2), holding("DP-1", 2, 11))
  same(kept_list(SCREENS, workspaces), { "1@eDP-1", "4@eDP-1", "11@DP-1", "12@DP-1" }, "kept")
end)

test("a group whose first is empty still keeps it", function()
  same(kept_list(SCREENS, holding("eDP-1", 1, 2)), { "1@eDP-1", "3@eDP-1", "11@DP-1" }, "kept")
end)

test("a group with windows on workspace 10 keeps no empty one after it", function()
  same(kept_list({ SCREENS[1] }, holding("eDP-1", 1, 1, 10)), { "1@eDP-1" }, "kept")
end)

test("a waiting group keeps its first and one empty after its last, on the screen holding it", function()
  local workspaces = concat(holding("eDP-1", 1, 1), holding("eDP-1", 1, 12, 13))
  same(kept_list({ SCREENS[1] }, workspaces), { "1@eDP-1", "2@eDP-1", "11@eDP-1", "14@eDP-1" }, "kept")
end)

test("an unplugged screen's group with no windows keeps nothing", function()
  same(kept_list({ SCREENS[1] }, holding("eDP-1", 0, 11)), { "1@eDP-1" }, "kept")
end)

test("a screen's own workspace left on another screen counts toward its group", function()
  same(kept_list(SCREENS, holding("DP-1", 1, 2)), { "1@eDP-1", "3@eDP-1", "11@DP-1" }, "kept")
end)

-- The module on the fake Hyprland.

test("at login each screen shows its own workspace 1", function()
  local world = two_screens()
  same({ world:shown_on("eDP-1"), world:shown_on("DP-1") }, { 1, 11 }, "shown")
  same(world:ids_on("DP-1"), { 11 }, "the number Hyprland gave DP-1 is gone")
end)

test("the screens file names each position's block, connector and screen", function()
  local world = two_screens()
  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. PANEL .. "\n1\tDP-1\t1\t" .. LG_A .. "\n", "file")
end)

test("SUPER + 2 opens the focused screen's own workspace 2", function()
  local world, module = two_screens()

  world:focus_monitor("DP-1")
  run(module.focus(2))
  world:focus_monitor("eDP-1")
  run(module.focus(2))

  same({ world:shown_on("eDP-1"), world:shown_on("DP-1") }, { 2, 12 }, "shown")
end)

test("a bar's click opens a slot on its own screen, focused or not", function()
  local world, module = two_screens()

  run(module.focus(3, "DP-1"))

  same(world:shown_on("DP-1"), 13, "shown")
  same(world.focused.name, "DP-1", "focus")
end)

test("SUPER + SHIFT + 4 moves the window to a new workspace 4 and follows it", function()
  local world, module = two_screens()
  local window = world:open_window()

  run(module.move_window(4))

  same(window.workspace.id, 4, "window")
  same(world:shown_on("eDP-1"), 4, "followed")
  same(world:ids_on("eDP-1"), { 1, 4, 5 }, "the empty 1 stays, and one empty after 4")
end)

test("SUPER + SHIFT + ALT + 4 moves the window without following", function()
  local world, module = two_screens()
  world:open_window()
  local window = world:open_window()

  run(module.move_window(4, false))

  same(window.workspace.id, 4, "window")
  same(world:shown_on("eDP-1"), 1, "stayed")
end)

test("moving a window on the external uses the external's block", function()
  local world, module = two_screens()
  world:focus_monitor("DP-1")
  local window = world:open_window()

  run(module.move_window(2))

  same(window.workspace.id, 12, "window")
end)

test("SUPER + SHIFT + CTRL + Right moves the window after the screen's last workspace", function()
  local world, module = two_screens()
  world:open_window()
  run(module.focus(3))
  world:open_window()
  local window = world:open_window()

  run(module.move_window_to_new())

  same(window.workspace.id, 4, "window")
  same(world:shown_on("eDP-1"), 4, "followed")
end)

-- Opens a window on each of the first `count` workspaces of a screen's group,
-- and returns them.
local function fill(world, module, name, count)
  local windows = {}
  world:focus_monitor(name)
  for slot = 1, count do
    run(module.focus(slot))
    windows[slot] = world:open_window()
  end
  return windows
end

-- Three workspaces with a window each on DP-1, two on the laptop.
local function busy()
  local world, module = two_screens()
  world:open_window()
  run(module.focus(2))
  world:open_window()
  world:focus_monitor("DP-1")
  for slot = 1, 3 do
    run(module.focus(slot))
    world:open_window()
  end
  return world, module
end

-- Regression: an unplugged screen's workspaces were renumbered into the
-- laptop's block, and the bar, which keeps a workspace's first id, could no
-- longer reach them.
test("unplugging a screen moves its group to the laptop under its own numbers", function()
  local world = busy()

  world:unplug("DP-1")

  same(world:ids_on("eDP-1"), { 1, 2, 3, 11, 12, 13, 14 }, "laptop")
  same(world.renumbered, 0, "renumbered")
end)

test("an unplugged screen's group goes to the laptop even when Hyprland puts it on another external", function()
  local world, module = busy()
  world:plug("DP-2", LG_B, 3840)
  fill(world, module, "DP-2", 2)

  world:unplug("DP-2", "DP-1")

  same(world:ids_on("eDP-1"), { 1, 2, 3, 21, 22, 23 }, "laptop")
  same(world:ids_on("DP-1"), { 11, 12, 13, 14 }, "the other external keeps its own")
end)

test("an unplugged screen with no windows leaves no group behind", function()
  local world = two_screens()

  world:unplug("DP-1")

  same(world:ids_on("eDP-1"), { 1 }, "laptop")
end)

test("plugging it back returns them under their old numbers", function()
  local world = busy()
  world:unplug("DP-1")

  world:plug("DP-3", LG_A, 1920)

  same(world:ids_on("eDP-1"), { 1, 2, 3 }, "laptop")
  same(world:ids_on("DP-3"), { 11, 12, 13, 14 }, "external, on a new connector")
end)

test("a config reload between unplug and replug keeps the record", function()
  local world = busy()
  world:unplug("DP-1")
  reload(world)

  world:plug("DP-1", LG_A, 1920)

  same(world:ids_on("DP-1"), { 11, 12, 13, 14 }, "external")
end)

test("a second external gets the next block and its position keeps it across plugs", function()
  local world, module = two_screens()
  world:plug("DP-2", LG_B, 3840)
  world:focus_monitor("DP-2")
  run(module.focus(1))

  same(world:shown_on("DP-2"), 21, "shown")

  world:unplug("DP-1")
  world:unplug("DP-2")
  world:plug("DP-5", LG_A, 1920)
  world:plug("DP-6", LG_B, 3840)
  same(world:shown_on("DP-6"), 21, "same block at the same position, on another connector")
end)

test("closing the lid moves the laptop's group to the external and opening it brings it back", function()
  local world = busy()

  world:unplug("eDP-1")
  same(world:ids_on("DP-1"), { 1, 2, 3, 11, 12, 13, 14 }, "external")

  world:plug("eDP-1", PANEL, 0)
  same(world:ids_on("eDP-1"), { 1, 2, 3 }, "laptop")
  same(world:ids_on("DP-1"), { 11, 12, 13, 14 }, "external")
end)

-- The laptop on the left and two externals beside it, with windows on each.
local function work_desk()
  local world, module = session()
  world:plug("eDP-1", PANEL, 0)
  world:plug("DP-1", LG_A, 1920)
  world:plug("DP-2", LG_B, 4480)
  world:fire("hyprland.start")
  fill(world, module, "eDP-1", 1)
  local left = fill(world, module, "DP-1", 2)
  local centre = fill(world, module, "DP-2", 1)
  return world, module, left, centre
end

local function on_screen(windows)
  local names = {}
  for _, window in ipairs(windows) do
    names[#names + 1] = window.workspace.monitor.name
  end
  return names
end

test("the home desk takes the work desk's groups by position, and the windows with them", function()
  local world, _, left, centre = work_desk()
  world:unplug("DP-1")
  world:unplug("DP-2")

  world:plug("HDMI-A-1", BENQ, 1920)
  world:plug("DP-3", LG_C, 4480)

  same(world:ids_on("HDMI-A-1"), { 11, 12, 13 }, "the left screen")
  same(world:ids_on("DP-3"), { 21, 22 }, "the centre screen")
  same(world:ids_on("eDP-1"), { 1, 2 }, "the laptop")
  same(on_screen(left), { "HDMI-A-1", "HDMI-A-1" }, "the left group's windows")
  same(on_screen(centre), { "DP-3" }, "the centre group's windows")
  same(world.renumbered, 0, "renumbered")
end)

test("the home desk plugged in centre first ends with each group at its position", function()
  local world, _, left, centre = work_desk()
  world:unplug("DP-1")
  world:unplug("DP-2")

  world:plug("DP-3", LG_C, 4480)
  world:plug("HDMI-A-1", BENQ, 1920)

  same(world:ids_on("HDMI-A-1"), { 11, 12, 13 }, "the left screen")
  same(world:ids_on("DP-3"), { 21, 22 }, "the centre screen")
  same({ blocks.block_of(world:shown_on("HDMI-A-1")), blocks.block_of(world:shown_on("DP-3")) }, { 1, 2 }, "each shows its group")
  same(on_screen(left), { "HDMI-A-1", "HDMI-A-1" }, "the left group's windows")
  same(on_screen(centre), { "DP-3" }, "the centre group's windows")
  same(world.renumbered, 0, "renumbered")
end)

-- Three externals beside the laptop, with a window on each.
local function three_externals()
  local world, module = session()
  world:plug("eDP-1", PANEL, 0)
  world:plug("DP-1", LG_A, 1920)
  world:plug("DP-2", LG_B, 3840)
  world:plug("DP-3", LG_C, 5760)
  world:fire("hyprland.start")
  for _, name in ipairs({ "DP-1", "DP-2", "DP-3" }) do
    fill(world, module, name, 1)
  end
  return world, module
end

test("from three externals to two, the third position's group waits on the laptop", function()
  local world = three_externals()

  world:unplug("DP-3")

  same(world:ids_on("eDP-1"), { 1, 31, 32 }, "the laptop")
  same({ world:ids_on("DP-1"), world:ids_on("DP-2") }, { { 11, 12 }, { 21, 22 } }, "the externals keep theirs")
end)

test("unplugging the leftmost of three, each group goes to the screen now at its position", function()
  local world = three_externals()

  world:unplug("DP-1")

  same(world:ids_on("DP-2"), { 11, 12 }, "now the leftmost")
  same(world:ids_on("DP-3"), { 21, 22 }, "now the second")
  same(world:ids_on("eDP-1"), { 1, 31, 32 }, "the third waits on the laptop")
  same(world.renumbered, 0, "renumbered")
end)

test("from two externals to three, a position never seen gets a fresh group", function()
  local world = work_desk()

  world:plug("DP-3", LG_C, 7040)

  same(world:shown_on("DP-3"), 31, "shown")
  same({ world:ids_on("DP-1"), world:ids_on("DP-2") }, { { 11, 12, 13 }, { 21, 22 } }, "the others keep theirs")
end)

test("from two externals to three, a third position seen before takes its group back", function()
  local world = three_externals()
  world:unplug("DP-3")

  world:plug("HDMI-A-1", BENQ, 5760)

  same(world:ids_on("HDMI-A-1"), { 31, 32 }, "the third")
  same(world:ids_on("eDP-1"), { 1 }, "the laptop")
end)

test("a screen plugged in left of the others takes position 1's group and shifts the rest right", function()
  local world, module = session()
  world:plug("eDP-1", PANEL, 0)
  world:plug("DP-1", LG_A, 3840)
  world:plug("DP-2", LG_B, 5760)
  world:fire("hyprland.start")
  fill(world, module, "DP-1", 1)
  fill(world, module, "DP-2", 1)

  world:plug("HDMI-A-1", BENQ, 1920)

  same(world:ids_on("HDMI-A-1"), { 11, 12 }, "the new leftmost")
  same(world:ids_on("DP-1"), { 21, 22 }, "the second")
  same(world:shown_on("DP-2"), 31, "the third, fresh")
  same(world.renumbered, 0, "renumbered")
end)

test("closing the lid at a desk leaves each external its group, and opening it brings the laptop's back", function()
  local world = work_desk()

  world:unplug("eDP-1")
  same(world:ids_on("DP-1"), { 1, 2, 11, 12, 13 }, "the laptop's group waits on the first external")
  same(world:ids_on("DP-2"), { 21, 22 }, "the other keeps its own")

  world:plug("eDP-1", PANEL, 0)
  same({ world:ids_on("eDP-1"), world:ids_on("DP-1"), world:ids_on("DP-2") }, { { 1, 2 }, { 11, 12, 13 }, { 21, 22 } }, "back")
end)

test("going to a desk position focuses the screen standing there", function()
  local world, module = work_desk()
  world:focus_monitor("eDP-1")

  run(module.go_to_position("2"))

  same(world.focused.name, "DP-2", "focus")
end)

test("going to a desk position no screen holds shows its waiting group", function()
  local world, module = work_desk()
  world:unplug("DP-2")
  world:focus_monitor("eDP-1")

  run(module.go_to_position("2"))

  same(world:shown_on("eDP-1"), 21, "shown")
end)

test("going to a desk position with nothing there does nothing", function()
  local world, module = work_desk()
  world:focus_monitor("eDP-1")

  run(module.go_to_position("7"))

  same({ world.focused.name, world:shown_on("eDP-1") }, { "eDP-1", 1 }, "unchanged")
end)

-- The laptop at work, the panel right of DP-8 and DP-9, before the module
-- reads a screens file keyed by screen: windows on the panel's 1 and on the
-- home desk's 11 waiting there, on DP-8's 31 and DP-9's 41 and 42.
local function before_migrating()
  local world = bare()
  world:plug("eDP-1", PANEL, 4480)
  world:plug("DP-8", LG_A, 2560)
  world:plug("DP-9", LG_B, 640)
  local windows = {}
  for _, place in ipairs({ { "eDP-1", 1 }, { "eDP-1", 11 }, { "DP-8", 31 }, { "DP-9", 41 }, { "DP-9", 42 } }) do
    world:focus_monitor(place[1])
    world:focus_workspace(place[2])
    windows[#windows + 1] = world:open_window()
  end
  for _, place in ipairs({ { "eDP-1", 1 }, { "DP-8", 31 }, { "DP-9", 41 } }) do
    world:focus_monitor(place[1])
    world:focus_workspace(place[2])
  end
  write(world.state .. "/workspace-screens", SCREENS_BY_SERIAL)
  return world, windows
end

local function load_after_update(world)
  package.loaded.workspaces = nil
  local module = require("workspaces")
  world:fire("config.reloaded")
  return module
end

test("migrating a screens file moves no window and renumbers no workspace", function()
  local world, windows = before_migrating()

  load_after_update(world)

  same(on_screen(windows), { "eDP-1", "eDP-1", "DP-8", "DP-9", "DP-9" }, "the windows")
  same({ world:ids_on("eDP-1"), world:ids_on("DP-8"), world:ids_on("DP-9") }, { { 1, 2, 11, 12 }, { 31, 32 }, { 41, 42, 43 } }, "ids")
  same({ world:shown_on("eDP-1"), world:shown_on("DP-8"), world:shown_on("DP-9") }, { 1, 31, 41 }, "shown")
  same(world.renumbered, 0, "renumbered")
  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. PANEL .. "\n3\tDP-8\t2\t" .. LG_A .. "\n4\tDP-9\t1\t" .. LG_B .. "\n", "file")
end)

test("after migrating, the home desk takes the work desk's groups by position", function()
  local world, windows = before_migrating()
  load_after_update(world)
  world:unplug("DP-8")
  world:unplug("DP-9")

  world:plug("DP-1", BENQ, 0)
  world:plug("DP-2", LG_C, 1920)

  same(on_screen(windows), { "eDP-1", "eDP-1", "DP-2", "DP-1", "DP-1" }, "the windows")
  same({ blocks.block_of(world:shown_on("DP-1")), blocks.block_of(world:shown_on("DP-2")) }, { 4, 3 }, "each shows its group")
  same(world.renumbered, 0, "renumbered")
end)

-- Kuba's desk: the panel on the right, the BenQ on the left and the LG in the
-- centre, and the screens file his laptop had, where each external's own
-- block waits at a position it never stood at (3 and 4) beside the position it
-- holds. `ghosts` adds windows to those waiting blocks.
local CSOT = "China Star Optoelectronics Technology Co. Ltd MNE007ZA3-2"
local LG_K = "LG Electronics LG ULTRAGEAR+ 510NTCZN7787"
local BNQ_K = "BNQ ZOWIE XL LCD G4L01663SL0"
local KUBA_FILE = "0\teDP-1\tpanel\t" .. CSOT .. "\n1\t\t3\t" .. LG_K .. "\n2\t\t4\t" .. BNQ_K
  .. "\n3\tHDMI-A-1\t2\t" .. LG_K .. "\n4\tDP-2\t1\t" .. BNQ_K .. "\n"
local KUBA_HEALED = "0\teDP-1\tpanel\t" .. CSOT .. "\n3\tHDMI-A-1\t2\t" .. LG_K .. "\n4\tDP-2\t1\t" .. BNQ_K .. "\n"

-- The laptop alone with the two groups waiting on it, as before the dock.
local function kuba_undocked(ghosts)
  local world = bare()
  write(world.state .. "/workspace-screens", KUBA_FILE)
  package.loaded.workspaces = nil
  local module = require("workspaces")
  world:plug("eDP-1", CSOT, 4480, 240)
  world:fire("hyprland.start")
  local ids = { 2, 31, 32, 33, 34, 41, 42 }
  for _, id in ipairs(ghosts or {}) do
    ids[#ids + 1] = id
  end
  for _, id in ipairs(ids) do
    world:window_on(id, "eDP-1")
  end
  world:open_window()
  return world, module
end

local function kuba_docked(world)
  same(world:ids_on("DP-2"), { 41, 42, 43 }, "the left screen")
  same(world:ids_on("HDMI-A-1"), { 31, 32, 33, 34, 35 }, "the centre screen")
  same(world:ids_on("eDP-1"), { 1, 2, 3 }, "the laptop")
  same({ world:shown_on("DP-2"), world:shown_on("HDMI-A-1") }, { 41, 31 }, "each shows its own")
  same(world.renumbered, 0, "renumbered")
  same(read(world.state .. "/workspace-screens"), KUBA_HEALED, "the file")
end

-- Regression: the centre group showed on the right and the centre screen the
-- left group, because the reconcile ran while the LG still sat at 0,0.
test("the BenQ, then the LG arriving at 0,0 before it is arranged, end with each group at its position", function()
  local world = kuba_undocked()
  world:attach("DP-2", BNQ_K)
  world:arrange("DP-2", 0, 360)
  world:settle()
  local file = world.state .. "/workspace-screens"
  local real_open, real_rename = io.open, os.rename
  local written_with, opened, renamed = nil, {}, {}
  io.open = function(path, mode)
    if mode == "w" and path:find(world.state, 1, true) then
      written_with = world:ids_on("HDMI-A-1")
      opened[#opened + 1] = path
    end
    return real_open(path, mode)
  end
  os.rename = function(from, to)
    renamed[#renamed + 1] = { from, to }
    return real_rename(from, to)
  end

  world:attach("HDMI-A-1", LG_K)
  world:arrange("HDMI-A-1", 1920, 0)
  world:settle()

  io.open, os.rename = real_open, real_rename
  kuba_docked(world)
  same(written_with, { 31, 32, 33, 34, 35 }, "the file is written once the workspaces have moved")
  same(opened, { file .. ".tmp" }, "written to a temp file beside it")
  same(renamed, { { file .. ".tmp", file } }, "then renamed over it")
end)

test("the LG first, then the BenQ, end the same", function()
  local world = kuba_undocked()

  world:attach("HDMI-A-1", LG_K)
  world:arrange("HDMI-A-1", 1920, 0)
  world:settle()
  world:attach("DP-2", BNQ_K)
  world:arrange("DP-2", 0, 360)
  world:settle()

  kuba_docked(world)
end)

test("both screens together move nothing until the layout has settled, then reconcile once", function()
  local world = kuba_undocked()
  local before = read(world.state .. "/workspace-screens")

  world:attach("DP-2", BNQ_K)
  world:attach("HDMI-A-1", LG_K)
  world:arrange("DP-2", 0, 360)
  world:arrange("HDMI-A-1", 1920, 0)
  same(world.moves, 0, "nothing moved before the layout settled")
  same(read(world.state .. "/workspace-screens"), before, "the file is untouched before the layout settled")

  world:settle()

  kuba_docked(world)
  same(world.moves, 8, "each of the eight waiting workspaces moved once")
end)

-- Regression: Hyprland listed the BenQ nowhere while it reported the LG's
-- arrival, so the BenQ lost its group and was left on the panel's 4.
test("a screen missing from the list while another arrives keeps its group", function()
  local world = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  local before = read(world.state .. "/workspace-screens")
  world.hidden["DP-2"] = true

  world:attach("HDMI-A-1", LG_K)
  world:arrange("HDMI-A-1", 1920, 0)
  world:tick()
  same(world:ids_on("DP-2"), { 41, 42, 43 }, "the missing screen keeps its group while the wait goes on")
  same(read(world.state .. "/workspace-screens"), before, "and the file is untouched")
  world.hidden["DP-2"] = nil
  world:tick()

  kuba_docked(world)
end)

test("a screen missing from the list for good is reconciled without it after the retries", function()
  local world = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world.hidden["DP-2"] = true

  world:attach("HDMI-A-1", LG_K)
  world:arrange("HDMI-A-1", 1920, 0)
  world:settle()

  same(world:ids_on("HDMI-A-1"), { 41, 42, 43 }, "the only external left is position 1")
end)

-- Regression: the left screen's removal was reconciled at once, so the centre
-- screen shifted to position 1 and took the left group for a moment, and
-- every workspace moved twice.
test("the dock's screens going one at a time move each group once, to the laptop", function()
  local world = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)
  world.moves = 0

  world:detach("DP-2", "HDMI-A-1")
  same(world.moves, 0, "nothing moves on the first removal")
  world:detach("HDMI-A-1")
  same(world.moves, 0, "nor on the second")
  world:settle()

  same(world:ids_on("eDP-1"), { 1, 2, 3, 31, 32, 33, 34, 35, 41, 42, 43 }, "both groups wait on the laptop")
  same(world.moves, 0, "where Hyprland's own backup put them; nothing moves again")
  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. CSOT .. "\n3\t\t2\t" .. LG_K .. "\n4\t\t1\t" .. BNQ_K .. "\n", "each position remembers its own screen")

  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)
  kuba_docked(world)
end)

-- Regression: Hyprland shows the unplugged screen's workspace on the screen
-- it backs up to when the unplugged one had focus, and the reconcile put the
-- backup screen back on its own only after the wait: two switches in view.
test("unplugging the focused screen leaves the screen Hyprland backs up to on its own workspace at once", function()
  local world = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)
  world:focus_monitor("HDMI-A-1")
  world.moves = 0

  world:detach("HDMI-A-1", "DP-2")

  same({ world:shown_on("DP-2"), world:shown_on("eDP-1") }, { 41, 1 }, "each on its own, before the wait")
  same(world.moves, 0, "and nothing moved")
  world:settle()
  same({ world:shown_on("DP-2"), world:shown_on("eDP-1") }, { 41, 1 }, "and after it")
  same(world:ids_on("eDP-1"), { 1, 2, 3, 31, 32, 33, 34, 35 }, "the group waits on the laptop")
end)

-- Regression: the panel was sent to its workspace 1 whenever any screen
-- went, though Hyprland changes its view only when the screen that went had
-- focus.
test("unplugging an unfocused screen leaves the panel on the empty workspace it chose", function()
  local world, module = two_screens()
  world:open_window()
  run(module.focus(5))

  world:detach("DP-1")
  same(world:shown_on("eDP-1"), 5, "still there")
  world:settle()
  same(world:shown_on("eDP-1"), 5, "and after the wait")
end)

test("unplugging an unfocused screen leaves the panel on the guest it chose", function()
  local world, module = two_screens()
  world:focus_monitor("DP-1")
  run(module.focus(1))
  world:open_window()
  run(module.go_to(11, "eDP-1"))

  world:detach("DP-1")
  same(world:shown_on("eDP-1"), 11, "still there")
  world:settle()
  same(world:shown_on("eDP-1"), 11, "and after the wait")
end)

test("after a config reload, unplugging the focused screen still leaves the backup on its own at once", function()
  local world, module = two_screens()
  world:focus_monitor("DP-1")
  run(module.focus(2))
  world:open_window()
  reload(world)
  world:fire("config.reloaded")

  world:detach("DP-1")

  same(world:shown_on("eDP-1"), 1, "home at once")
end)

test("focus moved from a screen by hand doesn't count as the move its removal makes", function()
  local world, module = two_screens()
  world:focus_monitor("DP-1")
  world:focus_monitor("eDP-1")
  world:open_window()
  run(module.focus(5))

  world:detach("DP-1")

  same(world:shown_on("eDP-1"), 5, "still there")
end)

test("moving a workspace to a screen that arrived within the wait joins that screen's block", function()
  local world, module = two_screens()
  world:focus_monitor("DP-1")
  run(module.focus(2))
  local window = world:open_window()
  world:attach("DP-2", LG_B)
  world:arrange("DP-2", 3840, 0)

  run(module.move_to_monitor("r"))

  same({ window.workspace.id, window.workspace.monitor.name }, { 22, "DP-2" }, "joined the new screen's block")
  world:settle()
  same({ window.workspace.id, window.workspace.monitor.name, world:shown_on("DP-2") }, { 22, "DP-2", 22 }, "and stays")
end)

test("Hyprland's fallback output holds no position and writes no row", function()
  local world, module = two_screens()
  world:open_window()
  world:unplug("eDP-1")
  local before = read(world.state .. "/workspace-screens")

  world:attach("FALLBACK", "")
  world:detach("DP-1")
  world:settle()
  same(read(world.state .. "/workspace-screens"), before, "the file is untouched")
  same(module.workspace_id(hl.get_monitor("FALLBACK"), 2), 12, "unmapped, the fallback addresses the group it shows")

  world:plug("eDP-1", PANEL, 0, 0)
  world:unplug("FALLBACK")
  same(world:ids_on("eDP-1"), { 1, 2 }, "the panel takes its group back")
  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. PANEL .. "\n1\t\t1\t" .. LG_A .. "\n", "and the file names no fallback")
end)

-- Regression: a window opening re-assigned the positions and pulled 41 to
-- the left screen beside the centre group.
test("a window opening while a screen is unarranged changes no mapping", function()
  local world = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world:attach("HDMI-A-1", LG_K)
  local before = read(world.state .. "/workspace-screens")

  world:focus_monitor("eDP-1")
  world:open_window()

  same(read(world.state .. "/workspace-screens"), before, "the file")
  same(world:ids_on("DP-2"), { 41, 42, 43 }, "the left screen keeps its group")
  world:arrange("HDMI-A-1", 1920, 0)
  world:settle()
  kuba_docked(world)
end)

test("unplugging both and plugging them back leaves one rule per kept workspace", function()
  local world = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)

  world:unplug("HDMI-A-1")
  world:unplug("DP-2")
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)

  kuba_docked(world)
  same(world:kept(), { "1@eDP-1", "3@eDP-1", "31@HDMI-A-1", "35@HDMI-A-1", "41@DP-2", "43@DP-2" }, "kept")
  same(world:all_rules(), { "1@eDP-1", "3@eDP-1", "31@HDMI-A-1", "35@HDMI-A-1", "41@DP-2", "43@DP-2" }, "no rule piles up")
end)

test("a screen that arrives on a parked number of another group ends on its own", function()
  local world = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world:focus_monitor("DP-2")
  world:focus_workspace(4)

  world:plug("HDMI-A-1", LG_K, 1920, 0)

  kuba_docked(world)
end)

test("a ghost position with windows keeps its row, and they wait on the laptop as a group", function()
  local world = kuba_undocked({ 11, 12 })

  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)

  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. CSOT .. "\n1\t\t3\t" .. LG_K
    .. "\n3\tHDMI-A-1\t2\t" .. LG_K .. "\n4\tDP-2\t1\t" .. BNQ_K .. "\n", "the file")
  same(world:ids_on("eDP-1"), { 1, 2, 3, 11, 12, 13 }, "the windows wait on the laptop")
  same(world:ids_on("HDMI-A-1"), { 31, 32, 33, 34, 35 }, "the centre screen")

  for _, window in ipairs(world.windows) do
    if window.workspace.id == 11 or window.workspace.id == 12 then
      world:close_window(window)
    end
  end
  world:unplug("DP-2")
  world:plug("DP-2", BNQ_K, 0, 360)
  same(read(world.state .. "/workspace-screens"), KUBA_HEALED, "the row goes once the block is empty")
end)

-- The legacy file Kuba's laptop wrote at home, then the work desk, then home.
test("migrating a file that knows only the home screens gives the work desk fresh groups and no ghost", function()
  local world = bare()
  world:plug("eDP-1", CSOT, 4480, 0)
  world:plug("DP-8", "Work A", 2560, 0)
  world:plug("DP-9", "Work B", 640, 0)
  world:window_on(11, "eDP-1")
  write(world.state .. "/workspace-screens", "0\teDP-1\t" .. CSOT .. "\n1\t\t" .. LG_K .. "\n2\t\t" .. BNQ_K .. "\n")

  load_after_update(world)

  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. CSOT .. "\n2\tDP-9\t1\tWork B\n3\tDP-8\t2\tWork A\n", "at work")
  same(world:ids_on("eDP-1"), { 1, 11, 12 }, "the home windows wait on the laptop")

  world:unplug("DP-8")
  world:unplug("DP-9")
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)

  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. CSOT .. "\n2\tDP-2\t1\t" .. BNQ_K .. "\n3\tHDMI-A-1\t2\t" .. LG_K .. "\n", "at home")
  same({ world:shown_on("DP-2"), world:shown_on("HDMI-A-1") }, { 21, 31 }, "each shows its position's group")
end)

test("a legacy file written with the lid closed keeps the panel's block for the panel", function()
  local world = bare()
  world:plug("DP-1", LG_A, 0, 0)
  write(world.state .. "/workspace-screens", "0\t\t" .. PANEL .. "\n1\tDP-1\t" .. LG_A .. "\n")

  load_after_update(world)
  world:plug("eDP-1", PANEL, 1920, 0)

  same({ world:shown_on("eDP-1"), world:shown_on("DP-1") }, { 1, 11 }, "shown")
end)

test("going to a desk position whose group has no workspace opens its first", function()
  local world, module = work_desk()
  world:unplug("DP-2")
  world:focus_monitor("DP-1")
  world:close_window(world.windows[#world.windows])
  world:focus_monitor("DP-1")

  run(module.go_to_position("2"))

  same({ world.focused.name, world:shown_on("eDP-1") }, { "eDP-1", 21 }, "shown on the primary screen")
end)

-- Regression: the move named the old id and the renumber came after, so the
-- bar, which keys a workspace by the first id it hears, never saw the new one.
test("moving a workspace to another screen renumbers it into that block before it moves", function()
  local world, module = two_screens()
  run(module.focus(2))
  local window = world:open_window()
  world.events = {}

  run(module.move_to_monitor("r"))

  same(world.events, { "changeworkspaceid>>2,12", "moveworkspacev2>>12,12,DP-1" }, "the move names the new id")
  same({ window.workspace.id, world:shown_on("DP-1"), world.focused.name }, { 12, 12, "DP-1" }, "joined and shown")
  same(world:ids_on("eDP-1"), { 1 }, "the laptop keeps its first")
  same(world:kept(), { "1@eDP-1", "11@DP-1", "13@DP-1" }, "kept")
end)

test("moving a workspace to a screen whose block is full keeps its number", function()
  local world, module = two_screens()
  fill(world, module, "DP-1", 10)
  world:focus_monitor("eDP-1")
  run(module.focus(2))
  local window = world:open_window()

  run(module.move_to_monitor("r"))

  same({ window.workspace.id, window.workspace.monitor.name, world.renumbered }, { 2, "DP-1", 0 }, "a guest")
end)

-- Regression: an empty workspace 1, once left, was out of reach of every
-- "previous workspace" but SUPER + 1.
test("previous reaches an empty workspace 1 again", function()
  local world, module = two_screens()
  run(module.focus(2))
  world:open_window()

  run(module.previous())

  same(world:shown_on("eDP-1"), 1, "shown")
end)

test("next from the last workspace opens one empty workspace and no more", function()
  local world, module = two_screens()
  world:open_window()

  run(module.next())
  same(world:shown_on("eDP-1"), 2, "a new one")
  run(module.next())
  same(world:shown_on("eDP-1"), 2, "stays on the empty one")
end)

test("previous from the first workspace with nothing on the left stays", function()
  local world, module = two_screens()
  world:open_window()

  run(module.previous())

  same(world:shown_on("eDP-1"), 1, "shown")
end)

test("next walks from the laptop's group into an unplugged screen's", function()
  local world, module = busy()
  world:unplug("DP-1")
  world:focus_monitor("eDP-1")
  run(module.focus(2))

  run(module.next())
  same(world:shown_on("eDP-1"), 3, "the empty workspace kept after the laptop's own")
  run(module.next())
  same(world:shown_on("eDP-1"), 11, "then the unplugged screen's first")
  run(module.previous())
  same(world:shown_on("eDP-1"), 3, "and back to the laptop's last, the empty one")
end)

test("the groups of two unplugged screens come in block order, not where the screens sat", function()
  local world, module = session()
  world:plug("eDP-1", PANEL, 0)
  world:plug("DP-1", LG_A, 3840)
  world:plug("DP-2", LG_B, 1920)
  world:fire("hyprland.start")
  world:open_window()
  for _, name in ipairs({ "DP-1", "DP-2" }) do
    world:focus_monitor(name)
    run(module.focus(1))
    world:open_window()
  end
  world:unplug("DP-1")
  world:unplug("DP-2")
  world:focus_monitor("eDP-1")
  run(module.focus(1))

  run(module.next_group())
  same(world:shown_on("eDP-1"), 11, "block 1 first, position 1's")
  run(module.next_group())
  same(world:shown_on("eDP-1"), 21, "then position 2's")
  run(module.next_group())
  same(world:shown_on("eDP-1"), 21, "the last group stays")
  run(module.previous_group())
  same(world:shown_on("eDP-1"), 11, "previous group")
end)

test("SUPER + 2 on an unplugged screen's group goes to that group's 2", function()
  local world, module = busy()
  world:unplug("DP-1")
  world:focus_monitor("eDP-1")
  run(module.focus(1))
  run(module.next_group())

  run(module.focus(2))

  same(world:shown_on("eDP-1"), 12, "shown")
end)

test("a bar's click reaches any group's workspace", function()
  local world, module = busy()
  world:unplug("DP-1")
  world:focus_monitor("eDP-1")

  run(module.go_to(12, "eDP-1"))
  same(world:shown_on("eDP-1"), 12, "the unplugged screen's")
  run(module.go_to(1, "eDP-1"))
  same(world:shown_on("eDP-1"), 1, "the laptop's own")
end)

test("the bar's scroll walks the groups of the screen it names", function()
  local world, module = busy()
  world:focus_monitor("eDP-1")
  run(module.focus(2))

  run(module.previous("DP-1"))

  same(world:shown_on("DP-1"), 12, "shown")
  same(world.focused.name, "DP-1", "focus")
end)

test("a window on a group's empty last workspace brings a new empty one after it", function()
  local world, module = two_screens()
  world:open_window()
  run(module.next())

  local window = world:open_window()
  same(world:ids_on("eDP-1"), { 1, 2, 3 }, "one empty after the window")

  world:close_window(window)
  run(module.previous())
  same(world:ids_on("eDP-1"), { 1, 2 }, "the one no longer needed goes")
end)

test("the kept workspaces are Hyprland's persistent workspaces, each bound to its screen", function()
  local world = two_screens()
  world:open_window()

  same(world:kept(), { "1@eDP-1", "2@eDP-1", "11@DP-1" }, "kept")
end)

test("a workspace kept, released and kept again reuses its rule", function()
  local world = two_screens()
  local window = world:open_window()
  world:close_window(window)
  world:open_window()

  same(world:rules_for("2"), 1, "rules for workspace 2")
  same(world:kept(), { "1@eDP-1", "2@eDP-1", "11@DP-1" }, "kept")
end)

test("unplugging an empty screen releases its kept workspace", function()
  local world = two_screens()

  world:unplug("DP-1")

  same(world:kept(), { "1@eDP-1" }, "kept")
end)

-- Workspace groups on the fake Hyprland. Kuba's work desk: two Dells, the
-- left one on DP-9 and the centre one on DP-8.
local DELL_L = "Dell Inc. DELL U2724D 5D6E7F8"
local DELL_C = "Dell Inc. DELL U2724D 1A2B3C4"
local KUBA_GROUPS = '[group.1]\nname = "Left"\nscreens = ["' .. BNQ_K .. '", "' .. DELL_L .. '"]\n\n'
  .. '[group.2]\nname = "Centre"\nscreens = ["' .. LG_K .. '", "' .. DELL_C .. '"]\n'

-- A fresh state, the groups file `text`, and the module loaded on them.
local function grouped(text)
  local world = bare()
  write(world.groups, text)
  package.loaded.workspaces = nil
  return world, require("workspaces")
end

-- Kuba at home with `text` as his groups file: two windows on the BenQ's
-- group and one on the LG's.
local function home_grouped(text)
  local world, module = grouped(text)
  world:plug("eDP-1", CSOT, 4480, 240)
  world:fire("hyprland.start")
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)
  local left = fill(world, module, "DP-2", 2)
  local centre = fill(world, module, "HDMI-A-1", 1)
  world:unplug("DP-2")
  world:unplug("HDMI-A-1")
  return world, module, left, centre
end

-- The work desk as Hyprland places it with no layout of Kuba's: in connector
-- order, the centre Dell (DP-8) leftmost.
local function work_grouped_ends_right(world, left, centre)
  same(world:ids_on("DP-9"), { 11, 12, 13 }, "the left Dell shows the Left group")
  same(world:ids_on("DP-8"), { 21, 22 }, "the centre Dell shows the Centre group")
  same({ world:shown_on("DP-9"), world:shown_on("DP-8") }, { 11, 21 }, "each on its first")
  same(on_screen(left), { "DP-9", "DP-9" }, "the Left group's windows")
  same(on_screen(centre), { "DP-8" }, "the Centre group's windows")
  same(world.renumbered, 0, "renumbered")
end

test("pinned screens take their groups whatever the layout, the centre one plugged in first", function()
  local world, _, left, centre = home_grouped(KUBA_GROUPS)

  world:plug("DP-8", DELL_C, 0, 0)
  world:plug("DP-9", DELL_L, 2560, 0)

  work_grouped_ends_right(world, left, centre)
end)

test("pinned screens take their groups whatever the layout, the left one plugged in first", function()
  local world, _, left, centre = home_grouped(KUBA_GROUPS)

  world:attach("DP-9", DELL_L)
  world:arrange("DP-9", 0, 0)
  world:settle()
  world:attach("DP-8", DELL_C)
  world:arrange("DP-8", 0, 0)
  world:arrange("DP-9", 2560, 0)
  world:settle()

  work_grouped_ends_right(world, left, centre)
end)

test("pinned screens plugged in together reconcile once, each group moving once", function()
  local world, _, left, centre = home_grouped(KUBA_GROUPS)
  world.moves = 0

  world:attach("DP-9", DELL_L)
  world:attach("DP-8", DELL_C)
  world:arrange("DP-8", 0, 0)
  world:arrange("DP-9", 2560, 0)
  same(world.moves, 0, "nothing moved before the layout settled")
  world:settle()

  work_grouped_ends_right(world, left, centre)
  same(world.moves, 5, "the three workspaces with windows and the empty one each group keeps after them")
end)

test("the screens file names each group beside its rows", function()
  local world = home_grouped(KUBA_GROUPS)

  world:plug("DP-8", DELL_C, 0, 0)
  world:plug("DP-9", DELL_L, 2560, 0)

  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. CSOT .. "\n1\tDP-9\t1\t" .. DELL_L
    .. "\tLeft\n2\tDP-8\t2\t" .. DELL_C .. "\tCentre\n", "file")
end)

test("in a clash the first listed takes the group and the other the next number no group pins", function()
  local world, _, left = home_grouped(KUBA_GROUPS)
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)

  world:plug("DP-9", DELL_L, 3840, 0)

  same(world:ids_on("DP-2"), { 11, 12, 13 }, "the BenQ, listed first, keeps the Left group")
  same(world:shown_on("DP-9"), 31, "the left Dell gets a group of its own, position 3")
  same(on_screen(left), { "DP-2", "DP-2" }, "the Left group's windows")

  world:unplug("DP-2")

  same(world:ids_on("DP-9"), { 11, 12, 13 }, "with the BenQ gone, the left Dell takes the Left group")
  same(on_screen(left), { "DP-9", "DP-9" }, "and its windows")
end)

test("unpinned screens fill the numbers left to right around a pinned one, and keep theirs while it is away", function()
  local world, module = grouped('[group.2]\nscreens = ["' .. LG_K .. '"]\n')
  world:plug("eDP-1", CSOT, 5760, 0)
  world:fire("hyprland.start")
  world:plug("DP-1", LG_A, 0, 0)
  world:plug("HDMI-A-1", LG_K, 1920, 0)
  world:plug("DP-3", LG_B, 3840, 0)
  same({ world:shown_on("DP-1"), world:shown_on("HDMI-A-1"), world:shown_on("DP-3") }, { 11, 21, 31 }, "1, 2, 3")
  local centre = fill(world, module, "HDMI-A-1", 1)
  local right = fill(world, module, "DP-3", 1)

  world:unplug("HDMI-A-1")

  same(world:ids_on("DP-3"), { 31, 32 }, "the right screen keeps position 3's group, never taking 2")
  same(on_screen(right), { "DP-3" }, "and its windows")
  same(on_screen(centre), { "eDP-1" }, "the pinned group's windows wait on the laptop")
  same(world:ids_on("DP-1"), { 11 }, "the left screen keeps its own")
end)

test("the panel pinned after an external shows the group with the lid open, and its own 1-10 waits on it", function()
  local world, module = grouped('[group.2]\nscreens = ["' .. LG_K .. '", "' .. CSOT .. '"]\n')
  world:plug("eDP-1", CSOT, 1920, 0)
  world:fire("hyprland.start")
  same(world:shown_on("eDP-1"), 11, "the panel shows group 2, block 1; 1-10 stays the panel's")
  local mine = fill(world, module, "eDP-1", 1)
  local own = world:window_on(1, "eDP-1")

  world:plug("HDMI-A-1", LG_K, 0, 0)
  same(world:ids_on("HDMI-A-1"), { 11, 12 }, "docked, the LG listed first takes group 2")
  same({ world:ids_on("eDP-1"), world:shown_on("eDP-1") }, { { 1, 2 }, 1 }, "and the panel its own 1-10")

  world:unplug("eDP-1")
  same(world:ids_on("HDMI-A-1"), { 1, 2, 11, 12 }, "the lid closed, 1-10 waits on the LG")

  world:plug("eDP-1", CSOT, 1920, 0)
  same({ world:ids_on("eDP-1"), world:ids_on("HDMI-A-1") }, { { 1, 2 }, { 11, 12 } }, "the lid open again")

  world:unplug("HDMI-A-1")
  same(world:ids_on("eDP-1"), { 1, 2, 11, 12 }, "undocked, the panel shows group 2 again beside its own")
  same(on_screen({ mine[1], own }), { "eDP-1", "eDP-1" }, "the windows")
  same(read(world.state .. "/workspace-screens"), "0\t\tpanel\t" .. CSOT .. "\n1\teDP-1\t2\t" .. CSOT .. "\n",
    "the panel's row keeps 1-10 while the panel stands at 2")
  same(world.renumbered, 0, "renumbered")
end)

test("the panel pinned first holds its group with the lid open, and the lid closed hands it to the next listed", function()
  local world, module = grouped('[group.1]\nscreens = ["' .. CSOT .. '", "' .. BNQ_K .. '"]\n[group.2]\nscreens = ["'
    .. LG_K .. '"]\n')
  world:plug("eDP-1", CSOT, 4480, 240)
  world:fire("hyprland.start")
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)
  same({ world:shown_on("eDP-1"), world:shown_on("HDMI-A-1"), world:shown_on("DP-2") }, { 11, 31, 21 },
    "the panel group 1, the LG group 2, the BenQ position 3")
  local first = fill(world, module, "eDP-1", 1)
  local benq = fill(world, module, "DP-2", 1)
  local lg = fill(world, module, "HDMI-A-1", 1)
  local own = world:window_on(1, "eDP-1")

  world:unplug("eDP-1")

  same(world:ids_on("DP-2"), { 1, 2, 11, 12, 21, 22 }, "the lid closed, the BenQ takes group 1, 1-10 and its own wait there")
  same(world:ids_on("HDMI-A-1"), { 31, 32 }, "the LG keeps group 2")
  same(on_screen({ first[1], benq[1], lg[1], own }), { "DP-2", "DP-2", "HDMI-A-1", "DP-2" }, "the windows")
  same(world:shown_on("DP-2"), 21, "the BenQ still shows what it showed, now waiting there")

  world:plug("eDP-1", CSOT, 4480, 240)

  same({ world:ids_on("eDP-1"), world:ids_on("DP-2"), world:ids_on("HDMI-A-1") }, { { 1, 2, 11, 12 }, { 21, 22 }, { 31, 32 } },
    "the lid open, each back")
  same(world:shown_on("eDP-1"), 11, "the panel shows group 1 again")
  same(world.renumbered, 0, "renumbered")
end)

test("a groups file edited while the screens are plugged in moves each group to the screen it pins at once", function()
  local world, module = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)
  world.moves = 0

  write(world.groups, '[group.1]\nname = "Main"\nscreens = ["' .. LG_K .. '"]\n')
  run(module.reload_groups())

  same(world:ids_on("HDMI-A-1"), { 41, 42, 43 }, "the LG pinned to 1 takes its group")
  same(world:ids_on("DP-2"), { 31, 32, 33, 34, 35 }, "the BenQ, unpinned, takes 2")
  same({ world:shown_on("HDMI-A-1"), world:shown_on("DP-2") }, { 41, 31 }, "each on its first")
  same(world.renumbered, 0, "renumbered")
  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. CSOT .. "\n3\tDP-2\t2\t" .. BNQ_K
    .. "\n4\tHDMI-A-1\t1\t" .. LG_K .. "\tMain\n", "the file")
  same(world:kept(), { "1@eDP-1", "3@eDP-1", "31@DP-2", "35@DP-2", "41@HDMI-A-1", "43@HDMI-A-1" }, "kept")
  same(world:rules_for("41") + world:rules_for("31"), 2, "one rule per kept workspace")

  world.moves = 0
  run(module.reload_groups())
  same(world.moves, 0, "the same file again moves nothing")
end)

test("a groups file edited while a screen arrives is applied once the layout settles", function()
  local world, module = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world:attach("HDMI-A-1", LG_K)
  world:arrange("HDMI-A-1", 1920, 0)
  world.moves = 0

  write(world.groups, '[group.2]\nscreens = ["' .. BNQ_K .. '"]\n')
  run(module.reload_groups())
  same(world.moves, 0, "nothing moves before the layout settles")
  world:settle()

  same(world:ids_on("DP-2"), { 31, 32, 33, 34, 35 }, "the BenQ pinned to 2")
  same(world:ids_on("HDMI-A-1"), { 41, 42, 43 }, "the LG, unpinned, at 1")
end)

test("a groups file that doesn't read is one toast and pins nothing, until it is fixed", function()
  local world, module = grouped('[group.1]\nscreens = ["' .. LG_A .. '"]\n[group.2]\nscreen = ["' .. LG_B .. '"]\n')
  world:plug("eDP-1", PANEL, 0, 0)
  world:fire("hyprland.start")
  world:plug("DP-1", LG_B, 1920, 0)
  world:plug("DP-2", LG_A, 3840, 0)

  same({ world:shown_on("DP-1"), world:shown_on("DP-2") }, { 11, 21 }, "left to right, as without the file")
  same(#world.commands, 1, "one toast")
  assert(world.commands[1]:find("notify-send -a", 1, true) and world.commands[1]:find("line 4: `screen`", 1, true),
    world.commands[1])

  run(module.problem_shown(1))
  module = reload(world)
  world:fire("config.reloaded")
  same(#world.commands, 1, "once shown, a config reload doesn't say it again")

  write(world.groups, '[group.1]\nscreens = ["' .. LG_A .. '"]\n')
  run(module.reload_groups())
  same({ world:shown_on("DP-2"), world:shown_on("DP-1") }, { 11, 21 }, "fixed, the pin applies at once")

  write(world.groups, "[group.1\n")
  run(module.reload_groups())
  same({ world:shown_on("DP-1"), world:shown_on("DP-2") }, { 11, 21 }, "broken again, left to right")
  same(#world.commands, 2, "and a toast again")
end)

-- Regression: at login the toast was sent before the shell, the notification
-- server, was up, and then counted as shown for good.
test("a broken groups file at login is toasted until the shell shows it, and only then counts as shown", function()
  local world, module = grouped("[group.1]\nscreen = []\n")

  same(#world.commands, 1, "one toast")
  assert(world.commands[1]:find("for try in $(seq 60); do notify-send -a", 1, true), world.commands[1])
  assert(world.commands[1]:find("&& exec hyprctl dispatch 'dot.workspaces.problem_shown(1)'; sleep 1; done", 1, true),
    world.commands[1])
  module = reload(world)
  world:fire("config.reloaded")
  same(#world.commands, 1, "a reload while it is still being tried doesn't try twice")

  dot.workspace_groups_toasting.expires = 0
  module = reload(world)
  world:fire("config.reloaded")
  same(#world.commands, 2, "once the tries ran out unshown, the next load tries again")
  assert(world.commands[2]:find("problem_shown(2)", 1, true), world.commands[2])

  run(module.problem_shown(1))
  same(dot.workspace_groups_problem, nil, "a stale token counts for nothing")
  run(module.problem_shown(2))
  module = reload(world)
  world:fire("config.reloaded")
  same(#world.commands, 2, "shown, it isn't said again")
end)

-- Regression: nvim saves by moving the file away and writing a new one, and
-- the read in between unpinned every screen for a moment.
test("a groups file missing for a moment, as an editor saves it, moves nothing", function()
  local world, module = kuba_undocked()
  world:plug("DP-2", BNQ_K, 0, 360)
  world:plug("HDMI-A-1", LG_K, 1920, 0)
  local text = '[group.1]\nscreens = ["' .. LG_K .. '"]\n'
  write(world.groups, text)
  run(module.reload_groups())
  world.moves = 0

  os.remove(world.groups)
  run(module.reload_groups())
  same(world.moves, 0, "nothing moves while the file is missing")
  write(world.groups, text)
  run(module.reload_groups())
  world:settle()
  same(world.moves, 0, "nor once it is back as it was")
  same(world:ids_on("HDMI-A-1"), { 41, 42, 43 }, "the LG keeps group 1")

  os.remove(world.groups)
  run(module.reload_groups())
  world:settle()
  same(world:ids_on("DP-2"), { 41, 42, 43 }, "still missing after the wait, nothing is pinned")
end)

test("going to a pinned group's position focuses the screen pinned there", function()
  local world, module = home_grouped(KUBA_GROUPS)
  world:plug("DP-8", DELL_C, 0, 0)
  world:plug("DP-9", DELL_L, 2560, 0)
  world:focus_monitor("eDP-1")

  run(module.go_to_position("1"))

  same(world.focused.name, "DP-9", "focus")
end)

-- The binds of fallback.lua and keybinds.lua, by their keys: each one's
-- description and action. Any `hl.dsp` dispatcher stands for itself, as
-- { dsp = "focus", args = ... }.
local function binds()
  local _, module = session()
  local function dispatcher(path)
    return setmetatable({}, {
      __index = function(_, key) return dispatcher(path == "" and key or path .. "." .. key) end,
      __call = function(_, args) return { dsp = path, args = args or {} } end,
    })
  end
  local dsp = dispatcher("")
  local found = {}
  local duplicates = {}
  local gestures = {}
  local config = {}
  _G.hl = setmetatable({
    dsp = dsp,
    config = function(settings)
      for section, values in pairs(settings) do
        config[section] = values
      end
    end,
    gesture = function(options)
      gestures[options.direction] = options
    end,
    bind = function(keys, action, options)
      local normal = keys:upper():gsub("%s+", "")
      if found[normal] then
        duplicates[#duplicates + 1] = keys
      end
      found[normal] = { description = options and options.description, action = action }
    end,
  }, { __index = function() return function() end end })
  _G.dot = { workspaces = module }
  dofile(repo .. "/config/hypr/fallback.lua")
  dofile(repo .. "/config/hypr/keybinds.lua")
  return found, duplicates, module, gestures, config
end

local function key(name)
  return name:upper():gsub("%s+", "")
end

test("no two binds share keys", function()
  local _, duplicates = binds()
  same(duplicates, {}, "keys bound twice")
end)

test("SUPER + 1-0 and their SHIFT and SHIFT + ALT forms reach the screen's workspaces 1-10", function()
  local found = binds()
  for slot = 1, 10 do
    local code = "code:" .. (slot + 9)
    same(found[key("SUPER + " .. code)].description, "Go to workspace " .. slot, code)
    same(found[key("SUPER + SHIFT + " .. code)].description, "Move window to workspace " .. slot, code)
    same(found[key("SUPER + SHIFT + ALT + " .. code)].description, "Move window silently to workspace " .. slot, code)
    same(type(found[key("SUPER + SHIFT + " .. code)].action), "function", "a per-screen action")
  end
end)

test("SUPER + hjkl focus the windows and the arrows the screens", function()
  local found = binds()
  local names = { H = "left", J = "down", K = "up", L = "right" }
  local arrows = { H = "Left", J = "Down", K = "Up", L = "Right" }
  for letter, name in pairs(names) do
    local window = found[key("SUPER + " .. letter)]
    same({ window.description, window.action.args.direction }, { "Focus window " .. name, name:sub(1, 1) }, letter)
    local screen = found[key("SUPER + " .. arrows[letter])]
    same({ screen.description, screen.action.dsp, screen.action.args.monitor }, { "Focus screen " .. name, "focus", name:sub(1, 1) }, arrows[letter])
  end
end)

test("the workspace cycling binds walk the focused screen's groups", function()
  local found = binds()
  local want = {
    ["SUPER + Tab"] = "Next workspace",
    ["SUPER + SHIFT + Tab"] = "Previous workspace",
    ["SUPER + CTRL + Right"] = "Next workspace",
    ["SUPER + CTRL + Left"] = "Previous workspace",
    ["SUPER + mouse_down"] = "Next workspace",
    ["SUPER + mouse_up"] = "Previous workspace",
    ["SUPER + CTRL + ALT + Right"] = "Next workspace group",
    ["SUPER + CTRL + ALT + Left"] = "Previous workspace group",
  }
  for keys, description in pairs(want) do
    same({ found[key(keys)].description, type(found[key(keys)].action) }, { description, "function" }, keys)
  end
  same(found[key("SUPER + CTRL + Tab")].action.args.workspace, "previous_per_monitor", "former workspace")
end)

test("the three-finger swipe is Hyprland's own, which follows the fingers", function()
  local _, _, _, gestures, config = binds()
  same({ gestures.horizontal.fingers, gestures.horizontal.action }, { 3, "workspace" }, "horizontal")
  same({ gestures.left, gestures.right }, { nil, nil }, "no swipe that switches only once it ends")
  same(config.gestures.workspace_swipe_create_new, false, "the kept empty workspaces end each group instead")
end)

test("the group binds are on SUPER + ALT + Tab", function()
  local found = binds()
  same(found[key("SUPER + ALT + Tab")].action.dsp, "group.next", "next")
  same(found[key("SUPER + ALT + SHIFT + Tab")].action.dsp, "group.prev", "previous")
end)

test("SUPER + SHIFT + CTRL + Right moves the window to a new workspace", function()
  local found = binds()
  same(found[key("SUPER + SHIFT + CTRL + Right")].description, "Move window to a new workspace", "description")
end)

test("SUPER + SHIFT + Space switches to the next keyboard layout", function()
  local found = binds()
  local switch = found[key("SUPER + SHIFT + Space")]
  same(switch.description, "Next keyboard layout", "description")
  same({ switch.action.dsp, switch.action.args }, { "exec_cmd", "hyprctl switchxkblayout all next" }, "action")
end)

os.exit(failed == 0 and 0 or 1)
