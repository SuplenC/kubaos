# The `kubaos` command (home/curated.nix), built from the release it ships
# with. home/updates.nix installs it in every home, so each switch puts the
# release's `kubaos` on the user's PATH, and /usr/bin/kubaos runs it
# (install/firstboot/kubaos-launcher). Its second binary, `kubaos-apps`, is
# the Apps catalog's root helper: root links it from this store path into
# /usr/local/libexec/ (install_apps_helper, system/lib.sh), where pkexec and
# sudo run it. Run from the user's PATH, it has no more rights than they do.
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
