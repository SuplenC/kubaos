"""Gives the bundled Material Symbols subset its own family name.

The installed system has Nix's full Material Symbols Rounded, whose
codepoints don't all match the subset's (0xe668 is a crossed-out eye
there, not a check). With the same family name the system font shadowed
ours; with its own name it can't. Needs fontTools (pip install fonttools).

    python3 rename-icon-font.py fonts/MaterialSymbolsRounded-subset.ttf
"""
import sys

from fontTools.ttLib import TTFont

FAMILY = "KubaOS Symbols"
path = sys.argv[1]
f = TTFont(path)
names = {
    1: FAMILY,
    3: "KubaOSSymbols-Regular (subset of Material Symbols Rounded)",
    4: FAMILY + " Regular",
    6: "KubaOSSymbols-Regular",
    16: FAMILY,
}
table = f["name"]
for r in list(table.names):
    if r.nameID in names:
        table.setName(names[r.nameID], r.nameID, r.platformID, r.platEncID, r.langID)
f.save(path)
print(f"{path}: family is now {FAMILY!r}")
