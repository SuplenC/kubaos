# `kubaos-clock` (home/curated.nix): the date engine the clock popup talks to,
# and the command the menu changes the clock settings with. home/clock.nix
# installs it.
{
  lib,
  rustPlatform,
  tzdata,
  # Hashes for Cargo.lock's git crates (home/curated.nix).
  cargoOutputHashes ? { },
}:

let
  root = ../..;
in
rustPlatform.buildRustPackage {
  pname = "kubaos-clock";
  version = (lib.importTOML ../../Cargo.toml).workspace.package.version;

  # The workspace alone: the holiday rules it embeds live in the crate.
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
    "kubaos-clock"
  ];
  cargoTestFlags = [
    "-p"
    "kubaos-clock"
  ];

  # The tests read zones (Europe/Warsaw, Asia/Tokyo) from the system's tz
  # database, and the build sandbox has no /usr/share/zoneinfo. At run time
  # Ubuntu's tzdata is the one read, so the clock follows its updates.
  preCheck = ''
    export TZDIR=${tzdata}/share/zoneinfo
  '';

  meta = {
    description = "KubaOS's date engine: the clock popup's calendar, date maths and time zones";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-clock";
    platforms = lib.platforms.linux;
  };
}
