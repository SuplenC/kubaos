# `kubaos-launches` (home/curated.nix): the launch watcher that tells
# Hyprland which workspace each app was launched from. home/launches.nix runs it.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-launches" {
  pname = "kubaos-launches";

  meta = {
    description = "KubaOS's launch watcher, so apps open on the workspace they were launched from";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-launches";
    platforms = lib.platforms.linux;
  };
}
