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

    # The wallpaper sets, one folder each (SuplenC/kubaos-wallpapers), pinned
    # to a commit. system/60-nix.sh links /usr/share/kubaos/wallpapers to it,
    # so a new set reaches machines with a KubaOS release: bump the rev, then
    # `nix flake update kubaos-wallpapers`.
    kubaos-wallpapers = {
      url = "github:SuplenC/kubaos-wallpapers/8a8465973564bfefd5abbfe82339cd645464ce92";
      flake = false;
    };

    # The Command menu (SuplenC/kotodama), following its main branch while
    # the menu is being polished. flake.lock holds the commit, and
    # `nix flake update kotodama` moves it to main's newest. It builds from
    # this flake's nixpkgs, so its QML module is built against the Qt that the
    # curated Quickshell loads it into.
    kotodama = {
      url = "github:SuplenC/kotodama/main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.home-manager.follows = "home-manager";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      home-manager,
      ...
    }@inputs:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      # The curated set (home/curated.nix), from this flake's nixpkgs.
      packages = forAllSystems (pkgs: import ./home/curated.nix { inherit pkgs inputs; });

      # KubaOS's home, with Kotodama's module (`programs.kotodama`) for the
      # Command menu (home/menu.nix).
      homeModules.default = {
        imports = [
          ./home
          inputs.kotodama.homeModules.default
        ];
      };

      # Store path of the wallpaper sets, for `nix eval --raw .#lib.wallpapers`
      # (system/lib.sh, link_wallpapers).
      lib.wallpapers = "${inputs.kubaos-wallpapers}";

      # A KubaOS home for `username`. pkgs is the caller's nixpkgs, for every
      # ordinary package; the curated set and home-manager come from this
      # flake's lock. The user flake (user-flake/flake.nix.in) calls this.
      lib.mkHome =
        {
          pkgs,
          username,
          modules ? [ ],
        }:
        let
          system = pkgs.stdenv.hostPlatform.system;
        in
        home-manager.lib.homeManagerConfiguration {
          inherit pkgs;
          extraSpecialArgs = {
            inherit username;
            # The packages, plus the package set itself for the GPU driver
            # bundle (targets.genericLinux.gpu.packages, home/desktop.nix).
            kubaosPkgs = self.packages.${system} // {
              nixpkgs = nixpkgs.legacyPackages.${system};
            };
          };
          modules = [ self.homeModules.default ] ++ modules;
        };

      # This checkout's own homes, for the payload build's user and the test
      # harness (docs/TESTING.md). Installed machines switch their user flake
      # (~/.config/kubaos/home) instead.
      homeConfigurations =
        let
          home =
            system:
            self.lib.mkHome {
              pkgs = nixpkgs.legacyPackages.${system};
              username = "kuba";
            };
        in
        {
          # The laptop (TUXEDO InfinityBook Pro 14 Gen10).
          "kuba@tuxedo" = home "x86_64-linux";
          # UTM VM on an Apple Silicon Mac, for testing.
          "kuba@vm" = home "aarch64-linux";
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
