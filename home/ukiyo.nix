# ukiyo (浮世): the live wallpaper engine, KubaOS's default wallpaper.
# docs/UKIYO.md has the whole picture.
#
# - the package (ukiyo, ukiyo-activityd, the scenes in share/ukiyo/scenes,
#   reachable through ~/.nix-profile/share on XDG_DATA_DIRS)
# - ukiyo.service, a systemd user unit bound to the graphical session. It
#   has no [Install]: dot-theme starts it when a scene is the wallpaper and
#   stops it when a still or a loop takes over (`dot-theme apply` at login
#   does the same), so a session with a photo as wallpaper runs no daemon.
# - ~/.config/ukiyo/defaults.toml, the declarative layer. The wallpaper
#   settings panel (and `ukiyo config set`) writes config.toml next to it,
#   which wins; `ukiyo config reset` goes back to these.
# The zsh hook that pulses the scene on finished builds is in shell.nix.
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  inherit (kubaosPkgs) ukiyo;
  toml = pkgs.formats.toml { };
  defaults = toml.generate "ukiyo-defaults.toml" {
    scene = "kabukicho-gate";
    performance = {
      # Quickshell's Settings.screenOffAfter: drawing stops when hypridle
      # turns the screens off.
      idle_timeout = 600;
      # Motion's first-run mode is "On AC" (Settings.wallpaperAnimate), which
      # is `still` here. The panel keeps the two in step (services/Ukiyo.qml).
      on_battery = "still";
    };
    # The compositor's idle notifications, no root helper. ukiyo-activityd
    # (exact typing rate) needs a system service and input group access.
    signals.typing.source = "idle";
  };
in
{
  home.packages = [ ukiyo ];

  xdg.configFile."ukiyo/defaults.toml".source = defaults;

  systemd.user.services.ukiyo = {
    Unit = {
      Description = "ukiyo live wallpaper";
      Documentation = [ "file:///usr/share/kubaos/docs/UKIYO.md" ];
      PartOf = [ "graphical-session.target" ];
      After = [ "graphical-session.target" ];
      # Hyprland's autostart imports it (config/hypr/autostart.lua); without
      # it the daemon would look for a display that isn't there.
      ConditionEnvironment = "WAYLAND_DISPLAY";
      X-Restart-Triggers = [ "${defaults}" ];
    };
    Service = {
      ExecStart = "${lib.getExe ukiyo} daemon";
      Restart = "on-failure";
      RestartSec = 2;
      # The scenes: the package's, and any other in the Nix profile.
      Environment = [
        "XDG_DATA_DIRS=%h/.nix-profile/share:${ukiyo}/share:/usr/local/share:/usr/share"
        "RUST_LOG=info"
      ];
    };
  };
}
