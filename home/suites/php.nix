# The PHP Suite: PHP at the chosen version with Xdebug, composer, phpactor,
# php-cs-fixer, phpcs, Xdebug's debug adapter and LazyVim's PHP extra
# (php.lua). Nothing until kubaos.d/suites/php.toml exists; the metadata is
# the `php` entry in crates/kubaos-core/src/suites/catalog.rs.
#
# Xdebug is loaded in step-debug mode and starts only when asked: from
# Neovim's debugger, or with XDEBUG_TRIGGER=1 for a run while it listens.
# composer's cache is ~/.cache/composer, the one folder the catalog lists;
# its home (config, auth.json) stays where composer keeps it.
#
# `kubaos-php-new DIR` is the plain composer scaffolder; Laravel's is
# composer's own `create-project`. Every PHP 8 release is a toolchain for
# Project pins, with its composer, installed or not.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  suite = config.kubaos.suites.php or null;

  # The catalog's versions, newest (the default) first: every php8N this
  # nixpkgs builds.
  versions = [
    "8.5"
    "8.4"
    "8.3"
    "8.2"
  ];
  version = if suite == null || suite.version == null then lib.head versions else suite.version;

  phpAt =
    version:
    pkgs."php${lib.replaceStrings [ "." ] [ "" ] version}".buildEnv {
      extensions = { enabled, all }: enabled ++ [ all.xdebug ];
      extraConfig = ''
        xdebug.mode = debug
        xdebug.start_with_request = trigger
      '';
    };
  # PHP and the composer that runs on it, so a pinned folder's composer
  # resolves dependencies for the pinned PHP.
  toolchainAt =
    version:
    let
      php = phpAt version;
    in
    pkgs.symlinkJoin {
      name = "php-${version}";
      paths = [
        php
        php.packages.composer
      ];
    };
  php = phpAt version;

  # LazyVim's PHP extra starts Mason's `php-debug-adapter`, which is Xdebug's
  # VS Code adapter run on Node; nixpkgs has the extension.
  phpDebugAdapter = pkgs.writeShellScriptBin "php-debug-adapter" ''
    exec ${lib.getExe pkgs.nodejs} ${pkgs.vscode-extensions.xdebug.php-debug}/share/vscode/extensions/xdebug.php-debug/out/phpDebug.js "$@"
  '';

  newProject = pkgs.writeShellApplication {
    name = "kubaos-php-new";
    runtimeInputs = [
      php
      php.packages.composer
      pkgs.coreutils
    ];
    text = ''
      # kubaos-php-new DIR: a composer project in DIR, which must not exist:
      # composer.json with PSR-4 autoloading of src/, and index.php, which
      # loads it. `php index.php` runs it.
      if (($# != 1)); then
        echo "usage: kubaos-php-new DIR" >&2
        exit 2
      fi
      dir=$1
      # composer's package names are vendor/name, lowercase.
      slug() { tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9_.\n-' '-'; }
      vendor=$(id -un | slug)
      name=$(basename -- "$dir" | slug)
      mkdir -- "$dir"
      cd -- "$dir"
      composer init --no-interaction --name "$vendor/$name" --type project --autoload src/
      cat >index.php <<'EOF'
      <?php

      declare(strict_types=1);

      require __DIR__ . '/vendor/autoload.php';

      echo "Hello from PHP\n";
      EOF
    '';
  };
in
{
  config = lib.mkMerge [
    { kubaos.suiteToolchains.php = lib.genAttrs versions toolchainAt; }
    (lib.mkIf (suite != null) {
      home.packages = [
        (toolchainAt version)
        pkgs.phpactor
        php.packages.php-cs-fixer
        php.packages.php-codesniffer
        phpDebugAdapter
        newProject
      ];
      home.sessionVariables.COMPOSER_CACHE_DIR = "${config.home.homeDirectory}/.cache/composer";
      kubaos.nvim.suiteSpecs = [ ''dofile("${./php.lua}")'' ];
    })
  ];
}
