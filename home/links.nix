# Defaults and the user layer (docs/CONTRACTS.md).
#
# config/<app>/ is KubaOS's defaults. Each file is linked read-only from the
# Nix store into ~/.config/<app>/, a real directory, so an update replaces the
# defaults whole and apps can still write their own files next to them. The
# user's changes live in ~/.config/kubaos/<group>/<app>/ (home/user-layer.nix),
# which the defaults load last and no update writes to. Editing config/ takes
# a home-manager switch.
#
# A few files can't be plain links (the `layer` table below); dot-config
# handles them: it seeds the ones the app rewrites, merges the ones with no
# include, and creates the user layer's missing files from user-layer/.
#
# The token outputs the repo doesn't keep (a terminal's tokens, the matugen
# templates written from the tokens, Hyprland's tokens.lua) come from their Nix
# build (design/package.nix) and are linked as if they were in config/.
#
# scripts/<file> is linked from the store to ~/.local/bin/<file> the same way.
# In a checkout, new files need `git add` before the next switch (flakes only
# see tracked files).
{
  config,
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  outOfStore = config.lib.file.mkOutOfStoreSymlink;
  userLayer = import ./user-layer.nix { configHome = config.xdg.configHome; };

  # Every seed folder, user-layer/<group>/<app>, as "<group>/<app>".
  seedFolders = lib.concatLists (
    lib.mapAttrsToList (
      group: type:
      lib.optionals (type == "directory") (
        map (app: "${group}/${app}") (
          lib.attrNames (lib.filterAttrs (_: t: t == "directory") (builtins.readDir ../user-layer/${group}))
        )
      )
    ) (builtins.readDir ../user-layer)
  );

  shipped = builtins.path {
    path = ../config;
    name = "kubaos-config";
  };

  # The token outputs built into config/, by their paths under it: .gitignore
  # lists them, and kubaos-tokens' tests hold that list to the generator's.
  built = map (lib.removePrefix "/config/") (
    lib.filter (lib.hasPrefix "/config/") (lib.splitString "\n" (builtins.readFile ../.gitignore))
  );

  # tms's `search_dirs`, from the Command menu's folders, so both find the
  # same projects. A folder without a depth, such as the Scripts root, is
  # searched one level down.
  menuFolders =
    (builtins.fromTOML (builtins.readFile ../config/kotodama/kubaos.toml)).extensions.folders.folders;
  tmsSearchDirs = lib.concatMapStrings (folder: ''
    [[search_dirs]]
    path = "${folder.path}"
    depth = ${toString (folder.depth or 1)}

  '') menuFolders;

  # config/ as shipped with the token outputs built into it, but for
  # tms/config.toml, which gains the search_dirs.
  defaults = pkgs.runCommand "kubaos-defaults" { } ''
    cp -r ${shipped} $out
    chmod -R u+w $out
    cp -r ${kubaosPkgs.kubaos-token-outputs}/config/. $out/
    chmod -R u+w $out
    { printf '%s' ${lib.escapeShellArg tmsSearchDirs}; cat ${shipped}/tms/config.toml; } >$out/tms/config.toml
  '';

  # dot-config reads this as ~/.local/share/kubaos/layer.
  layer = {
    # The app rewrites these, so they're the user's: copied from the
    # defaults once (source, then destination under ~/.config), never again.
    seed = {
      "btop/btop.conf" = "btop/btop.conf";
      "tms/config.toml" = "tms/config.toml";
      "nvim/lazy-lock.json" = "kubaos/${userLayer.apps.nvim}/lazy-lock.json";
      "nvim/lazyvim.json" = "kubaos/${userLayer.apps.nvim}/lazyvim.json";
    };
    # No include: ~/.config/<file> = defaults + the user layer's file (the
    # value, relative to the user layer), rewritten by dot-config apply.
    # concat appends; json merges objects key by key, with the wallpaper
    # theme's colours for the app between the two (dot-theme runs
    # `dot-config merge` after each theme change).
    concat = {
      "bat/config" = "${userLayer.apps.bat}/config";
      "git/ignore" = "${userLayer.apps.git}/ignore";
    };
    json = {
      "fastfetch/config.jsonc" = "${userLayer.apps.fastfetch}/config.jsonc";
      "fastfetch/greeting.jsonc" = "${userLayer.apps.fastfetch}/greeting.jsonc";
    };
  };
  layerFile = lib.concatLines (
    lib.mapAttrsToList (app: path: "app ${app} ${path}") userLayer.apps
    ++ lib.mapAttrsToList (src: dst: "seed ${src} ${dst}") layer.seed
    ++ lib.mapAttrsToList (src: user: "concat ${src} ${user}") layer.concat
    ++ lib.mapAttrsToList (src: user: "json ${src} ${user}") layer.json
  );
  notLinked = lib.attrNames layer.seed ++ lib.attrNames layer.concat ++ lib.attrNames layer.json;

  # Every regular file under dir, as paths relative to it.
  files =
    dir:
    lib.concatLists (
      lib.mapAttrsToList (
        name: type:
        if type == "directory" then
          map (f: "${name}/${f}") (files "${dir}/${name}")
        else if type == "regular" then
          [ name ]
        else
          [ ]
      ) (builtins.readDir dir)
    );
  linked = lib.subtractLists notLinked (lib.unique (files shipped ++ built));

  scriptsDir = builtins.path {
    path = ../scripts;
    name = "kubaos-scripts";
  };
  scripts = lib.attrNames (
    lib.filterAttrs (name: t: t == "regular" && !lib.hasPrefix "." name) (builtins.readDir ../scripts)
  );
  # Where the repo lived before the user flake: machines installed then
  # still have live links into it.
  oldCheckout = "${config.home.homeDirectory}/Projects/dotfiles";
in
{
  xdg.configFile =
    lib.genAttrs linked (f: {
      source = "${defaults}/${f}";
    })
    // {
      # Generated by dot-theme (config/matugen).
      "yazi/theme.toml".source = outOfStore "${config.xdg.stateHome}/theme/yazi.toml";
      "btop/themes/matugen.theme".source = outOfStore "${config.xdg.stateHome}/theme/btop.theme";
      "fastfetch/kubaos.txt".source = ../branding/fastfetch.txt;
      # KubaOS ships no yazi settings besides the theme: these are the user's.
      "yazi/yazi.toml".source = outOfStore "${userLayer.dir "yazi"}/yazi.toml";
      "yazi/keymap.toml".source = outOfStore "${userLayer.dir "yazi"}/keymap.toml";
      # The Daemon merges ~/.config/kotodama/*.toml in filename order: the
      # user's file sorts after kubaos.toml, so it wins.
      "kotodama/user.toml".source = outOfStore "${userLayer.dir "kotodama"}/user.toml";
    };

  # The seeds are where home/user-layer.nix puts their apps, so the table every
  # reader follows and the tree dot-config creates can't drift apart.
  assertions = map (folder: {
    assertion = lib.elem folder (lib.attrValues userLayer.apps);
    message = "user-layer/${folder} is not where home/user-layer.nix puts that app";
  }) seedFolders;

  xdg.dataFile = {
    # The defaults as shipped, for dot-config and for reading.
    "kubaos/defaults".source = defaults;
    "kubaos/skel".source = ../user-layer;
    "kubaos/layer".text = layerFile;
  };

  home.file = lib.listToAttrs (
    map (name: lib.nameValuePair ".local/bin/${name}" { source = "${scriptsDir}/${name}"; }) scripts
  );

  # Migration from live links: ~/.config/<app> used to be a link to
  # config/<app> in the checkout. Remove those before home-manager checks its
  # targets, or it would find (and back up) the checkout's own files there.
  # A machine-local config/hypr/local.lua becomes the user layer's
  # hyprland.lua, unless that already exists. It runs before dot-config moves
  # a flat layout into the groups, so the flat hyprland.lua counts as existing
  # too: copying local.lua beside it would leave two files to merge by hand.
  home.activation.liveLinksToDefaults = lib.hm.dag.entryBefore [ "checkLinkTargets" ] ''
    for l in "${config.xdg.configHome}"/*; do
      if [ -L "$l" ] && [ "$(readlink -f "$l")" = "${oldCheckout}/config/$(basename "$l")" ]; then
        run rm "$l"
      fi
    done
    if [ -f "${oldCheckout}/config/hypr/local.lua" ] && [ ! -e "${userLayer.dir "hypr"}/hyprland.lua" ] &&
      [ ! -e "${userLayer.root}/hypr/hyprland.lua" ]; then
      run install -D -m 0644 "${oldCheckout}/config/hypr/local.lua" "${userLayer.dir "hypr"}/hyprland.lua"
    fi
  '';

  home.activation.userLayer = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    run env PATH="${lib.makeBinPath [ pkgs.python3 ]}:$PATH" \
      XDG_CONFIG_HOME="${config.xdg.configHome}" XDG_DATA_HOME="${config.xdg.dataHome}" \
      ${pkgs.bash}/bin/bash ${../scripts/dot-config} apply
  '';
}
