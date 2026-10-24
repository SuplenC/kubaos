# The clock popup: kubaos-clock (crates/kubaos-clock), the date engine the
# shell's popup talks to over a pipe and the menu changes the clock settings
# with, and the sound its timers and alarms ring with.
#
# The sound goes at a fixed path under ~/.local/share, so the shell needn't
# know where the Nix profile is. pw-play (Ubuntu's PipeWire) plays it.
{
  kubaosPkgs,
  pkgs,
  ...
}:
{
  home.packages = [ kubaosPkgs.kubaos-clock ];

  xdg.dataFile."kubaos/clock/alarm.oga".source =
    "${pkgs.sound-theme-freedesktop}/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga";
}
