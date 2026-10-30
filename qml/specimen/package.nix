# `qml-specimen`: the QML specimen (qml/specimen/) laid out headless with Qt's
# `qml` runtime, the Qt Quickshell loads, at every supported scale, light and
# dark. The fonts are kubaos-ui's own and nothing else, so a page measures the
# same on every machine and in the build sandbox.
#
#   qml-specimen frames DIR   each frame as DIR/components-MODE-SCALEx.json
#   qml-specimen shots DIR    each page as DIR/components-MODE-SCALEx.png
#
# Either fails when one pass over the page arranges a layout more than twice.
#
# `kubaos-specimen parity DIR` holds the frames to the iced page's boxes
# (crates/checks.nix, design-parity).
{
  lib,
  qt6,
  writeShellApplication,
  writeText,
  kubaos-components,
}:
let
  specimen = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./main.qml
      ./Components.qml
      ./ShellOnly.qml
      ./sections
      ./shell
    ];
  };
  # HAZARD: nixpkgs' `makeFontsConf` also lists the user's and the system's font directories and
  # /etc/fonts/conf.d. Outside the sandbox Qt then draws mono text in another, proportional font
  # from those, so the frames and the screenshots differ from iced's.
  fonts = writeText "kubaos-ui-fonts.conf" ''
    <?xml version="1.0"?>
    <!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
    <fontconfig>
      <dir>${../../crates/kubaos-ui/assets/fonts}</dir>
    </fontconfig>
  '';
  plugins = lib.makeSearchPath "lib/qt-6/plugins" [
    qt6.qtbase
    qt6.qtsvg
    qt6.qtdeclarative
  ];
  imports = lib.makeSearchPath "lib/qt-6/qml" [
    qt6.qtdeclarative
    kubaos-components
  ];
in
writeShellApplication {
  name = "qml-specimen";
  text = ''
    usage() {
      echo "usage: qml-specimen frames DIR | qml-specimen shots DIR" >&2
      exit 2
    }
    [[ $# -eq 2 ]] || usage
    what=$1
    dir=$2
    [[ $what == frames || $what == shots ]] || usage
    mkdir -p "$dir"
    export QT_QPA_PLATFORM=offscreen
    export QT_PLUGIN_PATH=${plugins}
    export NIXPKGS_QT6_QML_IMPORT_PATH=${imports}
    export FONTCONFIG_FILE=${fonts}
    export QT_FORCE_STDERR_LOGGING=1
    export HOME=''${HOME:-$(mktemp -d)}
    export XDG_RUNTIME_DIR=''${XDG_RUNTIME_DIR:-$(mktemp -d)}
    # The desktop's own Qt settings (a platform theme, a style, its scaling, its QML imports)
    # would change what is laid out.
    unset WAYLAND_DISPLAY DISPLAY QML_IMPORT_PATH QML2_IMPORT_PATH QT_QPA_PLATFORMTHEME \
      QT_STYLE_OVERRIDE QT_QUICK_CONTROLS_STYLE QT_AUTO_SCREEN_SCALE_FACTOR QT_SCREEN_SCALE_FACTORS \
      QT_ENABLE_HIGHDPI_SCALING QT_FONT_DPI
    for scale in 1 1.25 1.5 2; do
      for mode in dark light; do
        name="components-$mode-''${scale}x"
        shot=""
        [[ $what == shots ]] && shot="$dir/$name.png"
        log=$(QT_SCALE_FACTOR=$scale ${qt6.qtdeclarative}/bin/qml ${specimen}/main.qml -- 1280 "$mode" "$shot" 2>&1 || true)
        frame=$(sed -n 's/^.*frame {/{/p' <<<"$log")
        if [[ -z $frame ]]; then
          echo "qml-specimen: $name reported no frame:" >&2
          echo "$log" >&2
          exit 1
        fi
        # One pass over the page arranges each layout once; twice is the most a layout may take,
        # when a child learns only as it lays out that it fills.
        passes=""
        layouts=""
        read -r passes layouts < <(sed -n 's/^.*passes \([0-9]*\) layouts \([0-9]*\).*$/\1 \2/p' <<<"$log") || true
        if [[ -z $passes || $passes -gt $((2 * layouts)) ]]; then
          echo "qml-specimen: $name took ''${passes:-no} arrangements in one pass over ''${layouts:-its} layouts" >&2
          exit 1
        fi
        if [[ $what == frames ]]; then
          echo "$frame" >"$dir/$name.json"
          echo "$dir/$name.json"
        else
          echo "$shot"
        fi
      done
    done
  '';
}
