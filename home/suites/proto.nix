# The proto + moon Suite (crates/kubaos-core/src/suites/catalog.rs, PROTO):
# everything of it but proto, which mise installs from the Suite's fragment,
# ~/.config/mise/conf.d/kubaos-proto.toml. Installs nothing without
# kubaos.d/suites/proto.toml.
#
# proto is a version manager itself: a project pins its tools, moon and proto
# among them, in its .prototools, and proto installs them into its home,
# ~/.local/share/proto, the one folder the catalog lists. The Suite has no
# versions and no Project pins, and installs no moon: the scaffolder, or
# `proto install` in a project, does.
#
# - proto's shims ahead of everything mise puts on PATH, so a tool a project
#   pins in its .prototools runs at that version there, over a Suite's. A shim
#   that finds no version for the folder runs the next one on PATH, the
#   Suite's. In an interactive zsh that is kubaos-proto_path.toml in mise's
#   conf.d (mise puts a config's `_.path` ahead of its tools); in a login
#   shell it is the line after mise's shims (shell.nix), and in the graphical
#   session config/hypr/env.lua, which reads PROTO_HOME. Only tools proto
#   installed have shims.
# - Defaults the user changes in home.nix: PROTO_HOME, which every path here
#   follows, and PROTO_VERSION_CHECK off, since the check's advice,
#   `proto upgrade`, overwrites the proto in mise's install folder and puts
#   another in ~/.local/share/proto/bin, which the shims then run. proto
#   moves with `kubaos suite update` and `upgrade`.
# - kubaos-moon-new, the scaffolder, which runs the git and proto on PATH.
#
# Neovim needs nothing of its own: LazyVim's YAML extra validates moon.yml
# and .moon/*.yml with SchemaStore's moon schemas.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.proto or null;
  protoHome = config.home.sessionVariables.PROTO_HOME;
  shims = "${protoHome}/shims";

  miseDefaults = (pkgs.formats.toml { }).generate "kubaos-proto_path.toml" {
    env._.path = [ shims ];
  };

  newWorkspace = pkgs.writeShellApplication {
    name = "kubaos-moon-new";
    text = ''
      # kubaos-moon-new DIR: a moon workspace in DIR, which must not exist: a
      # git repository (moon reads its default branch), .prototools pinning
      # the moon that made it, and .moon/workspace.yml.
      if (($# != 1)); then
        echo "usage: kubaos-moon-new DIR" >&2
        exit 2
      fi
      if ! command -v git >/dev/null; then
        echo "kubaos-moon-new: git isn't on PATH" >&2
        exit 1
      fi
      if ! command -v proto >/dev/null; then
        echo "kubaos-moon-new: proto isn't on PATH; \`kubaos suite configure proto\` installs the Suite's tools again" >&2
        exit 1
      fi
      mkdir -- "$1"
      cd -- "$1"
      git init --quiet
      # The version a parent's .prototools pins, else the newest, pinned here.
      proto install moon --pin local --quiet
      proto run moon -- init --yes .
    '';
  };
in
{
  config = lib.mkIf (suite != null) {
    home.packages = [ newWorkspace ];
    home.sessionVariables = {
      # Set rather than left to proto, which picks ~/.proto when that exists
      # and the XDG data folder otherwise: the paths here have to name the
      # one it uses.
      PROTO_HOME = lib.mkDefault "${config.xdg.dataHome}/proto";
      PROTO_VERSION_CHECK = lib.mkDefault "0";
    };
    # After mise's shims (shell.nix's line is mkAfter, 1500).
    home.sessionVariablesExtra = lib.mkOrder 1600 ''
      export PATH="${shims}:$PATH"
    '';
    home.sessionPath = [ "${protoHome}/bin" ];
    xdg.configFile."mise/conf.d/kubaos-proto_path.toml".source = miseDefaults;
  };
}
