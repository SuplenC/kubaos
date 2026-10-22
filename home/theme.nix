# Theme: wallpaper + matugen pipeline (scripts/dot-theme, config/matugen/),
# GTK / Qt / cursor / icons. Generated colors live in ~/.local/state/theme/.
{
  config,
  kubaosPkgs,
  lib,
  pkgs,
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
  # The recoloured pointer `kubaos cursor` writes, and the theme it inherits
  # (THEME_NAME and FALLBACK_THEME in crates/kubaos-cursor/src/lib.rs).
  cursorTheme = "KubaOS-Bibata";
  cursorFallback = "Bibata-Modern-Classic";
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

  # The pointer follows the wallpaper: `kubaos cursor` (crates/kubaos-cursor),
  # run by dot-theme, recolours Bibata's SVG sources into
  # ~/.local/share/icons/KubaOS-Bibata, a hyprcursor theme for Hyprland and an
  # XCursor theme for XWayland and GTK 3. Every client asks for that name. Its
  # index.theme inherits Bibata-Modern-Classic, installed below, so the
  # `classic` style, a failed render and a cursor it lacks all show the classic
  # pointer; without the directory, the `default` theme home-manager writes
  # inherits it too.
  home.pointerCursor = {
    enable = true;
    # Black with a white outline: the classic style and the fallback.
    name = cursorFallback;
    package = pkgs.bibata-cursors;
    size = 24;
    x11.enable = true;
  };
  home.sessionVariables = {
    XCURSOR_THEME = cursorTheme;
    HYPRCURSOR_THEME = cursorTheme;
    HYPRCURSOR_SIZE = 24;
  };
  xresources.properties."Xcursor.theme" = lib.mkForce cursorTheme;
  gtk.cursorTheme = {
    name = cursorTheme;
    size = 24;
  };
  # Bibata's sources for `kubaos cursor`: the release nixpkgs 26.05 builds
  # bibata-cursors from, so the recoloured pointer and the classic one are the
  # same drawing. Pinned here, not taken from the user's nixpkgs, because
  # `kubaos cursor` reads this release's file layout.
  xdg.dataFile."kubaos/cursor/bibata".source = pkgs.fetchFromGitHub {
    owner = "ful1e5";
    repo = "Bibata_Cursor";
    rev = "v2.0.7";
    hash = "sha256-kIKidw1vditpuxO1gVuZeUPdWBzkiksO/q2R/+DUdEc=";
  };
  # A home that never ran `kubaos cursor` (before its first login) gets the
  # classic stub, so the name every client asks for exists. `kubaos cursor`
  # replaces it whole.
  home.activation.cursorTheme = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    theme="${config.xdg.dataHome}/icons/${cursorTheme}"
    if [ ! -e "$theme" ]; then
      run mkdir -p "$theme"
      if [ -z "''${DRY_RUN:-}" ]; then
        printf '[Icon Theme]\nName=${cursorTheme}\nComment=Bibata Modern in the colours of the wallpaper\nInherits=${cursorFallback}\n' \
          >"$theme/index.theme"
      fi
    fi
  '';

  gtk = {
    enable = true;
    colorScheme = "dark";
    theme = {
      name = "adw-gtk3-dark";
      package = pkgs.adw-gtk3;
    };
    iconTheme = {
      name = iconTheme;
      package = kubaosPkgs.papirus-cyan;
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
    platformTheme = {
      name = "qtct";
      # Quickshell loads these plugins, so they're built against its Qt.
      package = [
        kubaosPkgs.qt5ct
        kubaosPkgs.qt6ct
      ];
    };
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
    # The size is `kubaos cursor`'s: it sets cursor-size from the user's
    # setting, which a switch would otherwise reset.
    cursor-theme = cursorTheme;
  };

  # libadwaita reads the button layout from here (through the portal).
  dconf.settings."org/gnome/desktop/wm/preferences".button-layout = ":";

  # KubaOS's wallpaper sets are read-only in /usr/share/kubaos/wallpapers
  # (system/60-nix.sh); ~/Pictures/wallpapers is yours, a real folder that
  # dot-theme and the picker show as the last tab.
  #
  # Migration: older generations linked it to the checkout's wallpapers/.
  # Replace the link with the folder; the sets are in /usr/share now, and
  # dot-theme moves a current wallpaper from the old path.
  #
  # ~/.config/kubaos/wallpapers/sets, when the install didn't write it
  # (a machine from before, or one set up with system/): the sets on disk
  # now, so the ones a later release adds stay opt-in. Never touched again.
  home.activation.wallpapersFolder = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    walls="${config.home.homeDirectory}/Pictures/wallpapers"
    if [ -L "$walls" ]; then
      run rm "$walls"
    fi
    run mkdir -p "$walls"
    sets="${config.xdg.configHome}/kubaos/wallpapers/sets"
    if [ ! -e "$sets" ] && [ -d /usr/share/kubaos/wallpapers/ ]; then
      run mkdir -p "$(dirname "$sets")"
      if [ -z "''${DRY_RUN:-}" ]; then
        {
          echo "# Wallpaper sets the picker shows, one per line (from /usr/share/kubaos/wallpapers)."
          echo "# Your own images go in ~/Pictures/wallpapers."
          for d in /usr/share/kubaos/wallpapers/*/; do if [ -d "$d" ]; then basename "$d"; fi; done
        } >"$sets"
      fi
    fi
  '';

  # First run: render defaults so every consumer has a file before login.
  # After linkGeneration, so ~/.local/bin/dot-theme exists on a fresh home.
  home.activation.dotThemeDefaults = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    if [ ! -e "${stateDir}/colors.json" ] && [ -x "${config.home.homeDirectory}/.local/bin/dot-theme" ]; then
      run env PATH="${
        lib.makeBinPath [
          pkgs.matugen
        ]
      }:$PATH" \
        "${config.home.homeDirectory}/.local/bin/dot-theme" apply || true
    fi
  '';
}
