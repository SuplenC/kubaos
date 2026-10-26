# `kubaos-clock` (home/curated.nix): the date engine the clock popup talks to,
# and the command the menu changes the clock settings with. home/clock.nix
# installs it. At run time it reads Ubuntu's tz database, so the clock follows
# its updates.
{
  lib,
  buildCrate,
}:

buildCrate "kubaos-clock" {
  pname = "kubaos-clock";

  meta = {
    description = "KubaOS's date engine: the clock popup's calendar, date maths and time zones";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-clock";
    platforms = lib.platforms.linux;
  };
}
