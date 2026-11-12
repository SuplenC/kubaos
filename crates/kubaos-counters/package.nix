# `kubaos-counters` (home/curated.nix): the counters' command, which keeps them in
# ~/.local/state/kubaos/counters.toml with their change log, resets them on time, and writes the
# state the bar reads. home/counters.nix installs it.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-counters" {
  pname = "kubaos-counters";

  meta = {
    description = "KubaOS's counters: named counts with auto resets and a history of every change";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-counters";
    platforms = lib.platforms.linux;
  };
}
