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

  # The crate's second binary, `kubaos-apps`, is the Apps catalog's root
  # helper, which only root may install where pkexec runs it; the home
  # package carries the `kubaos` command alone, so the helper never lands on
  # the user's PATH.
  postInstall = ''
    rm -f "$out/bin/kubaos-apps"
  '';

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
