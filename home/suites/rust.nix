# The Rust Suite (crates/kubaos-core/src/suites/catalog.rs, RUST): rustup and
# the cargo tools, ~/.cargo/config.toml and LazyVim's Rust extra. Does nothing
# without kubaos.d/suites/rust.toml.
#
# rustup owns the toolchains, in ~/.rustup, outside Nix: a project's
# rust-toolchain.toml picks its own, which rustup installs on first use, so
# the Suite has no versions and no Project pins. The switch installs no
# toolchain and needs no network: it only gives rustup a default (stable,
# with the default profile, which carries clippy and rustfmt) when rustup has
# no settings yet, and the first cargo or rustc downloads it.
#
# - rustup from nixpkgs, whose proxies (cargo, rustc, rustfmt, cargo-clippy)
#   are on PATH. nixpkgs' rustup points the toolchains it downloads at Nix's
#   glibc (patchelf), so rustc runs on Nix's glibc while what cargo links
#   uses the system's.
# - rust-analyzer: a wrapper around rustup's proxy that adds the component to
#   the active toolchain the first time, since no rustup profile has it.
# - bacon, cargo-nextest, sccache, and wild beside mold.
# - kubaos-rust-workspace, the workspace scaffolder.
# - ~/.cargo/config.toml: sccache as the rustc wrapper, clang (apt) as the
#   linker driver, with mold (apt), or wild with the `wild` extra.
# - ~/.cargo/bin on PATH, for what `cargo install` puts there.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.rust or null;
  extras = if suite == null then [ ] else suite.extras;
  rustup = pkgs.rustup;

  # wild-unwrapped is the bare binary; nixpkgs' `wild` is a bintools wrapper
  # for Nix's stdenv, not for apt's clang.
  linkArg =
    if lib.elem "wild" extras then "--ld-path=${lib.getExe pkgs.wild-unwrapped}" else "-fuse-ld=mold";
  linuxTarget = {
    linker = "clang";
    rustflags = [
      "-C"
      "link-arg=${linkArg}"
    ];
  };
  cargoConfig = (pkgs.formats.toml { }).generate "cargo-config.toml" {
    # Absolute path so builds started outside a login shell (IDEs) still find it.
    build.rustc-wrapper = lib.getExe pkgs.sccache;
    target.x86_64-unknown-linux-gnu = linuxTarget;
    target.aarch64-unknown-linux-gnu = linuxTarget;
  };

  # rustup's own format, as `rustup default stable` would write it, less the
  # download. The first cargo or rustc installs the toolchain (rustup's
  # auto-install).
  rustupSettings = pkgs.writeText "rustup-settings.toml" ''
    version = "12"
    default_toolchain = "stable"
    profile = "default"
  '';

  # HAZARD: hiPrio, since rustup's proxy of the same name is in the same
  # profile. Without it the profile doesn't build.
  rustAnalyzer = lib.hiPrio (
    pkgs.writeShellScriptBin "rust-analyzer" ''
      # Its output is the language server's protocol: rustup's messages go to stderr.
      if ! ${rustup}/bin/rustup which rust-analyzer >/dev/null 2>&1; then
        ${rustup}/bin/rustup component add rust-analyzer >&2
      fi
      exec ${rustup}/bin/rust-analyzer "$@"
    ''
  );

  newWorkspace = pkgs.writeShellApplication {
    name = "kubaos-rust-workspace";
    runtimeInputs = [ rustup ];
    text = ''
      # kubaos-rust-workspace PATH NAME: a cargo workspace at PATH, under git,
      # whose members are crates/*, with a first library crate, crates/NAME.
      if (($# != 2)); then
        echo "usage: kubaos-rust-workspace PATH NAME" >&2
        exit 2
      fi
      path=$1 name=$2
      mkdir -- "$path"
      cd -- "$path"
      git init --quiet
      printf '/target\n' >.gitignore
      printf '[workspace]\nresolver = "3"\nmembers = ["crates/*"]\n' >Cargo.toml
      cargo new --quiet --vcs none --lib "crates/$name"
    '';
  };
in
{
  config = lib.mkIf (suite != null) {
    home.packages = [
      rustup
      rustAnalyzer
      newWorkspace
      pkgs.bacon
      pkgs.cargo-nextest
      pkgs.sccache
      # On PATH even with mold, for per-project trials and benchmarks (5 MB).
      pkgs.wild-unwrapped
    ];

    # sccache's own default, small enough for any disk. A machine with room to
    # spare sets more in the user flake: home.sessionVariables.SCCACHE_CACHE_SIZE.
    home.sessionVariables.SCCACHE_CACHE_SIZE = lib.mkDefault "10G";
    home.sessionPath = [ "${config.home.homeDirectory}/.cargo/bin" ];
    home.file.".cargo/config.toml".source = cargoConfig;

    # Only when rustup has no settings at all: a default the user chose, or
    # `rustup default none`, stays theirs.
    home.activation.kubaosRustupDefault = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
      rustupHome="''${RUSTUP_HOME:-$HOME/.rustup}"
      if [[ ! -e $rustupHome/settings.toml ]]; then
        run install -D -m 0644 ${rustupSettings} "$rustupHome/settings.toml"
      fi
    '';

    kubaos.nvim.suiteSpecs = [ ''{ import = "lazyvim.plugins.extras.lang.rust" }'' ];
  };
}
