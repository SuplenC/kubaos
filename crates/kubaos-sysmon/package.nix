# `kubaos-sysmon` (home/curated.nix): the system monitor the bar's pill and the
# popup read a stream from, and the command the menu changes its settings with.
# home/sysmon.nix installs it.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-sysmon" {
  pname = "kubaos-sysmon";

  meta = {
    description = "KubaOS's system monitor: CPU, memory, GPU, disks, network and battery from /proc and /sys";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-sysmon";
    platforms = lib.platforms.linux;
  };
}
