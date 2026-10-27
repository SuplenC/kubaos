# Project pins: `use suite <name> <version>` in an .envrc puts that version of
# a Suite's toolchain on PATH there (config/direnv/lib/suite.sh, linked into
# ~/.config/direnv/lib/ with the rest of config/).
#
# Each Suite module declares a toolchain per version in
# kubaos.suiteToolchains.<name>.<version>, whether the Suite is installed or
# not, so a folder can pin a version the home doesn't have. This module writes
# one entry per version, ~/.local/share/kubaos/toolchains/<name>/<version>:
# the derivation as `drv!output`, then the output path, for the toolchain's
# bin/. The generation keeps the derivation alive but never builds the
# toolchain, so an unused version costs a few store files; the direnv function
# realises it on the first load of a pin. `kubaos pins` reads the same entries
# (crates/kubaos-core/src/pins.rs).
{
  config,
  lib,
  ...
}:
let
  entry =
    toolchain:
    let
      bin = lib.getBin toolchain;
    in
    # Neither path brings its build along. The derivation's context is the
    # .drv file alone (a plain drvPath would build everything the toolchain is
    # built from), and the output path has none (it would build the
    # toolchain): the generation keeps the .drv files and builds nothing.
    ''
      ${builtins.unsafeDiscardOutputDependency toolchain.drvPath}!${bin.outputName}
      ${builtins.unsafeDiscardStringContext bin.outPath}
    '';

  entries = lib.concatMapAttrs (
    name: versions:
    lib.mapAttrs' (
      version: toolchain:
      lib.nameValuePair "kubaos/toolchains/${name}/${version}" { text = entry toolchain; }
    ) versions
  ) config.kubaos.suiteToolchains;
in
{
  options.kubaos.suiteToolchains = lib.mkOption {
    type = lib.types.attrsOf (lib.types.attrsOf lib.types.package);
    default = { };
    example = lib.literalExpression ''{ node = { "18" = pkgs.nodejs_18; "20" = pkgs.nodejs_20; }; }'';
    description = "Each Suite's toolchain per version, by Suite name, for Project pins. Its bin/ goes on PATH.";
  };

  config.xdg.dataFile = entries;
}
