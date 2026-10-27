#!/usr/bin/env bash
# sched-ext-lavd: the power stops' root helper learns `scheduler=`, which runs
# scx_lavd through kubaos-scx.service in the stop's mode when LAVD is turned on
# in power.toml. This reinstalls the helper and its polkit action
# (install_power_helper in system/lib.sh; ppd's actions stay as the user set
# them). scx_lavd and its unit come right after, from system/relink.sh, which
# every update runs once the system migrations are done; LAVD stays off.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

kubaos_backup /etc/polkit-1/rules.d/50-kubaos-power.rules
install_power_helper
