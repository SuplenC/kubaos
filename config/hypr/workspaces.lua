-- Workspaces 1-3 on the laptop, 4-6 on external 1, 7-9 on external 2.
-- Persistent, so they exist (and show in the bar) even when empty.
--
-- When an external is unplugged Hyprland moves its workspaces to a remaining
-- monitor. It does not move them back on replug: scripts/dot-hypr-workspaces
-- (started from autostart.lua) does that, reading these same rules back with
-- `hyprctl -j workspacerules`, so this file stays the single source of truth.

local m = dot.monitors

local layout = {
  [m.laptop] = { 1, 2, 3 },
  [m.external_1] = { 4, 5, 6 },
  [m.external_2] = { 7, 8, 9 },
}

for monitor, ids in pairs(layout) do
  for index, id in ipairs(ids) do
    hl.workspace_rule({
      workspace = tostring(id),
      monitor = monitor,
      persistent = true,
      default = index == 1,
    })
  end
end

-- No wider gaps for a lone window: corners are concentric with the shell's
-- frame only at gaps_out 8 (looks.lua), so one window keeps the same 8 px.
