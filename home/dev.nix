# Rust helpers and PATH for tools installed outside Nix (rustup, proto, bun via proto).
# clang and mold come from apt; wild comes from nixpkgs.
{ lib, pkgs, ... }:
let
  # Linker cargo uses through clang: "mold" (apt, the default) or "wild"
  # (faster relinks, pre-1.0). Change it here, then home-manager switch.
  # README.md, "Rust", has a per-project way to try wild first.
  rustLinker = "mold";

  # wild-unwrapped is the bare binary; nixpkgs' `wild` is a bintools wrapper
  # for Nix's stdenv, not for apt's clang.
  linkArg =
    {
      mold = "-fuse-ld=mold";
      wild = "--ld-path=${lib.getExe pkgs.wild-unwrapped}";
    }
    .${rustLinker};

  linuxTarget = {
    linker = "clang";
    rustflags = [
      "-C"
      "link-arg=${linkArg}"
    ];
  };
in
{
  home.packages = with pkgs; [
    bacon
    cargo-nextest
    hyperfine
    sccache
    # On PATH even with mold selected (5 MB), for per-project trials and benchmarks.
    wild-unwrapped
  ];

  home.sessionVariables.SCCACHE_CACHE_SIZE = "50G";

  home.sessionPath = [
    "$HOME/.cargo/bin"
    "$HOME/.proto/shims"
    "$HOME/.proto/bin"
    "$HOME/.local/bin"
  ];

  home.file.".cargo/config.toml".source = (pkgs.formats.toml { }).generate "cargo-config.toml" {
    # Absolute path so builds started outside a login shell (IDEs) still find it.
    build.rustc-wrapper = lib.getExe pkgs.sccache;
    target.x86_64-unknown-linux-gnu = linuxTarget;
    target.aarch64-unknown-linux-gnu = linuxTarget;
  };
}
