#!/usr/bin/env bash
# inotify-limits: room for 1024 inotify instances and 524288 watches per user
# (install_inotify_limits in system/lib.sh), on a machine installed before
# system/20-base.sh shipped it, applied at once. With the kernel's 128
# instances, a desktop with an editor open runs out, and a program that can't
# get one stops seeing its files change, or fails with "Too many open files".
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

same_content "$FILES_DIR/sysctl.d/60-kubaos-inotify.conf" /etc/sysctl.d/60-kubaos-inotify.conf ||
  kubaos_backup /etc/sysctl.d/60-kubaos-inotify.conf
install_inotify_limits
