# The test Suite: one of everything a Suite declares, for KubaOS's own tests
# (the `suites` check in test/lib/e2e.sh). Only a `kubaos` built with the
# `test-suite` feature offers it; this module does nothing without
# kubaos.d/suites/test.toml.
#
# - kubaos-test-suite: `version`, `run` (fills the Suite's cache,
#   ~/.cache/kubaos-test-suite) and `new DIR` (the scaffolder).
# - kubaos-test-suite-greet, with the `greeting` extra.
# - KUBAOS_TEST_SUITE_VERSION, and ~/.local/share/kubaos-test-suite/bin on PATH.
# - A Neovim setting in the Defaults: LazyVim's opts gain kubaos_test_suite.
# - kubaos-test-suite at each version for Project pins (pins.nix), declared
#   only while the Suite is installed, so a real home never lists it. Its
#   command is in a `bin` output beside `out`, as most nixpkgs toolchains'
#   are, so the pins check covers a toolchain that isn't `out`.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.test or null;
  version = if suite == null || suite.version == null then "1" else suite.version;
  # The catalog's versions (crates/kubaos-core/src/suites/catalog.rs).
  versions = [
    "1"
    "2"
    "3"
  ];

  scriptAt =
    version:
    pkgs.writeShellScript "kubaos-test-suite" ''
      set -euo pipefail
      case "''${1:-}" in
        version) echo ${lib.escapeShellArg version} ;;
        run)
          mkdir -p "$HOME/.cache/kubaos-test-suite"
          echo "built with the test Suite ${version}" >"$HOME/.cache/kubaos-test-suite/build.log"
          ;;
        new)
          mkdir -- "''${2:?usage: kubaos-test-suite new DIR}"
          printf '# %s\n\nMade by the test Suite %s.\n' "$(basename -- "$2")" ${lib.escapeShellArg version} >"$2/README.md"
          ;;
        *) echo "usage: kubaos-test-suite version|run|new DIR" >&2; exit 2 ;;
      esac
    '';
  toolchainAt =
    version:
    pkgs.runCommand "kubaos-test-suite"
      {
        outputs = [
          "bin"
          "out"
        ];
      }
      ''
        install -D -m 0755 ${scriptAt version} $bin/bin/kubaos-test-suite
        mkdir -p $out/share/doc/kubaos-test-suite
        echo "The test Suite ${version}." >$out/share/doc/kubaos-test-suite/README
      '';
  greet = pkgs.writeShellScriptBin "kubaos-test-suite-greet" ''
    echo "hello from the test Suite"
  '';
in
{
  config = lib.mkIf (suite != null) {
    home.packages = [ (toolchainAt version) ] ++ lib.optional (lib.elem "greeting" suite.extras) greet;
    home.sessionVariables.KUBAOS_TEST_SUITE_VERSION = version;
    home.sessionPath = [ "${config.home.homeDirectory}/.local/share/kubaos-test-suite/bin" ];
    kubaos.nvim.suiteSpecs = [
      ''{ "LazyVim/LazyVim", opts = { kubaos_test_suite = { greeting = "from the test Suite", version = "${version}" } } }''
    ];
    kubaos.suiteToolchains.test = lib.genAttrs versions toolchainAt;
  };
}
