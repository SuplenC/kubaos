# The Rust Suite (crates/kubaos-core/src/suites/catalog.rs, RUST): everything
# of it but its tools, which mise installs from the Suite's fragment,
# ~/.config/mise/conf.d/kubaos-rust.toml: rustup, cargo-nextest, sccache,
# codelldb and, with the `wild` extra, wild. Does nothing without
# kubaos.d/suites/rust.toml.
#
# rustup owns the toolchains, in ~/.rustup, outside Nix and mise: a project's
# rust-toolchain.toml picks its own, which rustup installs on first use, so
# the Suite has no versions and no Project pins. mise brings rustup alone, not
# its own `rust`, which sets RUSTUP_TOOLCHAIN and so outranks that file. The
# switch installs no toolchain and needs no network.
#
# - rustup's proxies (cargo, rustc, rustfmt, cargo-clippy, …), on PATH through
#   ~/.nix-profile/bin as links to the rustup mise installed: rustup runs as
#   the tool it is called by. Its release ships rustup alone; its installer
#   would put the proxies and a second rustup in ~/.cargo/bin, outside mise.
# - rust-analyzer: a wrapper of rustup's proxy that adds the component to the active toolchain the
#   first time, since no rustup profile has it.
# - codelldb, the debug adapter: a link to the adapter in the VS Code extension
#   mise unpacks, which finds its own LLDB beside it.
# - bacon, from nixpkgs: its release is one archive for every platform, which
#   mise's release backends can't pick a binary from.
# - kubaos-rust-workspace, the workspace scaffolder, on the cargo on PATH.
# - rustup's settings, once, when it has none: stable with the default
#   profile (clippy and rustfmt), and no self-update, which only works for a
#   rustup in ~/.cargo/bin. The first cargo or rustc downloads stable.
# - cargo's defaults, ~/.cargo/kubaos.toml: sccache as the rustc wrapper, and
#   clang (apt) as the linker driver with mold (apt), or wild with the `wild`
#   extra. ~/.cargo/config.toml is the user's: the switch creates it once,
#   including kubaos.toml, and what it sets itself wins (cargo merges an
#   included file below the one that includes it). Cargo before 1.95 skips
#   the include, so a project pinned to one builds with cargo's own defaults.
# - SCCACHE_CACHE_SIZE, a default the user's own variable replaces.
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

  # HAZARD: link through mise's `latest` link, never a version's folder, which
  # the next `kubaos suite update` prunes. `latest` points at the tool's newest
  # installed version and follows every install and prune, which keep only the
  # fragment's version. Its `global` link would name the fragment's
  # version itself, but mise makes it only on a config-wide install, and
  # `kubaos suite add` installs before it writes the fragment. MISE_DATA_DIR is
  # mise's default here, as everywhere in the home (shell.nix).
  # https://mise.jdx.dev/ide-integration.html#global-version-link
  miseInstalls = "${config.xdg.dataHome}/mise/installs";
  rustup = "${miseInstalls}/aqua-rust-lang-rustup/latest/rustup";
  codelldb = "${miseInstalls}/github-vadimcn-codelldb/latest/extension/adapter/codelldb";

  # rustup's own proxy names, as its installer links them.
  # https://github.com/rust-lang/rustup/blob/master/src/cli/self_update.rs
  proxies = [
    "rustup"
    "cargo"
    "cargo-clippy"
    "cargo-fmt"
    "cargo-miri"
    "clippy-driver"
    "rls"
    "rust-gdb"
    "rust-gdbgui"
    "rust-lldb"
    "rustc"
    "rustdoc"
    "rustfmt"
  ];

  # Its output is the language server's protocol: rustup's messages go to stderr.
  rustAnalyzer = pkgs.writeShellScript "rust-analyzer" ''
    if ! ${rustup} which rust-analyzer >/dev/null 2>&1; then
      ${rustup} component add rust-analyzer >&2
    fi
    exec -a rust-analyzer ${rustup} "$@"
  '';

  rustupProxies = pkgs.runCommand "kubaos-rustup-proxies" { } ''
    mkdir -p $out/bin
    for proxy in ${lib.escapeShellArgs proxies}; do
      ln -s ${lib.escapeShellArg rustup} "$out/bin/$proxy"
    done
    ln -s ${rustAnalyzer} $out/bin/rust-analyzer
    ln -s ${lib.escapeShellArg codelldb} $out/bin/codelldb
  '';

  linkArg = if lib.elem "wild" extras then "--ld-path=wild" else "-fuse-ld=mold";
  linuxTarget = {
    linker = "clang";
    rustflags = [
      "-C"
      "link-arg=${linkArg}"
    ];
  };
  cargoDefaults = (pkgs.formats.toml { }).generate "kubaos.toml" {
    build.rustc-wrapper = "sccache";
    target.x86_64-unknown-linux-gnu = linuxTarget;
    target.aarch64-unknown-linux-gnu = linuxTarget;
  };

  # The string form of `include`: cargo before 1.95 refuses the table form,
  # and skips this one.
  cargoConfig = pkgs.writeText "cargo-config.toml" ''
    # Your cargo settings. KubaOS created this file with the Rust Suite and
    # never writes to it again. It includes the Suite's defaults, in
    # kubaos.toml beside it: sccache as the rustc wrapper, and clang with mold
    # (or wild) as the linker. What you set here wins over them; delete the
    # include to drop them all.
    # https://doc.rust-lang.org/cargo/reference/config.html
    include = ["kubaos.toml"]
  '';

  removedDefaults = pkgs.writeText "kubaos-rust-removed.toml" ''
    # The Rust Suite's cargo defaults, empty while the Suite is removed. Your
    # config.toml includes this file; adding the Suite again fills it.
  '';

  # rustup's own format, as `rustup default stable` would write it, less the
  # download. The first cargo or rustc installs the toolchain (rustup's
  # auto-install).
  rustupSettings = pkgs.writeText "rustup-settings.toml" ''
    version = "12"
    default_toolchain = "stable"
    profile = "default"
    auto_self_update = "disable"
  '';

  newWorkspace = pkgs.writeShellApplication {
    name = "kubaos-rust-workspace";
    text = ''
      # kubaos-rust-workspace PATH NAME: a cargo workspace at PATH, under git,
      # whose members are crates/*, with a first library crate, crates/NAME,
      # made by the cargo on PATH.
      if (($# != 2)); then
        echo "usage: kubaos-rust-workspace PATH NAME" >&2
        exit 2
      fi
      if ! command -v cargo >/dev/null; then
        echo "kubaos-rust-workspace: no cargo on PATH; \`kubaos suite configure rust\` installs rustup again" >&2
        exit 1
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
  config = lib.mkMerge [
    (lib.mkIf (suite != null) {
      home.packages = [
        rustupProxies
        newWorkspace
        pkgs.bacon
      ];

      # sccache's own default, small enough for any disk. A machine with room to
      # spare sets more in the user flake: home.sessionVariables.SCCACHE_CACHE_SIZE.
      home.sessionVariables.SCCACHE_CACHE_SIZE = lib.mkDefault "10G";
      home.sessionPath = [ "${config.home.homeDirectory}/.cargo/bin" ];
      # Replaces the empty file the switch leaves when the Suite is removed.
      home.file.".cargo/kubaos.toml" = {
        source = cargoDefaults;
        force = true;
      };

      # After linkGeneration, which takes away a ~/.cargo/config.toml an older
      # home linked. Only when missing: the user's settings stay theirs.
      home.activation.kubaosRustDefaults = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
        rustupHome="''${RUSTUP_HOME:-$HOME/.rustup}"
        if [[ ! -e $rustupHome/settings.toml ]]; then
          run install -D -m 0644 ${rustupSettings} "$rustupHome/settings.toml"
        fi
        if [[ ! -e $HOME/.cargo/config.toml && ! -L $HOME/.cargo/config.toml ]]; then
          run install -D -m 0644 ${cargoConfig} "$HOME/.cargo/config.toml"
        fi
      '';

      kubaos.nvim.suiteSpecs = [ ''dofile("${./rust.lua}")'' ];
    })
    # A config.toml that still includes kubaos.toml would fail every cargo
    # once the Suite's file is gone, so removal leaves an empty one.
    (lib.mkIf (suite == null) {
      home.activation.kubaosRustDefaults = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
        if [[ -f $HOME/.cargo/config.toml && ! -e $HOME/.cargo/kubaos.toml ]] \
          && grep -q 'kubaos\.toml' "$HOME/.cargo/config.toml"; then
          run install -m 0644 ${removedDefaults} "$HOME/.cargo/kubaos.toml"
        fi
      '';
    })
  ];
}
