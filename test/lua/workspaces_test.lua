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

-- A fresh state directory and a fresh copy of the module on a new fake.
local function session()
  local dir = os.tmpname()
  os.remove(dir)
  assert(os.execute("mkdir -p '" .. dir .. "'"))
  os.getenv = function(key)
    if key == "XDG_STATE_HOME" then
      return dir
    end
    return real_getenv(key)
  end
  local world = Fake.new()
  _G.hl = world.hl
  _G.dot = {}
  package.loaded.workspaces = nil
  local module = require("workspaces")
  world.state = dir .. "/kubaos/hypr"
  return world, module
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

local LG_A = "LG Electronics LG ULTRAGEAR 312NTAB1A234"
local LG_B = "LG Electronics LG ULTRAGEAR 312NTAB1A999"
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

test("the screens file reads back what it wrote", function()
  local text = blocks.format_blocks({ [PANEL] = 0, [LG_A] = 1, [LG_B] = 2 }, { ["eDP-1"] = PANEL, ["DP-3"] = LG_A })
  same(text, "0\teDP-1\t" .. PANEL .. "\n1\tDP-3\t" .. LG_A .. "\n2\t\t" .. LG_B .. "\n", "text")
  local parsed, current = blocks.parse_blocks(text)
  same({ parsed[PANEL], parsed[LG_A], parsed[LG_B], current["DP-3"] }, { 0, 1, 2, LG_A }, "parsed")
end)

test("the merged file reads back what it wrote", function()
  local merged = blocks.parse_merged(blocks.format_merged({ [3] = { id = 11, identity = LG_A } }))
  same({ merged[3].id, merged[3].identity }, { 11, LG_A }, "record")
end)

-- The module on the fake Hyprland.

test("at login each screen shows its own workspace 1", function()
  local world = two_screens()
  same({ world:shown_on("eDP-1"), world:shown_on("DP-1") }, { 1, 11 }, "shown")
  same(world:ids_on("DP-1"), { 11 }, "the number Hyprland gave DP-1 is gone")
end)

test("the screens file names each screen's block and connector", function()
  local world = two_screens()
  same(read(world.state .. "/workspace-screens"), "0\teDP-1\t" .. PANEL .. "\n1\tDP-1\t" .. LG_A .. "\n", "file")
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
  same(world:ids_on("eDP-1"), { 4 }, "the empty 1 is gone")
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

test("unplugging a screen puts its workspaces after the other screen's own", function()
  local world = busy()

  world:unplug("DP-1")

  same(world:ids_on("eDP-1"), { 1, 2, 3, 4, 5 }, "laptop")
end)

test("plugging it back returns them under their old numbers", function()
  local world = busy()
  world:unplug("DP-1")

  world:plug("DP-3", LG_A, 1920)

  same(world:ids_on("eDP-1"), { 1, 2 }, "laptop")
  same(world:ids_on("DP-3"), { 11, 12, 13 }, "external, on a new connector")
end)

test("a config reload between unplug and replug keeps the record", function()
  local world = busy()
  world:unplug("DP-1")
  reload(world)

  world:plug("DP-1", LG_A, 1920)

  same(world:ids_on("DP-1"), { 11, 12, 13 }, "external")
end)

test("a new session forgets what joined another screen", function()
  local world = busy()
  world:unplug("DP-1")

  world:fire("hyprland.start")

  same(read(world.state .. "/workspace-merged"), "", "merged file")
end)

test("a second external gets the next block and keeps it across plugs", function()
  local world, module = two_screens()
  world:plug("DP-2", LG_B, 3840)
  world:focus_monitor("DP-2")
  run(module.focus(1))

  same(world:shown_on("DP-2"), 21, "shown")

  world:unplug("DP-1")
  world:unplug("DP-2")
  world:plug("DP-5", LG_B, 1920)
  same(world:shown_on("DP-5"), 21, "same block on another connector")
end)

test("closing the lid puts the laptop's workspaces after the external's", function()
  local world = busy()

  world:unplug("eDP-1")
  same(world:ids_on("DP-1"), { 11, 12, 13, 14, 15 }, "external")

  world:plug("eDP-1", PANEL, 0)
  same(world:ids_on("eDP-1"), { 1, 2 }, "laptop")
  same(world:ids_on("DP-1"), { 11, 12, 13 }, "external")
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
  _G.hl = setmetatable({
    dsp = dsp,
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
  return found, duplicates, module
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

test("SUPER + hjkl focus the screens and the arrows the windows", function()
  local found = binds()
  local names = { H = "left", J = "down", K = "up", L = "right" }
  local arrows = { H = "Left", J = "Down", K = "Up", L = "Right" }
  for letter, name in pairs(names) do
    local screen = found[key("SUPER + " .. letter)]
    same({ screen.description, screen.action.dsp, screen.action.args.monitor }, { "Focus screen " .. name, "focus", name:sub(1, 1) }, letter)
    local window = found[key("SUPER + " .. arrows[letter])]
    same({ window.description, window.action.args.direction }, { "Focus window " .. name, name:sub(1, 1) }, arrows[letter])
  end
end)

test("the workspace cycling binds stay on the focused screen", function()
  local found = binds()
  local want = {
    ["SUPER + Tab"] = "m+1",
    ["SUPER + SHIFT + Tab"] = "m-1",
    ["SUPER + CTRL + Right"] = "m+1",
    ["SUPER + CTRL + Left"] = "m-1",
    ["SUPER + mouse_down"] = "m+1",
    ["SUPER + mouse_up"] = "m-1",
    ["SUPER + CTRL + Tab"] = "previous_per_monitor",
  }
  for keys, workspace in pairs(want) do
    same(found[key(keys)].action.args.workspace, workspace, keys)
  end
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
