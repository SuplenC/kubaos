# `kubaos-power` (home/curated.nix): the power stops' session service, the
# command the slider and the menu move them with, and `kubaos power status` and
# `measure`. home/power.nix installs it and its user unit.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-power" {
  pname = "kubaos-power";

  meta = {
    description = "KubaOS's power stops, from Super Saver to Max Performance, the charge limit and the low-battery drop";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-power";
    platforms = lib.platforms.linux;
  };
}
