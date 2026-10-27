# The flake's checks (`nix flake check`): the design checks of
# docs/DESIGN.md, "Checks", run on the flake's own source.
#
# - design-lint: no literal colour, size, spacing, radius, duration or font,
#   and no toolkit control, in QML and Rust UI code beyond what
#   design/lint-baseline.toml still allows.
# - design-geometry: every page of the design specimen, at every scale, light
#   and dark, held to the geometry rules, with the curated kubaos-specimen.
#
# `cargo test --workspace` (`just test`) runs both as tests too; these run
# them from the built binaries, in the sandbox.
{
  pkgs,
  inputs,
  src,
  kubaos-specimen,
}:
let
  buildCrate = import ./build.nix { inherit pkgs inputs; };
  kubaos-design = buildCrate "kubaos-design" {
    pname = "kubaos-design";
    meta.mainProgram = "kubaos-design";
  };
in
{
  design-lint = pkgs.runCommand "kubaos-design-lint" { } ''
    ${kubaos-design}/bin/kubaos-design lint --root ${src}
    touch $out
  '';
  design-geometry = pkgs.runCommand "kubaos-design-geometry" { } ''
    ${kubaos-specimen}/bin/kubaos-specimen check
    touch $out
  '';
}
