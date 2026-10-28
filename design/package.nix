# `kubaos-token-outputs` (home/curated.nix): the token outputs git ignores,
# written by `kubaos-tokens build` at their paths in the repo. KubaOS.Components
# takes its Tokens.qml and Symbols.qml (qml/package.nix), and the home links the
# configs, the matugen templates and Hyprland's tokens.lua beside config/'s own
# files (home/links.nix).
#
# The source is the tokens and the files the outputs read: oh-my-posh's
# template, Geist Mono for the terminals' line and KubaOS Symbols' drawings. An
# output that reads another file fails here, naming it, until it is added.
{
  lib,
  runCommand,
  kubaos-tokens,
}:
let
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ./tokens.toml
      ../config/ohmyposh/config.toml.tmpl
      ../crates/kubaos-ui/assets/fonts/GeistMono-Regular.ttf
      ../crates/kubaos-ui/assets/symbols
    ];
  };
in
runCommand "kubaos-token-outputs" { } ''
  ${lib.getExe kubaos-tokens} build $out --root ${src}
''
