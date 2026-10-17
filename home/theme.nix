# Theme: wallpaper + matugen pipeline (scripts/dot-theme, config/matugen/),
# GTK / Qt / cursor / icons. Generated colors live in ~/.local/state/theme/.
{
  config,
  inputs,
  lib,
  pkgs,
  dotfilesDir,
  ...
}:
let
  stateDir = "${config.home.homeDirectory}/.local/state/theme";
  # GTK silently ignores a missing @import, so this is safe before the first run.
  importGenerated = ''
    /* Wallpaper colors from dot-theme (matugen). */
    @import url("file://${stateDir}/gtk.css");
  '';
  iconTheme = "Papirus-Dark";
  # Papirus with cyan folders, the neon accent most Tokyo wallpapers share.
  iconPackage = pkgs.papirus-icon-theme.override { color = "cyan"; };
  # Interface font, the same one the shell uses (home/desktop.nix installs it).
  # 10 pt is sized for Full HD density: the 1080p externals at scale 1 and the
  # laptop panel at scale 1.5 (config/hypr/monitors.lua) look the same.
  uiFont = "Geist";
  uiFontSize = 10;
  qtFont =
    family: size: weight:
    ''"${family},${toString size},-1,5,${toString weight},0,0,0,0,0,0,0,0,0,0,1"'';
  # qt5ct/qt6ct list schemes from their colors/ dirs; the generated palette is
  # linked there (below) so the settings dialog shows it by name.
  qtctSettings = dir: {
    Fonts = {
      general = qtFont uiFont uiFontSize 400;
      fixed = qtFont "JetBrainsMono Nerd Font" uiFontSize 400;
    };
    Appearance = {
      style = "Fusion";
      custom_palette = true;
      color_scheme_path = "${config.xdg.configHome}/${dir}/colors/matugen.conf";
      icon_theme = iconTheme;
      standard_dialogs = "xdgdesktopportal";
    };
  };
in
{
  home.packages = with pkgs; [
    matugen
    # dot-theme uses coreutils, flock (util-linux), pgrep (procps); all in
    # Ubuntu's base system, so nothing else is needed here.
  ];

  home.pointerCursor = {
    enable = true;
    # Black with a white outline: crisp on dark, neon-lit wallpapers.
    name = "Bibata-Modern-Classic";
    package = pkgs.bibata-cursors;
    size = 24;
    gtk.enable = true;
    x11.enable = true;
    # Sets HYPRCURSOR_THEME/SIZE. Bibata ships XCursor only; Hyprland falls back
    # to it when no hyprcursor theme of that name exists, at the same size.
    hyprcursor = {
      enable = true;
      size = 24;
    };
  };

  gtk = {
    enable = true;
    colorScheme = "dark";
    theme = {
      name = "adw-gtk3-dark";
      package = pkgs.adw-gtk3;
    };
    iconTheme = {
      name = iconTheme;
      package = iconPackage;
    };
    font = {
      name = uiFont;
      size = uiFontSize;
    };
    # Hyprland tiles and closes windows itself (SUPER+Q); no title bar
    # buttons on GTK header bars.
    gtk3.extraConfig.gtk-decoration-layout = ":";
    gtk4.extraConfig.gtk-decoration-layout = ":";
    # GTK 4.20 parses gtk-interface-color-scheme by name; home-manager writes
    # the number (2), which GTK rejects with a warning, plus
    # gtk-application-prefer-dark-theme, which libadwaita warns about. Write
    # the name ourselves; libadwaita gets dark from the portal (dconf below).
    gtk4.colorScheme = null;
    gtk4.extraConfig.gtk-interface-color-scheme = "dark";
    gtk3.extraCss = importGenerated;
    # libadwaita: no theme import (stateVersion 26.05 leaves gtk4.theme null),
    # only the named-color overrides.
    gtk4.extraCss = importGenerated;
  };

  qt = {
    enable = true;
    platformTheme.name = "qtct";
    qt5ctSettings = qtctSettings "qt5ct";
    qt6ctSettings = qtctSettings "qt6ct";
  };

  # The generated palette under the name qt5ct/qt6ct show in their dialog
  # ("matugen"). Dangling until dot-theme's first run; Qt then uses Fusion's
  # own palette.
  xdg.configFile."qt5ct/colors/matugen.conf".source =
    config.lib.file.mkOutOfStoreSymlink "${stateDir}/qt6ct.colors";
  xdg.configFile."qt6ct/colors/matugen.conf".source =
    config.lib.file.mkOutOfStoreSymlink "${stateDir}/qt6ct.colors";

  # xdg-desktop-portal-gtk serves these to apps (libadwaita, Chrome, Electron).
  dconf.settings."org/gnome/desktop/interface" = {
    color-scheme = "prefer-dark";
    gtk-theme = "adw-gtk3-dark";
    icon-theme = iconTheme;
    font-name = "${uiFont} ${toString uiFontSize}";
    monospace-font-name = "JetBrainsMono Nerd Font ${toString uiFontSize}";
    cursor-theme = config.home.pointerCursor.name;
    cursor-size = config.home.pointerCursor.size;
  };

  # libadwaita reads the button layout from here (through the portal).
  dconf.settings."org/gnome/desktop/wm/preferences".button-layout = ":";

  # The wallpapers live in the repo (wallpapers/{japanese,cyberpunk}/), linked
  # live like config/, so adding an image there needs no rebuild.
  home.file."Pictures/wallpapers".source =
    config.lib.file.mkOutOfStoreSymlink "${dotfilesDir}/wallpapers";

  # Migration: older generations made ~/Pictures/wallpapers a real directory
  # holding only .keep markers. Replace it with the link above; if anything
  # else is inside, leave it and let home-manager report the conflict.
  home.activation.wallpapersToLink = lib.hm.dag.entryBefore [ "checkLinkTargets" ] ''
    walls="${config.home.homeDirectory}/Pictures/wallpapers"
    if [ -d "$walls" ] && [ ! -L "$walls" ]; then
      if [ -z "$(find "$walls" -mindepth 1 ! -type d ! -name .keep -print -quit)" ]; then
        run rm -rf "$walls"
      else
        warnEcho "$walls has your own files; move them into ${dotfilesDir}/wallpapers and remove it."
      fi
    fi
  '';

  # First run: render defaults so every consumer has a file before login.
  # After linkGeneration, so ~/.local/bin/dot-theme exists on a fresh home.
  # The default wallpaper is a ukiyo scene: dot-theme asks ukiyo for its
  # poster, from the profile's share, the same path the session sees.
  home.activation.dotThemeDefaults = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    if [ ! -e "${stateDir}/colors.json" ] && [ -x "${config.home.homeDirectory}/.local/bin/dot-theme" ]; then
      run env PATH="${
        lib.makeBinPath [
          pkgs.matugen
          pkgs.jq
          inputs.ukiyo.packages.${pkgs.stdenv.hostPlatform.system}.default
        ]
      }:$PATH" \
        XDG_DATA_DIRS="${config.home.profileDirectory}/share:''${XDG_DATA_DIRS:-/usr/local/share:/usr/share}" \
        "${config.home.homeDirectory}/.local/bin/dot-theme" apply || true
    fi
  '';
}
