# Kōbō (home/curated.nix), built from the release it ships with, for setup
# mode on an installed machine. home/setup.nix installs it in every home, and
# /usr/bin/kubaos-setup runs that one (install/firstboot/kubaos-setup), so a
# new setup card reaches the machine with the update that carries it. The
# live ISO has its own static `kobo`, built by install/iso/build.
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

buildCrate "kubaos-kobo" {
  pname = "kobo";

  # winit and wgpu open the windowing and GPU libraries with dlopen, so the
  # linker never puts them in the runpath. Nix's vulkan-loader and libGL find
  # the machine's drivers in /run/opengl-driver (home/desktop.nix). Fonts and
  # images are compiled into the binary.
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
    } $out/bin/kobo
  '';

  meta = {
    description = "Kōbō, KubaOS's setup app: the first-boot cards for drivers, fingerprint, YubiKey, Bitwarden, git and the phone";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kobo";
    platforms = lib.platforms.linux;
  };
}
