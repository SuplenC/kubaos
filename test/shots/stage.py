"""Stages one tree's shell for the shot drivers.

    stage.py TREE HARNESS OUT WALLPAPERS ICONS

TREE holds `config/quickshell` and `qml/KubaOS` (the working tree, or a rev extracted from git).
OUT gets `imports/`, the QML import path the drivers run with:

- the stand-ins from HARNESS/fakes (Quickshell's modules, Kotodama and the shell's services and
  settings), with `@WALLPAPERS@` and `@ICONS@` filled in;
- TREE's `KubaOS.Components`, as it is;
- TREE's theme, components and modules as `qs.theme`, `qs.components` and `qs.modules.<name>`,
  each with a generated qmldir.

What can't run under Qt's own `qml` runtime is changed on the way: `WlrLayershell.*` bindings
(an attached object QML can't stand in for) are dropped, and the files in STAND_INS are replaced
by the harness's own (HARNESS/drivers/stand-ins): no video, no exclusive zones, no PAM.

The drivers in HARNESS/drivers go to OUT/drivers.
"""

import pathlib
import re
import shutil
import sys

LAYERSHELL = re.compile(r"^\s*WlrLayershell\.\w+\s*:.*$\n?", re.M)
# The shell's files the harness replaces, by their path under config/quickshell.
STAND_INS = {
    "components/WallpaperView.qml": "WallpaperView.qml",
    "modules/frame/Exclusions.qml": "Exclusions.qml",
    "modules/lock/Auth.qml": "Auth.qml",
}


def fill(path: pathlib.Path, wallpapers: str, icons: str) -> None:
    text = path.read_text()
    if "@WALLPAPERS@" in text or "@ICONS@" in text:
        path.write_text(text.replace("@WALLPAPERS@", wallpapers).replace("@ICONS@", icons))


def module(name: str, shell: pathlib.Path, rel: str, dest: pathlib.Path, stand_ins: pathlib.Path) -> None:
    """Copies the QML and JS files of `shell/rel` into `dest` as module `name`."""
    dest.mkdir(parents=True, exist_ok=True)
    lines = [f"module {name}"]
    for f in sorted((shell / rel).iterdir()):
        if f.suffix not in (".qml", ".js"):
            continue
        out = dest / f.name
        stand_in = STAND_INS.get(f"{rel}/{f.name}")
        if stand_in:
            shutil.copy(stand_ins / stand_in, out)
        else:
            out.write_text(LAYERSHELL.sub("", f.read_text()))
        if f.suffix == ".qml":
            single = "pragma Singleton" in out.read_text()
            lines.append(f"{'singleton ' if single else ''}{f.stem} 1.0 {f.name}")
    (dest / "qmldir").write_text("\n".join(lines) + "\n")


def main() -> None:
    tree, harness, out = (pathlib.Path(p) for p in sys.argv[1:4])
    wallpapers, icons = sys.argv[4:6]
    shell = tree / "config/quickshell"
    stand_ins = harness / "drivers/stand-ins"
    if out.exists():
        shutil.rmtree(out)
    imports = out / "imports"
    shutil.copytree(harness / "fakes", imports)
    for f in imports.rglob("*.qml"):
        fill(f, wallpapers, icons)
    shutil.copytree(tree / "qml/KubaOS", imports / "KubaOS")
    module("qs.theme", shell, "theme", imports / "qs/theme", stand_ins)
    module("qs.components", shell, "components", imports / "qs/components", stand_ins)
    for d in sorted((shell / "modules").iterdir()):
        if d.is_dir():
            module(f"qs.modules.{d.name}", shell, f"modules/{d.name}", imports / "qs/modules" / d.name, stand_ins)
    drivers = out / "drivers"
    drivers.mkdir()
    for f in (harness / "drivers").glob("*.qml"):
        shutil.copy(f, drivers)
        fill(drivers / f.name, wallpapers, icons)


if __name__ == "__main__":
    main()
