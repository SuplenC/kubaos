# How the curated set builds KubaOS's crates (home/curated.nix): `buildCrate`
# makes the package of one crate, with crane.
#
# Every package builds on a shared derivation of the workspace's dependencies
# (crane's buildDepsOnly, one for the command-line tools and one for Kōbō),
# which changes only with Cargo.lock and the crates' manifests. An update that changes KubaOS's code compiles that
# code alone. A package's source is its own crate and the workspace crates it
# depends on, so a change to one crate rebuilds only the packages built from
# it. The tests don't run here: `cargo test --workspace` (`just test`) runs
# them, with the repo's files and the programs they need.
{
  lib,
  craneLib,
  # Hashes for Cargo.lock's git sources, by source (home/curated.nix).
  cargoOutputHashes ? { },
}:

let
  root = ../.;
  workspace = (lib.importTOML ../Cargo.toml).workspace;
  inherit (workspace.package) version;

  # Each workspace crate's manifest, by crate name.
  manifests = lib.listToAttrs (
    map (
      dir:
      let
        manifest = lib.importTOML (root + "/${dir}/Cargo.toml");
      in
      lib.nameValuePair manifest.package.name (manifest // { inherit dir; })
    ) workspace.members
  );

  # Files outside the crates that a crate embeds when it builds.
  embeds = {
    kubaos-core = [ ../install/firstboot ];
  };

  # The workspace crates `crate` builds against, as cargo resolves them: its
  # own and its targets' [dependencies] that are workspace members, and
  # theirs. Dev-dependencies only build for the tests.
  workspaceDeps =
    crate:
    let
      manifest = manifests.${crate};
      targetDeps = lib.concatMap (target: lib.attrNames (target.dependencies or { })) (
        lib.attrValues (manifest.target or { })
      );
    in
    lib.filter (dep: manifests ? ${dep}) (lib.attrNames (manifest.dependencies or { }) ++ targetDeps);
  closure =
    crate:
    map (item: item.key) (
      builtins.genericClosure {
        startSet = [ { key = crate; } ];
        operator = item: map (key: { inherit key; }) (workspaceDeps item.key);
      }
    );

  # The crates the curated set ships, by the dependencies build they share.
  # Each build compiles its crates' dependencies one crate at a time, as each
  # package does: cargo picks a dependency's features from the crates being
  # built, so a dependency built for all of them at once would not match what
  # one package asks for and would build again in it.
  shipped = {
    tools = [
      "kubaos-cli"
      "kotodama-kubaos"
      "kubaos-files"
      "kubaos-automount"
      "kubaos-phone"
      "kubaos-bluetooth"
      "kubaos-clock"
      "kubaos-sysmon"
      "kubaos-idle"
    ];
    # Kōbō's iced and wgpu are most of its dependencies and no other crate
    # uses them. Apart, they build beside the tools' dependencies when
    # Cargo.lock changes, instead of before every tool.
    kobo = [ "kubaos-kobo" ];
  };
  groupOf =
    crate:
    lib.findFirst (group: lib.elem crate shipped.${group})
      (throw "${crate} isn't in crates/workspace.nix's `shipped`, so its dependencies aren't prebuilt")
      (lib.attrNames shipped);

  commonArgs = {
    inherit version;
    strictDeps = true;
    doCheck = false;
    # One vendor directory for every package, from Cargo.lock alone.
    cargoVendorDir = craneLib.vendorCargoDeps {
      cargoLock = ../Cargo.lock;
      outputHashes = cargoOutputHashes;
    };
  };

  cargoArtifacts = lib.mapAttrs (
    group: crates:
    craneLib.buildDepsOnly (
      commonArgs
      // {
        pname = "kubaos-${group}";
        # crane keeps only the manifests and Cargo.lock of this, at evaluation
        # time, so a change to the code doesn't rebuild the dependencies.
        src = lib.fileset.toSource {
          inherit root;
          fileset = lib.fileset.unions [
            ../Cargo.toml
            ../Cargo.lock
            ../crates
          ];
        };
        buildPhaseCargoCommand = lib.concatMapStringsSep "\n" (
          crate: "cargoWithProfile build --locked -p ${crate}"
        ) crates;
      }
    )
  ) shipped;

  # The source `crate` builds from: the workspace's manifests, so cargo loads
  # the workspace, and the whole of the crates it builds, with what they embed.
  srcFor =
    crate:
    let
      crates = closure crate;
    in
    lib.fileset.toSource {
      inherit root;
      fileset = lib.fileset.unions (
        [
          ../Cargo.toml
          ../Cargo.lock
          (lib.fileset.fileFilter (file: file.name == "Cargo.toml") ../crates)
        ]
        ++ map (dep: root + "/${manifests.${dep}.dir}") crates
        ++ lib.concatMap (dep: embeds.${dep} or [ ]) crates
      );
    };
in
{
  # buildCrate CRATE ARGS: the package of the workspace crate CRATE, built with
  # `cargo build -p CRATE`. ARGS go to crane's buildPackage: pname, meta, and
  # any hooks.
  buildCrate =
    crate: args:
    craneLib.buildPackage (
      commonArgs
      // {
        cargoArtifacts = cargoArtifacts.${groupOf crate};
        src = srcFor crate;
        cargoExtraArgs = "--locked -p ${crate}";
        # The workspace members left out of the source get an empty library,
        # which cargo needs to load the workspace and never builds.
        postPatch = ''
          for manifest in crates/*/Cargo.toml; do
            dir=''${manifest%/Cargo.toml}
            if [ ! -e "$dir/src" ]; then
              mkdir "$dir/src"
              touch "$dir/src/lib.rs"
            fi
          done
        '';
      }
      // args
    );
}
