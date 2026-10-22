# The Suite framework's Nix half. Each Suite is a module, suites/<name>.nix,
# imported here, that does nothing until kubaos.suites.<name> is set. state.nix
# sets it from kubaos.d/suites/<name>.toml, the file `kubaos suite add` writes;
# the Suite's metadata is in crates/kubaos-core/src/suites/catalog.rs.
#
# A Suite module installs with the usual options (home.packages,
# home.sessionVariables, home.sessionPath). Its editor settings go into the
# Defaults through kubaos.nvim.suiteSpecs: this module writes them to
# ~/.config/nvim/lua/kubaos/suites.lua, which config/nvim/lua/config/lazy.lua
# loads after KubaOS's LazyVim extras and before the user layer's plugin
# specs, so the user's settings win.
{
  config,
  lib,
  ...
}:
let
  suiteModules = lib.mapAttrsToList (name: _: ./suites + "/${name}") (
    lib.filterAttrs (name: type: type == "regular" && lib.hasSuffix ".nix" name) (builtins.readDir ./suites)
  );
in
{
  imports = suiteModules;

  options.kubaos = {
    suites = lib.mkOption {
      type = lib.types.attrsOf (
        lib.types.submodule {
          options = {
            version = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "The chosen version, for a Suite that has versions.";
            };
            extras = lib.mkOption {
              type = lib.types.listOf lib.types.str;
              default = [ ];
              description = "The Suite's extras that are turned on.";
            };
          };
        }
      );
      default = { };
      description = "The installed Suites and their options, by name, from kubaos.d/suites/*.toml.";
    };

    nvim.suiteSpecs = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ ''{ import = "lazyvim.plugins.extras.lang.go" }'' ];
      description = "lazy.nvim specs, each a Lua expression, that the installed Suites add to the Defaults.";
    };
  };

  config.xdg.configFile."nvim/lua/kubaos/suites.lua".text = ''
    -- The installed Suites' Neovim settings, written by KubaOS's home/suites.nix.
    -- Part of the Defaults: config/nvim/lua/config/lazy.lua loads these before
    -- the user layer's plugins/*.lua, so the user's settings win.
    return {
    ${lib.concatMapStrings (spec: "  ${spec},\n") config.kubaos.nvim.suiteSpecs}}
  '';
}
