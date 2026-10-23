# Bluetooth pairing: kubaos-bluetooth (crates/kubaos-bluetooth) as a user
# unit. It registers with bluetoothd as the default pairing agent, so a
# keyboard's passkey or a phone's code shows up in the control center's
# Bluetooth tab, and it pairs and scans for that tab. Without an agent,
# bluetoothd can't pair anything that needs a code.
#
# The shell reads the pairing state from $XDG_RUNTIME_DIR/kubaos-bluetooth.json
# and answers through `kubaos-bluetooth`, which calls the agent on the session
# bus (org.kubaos.Bluetooth).
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  agent = lib.getExe kubaosPkgs.kubaos-bluetooth;
in
{
  home.packages = [ kubaosPkgs.kubaos-bluetooth ];

  systemd.user.services.kubaos-bluetooth = {
    Unit = {
      Description = "KubaOS Bluetooth pairing agent";
      # bluetoothd may start after the user's session, or restart under it:
      # the agent registers whenever bluetoothd appears.
      After = [ "dbus.socket" ];
    };
    Service = {
      Type = "dbus";
      BusName = "org.kubaos.Bluetooth";
      ExecStart = "${agent} agent";
      # A state file left behind would show an agent that isn't there.
      ExecStopPost = "${pkgs.coreutils}/bin/rm -f %t/kubaos-bluetooth.json";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install.WantedBy = [ "default.target" ];
  };
}
