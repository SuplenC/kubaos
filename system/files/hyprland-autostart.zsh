# Start Hyprland on tty1 after autologin.
#
# ~/.zprofile sources this from the Nix store (home/shell.nix), AFTER the Nix
# / home-manager environment is set up, so Hyprland and everything it launches
# inherit PATH, XDG_DATA_DIRS etc.
#
# start-hyprland ships in the hyprland package (PPA and Ubuntu archive). It is
# the launcher the Hyprland wiki documents; it watches the compositor and
# brings it back in safe mode after a crash. uwsm is not used: the PPA doesn't
# ship it and the wiki marks it "for advanced users".
#
# Its output (the ASCII logo, the startup log) would flash on tty1 between the
# boot splash and the first frame, so it goes to a file instead:
# $XDG_RUNTIME_DIR/start-hyprland.log. Hyprland keeps its own log in
# $XDG_RUNTIME_DIR/hypr/<instance>/hyprland.log.
#
# Escape hatches:
#   touch ~/.config/hypr/no-autostart   tty1 stays a plain shell
#   Ctrl+Alt+F2                         another tty, log in normally
if [[ -z ${WAYLAND_DISPLAY-} && -z ${DISPLAY-} && ${XDG_VTNR-0} -eq 1 \
      && ! -e ${XDG_CONFIG_HOME:-$HOME/.config}/hypr/no-autostart ]] \
   && (( $+commands[start-hyprland] )); then
  exec start-hyprland >"${XDG_RUNTIME_DIR:-/tmp}/start-hyprland.log" 2>&1
fi
