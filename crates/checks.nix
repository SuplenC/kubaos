# The flake's checks (`nix flake check`): the design checks of
# docs/DESIGN.md, "Checks", run on the flake's own source.
#
# - design-lint: no literal colour, size, spacing, radius, duration or font,
#   and no toolkit control, in QML and Rust UI code beyond what
#   design/lint-baseline.toml still allows.
# - design-geometry: every page of the design specimen, at every scale, light
#   and dark, held to the geometry rules, with the curated kubaos-specimen.
# - design-parity: the components page drawn with `KubaOS.Components`, laid
#   out headless by Qt (`qml-specimen`), held to the same rules and to the
#   iced page's boxes, node for node, at every scale, light and dark.
# - qml-specimen: that runner, also for screenshots:
#   `nix run .#checks.<system>.qml-specimen -- shots DIR`.
# - qml-logic: the shell's logic in QML and JavaScript (test/qml), as Qt's
#   test runner runs it, headless (`just qml-test`).
#
# `cargo test --workspace` (`just test`) runs the lint and the geometry check
# as tests too, and `just design-check` runs all three outside the sandbox;
# these run them from the built binaries, in the sandbox.
{
  pkgs,
  inputs,
  src,
  kubaos-specimen,
  kubaos-components,
}:
let
  buildCrate = import ./build.nix { inherit pkgs inputs; };
  kubaos-design = buildCrate "kubaos-design" {
    pname = "kubaos-design";
    meta.mainProgram = "kubaos-design";
  };
  qml-specimen = pkgs.callPackage ../qml/specimen/package.nix { inherit kubaos-components; };
in
{
  inherit qml-specimen;
  design-lint = pkgs.runCommand "kubaos-design-lint" { } ''
    ${kubaos-design}/bin/kubaos-design lint --root ${src}
    touch $out
  '';
  design-geometry = pkgs.runCommand "kubaos-design-geometry" { } ''
    ${kubaos-specimen}/bin/kubaos-specimen check
    touch $out
  '';
  # The software renderer: the boxes are what is checked, and the sandbox has
  # no GPU.
  design-parity = pkgs.runCommand "kubaos-design-parity" { QT_QUICK_BACKEND = "software"; } ''
    ${qml-specimen}/bin/qml-specimen frames frames
    ${kubaos-specimen}/bin/kubaos-specimen parity frames
    touch $out
  '';
  # The shell's logic that needs no Quickshell, beside KubaOS.Components: keyed list diffs, a live
  # list's rows coming and going, the scan schedule, how the radio tabs order what is in range.
  # Offscreen, as the sandbox has no display.
  qml-logic =
    pkgs.runCommand "kubaos-qml-logic"
      {
        QT_QPA_PLATFORM = "offscreen";
        QT_PLUGIN_PATH = pkgs.lib.makeSearchPath "lib/qt-6/plugins" [ pkgs.qt6.qtbase ];
        NIXPKGS_QT6_QML_IMPORT_PATH = "${pkgs.qt6.qtdeclarative}/lib/qt-6/qml";
      }
      ''
        export HOME=$(mktemp -d)
        export XDG_RUNTIME_DIR=$HOME
        ${pkgs.qt6.qtdeclarative}/bin/qmltestrunner -import ${src}/qml -input ${src}/test/qml
        touch $out
      '';
}
