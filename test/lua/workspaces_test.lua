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

-- A new fake on a fresh state directory, without the module.
local function bare()
  local dir = os.tmpname()
  os.remove(dir)
  assert(os.execute("mkdir -p '" .. dir .. "/kubaos/hypr'"))
  os.getenv = function(key)
    if key == "XDG_STATE_HOME" then
      return dir
    end
    return real_getenv(key)
  end
  local world = Fake.new()
  _G.hl = world.hl
  _G.dot = {}
  world.state = dir .. "/kubaos/hypr"
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
local LG_C = "LG Electronics LG ULTRAGEAR+ 510NTCZN7787"
local BENQ = "BNQ ZOWIE XL LCD G4L01663SL0"
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

test("migrating, the plugged-in externals take the positions they stand at and the rest follow", function()
  local _, _, legacy = blocks.parse_places(SCREENS_BY_SERIAL)
  local places, seen = blocks.migrate(legacy, {
    { name = "eDP-1", id = 0, x = 4480, y = 0, description = PANEL },
    { name = "DP-8", id = 1, x = 2560, y = 0, description = LG_A },
    { name = "DP-9", id = 2, x = 640, y = 0, description = LG_B },
  })
  same({ places.panel, places["1"], places["2"], places["3"], places["4"] }, { 0, 4, 3, 1, 2 }, "places")
  same({ seen["1"], seen["3"], seen["4"] }, { LG_B, LG_C, BENQ }, "seen")
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
  same(read(world.state .. "/workspace-screens"), "0\teDP-1\tpanel\t" .. PANEL .. "\n1\t\t3\t" .. LG_C .. "\n2\t\t4\t" .. BENQ
    .. "\n3\tDP-8\t2\t" .. LG_A .. "\n4\tDP-9\t1\t" .. LG_B .. "\n", "file")
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
