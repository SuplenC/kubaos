# `kubaos-proxy` (home/curated.nix): the Proxy's command and its session
# service, which take the work network up and down as one. home/proxy.nix
# installs it and its user unit.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-proxy" {
  pname = "kubaos-proxy";

  meta = {
    description = "KubaOS's Proxy: one switch for the work network, its WireGuard tunnel, DNS, HTTP proxy and company CA";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-proxy";
    platforms = lib.platforms.linux;
  };
}
