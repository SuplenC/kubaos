# Quickshell, GPU drivers for Nix apps on Ubuntu, fonts.
{
  inputs,
  lib,
  pkgs,
  ...
}:
let
  # Extra Qt modules on Quickshell's QML import path, beyond its built-in qtsvg.
  quickshell = inputs.quickshell.packages.${pkgs.stdenv.hostPlatform.system}.default.withModules (
    with pkgs.qt6;
    [
      qt5compat # Qt5Compat.GraphicalEffects
      qtmultimedia
      qtimageformats # webp, avif, tiff wallpapers
    ]
  );
in
{
  # home-manager's non-NixOS GPU support: every Nix GL/Vulkan app (Quickshell,
  # Ghostty, ...) loads Mesa from /run/opengl-driver, no per-app wrapping.
  # One-time, and again whenever activation warns that drivers changed:
  #   sudo "$(command -v non-nixos-gpu-setup)"
  targets.genericLinux.gpu.enable = true;

  # No systemd service: Hyprland's autostart runs `qs`.
  home.packages = [
    quickshell

    # Fonts for the shell (config/quickshell/theme/Theme.qml): Geist for the
    # interface, M PLUS 1 for Japanese accents, Material Symbols for icons.
    pkgs.geist-font
    pkgs.mplus-outline-fonts.githubRelease
    pkgs.material-symbols
    pkgs.nerd-fonts.jetbrains-mono
    pkgs.nerd-fonts.caskaydia-cove

    # Screen recording and screenshot annotation (scripts/dot-hypr-record,
    # dot-hypr-screenshot). gsr's KMS capture finds the setcap'd
    # gsr-kms-server in /run/wrappers/bin (system/65-recorder.sh).
    pkgs.gpu-screen-recorder
    pkgs.satty
  ]
  # aarch64 is the VM. The Hyprland PPA builds no arm64 binaries and Ubuntu's
  # own is 0.53, which predates the Lua config: it ignores config/hypr and
  # writes a stock hyprland.conf into the repo. nixpkgs' Hyprland reads
  # hyprland.lua, and ~/.nix-profile/bin comes first on PATH, so tty1's
  # start-hyprland is this one. It renders through /run/opengl-driver
  # (llvmpipe in QEMU). The laptop keeps the PPA's.
  ++ lib.optionals pkgs.stdenv.hostPlatform.isAarch64 [ pkgs.hyprland ];

  # start-hyprland (no uwsm) never activates graphical-session.target, and
  # xdg-desktop-portal refuses to start without it (Requisite=), which leaves
  # Chrome without the portal's dark-mode setting, file chooser and screen
  # sharing. Hyprland's autostart (config/hypr/autostart.lua) starts this
  # target once the session environment is imported, and stops it on exit.
  systemd.user.targets.hyprland-session.Unit = {
    Description = "Hyprland session";
    Documentation = [ "man:systemd.special(7)" ];
    BindsTo = [ "graphical-session.target" ];
    Wants = [ "graphical-session-pre.target" ];
    After = [ "graphical-session-pre.target" ];
  };
}
