# Live links: config/<name>/ -> ~/.config/<name>, scripts/<file> -> ~/.local/bin/<file>.
# The targets are the working tree, so edits apply without a rebuild. New entries
# need `git add` before the next `home-manager switch` (flakes only see tracked files).
{
  config,
  lib,
  dotfilesDir,
  ...
}:
let
  link = path: config.lib.file.mkOutOfStoreSymlink "${dotfilesDir}/${path}";

  entries =
    type: dir:
    lib.optionals (builtins.pathExists dir) (
      lib.attrNames (
        lib.filterAttrs (name: t: t == type && !lib.hasPrefix "." name) (builtins.readDir dir)
      )
    );
in
{
  xdg.configFile = lib.genAttrs (entries "directory" ../config) (name: {
    source = link "config/${name}";
  });

  home.file = lib.listToAttrs (
    map (name: lib.nameValuePair ".local/bin/${name}" { source = link "scripts/${name}"; }) (
      entries "regular" ../scripts
    )
  );
}
