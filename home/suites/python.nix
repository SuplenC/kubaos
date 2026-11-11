# The Python Suite (crates/kubaos-core/src/suites/catalog.rs, PYTHON):
# everything of it but uv, basedpyright, ruff and debugpy, which mise installs
# from the Suite's fragment, ~/.config/mise/conf.d/kubaos-python.toml, and
# puts on PATH (shell.nix). Installs nothing without
# kubaos.d/suites/python.toml.
#
# - LazyVim's Python extra (python.lua), on the Suite's basedpyright, ruff and
#   debugpy instead of Mason's copies.
# - uv's command folder, ~/.local/share/uv/bin, on PATH: uv puts the commands
#   of the Pythons and tools it installs there rather than among the user's
#   own in ~/.local/bin, so purging the Suite's caches takes them whole. The
#   catalog lists ~/.local/share/uv as a cache.
#
# The Suite has no versions and no Project pins: a project's Python is uv's,
# from its .python-version (`uv python pin`) and requires-python. uv's
# Pythons are its own standalone builds, which run on the system's loader.
#
# uv's settings stay the user's: its two command folders are environment
# variables only (uv.toml has no key for them), set as defaults a plain
# `home.sessionVariables.UV_TOOL_BIN_DIR = …` in the user's home.nix replaces;
# everything else uv reads from its own ~/.config/uv/uv.toml.
{
  config,
  lib,
  ...
}:
let
  suite = config.kubaos.suites.python or null;
  uvBin = "${config.home.homeDirectory}/.local/share/uv/bin";
in
{
  config = lib.mkIf (suite != null) {
    home.sessionVariables = {
      UV_TOOL_BIN_DIR = lib.mkDefault uvBin;
      UV_PYTHON_BIN_DIR = lib.mkDefault uvBin;
    };
    home.sessionPath = [ uvBin ];
    kubaos.nvim.suiteSpecs = [ ''dofile("${./python.lua}")'' ];
  };
}
