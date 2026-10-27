# Quickshell, GPU drivers for Nix apps on Ubuntu, fonts.
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
{
  # home-manager's non-NixOS GPU support: every Nix GL/Vulkan app (Quickshell,
  # Ghostty, ...) loads Mesa from /run/opengl-driver, no per-app wrapping.
  # One-time, and again whenever activation warns that drivers changed:
  #   sudo "$(command -v non-nixos-gpu-setup)"
  # Mesa comes from KubaOS's nixpkgs, never newer than the user's: an app
  # built against a newer glibc can load an older Mesa, not the other way.
  targets.genericLinux.gpu = {
    enable = true;
    packages = kubaosPkgs.nixpkgs;
  };

  # No systemd service: Hyprland's autostart runs `qs`.
  home.packages = [
    kubaosPkgs.quickshell

    # Fonts for the shell, whose type styles are `design/tokens.toml`'s
    # `[font.family]`: Geist and Geist Mono for the interface, M PLUS 1 for
    # the Japanese accents. Its icons are KubaOS Symbols, SVGs in
    # `KubaOS.Components`, so no icon font. Geist and Geist Mono are curated,
    # at the release the iced apps embed; Geist Mono's Nerd Font is the one
    # terminals get the prompt's and the file managers' glyphs from once they
    # move onto it. JetBrains Mono's Nerd Font is GTK's and Qt's monospace
    # font until then (`home/theme.nix`).
    kubaosPkgs.geist-font
    pkgs.nerd-fonts.geist-mono
    pkgs.mplus-outline-fonts.githubRelease
    pkgs.nerd-fonts.jetbrains-mono
    pkgs.nerd-fonts.caskaydia-cove

    # The design specimen: every token and component, the menu's "Design
    # specimen" Item.
    kubaosPkgs.kubaos-specimen

    # Screen recording and screenshot annotation (scripts/dot-hypr-record,
    # dot-hypr-screenshot). gsr's KMS capture finds the setcap'd
    # gsr-kms-server in /run/wrappers/bin (system/65-recorder.sh).
    kubaosPkgs.gpu-screen-recorder
    pkgs.satty
  ]
  # aarch64 is the VM. The Hyprland PPA builds no arm64 binaries and Ubuntu's
  # own is 0.53, which predates the Lua config: it ignores config/hypr and
  # writes a stock hyprland.conf into the repo. nixpkgs' Hyprland reads
  # hyprland.lua, and ~/.nix-profile/bin comes first on PATH, so tty1's
  # start-hyprland is this one. It renders through /run/opengl-driver
  # (llvmpipe in QEMU). The laptop keeps the PPA's.
  ++ lib.optionals pkgs.stdenv.hostPlatform.isAarch64 [ kubaosPkgs.hyprland ];

  # start-hyprland (no uwsm) never activates graphical-session.target, and
  # xdg-desktop-portal refuses to start without it (Requisite=), which leaves
  # Chrome without the portal's dark-mode setting, file chooser and screen
  # sharing. Hyprland's autostart (config/hypr/autostart.lua) starts this
  # target once the session environment is imported. Leaving the session
  # (`dot-session leave`, the fallback exit bind) stops graphical-session.target
  # and waits for it before Hyprland goes.
  systemd.user.targets.hyprland-session.Unit = {
    Description = "Hyprland session";
    Documentation = [ "man:systemd.special(7)" ];
    BindsTo = [ "graphical-session.target" ];
    Wants = [ "graphical-session-pre.target" ];
    After = [ "graphical-session-pre.target" ];
  };

  # The portal and the polkit agent are Wayland clients that stop with the
  # session, before Hyprland goes. Should one hang on its way out, 5 s bound
  # it, under the 10 s every user unit gets
  # (`system/files/systemd/user.conf.d`). Drop-ins, so the units stay apt's.
  xdg.configFile =
    lib.genAttrs
      [
        "systemd/user/xdg-desktop-portal-hyprland.service.d/kubaos-stop.conf"
        "systemd/user/hyprpolkitagent.service.d/kubaos-stop.conf"
      ]
      (_: {
        text = ''
          [Service]
          TimeoutStopSec=5s
        '';
      });
}
