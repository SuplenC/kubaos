# `kubaos-phone` (home/curated.nix): the phone link's command, which turns
# KDE Connect on and off and sends to the phone. home/phone.nix installs it
# and the user unit that runs kdeconnectd.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-phone" {
  pname = "kubaos-phone";

  meta = {
    description = "KubaOS's phone link through KDE Connect";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-phone";
    platforms = lib.platforms.linux;
  };
}
