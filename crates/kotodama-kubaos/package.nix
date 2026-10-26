# The Command menu's KubaOS Extension (home/curated.nix), laid out as Kotodama
# finds an Extension: share/kotodama/extensions/kubaos/, with manifest.toml and
# the binary it names, and the "Add a keyboard layout" app beside it in
# share/kotodama/extensions/kubaos-layouts/. home/menu.nix lists it in
# `programs.kotodama.extraExtensions`.
{
  lib,
  stdenv,
  buildCrate,
}:

buildCrate "kotodama-kubaos" {
  pname = "kotodama-kubaos";

  postInstall = ''
    dir=$out/share/kotodama/extensions/kubaos
    install -Dm644 crates/kotodama-kubaos/manifest.toml $dir/manifest.toml
    mv $out/bin/kotodama-kubaos $dir/
    rmdir $out/bin
    # The "Add a keyboard layout" app: the same binary, run with `layouts`.
    layouts=$out/share/kotodama/extensions/kubaos-layouts
    install -Dm644 crates/kotodama-kubaos/layouts/manifest.toml $layouts/manifest.toml
    ln -s ../kubaos/kotodama-kubaos $layouts/kotodama-kubaos
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
