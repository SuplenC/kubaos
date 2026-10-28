# The state directory, kubaos.d/ in the user flake: what the kubaos command,
# the Command menu and Kōbō install, as TOML files the tool owns
# (crates/kubaos-core/src/state.rs). home.nix points here with
# `kubaos.stateDir = ./kubaos.d;`, a path inside the user flake, so the files
# reach this module through the flake's own source.
#
# packages.toml and every suites/<name>.toml may list `packages`, attribute
# paths into the user's pkgs; the home gets the union. Each suites/<name>.toml
# also turns on the Suite <name> (suites.nix) with the file's `version` and
# `extras`. nixpkgs.toml's `allow_unfree` (true without it) decides whether
# the user's pkgs takes unfree packages. Each generation also links its copy
# of the directory to ~/.local/share/kubaos/kubaos.d, which is what a rollback
# restores the directory from.
{
  config,
  lib,
  pkgs,
  ...
}:
let
  dir = config.kubaos.stateDir;
  present = dir != null && builtins.pathExists dir;

  readToml = file: builtins.fromTOML (builtins.readFile file);
  tomlFiles =
    sub:
    let
      folder = dir + "/${sub}";
      entries = if builtins.pathExists folder then builtins.readDir folder else { };
    in
    map (name: folder + "/${name}") (
      lib.attrNames (
        lib.filterAttrs (name: type: type == "regular" && lib.hasSuffix ".toml" name) entries
      )
    );
  files =
    lib.optional (builtins.pathExists (dir + "/packages.toml")) (dir + "/packages.toml")
    ++ tomlFiles "suites";

  suites = lib.listToAttrs (
    map (
      file:
      let
        toml = readToml file;
      in
      lib.nameValuePair (lib.removeSuffix ".toml" (baseNameOf file)) {
        version = toml.version or null;
        extras = toml.extras or [ ];
      }
    ) (tomlFiles "suites")
  );

  # name -> the files that list it, so a missing package names its file.
  listed = lib.foldl' (
    acc: file:
    lib.foldl' (inner: name: inner // { ${name} = (inner.${name} or [ ]) ++ [ file ]; }) acc (
      (readToml file).packages or [ ]
    )
  ) { } (lib.optionals present files);

  settingsFile = dir + "/nixpkgs.toml";
  allowUnfree =
    !(present && builtins.pathExists settingsFile) || (readToml settingsFile).allow_unfree or true;

  package =
    name: from:
    let
      found = lib.attrByPath (lib.splitString "." name) null pkgs;
    in
    if found == null then
      throw "kubaos.d: ${name} is not a package in your nixpkgs (listed in ${
        lib.concatMapStringsSep ", " toString from
      })"
    else
      found;
in
{
  options.kubaos.stateDir = lib.mkOption {
    type = lib.types.nullOr lib.types.path;
    default = null;
    example = lib.literalExpression "./kubaos.d";
    description = "The user flake's kubaos.d/ directory, whose TOML files list packages and Suites to install.";
  };

  config = lib.mkMerge [
    # home-manager imports the user's nixpkgs again with `nixpkgs.config`.
    # Set here, it takes the place of the config of the pkgs mkHome was given
    # (nixpkgs' defaults, for the user flake's legacyPackages), and a user's
    # own `nixpkgs.config` in home.nix merges with it key by key. Off, nothing
    # is set, so nixpkgs' default refuses unfree packages and the user's
    # `allowUnfreePredicate` or `allowUnfree` still decide.
    (lib.mkIf allowUnfree { nixpkgs.config.allowUnfree = true; })
    (lib.mkIf present {
      home.packages = lib.mapAttrsToList package listed;
      kubaos.suites = suites;
      xdg.dataFile."kubaos/kubaos.d".source = dir;
    })
  ];
}
