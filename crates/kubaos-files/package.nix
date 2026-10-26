# `kubaos-files` (home/curated.nix): yazi in Ghostty for xdg-open's folders
# and for apps' "Show in folder". home/files.nix installs it and registers it.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-files" {
  pname = "kubaos-files";

  meta = {
    description = "yazi in a terminal window, for xdg-open and org.freedesktop.FileManager1";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-files";
    platforms = lib.platforms.linux;
  };
}
