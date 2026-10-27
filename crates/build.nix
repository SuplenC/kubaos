# `buildCrate` (crates/workspace.nix) for this flake's nixpkgs, with the
# hashes of Cargo.lock's git sources. The curated set (home/curated.nix) and
# the flake's checks (crates/checks.nix) build KubaOS's crates through it.
{
  pkgs,
  inputs,
}:
let
  # Kotodama's crates in Cargo.lock come from git, at the rev of the
  # `kotodama` input. With the input's hash, crane fetches them as a
  # fixed-output derivation, which evaluates without the network. Without
  # it, it falls back to builtins.fetchGit, which clones during evaluation:
  # the machine's home then can't evaluate offline (home-manager switch, the
  # installer's build for a username other than the payload's).
  # HAZARD: only the source at the input's own rev gets the hash. Crates at
  # another rev are cloned during evaluation, with the network.
  kotodamaSource = "git+https://github.com/SuplenC/kotodama?rev=${inputs.kotodama.rev}#${inputs.kotodama.rev}";
  cargoOutputHashes = {
    ${kotodamaSource} = inputs.kotodama.narHash;
  };
in
(pkgs.callPackage ./workspace.nix {
  craneLib = inputs.crane.mkLib pkgs;
  inherit cargoOutputHashes;
}).buildCrate
