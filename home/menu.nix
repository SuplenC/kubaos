# The Command menu: Kotodama, through its home-manager module
# (`programs.kotodama`, imported by `homeModules.default` in flake.nix).
#
# The module installs the command, the Daemon's socket-activated user units
# and the Bundled extensions turned on here. Its settings are KubaOS's
# Defaults in config/kotodama/ and the user layer's ~/.config/kubaos/kotodama/
# (home/links.nix), so none are set here and the module writes no config file.
# The menu itself runs inside KubaOS's own Quickshell (config/quickshell/shell.qml).
{ kubaosPkgs, ... }:
{
  programs.kotodama = {
    enable = true;
    package = kubaosPkgs.kotodama;
    # KubaOS's shell places `KotodamaMenu` itself. With the ready menu running
    # too, the keybind opens only one of the two and the other wastes memory.
    frontEnd.enable = false;

    # Apps, calculator, clipboard, emoji and web search are on by default.
    extensions = {
      folders.enable = true;
      hyprland.enable = true;
      system.enable = true;
    };
  };
}
