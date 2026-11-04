# The Python Suite: uv, which installs the Pythons and makes each project's
# venv, basedpyright, ruff, debugpy's adapter and LazyVim's Python extra
# (python.lua). Nothing until kubaos.d/suites/python.toml exists; the metadata
# is the `python` entry in crates/kubaos-core/src/suites/catalog.rs.
#
# The Suite has no versions and no Project pins: a project's Python is uv's,
# from its .python-version (`uv python pin`) and requires-python, so a Nix
# Python on PATH would only compete with it.
#
# uv's cache is ~/.cache/uv. The Pythons and tools it installs live under
# ~/.local/share/uv, their commands in its bin/, on PATH, rather than among
# the user's own in ~/.local/bin, so purging the Suite's caches takes them
# whole. The catalog lists both folders.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.python or null;
  uvBin = "${config.home.homeDirectory}/.local/share/uv/bin";

  # LazyVim's Python extra starts Mason's `debugpy-adapter`. The adapter runs
  # the program on the project's venv with its own copy of debugpy, so the
  # venv needs none.
  debugpyAdapter = pkgs.writeShellScriptBin "debugpy-adapter" ''
    exec ${lib.getExe' pkgs.python3Packages.debugpy "debugpy-adapter"} "$@"
  '';
in
{
  config = lib.mkIf (suite != null) {
    home.packages = [
      pkgs.uv
      pkgs.basedpyright
      pkgs.ruff
      debugpyAdapter
    ];
    home.sessionVariables = {
      UV_TOOL_BIN_DIR = uvBin;
      UV_PYTHON_BIN_DIR = uvBin;
    };
    home.sessionPath = [ uvBin ];
    kubaos.nvim.suiteSpecs = [ ''dofile("${./python.lua}")'' ];
  };
}
