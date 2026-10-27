# `KubaOS.Components`, the QML component library, as a QML module on Qt's
# import path (lib/qt-6/qml/KubaOS/Components): Quickshell loads it through
# `withModules` (home/curated.nix), so the shell and the Command menu
# `import KubaOS.Components`. Quickshell's wrapper puts this ahead of any
# `QML_IMPORT_PATH`, so trying a clone's library in the shell is a home switch
# onto the clone (config/quickshell/README.md, "Theme").
{
  lib,
  stdenvNoCC,
}:
stdenvNoCC.mkDerivation {
  pname = "kubaos-components";
  version = "0.1.0";
  src = lib.fileset.toSource {
    root = ./.;
    fileset = ./KubaOS;
  };
  installPhase = ''
    runHook preInstall
    mkdir -p $out/lib/qt-6/qml
    cp -r KubaOS $out/lib/qt-6/qml/
    runHook postInstall
  '';
  meta = {
    description = "KubaOS.Components, the QML component library of KubaOS's design language";
    license = with lib.licenses; [
      mit
      asl20
    ];
    platforms = lib.platforms.all;
  };
}
