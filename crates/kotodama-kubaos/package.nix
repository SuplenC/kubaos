# The Command menu's KubaOS Extension (home/curated.nix), laid out as Kotodama
# finds an Extension: share/kotodama/extensions/kubaos/, with manifest.toml and
# the binary it names. home/menu.nix lists it in `programs.kotodama.extraExtensions`.
{
  lib,
  stdenv,
  rustPlatform,
}:

let
  root = ../..;
in
rustPlatform.buildRustPackage {
  pname = "kotodama-kubaos";
  version = (lib.importTOML ../../Cargo.toml).workspace.package.version;

  # The workspace, and the first-boot files kubaos-core embeds (the keyboard
  # layouts come from it), so a change elsewhere doesn't rebuild it.
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
    "kotodama-kubaos"
  ];
  # Its tests read config/ and scripts/, outside src; `cargo test` runs them.
  doCheck = false;

  postInstall = ''
    dir=$out/share/kotodama/extensions/kubaos
    install -Dm644 crates/kotodama-kubaos/manifest.toml $dir/manifest.toml
    mv $out/bin/kotodama-kubaos $dir/
    rmdir $out/bin
  '';

  # A Rust binary needs only libgcc_s from GCC, but the linker leaves a
  # runpath to GCC's `lib` output, with libstdc++ in it too. Kotodama's
  # strict check (`strictExtensions`, home/menu.nix) refuses an Extension
  # that refers to it, so the runpath points at GCC's `libgcc` output, which
  # holds libgcc_s alone, as Kotodama does for its own Extensions.
  postFixup = ''
    bin=$out/share/kotodama/extensions/kubaos/kotodama-kubaos
    rpath=":$(patchelf --print-rpath $bin):"
    rpath=''${rpath//":${lib.getLib stdenv.cc.cc}/lib:"/":${stdenv.cc.cc.libgcc}/lib:"}
    rpath=''${rpath#:}
    patchelf --set-rpath "''${rpath%:}" $bin
    if grep -q ${lib.getLib stdenv.cc.cc} $bin; then
      echo "kotodama-kubaos still refers to ${lib.getLib stdenv.cc.cc}" >&2
      exit 1
    fi
  '';

  meta = {
    description = "The Command menu's KubaOS Extension: KubaOS's actions and the system toggles";
    license = with lib.licenses; [
      mit
      asl20
    ];
    platforms = lib.platforms.linux;
  };
}
