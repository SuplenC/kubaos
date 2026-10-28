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
      url = "github:SuplenC/kubaos-wallpapers/a817336e531f443ce4f2d2a9fbd5a0122aacce76";
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
      inputs.crane.follows = "crane";
    };

    # Builds KubaOS's crates (crates/workspace.nix), and Kotodama's.
    crane.url = "github:ipetkov/crane";
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
      # rustfmt, built in nightly mode, so it reads rustfmt.toml's unstable
      # options; a stable rustfmt skips them with a warning and formats the rest
      # with the defaults. The same release as the devshell's rustc, so no
      # second toolchain is fetched.
      rustfmtNightly = pkgs: pkgs.rustfmt.override { asNightly = true; };
    in
    {
      # The curated set (home/curated.nix), from this flake's nixpkgs, and
      # `cache`.
      #
      # `cache` links every output of the curated set, for the binary cache
      # workflow (.github/workflows/cache.yml) to build and push. They are the
      # very derivations a machine's home builds: lib.mkHome takes the curated
      # set from this flake's lock, whatever the user's nixpkgs, so a pushed
      # path is the one a machine asks for. Cachix skips what cache.nixos.org
      # has, which leaves KubaOS's own builds. The debug outputs stay out: no
      # machine installs them.
      packages = forAllSystems (
        pkgs:
        let
          curated = import ./home/curated.nix { inherit pkgs inputs; };
          outputsOf =
            name: package:
            map (output: {
              name = "${name}-${output}";
              path = package.${output};
            }) (pkgs.lib.remove "debug" (package.outputs or [ "out" ]));
        in
        curated
        // {
          cache = pkgs.linkFarm "kubaos-cache" (
            pkgs.lib.concatLists (pkgs.lib.mapAttrsToList outputsOf curated)
          );
        }
      );

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
            kubaosPkgs = removeAttrs self.packages.${system} [ "cache" ] // {
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

      # The design checks (crates/checks.nix): the lint, the specimen's
      # geometry and QML's parity with it, run by `nix flake check`.
      checks = forAllSystems (
        pkgs:
        import ./crates/checks.nix {
          inherit pkgs inputs;
          src = self;
          inherit (self.packages.${pkgs.stdenv.hostPlatform.system}) kubaos-specimen kubaos-components;
        }
      );

      # `just shots` (test/shots/shots): before/after screenshots of the
      # shell, Kōbō and the specimen for a design PR, with Qt, sway, Mesa and
      # the fonts pinned here.
      apps = forAllSystems (pkgs: {
        shots = {
          type = "app";
          program = pkgs.lib.getExe (
            pkgs.callPackage ./test/shots/package.nix { wallpapers = inputs.kubaos-wallpapers; }
          );
          meta.description = "Before/after screenshots of KubaOS's shell, Kōbō and the specimen, diffed";
        };
      });

      # `nix fmt`: nixfmt for the Nix files, and the Rust files as `just fmt`
      # formats them. The patched dependencies under vendor/ keep upstream's
      # layout.
      formatter = forAllSystems (
        pkgs:
        let
          rustfmt = pkgs.writeShellApplication {
            name = "kubaos-rustfmt";
            runtimeInputs = [ (rustfmtNightly pkgs) ];
            text = ''
              rustfmt --edition 2024 "$@"
              ${./scripts/self-imports} "$@"
              rustfmt --edition 2024 "$@"
            '';
          };
        in
        pkgs.treefmt.withConfig {
          runtimeInputs = [
            pkgs.nixfmt
            rustfmt
          ];
          settings = {
            on-unmatched = "info";
            formatter.nixfmt = {
              command = "nixfmt";
              includes = [ "*.nix" ];
            };
            formatter.rustfmt = {
              command = "kubaos-rustfmt";
              includes = [ "*.rs" ];
              excludes = [ "vendor/*" ];
            };
          };
        }
      );

      # The toolchain `just fmt`, `just lint` and `just check` run on, here and
      # in CI (.github/workflows/check.yml): rustc, cargo, clippy and nextest
      # from this flake's nixpkgs, so flake.lock pins them.
      #
      # rustfmt is the nightly one (rustfmtNightly). RUSTFMT points `cargo fmt`
      # and rust-analyzer at it, and it comes first on PATH for an editor that
      # runs `rustfmt` itself; editors get the shell through direnv (.envrc).
      # `just fmt` and `just fmt-check` refuse a rustfmt that isn't nightly.
      #
      # Cargo compiles through sccache, its cache under ~/.cache/sccache, so a
      # fresh worktree reuses the crates.io dependencies another checkout
      # already built. Only the shell's cargo: crane and the flake's packages
      # build in the sandbox, without it.
      devShells = forAllSystems (
        pkgs:
        let
          rustfmt-nightly = rustfmtNightly pkgs;
        in
        {
          default = pkgs.mkShell {
            packages = [
              rustfmt-nightly
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.cargo-nextest
              pkgs.just
              pkgs.actionlint
              pkgs.nixfmt
              self.formatter.${pkgs.stdenv.hostPlatform.system}
              pkgs.sccache
              home-manager.packages.${pkgs.stdenv.hostPlatform.system}.default
            ];
            RUSTFMT = pkgs.lib.getExe' rustfmt-nightly "rustfmt";
            RUSTC_WRAPPER = pkgs.lib.getExe pkgs.sccache;
            shellHook = ''
              export SCCACHE_DIR=''${SCCACHE_DIR:-''${XDG_CACHE_HOME:-$HOME/.cache}/sccache}
            '';
          };
        }
      );
    };
}
