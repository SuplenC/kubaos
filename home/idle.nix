# Idle: kubaos-idle (crates/kubaos-idle), which writes hypridle's listeners
# from ~/.config/kubaos/desktop/idle/idle.toml, holds the lid and Keep awake,
# schedules a lock, and is the command the menu changes the idle settings
# with. config/hypr/autostart.lua starts hypridle through it.
#
# Every switch writes the listeners before the session reloads, so the
# hypridle that `dot-session restart-idle` starts on new defaults (and the
# first one after an update) finds the file config/hypr/hypridle.conf sources.
{
  kubaosPkgs,
  lib,
  ...
}:
{
  home.packages = [ kubaosPkgs.kubaos-idle ];

  home.activation.idleListeners = lib.hm.dag.entryBetween [ "reloadSession" ] [ "userLayer" ] ''
    run ${lib.getExe kubaosPkgs.kubaos-idle} write || true
  '';
}
