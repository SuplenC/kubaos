{
  pkgs,
  username,
  ...
}:
{
  imports = [
    ./links.nix
    ./cli.nix
    ./dev.nix
    ./desktop.nix
    ./menu.nix
    ./secrets.nix
    ./shell.nix
    ./state.nix
    ./suites.nix
    ./terminal.nix
    ./theme.nix
    ./updates.nix
  ];

  home = {
    inherit username;
    homeDirectory = "/home/${username}";
    stateVersion = "26.05";
  };

  programs.home-manager.enable = true;

  # Ubuntu, not NixOS: session vars, XDG_DATA_DIRS, GPU drivers (see desktop.nix).
  targets.genericLinux.enable = true;
  xdg.enable = true;
  fonts.fontconfig.enable = true;

  nix = {
    package = pkgs.nix;
    settings.experimental-features = [
      "nix-command"
      "flakes"
    ];
  };
}
