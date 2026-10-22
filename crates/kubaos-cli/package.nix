# The `kubaos` command (home/curated.nix), built from the release it ships
# with. home/updates.nix installs it in every home, so each switch puts the
# release's `kubaos` on the user's PATH, and /usr/bin/kubaos runs it
# (install/firstboot/kubaos-launcher).
{
  lib,
  rustPlatform,
}:

let
  root = ../..;
in
rustPlatform.buildRustPackage {
  pname = "kubaos";
  version = (lib.importTOML ../../Cargo.toml).workspace.package.version;

  # The workspace and the first-boot files the engine embeds, so a change
  # elsewhere doesn't rebuild it.
  src = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions [
      ../../Cargo.toml
      ../../Cargo.lock
      ../../crates
      ../../install/firstboot
    ];
  };

  cargoLock = {
    lockFile = ../../Cargo.lock;
    # Kotodama's crates come from git, at the rev in Cargo.lock.
    allowBuiltinFetchGit = true;
  };
  cargoBuildFlags = [
    "-p"
    "kubaos-cli"
  ];
  # Its tests run git, bash and the repo's files outside src; `cargo test` runs them.
  doCheck = false;

  meta = {
    description = "The kubaos command: packages, Suites, switches and KubaOS updates";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos";
    platforms = lib.platforms.linux;
  };
}
