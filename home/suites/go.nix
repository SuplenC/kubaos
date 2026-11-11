# The Go Suite (crates/kubaos-core/src/suites/catalog.rs, GO): everything of it
# but Go and its tools, which mise installs from the Suite's fragment,
# ~/.config/mise/conf.d/kubaos-go.toml, and puts on PATH (shell.nix). Installs
# nothing without kubaos.d/suites/go.toml.
#
# - LazyVim's Go extra (go.lua), on the Suite's gopls, gofumpt, goimports,
#   golangci-lint and delve instead of Mason's copies.
# - ~/go/bin on PATH, for what `go install` builds.
# - A default for mise, kubaos-go_defaults.toml in its conf.d (a `_` no
#   Suite's name has, so it is never a Suite's fragment): mise leaves GOBIN
#   alone. By default mise points GOBIN at the bin of the Go it
#   activates, so what `go install` built would be pruned with that Go at the
#   next patch update; this way it goes to Go's own default, ~/go/bin. The
#   user's ~/.config/mise/config.toml loads after it and wins.
# - kubaos-go-new, the scaffolder, which runs the go on PATH.
#
# Go's own settings stay the user's: the Suite sets no GOPATH, GOBIN, GOFLAGS
# or GOTOOLCHAIN, so `go env -w` and the environment decide them.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.go or null;
  miseDefaults = (pkgs.formats.toml { }).generate "kubaos-go_defaults.toml" {
    settings.go_set_gobin = false;
  };

  newProject = pkgs.writeShellApplication {
    name = "kubaos-go-new";
    text = ''
      # kubaos-go-new PATH NAME: a Go module called NAME at PATH, under git,
      # with a main package, made by the go on PATH.
      if (($# != 2)); then
        echo "usage: kubaos-go-new PATH NAME" >&2
        exit 2
      fi
      if ! command -v go >/dev/null; then
        echo "kubaos-go-new: no go on PATH; \`kubaos suite configure go\` installs it again" >&2
        exit 1
      fi
      path=$1 name=$2
      mkdir -- "$path"
      cd -- "$path"
      git init --quiet
      go mod init "$name"
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
  config = lib.mkIf (suite != null) {
    home.packages = [ newProject ];
    home.sessionPath = [ "${config.home.homeDirectory}/go/bin" ];
    xdg.configFile."mise/conf.d/kubaos-go_defaults.toml".source = miseDefaults;

    kubaos.nvim.suiteSpecs = [ ''dofile("${./go.lua}")'' ];
  };
}
