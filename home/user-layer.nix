# Where each app's settings live in the user layer, ~/.config/kubaos, grouped
# by area (docs/CONTRACTS.md, "Defaults and the user layer"). Not a module:
# links.nix, shell.nix and theme.nix import it with the config home.
# links.nix also writes the table into the layer file, which is where
# scripts/dot-config reads it, so the shell has no copy of its own.
#
# The same table in the other languages: crates/kubaos-user-layer (Rust),
# config/hypr/user_layer.lua, config/nvim/lua/config/user.lua and
# config/quickshell/settings/UserLayer.qml. links.nix checks this one against
# the seeds in user-layer/ at evaluation.
{ configHome }:
let
  groups = {
    desktop = [
      "hypr"
      "quickshell"
      "cursor"
      "clock"
      "keyboard"
      "sysmon"
      "files"
      "wallpapers"
    ];
    terminal = [
      "ghostty"
      "foot"
      "tmux"
      "zsh"
      "fastfetch"
    ];
    editor = [ "nvim" ];
    tools = [
      "git"
      "lazygit"
      "yazi"
      "bat"
      "bacon"
      "fuzzel"
    ];
    menu = [ "kotodama" ];
  };
in
rec {
  inherit groups;
  root = "${configHome}/kubaos";
  # Each app's folder relative to the user layer: `apps.clock` is "desktop/clock".
  apps = builtins.listToAttrs (
    builtins.concatLists (
      builtins.attrValues (
        builtins.mapAttrs (
          group: names:
          map (name: {
            inherit name;
            value = "${group}/${name}";
          }) names
        ) groups
      )
    )
  );
  # `dir "clock"` is ~/.config/kubaos/desktop/clock.
  dir = app: "${root}/${apps.${app}}";
}
