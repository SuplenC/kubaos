# `kubaos-bluetooth` (home/curated.nix): the Bluetooth pairing agent, and the
# command the shell pairs and scans through. home/bluetooth.nix runs it.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-bluetooth" {
  pname = "kubaos-bluetooth";

  meta = {
    description = "KubaOS's Bluetooth pairing agent";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-bluetooth";
    platforms = lib.platforms.linux;
  };
}
