# The proto + moon Suite: proto, and moon through it. Nothing until
# kubaos.d/suites/proto.toml exists; the metadata is the `proto` entry in
# crates/kubaos-core/src/suites/catalog.rs.
#
# proto is a version manager itself: a project pins its tools, moon and proto
# among them, in its .prototools, so the Suite has no versions and no Project
# pins. moon comes from proto, not nixpkgs, whose moon is a major behind: the
# `moon` and `moonx` here install it with `proto install moon` the first time
# they run, then proto's shims, ahead of them on PATH, take over. A switch
# downloads nothing.
#
# Everything proto installs lives in its home, ~/.local/share/proto, the one
# folder the catalog lists. Neovim needs nothing of its own: LazyVim's YAML
# extra validates moon.yml and .moon/*.yml with SchemaStore's moon schemas.
#
# `kubaos-moon-new DIR` is the scaffolder.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.proto or null;
  protoHome = "${config.xdg.dataHome}/proto";

  firstUse =
    name:
    pkgs.writeShellApplication {
      inherit name;
      runtimeInputs = [ pkgs.proto ];
      text = ''
        # proto installs the version the folder's .prototools pins, else the
        # latest, which it then pins globally.
        proto install moon >&2
        exec ${protoHome}/shims/${name} "$@"
      '';
    };
  moon = firstUse "moon";

  newWorkspace = pkgs.writeShellApplication {
    name = "kubaos-moon-new";
    runtimeInputs = [
      pkgs.git
      pkgs.proto
    ];
    text = ''
      # kubaos-moon-new DIR: a moon workspace in DIR, which must not exist: a
      # git repository (moon reads its default branch), .moon/workspace.yml,
      # and .prototools pinning the moon that made it.
      if (($# != 1)); then
        echo "usage: kubaos-moon-new DIR" >&2
        exit 2
      fi
      mkdir -- "$1"
      cd -- "$1"
      git init --quiet
      ${lib.getExe moon} init --yes .
      version=$(${lib.getExe moon} --version)
      proto pin moon "''${version#moon }"
    '';
  };
in
{
  config = lib.mkIf (suite != null) {
    home.packages = [
      pkgs.proto
      moon
      (firstUse "moonx")
      newWorkspace
    ];
    home.sessionVariables = {
      # Set rather than left to proto: newer versions pick the XDG data dir
      # when ~/.proto doesn't exist, older ones ~/.proto, and the PATH below
      # has to name the one it uses.
      PROTO_HOME = protoHome;
      # The check's advice, `proto upgrade`, can't replace Nix's read-only
      # proto; a newer one comes with KubaOS's next nixpkgs.
      PROTO_VERSION_CHECK = "0";
    };
    home.sessionPath = [
      "${protoHome}/shims"
      "${protoHome}/bin"
    ];
  };
}
