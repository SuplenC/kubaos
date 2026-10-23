# `kubaos-files` (home/curated.nix): yazi in Ghostty for xdg-open's folders
# and for apps' "Show in folder". home/files.nix installs it and registers it.
{
  lib,
  rustPlatform,
}:

let
  root = ../..;
in
rustPlatform.buildRustPackage {
  pname = "kubaos-files";
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
    # Kotodama's crates come from git, at the rev in Cargo.lock.
    allowBuiltinFetchGit = true;
  };
  cargoBuildFlags = [
    "-p"
    "kubaos-files"
  ];
  cargoTestFlags = [
    "-p"
    "kubaos-files"
  ];

  meta = {
    description = "yazi in a terminal window, for xdg-open and org.freedesktop.FileManager1";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-files";
    platforms = lib.platforms.linux;
  };
}
