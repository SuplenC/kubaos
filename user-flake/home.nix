# Your own home-manager settings, on top of KubaOS's. pkgs is your nixpkgs
# (flake.nix). Apply with
#   home-manager switch --flake ~/.config/kubaos/home
{ pkgs, ... }:
{
  home.packages = [
    # pkgs.hello
  ];

  # What `kubaos package add`, the Command menu and Kōbō install (kubaos.d/).
  kubaos.stateDir = ./kubaos.d;
}
