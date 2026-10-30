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
# font before they took the tokens'), Noto Color Emoji and DejaVu; and fuzzel's icons, the curated
# Colloid and Papirus, the icon theme before it.
#
# The mockups (test/shots/mockups) are drawn by headless Chromium, nixpkgs' Playwright build, which
# the site's playwright-core of the same version drives, with Node to build and run them.
#
# The boot screens (test/shots/boot) take plymouthd with its X11 renderer in Xvfb, sandboxed by
# bubblewrap; GRUB under OVMF in QEMU; and Bibata's sources at the release `home/theme.nix` pins,
# for the pointer.
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
  fetchFromGitHub,
  plymouth,
  xvfb,
  xdotool,
  xwd,
  bubblewrap,
  fontconfig,
  qemu_kvm,
  grub2_efi,
  OVMF,
  xorriso,
  mtools,
  socat,
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
  util-linux,
  wallpapers,
  geist-font,
  colloid-orange,
  papirus-icon-theme,
  hicolor-icon-theme,
  nodejs_24,
  playwright-driver,
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
  # Bibata's sources, as `home/theme.nix` pins them for `kubaos cursor`.
  bibata = fetchFromGitHub {
    owner = "ful1e5";
    repo = "Bibata_Cursor";
    rev = "v2.0.7";
    hash = "sha256-kIKidw1vditpuxO1gVuZeUPdWBzkiksO/q2R/+DUdEc=";
  };
  python = python3.withPackages (p: [ p.pillow ]);
  tools = [
    qt6.qtdeclarative
    sway-unwrapped
    imagemagick
    python
    qemu_kvm
    grub2_efi
    xorriso
    mtools
    socat
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
    util-linux
    nodejs_24
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
    export KUBAOS_SHOTS_ICON_THEMES=${
      lib.makeSearchPath "share" [
        colloid-orange
        papirus-icon-theme
        hicolor-icon-theme
      ]
    }
    export KUBAOS_SHOTS_PLYMOUTH=${plymouth}
    export KUBAOS_SHOTS_XVFB=${xvfb}/bin/Xvfb
    export KUBAOS_SHOTS_XDOTOOL=${xdotool}/bin/xdotool
    export KUBAOS_SHOTS_XWD=${xwd}/bin/xwd
    export KUBAOS_SHOTS_BWRAP=${bubblewrap}/bin/bwrap
    # label-freetype asks this fc-match for a font; the initramfs has none, so the sandbox hides it.
    export KUBAOS_SHOTS_FCMATCH=${fontconfig}/bin/fc-match
    export KUBAOS_SHOTS_SH=${bash}/bin/bash
    export KUBAOS_SHOTS_SANDBOX_PATH=${
      lib.makeBinPath [
        coreutils
        bash
      ]
    }
    export KUBAOS_SHOTS_GRUB=${grub2_efi}
    export KUBAOS_SHOTS_OVMF=${OVMF.fd}/FV/OVMF.fd
    export KUBAOS_SHOTS_BIBATA=${bibata}
    export KUBAOS_SHOTS_PLAYWRIGHT=${playwright-driver.browsers-chromium}
    repo=$(git rev-parse --show-toplevel 2>/dev/null) || {
      echo "kubaos-shots: run me inside a KubaOS checkout" >&2
      exit 2
    }
    exec bash "$repo/test/shots/shots" "$@"
  '';
  meta = {
    description = "Before/after screenshots of KubaOS's shell, Kōbō, the specimen, the terminals and the boot screens, diffed";
    mainProgram = "kubaos-shots";
  };
}
