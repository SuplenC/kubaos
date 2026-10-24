# Removable drives: kubaos-automount (crates/kubaos-automount) as a user unit.
# It mounts a USB stick or SD card as it's plugged in, through udisks2, and
# notifies with Open and Eject (and Import photos when the drive has a DCIM
# folder). Internal disks and drives already mounted are left alone; an
# encrypted drive is unlocked from the Files sidebar, which asks for its
# passphrase.
#
# The Command menu's Eject Items read $XDG_RUNTIME_DIR/kubaos-drives, which
# the service keeps, and eject through `kubaos-automount eject`.
# `kubaos-automount off` turns mounting off without stopping the unit, so
# Eject still works.
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  automount = lib.getExe kubaosPkgs.kubaos-automount;
in
{
  home.packages = [ kubaosPkgs.kubaos-automount ];

  systemd.user.services.kubaos-automount = {
    Unit = {
      Description = "KubaOS automount for removable drives";
      After = [ "dbus.socket" ];
    };
    Service = {
      Type = "simple";
      ExecStart = "${automount} watch";
      # A drives file left behind would list drives the menu can't eject.
      ExecStopPost = "${pkgs.coreutils}/bin/rm -f %t/kubaos-drives";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install.WantedBy = [ "default.target" ];
  };
}
