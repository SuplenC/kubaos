# `kubaos-bluetooth` (home/curated.nix): the Bluetooth pairing agent, and the
# command the shell pairs and scans through. home/bluetooth.nix runs it.
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
  pname = "kubaos-bluetooth";
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
    "kubaos-bluetooth"
  ];
  cargoTestFlags = [
    "-p"
    "kubaos-bluetooth"
  ];

  meta = {
    description = "KubaOS's Bluetooth pairing agent";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-bluetooth";
    platforms = lib.platforms.linux;
  };
}
