# The power stops: kubaos-power (crates/kubaos-power) as a user unit of the
# graphical session. It follows power-profiles-daemon's profile and UPower's
# power source, keeps the stop each source last had, and puts the rest of each
# stop to work: the root helper's sysfs writes
# (/usr/local/libexec/kubaos-power-write, system/30-hardware.sh), the laptop
# panel's refresh and VRR through Hyprland, and the file indexer's pause.
#
# The slider and the bar read $XDG_RUNTIME_DIR/kubaos-power.json, which the
# service keeps; `kubaos power set …` reaches it on kubaos-power.sock beside
# it. Both go when the unit stops, and with them the indexer's pause.
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  power = lib.getExe kubaosPkgs.kubaos-power;
in
{
  home.packages = [ kubaosPkgs.kubaos-power ];

  systemd.user.services.kubaos-power = {
    Unit = {
      Description = "KubaOS power stops";
      # hyprctl finds the session's Hyprland through the environment
      # autostart.lua imports before graphical-session.target starts.
      After = [
        "graphical-session.target"
        "dbus.socket"
      ];
      PartOf = [ "graphical-session.target" ];
    };
    Service = {
      Type = "simple";
      ExecStart = "${power} serve";
      ExecStopPost = [
        "${pkgs.coreutils}/bin/rm -f %t/kubaos-power.json %t/kubaos-power.sock"
        # Ubuntu's systemctl, the one of the user manager that runs this.
        "-systemctl --user stop kubaos-power-quiet.service"
      ];
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install.WantedBy = [ "graphical-session.target" ];
  };
}
