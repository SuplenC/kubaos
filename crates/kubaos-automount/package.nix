# `kubaos-automount` (home/curated.nix): the automount service for removable
# drives, the eject command and the photo import. home/automount.nix runs it.
{
  lib,
  rustPlatform,
  # Hashes for Cargo.lock's git crates (home/curated.nix).
  cargoOutputHashes ? { },
}:

let
  root = ../..;
in
rustPlatform.buildRustPackage {
  pname = "kubaos-automount";
  version = (lib.importTOML ../../Cargo.toml).workspace.package.version;

  # The workspace alone: this crate embeds nothing from the rest of the repo.
  src = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions [
      ../../Cargo.toml
      ../../Cargo.lock
      ../../crates
    ];
  };

  cargoLock = {
    lockFile = ../../Cargo.lock;
    # Kotodama's crates come from git, at the rev in Cargo.lock. One with no
    # hash in outputHashes is cloned during evaluation.
    outputHashes = cargoOutputHashes;
    allowBuiltinFetchGit = true;
  };
  cargoBuildFlags = [
    "-p"
    "kubaos-automount"
  ];
  cargoTestFlags = [
    "-p"
    "kubaos-automount"
  ];

  meta = {
    description = "KubaOS's automount for removable drives";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-automount";
    platforms = lib.platforms.linux;
  };
}
