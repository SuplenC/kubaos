# Apps open on the workspace they were launched from: the launch watcher,
# kubaos-launches (crates/kubaos-launches), as a user unit that tells
# Hyprland of each app's launch (config/hypr/launches.lua places the window).
#
# It runs with the graphical session, after autostart.lua has imported
# HYPRLAND_INSTANCE_SIGNATURE into the user manager, so it reaches the
# Hyprland of this session; a new session starts it again.
# `kubaos-launches off` turns placing off without stopping the unit.
{
  kubaosPkgs,
  lib,
  ...
}:
{
  home.packages = [ kubaosPkgs.kubaos-launches ];

  systemd.user.services.kubaos-launches = {
    Unit = {
      Description = "KubaOS launch watcher, so apps open where they were launched";
      After = [
        "graphical-session.target"
        "dbus.socket"
      ];
      PartOf = [ "graphical-session.target" ];
    };
    Service = {
      Type = "simple";
      ExecStart = "${lib.getExe kubaosPkgs.kubaos-launches} watch";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install.WantedBy = [ "graphical-session.target" ];
  };
}
