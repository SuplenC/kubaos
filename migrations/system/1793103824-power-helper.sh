#!/usr/bin/env bash
# power-helper: what the power stops need as root (install_power_helper in
# system/lib.sh), on a machine installed before system/30-hardware.sh shipped
# it: the helper that writes their knobs in /sys, its polkit action and rule,
# amdgpu's FreeSync video modes for a 60 Hz panel without a modeset (the
# initramfs is rebuilt; it takes effect at the next boot), and
# power-profiles-daemon's GPU action on and its panel action off. Without them
# a stop changes only power-profiles-daemon's profile, and Super Saver keeps
# the panel at its native rate with VRR.
# kubaos update runs it once, as root, after apt and before the home-manager switch,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"
# shellcheck source=system/lib.sh
source "$KUBAOS_DIR/system/lib.sh"

kubaos_backup /etc/polkit-1/rules.d/50-kubaos-power.rules
kubaos_backup /etc/modprobe.d/kubaos-amdgpu.conf
install_power_helper
