# The curated set (docs/adr/0001-release-and-update-model.md): what moves
# only with a KubaOS release. Built from KubaOS's own nixpkgs and exported as
# packages.<system>; the home modules take them from the kubaosPkgs argument,
# and everything else from the user's nixpkgs (lib.mkHome in flake.nix).
#
# Curated means built from source outside nixpkgs, loaded into a curated
# process (Quickshell's Qt plugins, the lock screen's PAM modules), or given
# privileges by system/ (gsr-kms-server).
{ pkgs, inputs }:
let
  inherit (pkgs.stdenv.hostPlatform) system;
in
rec {
  # The Command menu: the `kotodama` command, the Daemon, the Bundled
  # extensions and the QML module. The flake input follows KubaOS's nixpkgs,
  # so the module is built against the Qt that Quickshell loads it into.
  kotodama = inputs.kotodama.packages.${system}.default;

  # Extra Qt modules on Quickshell's QML import path, beyond its built-in qtsvg.
  quickshell = inputs.quickshell.packages.${system}.default.withModules (
    with pkgs.qt6;
    [
      qt5compat # Qt5Compat.GraphicalEffects
      qtmultimedia
      qtimageformats # webp, avif, tiff wallpapers
      # `import Kotodama`, the Command menu in config/quickshell/shell.qml.
      kotodama.qml
    ]
  );
  # The qtct platform theme plugins Quickshell loads: its Qt, not the user's.
  inherit (pkgs.libsForQt5) qt5ct;
  inherit (pkgs.qt6Packages) qt6ct;

  # Papirus with cyan folders, the neon accent most Tokyo wallpapers share.
  # An override, so cache.nixos.org doesn't have it.
  papirus-cyan = pkgs.papirus-icon-theme.override { color = "cyan"; };

  # system/65-recorder.sh setcaps this package's gsr-kms-server; the gsr in
  # the profile must be the same build.
  inherit (pkgs) gpu-screen-recorder;

  # The lock screen's PAM modules (system/lib.sh, link_nix_pam_module), which
  # load into Quickshell next to Nix's libpam.
  inherit (pkgs) fprintd pam_u2f;
}
# aarch64 is the VM, where Hyprland comes from nixpkgs (home/desktop.nix).
// pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isAarch64 { inherit (pkgs) hyprland; }
