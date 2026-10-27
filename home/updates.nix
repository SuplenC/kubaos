# The `kubaos` command and the daily update check.
#
# `kubaos` comes with the release (home/curated.nix): each switch puts the
# release's build on PATH, and /usr/bin/kubaos runs it, so a fix to
# `kubaos update` reaches the machine with the update that carries it.
#
# `kubaos update --check` asks origin, as the user and without changing
# anything, whether the channel has a new release. It writes
# ~/.local/state/kubaos/update-available, which the bar's marker and the
# Command menu's "Update KubaOS" read, and sends one notification per release.
# It also counts the apt security updates waiting, as the user and on lists of
# its own (Ubuntu's apt-daily refresh is retired), and notifies once per set.
#
# Persistent: a laptop that was off at the time checks when it next starts.
# The randomized delay keeps every machine from asking GitHub at once.
{ kubaosPkgs, lib, ... }:
{
  home.packages = [ kubaosPkgs.kubaos ];

  systemd.user.services.kubaos-update-check = {
    Unit = {
      Description = "Check for a new KubaOS release and pending apt security updates";
      # A machine set up by hand has no KubaOS checkout to update.
      ConditionPathExists = "/usr/share/kubaos/.git";
      After = [ "network-online.target" ];
    };
    Service = {
      Type = "oneshot";
      ExecStart = "${lib.getExe kubaosPkgs.kubaos} update --check";
    };
  };

  systemd.user.timers.kubaos-update-check = {
    Unit.Description = "Check for a new KubaOS release and pending apt security updates daily";
    Timer = {
      OnCalendar = "daily";
      Persistent = true;
      RandomizedDelaySec = "1h";
    };
    Install.WantedBy = [ "timers.target" ];
  };
}
