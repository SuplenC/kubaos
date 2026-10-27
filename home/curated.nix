# The curated set (docs/adr/0001-release-and-update-model.md): what moves
# only with a KubaOS release. Built from KubaOS's own nixpkgs and exported as
# packages.<system>; the home modules take them from the kubaosPkgs argument,
# and everything else from the user's nixpkgs (lib.mkHome in flake.nix).
#
# Curated means built from source outside nixpkgs, loaded into a curated
# process (Quickshell's Qt plugins, the lock screen's PAM modules), given
# privileges by system/ (gsr-kms-server), or driven by a curated program
# through an interface that changes between its releases (kdeconnect-kde).
{ pkgs, inputs }:
let
  inherit (pkgs.stdenv.hostPlatform) system;
  inherit (pkgs) lib;

  # KubaOS's own crates, on one shared build of their dependencies
  # (crates/workspace.nix, through crates/build.nix).
  buildCrate = import ../crates/build.nix { inherit pkgs inputs; };
  callCrate = path: pkgs.callPackage path { inherit buildCrate; };
in
rec {
  # The Command menu: the `kotodama` command, the Daemon, the Bundled
  # extensions and the QML module. The flake input follows KubaOS's nixpkgs,
  # so the module is built against the Qt that Quickshell loads it into.
  kotodama = inputs.kotodama.packages.${system}.default;
  # The menu's KubaOS Extension: the actions and the system toggles
  # (home/menu.nix installs it with the Bundled ones).
  kotodama-kubaos = callCrate ../crates/kotodama-kubaos/package.nix;
  # The `kubaos` command, so an update's fixes to it arrive with the release
  # (home/updates.nix installs it).
  kubaos = callCrate ../crates/kubaos-cli/package.nix;
  # Kōbō, for first-boot setup (home/setup.nix), so a new setup card arrives
  # with the release, as `kubaos` does.
  kubaos-kobo = callCrate ../crates/kubaos-kobo/package.nix;
  # yazi's launcher and the Downloads tidy (home/files.nix).
  kubaos-files = callCrate ../crates/kubaos-files/package.nix;
  # The automount for removable drives, and their Eject and photo import
  # (home/automount.nix).
  kubaos-automount = callCrate ../crates/kubaos-automount/package.nix;
  # The phone link's command (home/phone.nix).
  kubaos-phone = callCrate ../crates/kubaos-phone/package.nix;
  # The phone link's daemon. Pinned with the release: kubaos-phone speaks
  # kdeconnectd's D-Bus interface, which can change between KDE Gear releases.
  inherit (pkgs.kdePackages) kdeconnect-kde;
  # The Bluetooth pairing agent the shell pairs through (home/bluetooth.nix).
  kubaos-bluetooth = callCrate ../crates/kubaos-bluetooth/package.nix;
  # The date engine behind the clock popup (home/clock.nix).
  kubaos-clock = callCrate ../crates/kubaos-clock/package.nix;
  # The system monitor behind the bar's pill and its popup (home/sysmon.nix).
  kubaos-sysmon = callCrate ../crates/kubaos-sysmon/package.nix;
  # The idle settings, Keep awake and the scheduled power actions (home/idle.nix).
  kubaos-idle = callCrate ../crates/kubaos-idle/package.nix;
  # The power stops' service and command (home/power.nix).
  kubaos-power = callCrate ../crates/kubaos-power/package.nix;
  # The design specimen, the Command menu's "Design specimen" (home/desktop.nix).
  kubaos-specimen = callCrate ../crates/kubaos-specimen/package.nix;

  # Geist and Geist Mono, the system's interface and monospace fonts, at the
  # release kubaos-ui embeds (crates/kubaos-ui/assets/make-fonts.sh), so the
  # shell and the iced apps draw the same letters. nixpkgs trails it. Only
  # the static TTFs: the OTFs and variable fonts would list every family twice
  # in fontconfig, and Geist Pixel isn't ours.
  geist-font = pkgs.stdenvNoCC.mkDerivation (finalAttrs: {
    pname = "geist-font";
    version = "1.8.1";
    src = pkgs.fetchzip {
      url = "https://github.com/vercel/geist-font/releases/download/v${finalAttrs.version}/geist-font-v${finalAttrs.version}.zip";
      hash = "sha256-5iMNUydI611EARjQ9jYWZdastq5m2dalGTWAvGU82vU=";
    };
    installPhase = ''
      runHook preInstall
      install -Dm444 -t $out/share/fonts/truetype Geist/ttf/*.ttf GeistMono/ttf/*.ttf
      runHook postInstall
    '';
    meta = {
      inherit (pkgs.geist-font.meta) homepage license;
      description = "Geist and Geist Mono, KubaOS's interface and monospace fonts";
      platforms = lib.platforms.all;
    };
  });

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
