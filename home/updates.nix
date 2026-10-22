# The daily update check: `kubaos update --check` asks origin, as the user and
# without changing anything, whether the channel has a new release. It writes
# ~/.local/state/kubaos/update-available, which the bar's marker and the
# Command menu's "Update KubaOS" read, and sends one notification per release.
# Package updates never notify: they wait for the next `kubaos update`.
#
# Persistent: a laptop that was off at the time checks when it next starts.
# The randomized delay keeps every machine from asking GitHub at once.
{
  systemd.user.services.kubaos-update-check = {
    Unit = {
      Description = "Check for a new KubaOS release";
      # A machine set up by hand has no installed CLI to ask with.
      ConditionPathExists = "/usr/bin/kubaos";
      After = [ "network-online.target" ];
    };
    Service = {
      Type = "oneshot";
      ExecStart = "/usr/bin/kubaos update --check";
    };
  };

  systemd.user.timers.kubaos-update-check = {
    Unit.Description = "Check for a new KubaOS release daily";
    Timer = {
      OnCalendar = "daily";
      Persistent = true;
      RandomizedDelaySec = "1h";
    };
    Install.WantedBy = [ "timers.target" ];
  };
}
