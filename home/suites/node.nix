# The Node Suite: Node at the chosen version, Bun (the `bun` extra), the
# TypeScript language server (vtsls), eslint's language server, the
# JavaScript debugger, and LazyVim's TypeScript and eslint extras (node.lua).
# prettier is everyone's (home/cli.nix). Nothing
# until kubaos.d/suites/node.toml exists; the metadata is the `node` entry in
# crates/kubaos-core/src/suites/catalog.rs.
#
# Global installs land in the home, not the read-only Nix store: npm's under
# ~/.local/share/npm, Bun's under ~/.local/share/bun, both on PATH. npm's
# cache stays at its own ~/.npm and Bun's is ~/.cache/bun. The catalog lists
# all four as the Suite's caches.
#
# `kubaos-node-new DIR` is the Node + TypeScript scaffolder. Every Node major
# is a toolchain for Project pins, installed or not.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.node or null;
  home = config.home.homeDirectory;

  # The catalog's versions, the default (the active LTS) first: every
  # nodejs_NN this nixpkgs builds. 20 is past its end of life, and nixpkgs
  # refuses it.
  versions = [
    "26"
    "24"
    "22"
  ];
  version = if suite == null || suite.version == null then lib.head versions else suite.version;
  nodeAt = version: pkgs."nodejs_${version}";
  node = nodeAt version;

  # LazyVim's TypeScript extra starts Mason's `js-debug-adapter`; nixpkgs'
  # build of the same server is `js-debug`.
  jsDebugAdapter = pkgs.writeShellScriptBin "js-debug-adapter" ''
    exec ${lib.getExe pkgs.vscode-js-debug} "$@"
  '';

  # Node strips TypeScript's types itself, so the app runs its .ts files with
  # plain `node`, and `tsc` only checks them. The tsconfig allows only the
  # syntax that stripping can run.
  # https://nodejs.org/api/typescript.html
  newProject = pkgs.writeShellApplication {
    name = "kubaos-node-new";
    runtimeInputs = [ node ];
    text = ''
      # kubaos-node-new DIR: a Node + TypeScript app in DIR, which must not
      # exist: src/index.ts, `npm start` runs it, `npm run check` type-checks.
      if (($# != 1)); then
        echo "usage: kubaos-node-new DIR" >&2
        exit 2
      fi
      dir=$1
      name=$(basename -- "$dir" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9._\n-' '-')
      mkdir -- "$dir"
      cd -- "$dir"
      mkdir src
      cat >package.json <<EOF
      {
        "name": "$name",
        "version": "0.1.0",
        "private": true,
        "type": "module",
        "scripts": {
          "start": "node src/index.ts",
          "dev": "node --watch src/index.ts",
          "check": "tsc"
        }
      }
      EOF
      cat >tsconfig.json <<'EOF'
      {
        "compilerOptions": {
          "target": "esnext",
          "module": "nodenext",
          "types": ["node"],
          "strict": true,
          "noEmit": true,
          "allowImportingTsExtensions": true,
          "verbatimModuleSyntax": true,
          "erasableSyntaxOnly": true,
          "skipLibCheck": true
        },
        "include": ["src"]
      }
      EOF
      cat >src/index.ts <<'EOF'
      const greeting: string = "Hello from Node";
      console.log(greeting);
      EOF
      printf 'node_modules/\n' >.gitignore
      major=$(node -p 'process.versions.node.split(".")[0]')
      npm install --save-dev --no-fund --no-audit --no-update-notifier typescript "@types/node@$major"
    '';
  };
in
{
  config = lib.mkMerge [
    { kubaos.suiteToolchains.node = lib.genAttrs versions nodeAt; }
    (lib.mkIf (suite != null) {
      home.packages = [
        node
        pkgs.vtsls
        pkgs.vscode-langservers-extracted
        jsDebugAdapter
        newProject
      ];
      home.sessionVariables.NPM_CONFIG_PREFIX = "${home}/.local/share/npm";
      home.sessionPath = [ "${home}/.local/share/npm/bin" ];
      kubaos.nvim.suiteSpecs = [ ''dofile("${./node.lua}")'' ];
    })
    (lib.mkIf (suite != null && lib.elem "bun" suite.extras) {
      home.packages = [ pkgs.bun ];
      home.sessionVariables = {
        BUN_INSTALL = "${home}/.local/share/bun";
        BUN_INSTALL_CACHE_DIR = "${home}/.cache/bun";
      };
      home.sessionPath = [ "${home}/.local/share/bun/bin" ];
    })
  ];
}
