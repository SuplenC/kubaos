# `kubaos-automount` (home/curated.nix): the automount service for removable
# drives, the eject command and the photo import. home/automount.nix runs it.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-automount" {
  pname = "kubaos-automount";

  meta = {
    description = "KubaOS's automount for removable drives";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-automount";
    platforms = lib.platforms.linux;
  };
}
