# `kubaos-specimen` (home/curated.nix): the design specimen, every token and
# component of kubaos-ui in a window. home/desktop.nix installs it, and the
# Command menu's "Design specimen" Item opens it (config/kotodama/kubaos.toml).
{
  lib,
  buildCrate,
  libGL,
  libx11,
  libxcursor,
  libxi,
  libxkbcommon,
  libxrandr,
  vulkan-loader,
  wayland,
}:

buildCrate "kubaos-specimen" {
  pname = "kubaos-specimen";

  # winit and wgpu open the windowing and GPU libraries with dlopen, so the
  # linker never puts them in the runpath, as for Kōbō (crates/kubaos-kobo/
  # package.nix). Fonts are compiled into the binary.
  postFixup = ''
    patchelf --add-rpath ${
      lib.makeLibraryPath [
        libGL
        libx11
        libxcursor
        libxi
        libxkbcommon
        libxrandr
        vulkan-loader
        wayland
      ]
    } $out/bin/kubaos-specimen
  '';

  meta = {
    description = "KubaOS's design specimen: every design token and kubaos-ui component, live or as screenshots";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-specimen";
    platforms = lib.platforms.linux;
  };
}
