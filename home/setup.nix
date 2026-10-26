# Kōbō, for setup mode any time. /usr/bin/kubaos-setup and the first-boot
# unit run this one (install/firstboot/kubaos-setup), so a new setup card
# reaches the machine with the update that carries it.
{ kubaosPkgs, ... }:
{
  home.packages = [ kubaosPkgs.kubaos-kobo ];
}
