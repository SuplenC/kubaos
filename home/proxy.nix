# The Proxy, one switch for the work network: kubaos-proxy
# (crates/kubaos-proxy), and its service as a user unit of the graphical
# session. The switch itself is `kubaos-proxy on|off`, from the control
# center's tile, the Command menu or a terminal.
#
# The service changes nothing at login but what a session left behind: a
# login starts with the Proxy off, so a part a crash or a reboot left up (the
# kill switch, apt's proxy) goes. While the user wants it on, it brings the
# tunnel back after a resume, since NetworkManager drops it when the machine
# sleeps. The root side is /usr/local/libexec/kubaos-proxy-write
# (install_proxy_helper, system/lib.sh).
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  proxy = lib.getExe kubaosPkgs.kubaos-proxy;
in
{
  home.packages = [ kubaosPkgs.kubaos-proxy ];

  systemd.user.services.kubaos-proxy = {
    Unit = {
      Description = "KubaOS Proxy, the work network's switch";
      # notify-send reaches the shell's notifications through the session
      # bus, and pkexec finds the session's polkit agent.
      After = [
        "graphical-session.target"
        "dbus.socket"
      ];
      PartOf = [ "graphical-session.target" ];
    };
    Service = {
      Type = "simple";
      ExecStart = "${proxy} serve";
      ExecStopPost = "${pkgs.coreutils}/bin/rm -f %t/kubaos-proxy.json";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install.WantedBy = [ "graphical-session.target" ];
  };
}
