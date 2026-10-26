# The `kubaos` command (home/curated.nix), built from the release it ships
# with. home/updates.nix installs it in every home, so each switch puts the
# release's `kubaos` on the user's PATH, and /usr/bin/kubaos runs it
# (install/firstboot/kubaos-launcher).
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-cli" {
  pname = "kubaos";

  meta = {
    description = "The kubaos command: packages, Suites, switches, KubaOS updates and the pointer";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos";
    platforms = lib.platforms.linux;
  };
}
