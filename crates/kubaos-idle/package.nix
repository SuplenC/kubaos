# `kubaos-idle` (home/curated.nix): writes hypridle's listeners from the idle
# settings, holds the lid and Keep awake inhibitors, and is the command the
# menu changes the idle settings with. home/idle.nix installs it.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-idle" {
  pname = "kubaos-idle";

  meta = {
    description = "KubaOS's idle settings: lock, screens off, suspend, the lid and Keep awake";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-idle";
    platforms = lib.platforms.linux;
  };
}
