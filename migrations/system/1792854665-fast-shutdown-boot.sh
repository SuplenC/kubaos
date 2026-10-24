#!/usr/bin/env bash
# fast-shutdown-boot: the 10 s stop timeouts for the system and user managers,
# Docker started by its socket instead of at boot, NetworkManager-wait-online
# off and setvtrgb masked (install_fast_shutdown_boot in system/lib.sh), on a
# machine installed before system/20-base.sh and 40-apps.sh did this, and the
# splash theme that plays the sign backwards on shutdown. Without them a stuck
# service holds a shutdown for 90 s, and the boot waits for the network before
# the desktop.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

install_fast_shutdown_boot

# plymouthd reads the theme from the root file system on shutdown and reboot,
# so the script alone is enough there. The initramfs keeps its copy for the
# boot, which plays the same, until the next kernel update rebuilds it.
theme=/usr/share/plymouth/themes/kubaos
if [[ -d $theme ]]; then
  install_root_file "$KUBAOS_DIR/system/files/plymouth/kubaos/kubaos.script" "$theme/kubaos.script" 0644
fi
