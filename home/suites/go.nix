# The Go Suite (crates/kubaos-core/src/suites/catalog.rs, GO): Go from the
# user's nixpkgs at the chosen version, its tools, and LazyVim's Go extra.
# Installs nothing without kubaos.d/suites/go.toml; its toolchains for Project
# pins are declared in every home.
#
# - go, at the version kubaos.d/suites/go.toml names, with GOTOOLCHAIN=local:
#   a go.mod asking for a newer Go fails with a message instead of go
#   downloading one past Nix, and a Project pin is the way to another version.
# - gopls, gofumpt, goimports (gotools), golangci-lint and delve (dlv), from
#   nixpkgs, which LazyVim's Go extra finds on PATH instead of installing its
#   own copies through Mason (go.lua).
# - ~/go/bin on PATH, for what `go install` puts there.
# - kubaos-go-new, the scaffolder.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.go or null;
  # The catalog's versions, the default first.
  versions = [
    "1.26"
    "1.27"
    "1.25"
  ];
  attrOf = version: "go_" + lib.replaceStrings [ "." ] [ "_" ] version;
  # A version the user's nixpkgs has dropped (gone, or an alias that throws
  # once it reaches end of life) has no toolchain rather than failing the
  # home.
  available =
    version: pkgs ? ${attrOf version} && (builtins.tryEval pkgs.${attrOf version}.drvPath).success;
  toolchains = lib.genAttrs (lib.filter available versions) (version: pkgs.${attrOf version});

  version = if suite == null || suite.version == null then lib.head versions else suite.version;
  go =
    toolchains.${version}
      or (throw "kubaos.d/suites/go.toml: your nixpkgs has no Go ${version}; choose another with `kubaos suite configure go --version`");

  # gopls, golangci-lint, gofumpt and goimports read code with the go/parser
  # and go/types of the Go they were built with, and refuse or misread code
  # for a newer one. nixpkgs builds them with its default Go, so a newer
  # chosen version builds them again with it: about a minute, once. delve
  # stays as it is: the Go versions it debugs are its own release's.
  newerThanNixpkgs = lib.versionOlder (lib.versions.majorMinor pkgs.go.version) version;
  builder = pkgs."buildGo${lib.replaceStrings [ "." ] [ "" ] version}Module";
  withChosenGo =
    tool:
    if newerThanNixpkgs then
      tool.override (
        lib.mapAttrs (_: _: builder) (
          lib.filterAttrs (name: _: lib.hasPrefix "buildGo" name) (lib.functionArgs tool.override)
        )
      )
    else
      tool;

  newProject = pkgs.writeShellApplication {
    name = "kubaos-go-new";
    runtimeInputs = [ go ];
    text = ''
      # kubaos-go-new PATH NAME: a Go module called NAME at PATH, under git,
      # with a main package.
      if (($# != 2)); then
        echo "usage: kubaos-go-new PATH NAME" >&2
        exit 2
      fi
      path=$1 name=$2
      mkdir -- "$path"
      cd -- "$path"
      git init --quiet
      GOTOOLCHAIN=local go mod init "$name"
      cat >main.go <<EOF
      package main

      import "fmt"

      func main() {
      	fmt.Println("Hello from $name")
      }
      EOF
    '';
  };

in
{
  config = lib.mkMerge [
    { kubaos.suiteToolchains.go = toolchains; }

    (lib.mkIf (suite != null) {
      home.packages = [
        go
        (withChosenGo pkgs.gopls)
        (withChosenGo pkgs.gofumpt)
        (withChosenGo pkgs.gotools)
        (withChosenGo pkgs.golangci-lint)
        pkgs.delve
        newProject
      ];
      home.sessionVariables.GOTOOLCHAIN = "local";
      home.sessionPath = [ "${config.home.homeDirectory}/go/bin" ];

      kubaos.nvim.suiteSpecs = [ ''dofile("${./go.lua}")'' ];
    })
  ];
}
