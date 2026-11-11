# The Node Suite (crates/kubaos-core/src/suites/catalog.rs, NODE): everything
# of it but Node, Bun and vtsls, which mise installs from the Suite's fragment,
# ~/.config/mise/conf.d/kubaos-node.toml, and puts on PATH (shell.nix).
# Installs nothing without kubaos.d/suites/node.toml.
#
# - LazyVim's TypeScript and eslint extras (node.lua), on the Suite's vtsls,
#   eslint's server (home/cli.nix, everyone's) and the JavaScript debugger
#   instead of Mason's copies.
# - The JavaScript debugger, js-debug, from nixpkgs: it ships as a VS Code
#   extension and a tarball of JavaScript, no program mise can install.
# - Where global installs land, outside the folder of the Node or Bun mise
#   activates, which the next patch update would prune: npm's in
#   ~/.local/share/npm and Bun's in ~/.local/share/bun, both on PATH. npm's
#   is `prefix` in a global npmrc (NPM_CONFIG_GLOBALCONFIG), so the user's
#   ~/.npmrc and a project's .npmrc win over it; Bun's is BUN_INSTALL, which
#   `[install]` in the user's bunfig wins over. Bun's download cache is in
#   BUN_INSTALL too.
# - kubaos-node-new, the scaffolder, which runs the node and npm on PATH.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.node or null;
  home = config.home.homeDirectory;
  npmGlobals = "${home}/.local/share/npm";
  bunGlobals = "${home}/.local/share/bun";

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
    text = ''
      # kubaos-node-new DIR: a Node + TypeScript app in DIR, which must not
      # exist: src/index.ts, `npm start` runs it, `npm run check` type-checks.
      if (($# != 1)); then
        echo "usage: kubaos-node-new DIR" >&2
        exit 2
      fi
      for tool in node npm; do
        if ! command -v "$tool" >/dev/null; then
          echo "kubaos-node-new: no $tool on PATH; \`kubaos suite configure node\` installs it again" >&2
          exit 1
        fi
      done
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
    (lib.mkIf (suite != null) {
      home.packages = [
        jsDebugAdapter
        newProject
      ];
      xdg.configFile."npm/npmrc".text = ''
        prefix=${npmGlobals}
      '';
      home.sessionVariables.NPM_CONFIG_GLOBALCONFIG = "${config.xdg.configHome}/npm/npmrc";
      home.sessionPath = [ "${npmGlobals}/bin" ];
      kubaos.nvim.suiteSpecs = [ ''dofile("${./node.lua}")'' ];
    })
    (lib.mkIf (suite != null && lib.elem "bun" suite.extras) {
      home.sessionVariables.BUN_INSTALL = bunGlobals;
      home.sessionPath = [ "${bunGlobals}/bin" ];
    })
  ];
}
