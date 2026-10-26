# The phone link: KDE Connect (kdeconnectd) as a user unit that runs only
# once the user sets up a phone, and `kubaos-phone` (crates/kubaos-phone),
# which turns it on and off and sends to the phone. Until then nothing
# listens for phones or announces the laptop on the network.
#
# The opt-in is the flag file ~/.config/kubaos/desktop/phone/phone-link:
# Kōbō's "Your phone" card and `kubaos-phone on` write it and start the
# unit, `kubaos-phone off` removes it and stops the unit. The unit is wanted
# by every graphical session but starts only while the flag exists, so a
# login brings the link back exactly when it was left on.
#
# Only the daemon's path is used: KDE Connect's package isn't in the profile,
# so its D-Bus activation file and its autostart entry are nowhere a bus or a
# session would find them, and nothing can start kdeconnectd behind the
# flag's back.
{
  config,
  kubaosPkgs,
  lib,
  ...
}:
let
  userLayer = import ./user-layer.nix { configHome = config.xdg.configHome; };
in
{
  home.packages = [ kubaosPkgs.kubaos-phone ];

  systemd.user.services.kubaos-phone = {
    Unit = {
      Description = "KDE Connect, for the phone link";
      # crates/kubaos-phone/src/setting.rs owns this path.
      ConditionPathExists = "${userLayer.dir "phone"}/phone-link";
      # The clipboard plugin needs the session's WAYLAND_DISPLAY, which
      # autostart.lua imports before graphical-session.target starts.
      After = [
        "graphical-session.target"
        "dbus.socket"
      ];
      PartOf = [ "graphical-session.target" ];
    };
    Service = {
      Type = "dbus";
      BusName = "org.kde.kdeconnect";
      ExecStart = lib.getExe' kubaosPkgs.kdeconnect-kde "kdeconnectd";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install.WantedBy = [ "graphical-session.target" ];
  };
}
