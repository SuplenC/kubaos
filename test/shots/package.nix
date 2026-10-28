# `kubaos-shots`: the tools `test/shots/shots` renders with, pinned, then that script from the
# checkout it runs in (`just shots`, `nix run .#shots`). The script, its stand-ins and its drivers
# are read from the working tree, so a change to the harness takes effect without a rebuild.
#
# The renders are Qt's `qml` runtime in a private headless sway (the frame's shadow and blur
# need GL, which Qt's offscreen platform lacks), drawn by Mesa's llvmpipe, so a shot is the same
# on every machine. The fonts are the shell's and nothing else: kubaos-ui's Geist and Geist Mono,
# M PLUS 1 and Noto Color Emoji, then DejaVu for the symbols they lack (⌃, ⇧), as Ubuntu falls
# back to it.
#
# The terminals' shots (`test/shots/terminals`) run Ghostty, foot and fuzzel in the same sway,
# with the TUIs they show and matugen pinned here, and the fonts a terminal finds on KubaOS:
# Geist and Geist Mono (the curated release), Geist Mono's and Meslo's Nerd Fonts (the terminals'
# font before they took the tokens'), Noto Color Emoji and DejaVu.
{
  lib,
  writeShellApplication,
  writeText,
  linkFarm,
  runCommand,
  qt6,
  sway-unwrapped,
  mesa,
  imagemagick,
  python3,
  git,
  coreutils,
  findutils,
  gnugrep,
  gnused,
  gawk,
  gnutar,
  bash,
  adwaita-icon-theme,
  mplus-outline-fonts,
  noto-fonts-color-emoji,
  dejavu_fonts,
  nerd-fonts,
  ghostty,
  foot,
  fuzzel,
  tmux,
  neovim-unwrapped,
  vimPlugins,
  btop,
  lazygit,
  bat,
  fastfetch,
  oh-my-posh,
  matugen,
  grim,
  swaybg,
  ncurses,
  procps,
  wallpapers,
  geist-font,
  ...
}:
let
  shellFonts = linkFarm "kubaos-shots-fonts" [
    {
      name = "kubaos-ui";
      path = ../../crates/kubaos-ui/assets/fonts;
    }
    {
      name = "mplus";
      path = "${mplus-outline-fonts.githubRelease}/share/fonts";
    }
    {
      name = "emoji";
      path = "${noto-fonts-color-emoji}/share/fonts";
    }
    {
      name = "fallback";
      path = "${dejavu_fonts}/share/fonts";
    }
  ];
  # HAZARD: only these directories. nixpkgs' `makeFontsConf` adds the system's and the user's
  # fonts and /etc/fonts/conf.d, and a shot would then depend on the machine. kubaos-ui's M PLUS 1
  # is a subset for iced; the full family is listed first, so fontconfig prefers it.
  fonts = writeText "kubaos-shots-fonts.conf" ''
    <?xml version="1.0"?>
    <!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
    <fontconfig>
      <dir>${shellFonts}/mplus</dir>
      <dir>${shellFonts}/kubaos-ui</dir>
      <dir>${shellFonts}/emoji</dir>
      <dir>${shellFonts}/fallback</dir>
      <cachedir prefix="xdg">fontconfig</cachedir>
    </fontconfig>
  '';
  terminalFonts = writeText "kubaos-shots-terminal-fonts.conf" ''
    <?xml version="1.0"?>
    <!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
    <fontconfig>
      <dir>${geist-font}/share/fonts</dir>
      <dir>${nerd-fonts.geist-mono}/share/fonts</dir>
      <dir>${nerd-fonts.meslo-lg}/share/fonts</dir>
      <dir>${shellFonts}/emoji</dir>
      <dir>${shellFonts}/fallback</dir>
      <cachedir prefix="xdg">fontconfig</cachedir>
    </fontconfig>
  '';
  # Neovim's plugins the terminals' shots load beside `config/nvim`: mini.base16, which draws
  # the theme, and lualine, which draws its status line.
  nvimPlugins = lib.makeSearchPath "" [
    vimPlugins.mini-nvim
    vimPlugins.lualine-nvim
  ];
  # The tray's icons: Adwaita's symbolic one as it ships (no fill), the same kind filled #2e3436
  # (as apps draw them for a light theme), and colour ones, for the tray and the menu's Results.
  icons = runCommand "kubaos-shots-icons" { } ''
    a=${adwaita-icon-theme}/share/icons/Adwaita
    mkdir -p $out
    cp $a/symbolic/status/audio-volume-high-symbolic.svg $out/
    sed -e 's|<svg \([^>]*\)>|<svg \1><g fill="#2e3436">|' -e 's|</svg>|</g></svg>|' \
      $a/symbolic/status/network-wireless-signal-good-symbolic.svg >$out/network-wireless-dark-symbolic.svg
    for f in places/folder places/folder-download mimetypes/text-html mimetypes/x-office-document mimetypes/image-x-generic; do
      cp $a/scalable/$f.svg $out/
    done
  '';
  tools = [
    qt6.qtdeclarative
    sway-unwrapped
    imagemagick
    python3
    git
    coreutils
    findutils
    gnugrep
    gnused
    gawk
    gnutar
    bash
    ghostty
    foot
    fuzzel
    tmux
    neovim-unwrapped
    btop
    lazygit
    bat
    fastfetch
    oh-my-posh
    matugen
    grim
    swaybg
    ncurses
    procps
  ];
in
writeShellApplication {
  name = "kubaos-shots";
  runtimeInputs = tools;
  text = ''
    export KUBAOS_SHOTS_PATH=${lib.makeBinPath tools}
    export KUBAOS_SHOTS_QML=${qt6.qtdeclarative}/bin/qml
    export KUBAOS_SHOTS_SWAY=${lib.getExe sway-unwrapped}
    export KUBAOS_SHOTS_QT_PLUGINS=${
      lib.makeSearchPath "lib/qt-6/plugins" [
        qt6.qtbase
        qt6.qtsvg
        qt6.qtdeclarative
        qt6.qtwayland
      ]
    }
    export KUBAOS_SHOTS_QML_IMPORTS=${qt6.qtdeclarative}/lib/qt-6/qml
    export KUBAOS_SHOTS_FONTS=${fonts}
    export KUBAOS_SHOTS_EGL_VENDOR=${mesa}/share/glvnd/egl_vendor.d/50_mesa.json
    export KUBAOS_SHOTS_DRI=${mesa}/lib/dri
    export KUBAOS_SHOTS_WALLPAPERS=${wallpapers}
    export KUBAOS_SHOTS_ICONS=${icons}
    export KUBAOS_SHOTS_TERMINAL_FONTS=${terminalFonts}
    export KUBAOS_SHOTS_NVIM_PLUGINS=${nvimPlugins}
    export KUBAOS_SHOTS_TERMINFO=${ncurses}/share/terminfo
    repo=$(git rev-parse --show-toplevel 2>/dev/null) || {
      echo "kubaos-shots: run me inside a KubaOS checkout" >&2
      exit 2
    }
    exec bash "$repo/test/shots/shots" "$@"
  '';
  meta = {
    description = "Before/after screenshots of KubaOS's shell, Kōbō, the specimen and the terminals, diffed";
    mainProgram = "kubaos-shots";
  };
}
