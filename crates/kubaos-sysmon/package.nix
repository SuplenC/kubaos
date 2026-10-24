# `kubaos-sysmon` (home/curated.nix): the system monitor the bar's pill and the
# popup read a stream from, and the command the menu changes its settings with.
# home/sysmon.nix installs it.
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
  pname = "kubaos-sysmon";
  version = (lib.importTOML ../../Cargo.toml).workspace.package.version;

  # The workspace alone: the tests read fixture /proc and /sys trees they write
  # themselves.
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
    "kubaos-sysmon"
  ];
  cargoTestFlags = [
    "-p"
    "kubaos-sysmon"
  ];

  meta = {
    description = "KubaOS's system monitor: CPU, memory, GPU, disks, network and battery from /proc and /sys";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-sysmon";
    platforms = lib.platforms.linux;
  };
}
