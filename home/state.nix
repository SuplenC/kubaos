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
#
# held.toml, which only the switch writes (crates/kubaos-core/src/held.rs),
# names the listed packages that couldn't be built at the last switch, each
# with the store path the home had: a held package installs that path instead
# of the attribute. Each generation records the store path it installed for
# every listed package in ~/.local/share/kubaos/installed.json, and exposes
# the attributes as nixpkgs has them in kubaos.listedPackages, which the
# switch builds before it switches.
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

  heldFile = dir + "/held.toml";
  held = if present && builtins.pathExists heldFile then (readToml heldFile).held or { } else { };

  # A store path as a package home.packages takes. Pure evaluation refuses
  # builtins.storePath, but a string with the path in its context makes the
  # generation depend on the path all the same.
  storePackage =
    name: path:
    let
      outPath = builtins.appendContext path {
        ${path} = {
          path = true;
        };
      };
      package = {
        type = "derivation";
        name = builtins.substring 33 (-1) (baseNameOf path);
        inherit outPath;
        outputs = [ "out" ];
        out = package;
        outputName = "out";
        meta = { };
      };
    in
    if
      builtins.isString path && builtins.match "${builtins.storeDir}/[a-z0-9]{32}-[^/]+" path != null
    then
      package
    else
      throw "kubaos.d/held.toml: ${name} is held at ${toString path}, which is no store path";

  fresh = name: lib.attrByPath (lib.splitString "." name) null pkgs;

  package =
    name: from:
    let
      found = fresh name;
    in
    if held ? ${name} then
      storePackage name held.${name}
    else if found == null then
      throw "kubaos.d: ${name} is not a package in your nixpkgs (listed in ${
        lib.concatMapStringsSep ", " toString from
      })"
    else
      found;

  installed = lib.mapAttrs package listed;
in
{
  options.kubaos.stateDir = lib.mkOption {
    type = lib.types.nullOr lib.types.path;
    default = null;
    example = lib.literalExpression "./kubaos.d";
    description = "The user flake's kubaos.d/ directory, whose TOML files list packages and Suites to install.";
  };

  options.kubaos.listedPackages = lib.mkOption {
    type = lib.types.lazyAttrsOf (lib.types.nullOr lib.types.raw);
    internal = true;
    readOnly = true;
    description = "Every package the state directory lists, by name, as the user's nixpkgs has it (null when it has none), held or not: what the switch builds before it switches.";
  };

  config = lib.mkMerge [
    # home-manager imports the user's nixpkgs again with `nixpkgs.config`.
    # Set here, it takes the place of the config of the pkgs mkHome was given
    # (nixpkgs' defaults, for the user flake's legacyPackages), and a user's
    # own `nixpkgs.config` in home.nix merges with it key by key. Off, nothing
    # is set, so nixpkgs' default refuses unfree packages and the user's
    # `allowUnfreePredicate` or `allowUnfree` still decide.
    (lib.mkIf allowUnfree { nixpkgs.config.allowUnfree = true; })
    { kubaos.listedPackages = lib.mapAttrs (name: _: fresh name) listed; }
    (lib.mkIf present {
      home.packages = lib.attrValues installed;
      kubaos.suites = suites;
      xdg.dataFile."kubaos/kubaos.d".source = dir;
      # The interpolated paths keep their context, so the record holds each package in the
      # generation's closure, as a hold needs it to stay.
      xdg.dataFile."kubaos/installed.json".text = builtins.toJSON (
        lib.mapAttrs (_: package: "${package}") installed
      );
    })
  ];
}
