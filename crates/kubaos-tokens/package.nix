# `kubaos-tokens` (home/curated.nix): the tokens' generator, which the Nix
# build of the token outputs git ignores runs (design/package.nix).
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-tokens" {
  pname = "kubaos-tokens";

  meta = {
    description = "KubaOS's design tokens: checks design/tokens.toml and writes each toolkit's copy";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-tokens";
    platforms = lib.platforms.all;
  };
}
