# `kubaos-phone` (home/curated.nix): the phone link's command, which turns
# KDE Connect on and off and sends to the phone. home/phone.nix installs it
# and the user unit that runs kdeconnectd.
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
  pname = "kubaos-phone";
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
    "kubaos-phone"
  ];
  cargoTestFlags = [
    "-p"
    "kubaos-phone"
  ];

  meta = {
    description = "KubaOS's phone link through KDE Connect";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-phone";
    platforms = lib.platforms.linux;
  };
}
