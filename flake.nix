{
  description = "KubaOS: Kuba's own Linux. Home-manager flake for the Ubuntu + Hyprland + Quickshell desktop";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

    home-manager = {
      url = "github:nix-community/home-manager/release-26.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Pinned to a release tag. Bump the ref on purpose, then `nix flake update quickshell`.
    quickshell = {
      url = "git+https://git.outfoxxed.me/quickshell/quickshell?ref=refs/tags/v0.3.2";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # The live wallpaper engine (docs/UKIYO.md). To build against a local
    # checkout instead: --override-input ukiyo git+file://$HOME/Projects/ukiyo
    # (the Docker builds bind-mount it, see test/lib/local-inputs.sh).
    ukiyo = {
      url = "github:SuplenC/ukiyo/main";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      home-manager,
      ...
    }@inputs:
    let
      # The account the generations are for. The installer rewrites this file
      # in the installed checkout when the user picks another name
      # (docs/INSTALLER.md, "Another username").
      username = nixpkgs.lib.trim (builtins.readFile ./home/username);

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      mkHome =
        system:
        home-manager.lib.homeManagerConfiguration {
          pkgs = nixpkgs.legacyPackages.${system};
          extraSpecialArgs = { inherit inputs username; };
          modules = [ ./home ];
        };
    in
    {
      homeConfigurations = {
        # The laptop (TUXEDO InfinityBook Pro 14 Gen10).
        "${username}@tuxedo" = mkHome "x86_64-linux";
        # UTM VM on an Apple Silicon Mac, for testing.
        "${username}@vm" = mkHome "aarch64-linux";
      };

      formatter = forAllSystems (pkgs: pkgs.nixfmt-tree);

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.nixfmt
            pkgs.nixfmt-tree
            home-manager.packages.${pkgs.stdenv.hostPlatform.system}.default
          ];
        };
      });
    };
}
