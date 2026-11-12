# Counters: kubaos-counters (crates/kubaos-counters), the command the menu, the key
# (config/hypr/keybinds.lua) and the bar change counters with. It keeps them in
# ~/.local/state/kubaos/counters.toml, every change in counters.log beside it, and the state the
# bar watches in $XDG_RUNTIME_DIR/kubaos-counters.json.
#
# A counter's auto reset runs from a transient timer, kubaos-counters-reset.timer, which each
# change arms at the next reset; it is on the calendar, so one that came due asleep fires on resume.
# The timer doesn't outlive a reboot: kubaos-counters.service runs `tick` at login, which does the
# resets missed while switched off, writes the bar's state file and arms the timer again.
{
  kubaosPkgs,
  lib,
  ...
}:
{
  home.packages = [ kubaosPkgs.kubaos-counters ];

  systemd.user.services.kubaos-counters = {
    Unit.Description = "KubaOS counters: the resets missed while off, and the bar's state";
    Service = {
      Type = "oneshot";
      ExecStart = "${lib.getExe kubaosPkgs.kubaos-counters} tick";
    };
    Install.WantedBy = [ "default.target" ];
  };
}
