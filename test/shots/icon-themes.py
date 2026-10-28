"""The icon theme comparison sheet: the same 24 icons in each candidate GTK and Qt icon theme.

    icon-themes.py REPO OUT ROOT...

Draws OUT/icon-themes-dark.png and OUT/icon-themes-light.png at 2× (one logical pixel is two):
a row per theme, 12 colour icons at 48 (places, files, apps) then 12 symbolic ones at 16 drawn
in `on_surface` as GTK recolours them, on the tokens' `surface` of that mode, with Geist from
`kubaos-ui`. Each ROOT is a `share/icons` directory; an icon is looked up the way GTK does
(the theme, then what it inherits, then hicolor), and one that only an inherited theme has gets
a dot under it in `outline`, so a theme's own coverage shows. `rsvg-convert` draws the SVGs.
`test/shots/icon-themes` builds the themes and runs it.
"""

import configparser
import pathlib
import subprocess
import sys
import tempfile
import tomllib

from PIL import Image, ImageDraw, ImageFont

SCALE = 2
# The candidates: their name on the sheet, the theme in dark and in light, and a note.
THEMES = [
    ("Papirus (today)", "Papirus-Dark", "Papirus-Light"),
    ("Adwaita", "Adwaita", "Adwaita"),
    ("MoreWaita", "MoreWaita", "MoreWaita"),
    ("Colloid orange", "Colloid-Orange-Dark", "Colloid-Orange-Light"),
    ("Fluent orange", "Fluent-orange-dark", "Fluent-orange-light"),
    ("Tela orange", "Tela-orange-dark", "Tela-orange-light"),
]
COLOUR = [
    ("folder", "folder"),
    ("folder-download", "downloads"),
    ("user-home", "home"),
    ("user-trash", "trash"),
    ("text-x-generic", "text"),
    ("image-x-generic", "image"),
    ("application-pdf", "PDF"),
    ("package-x-generic", "package"),
    ("org.gnome.Nautilus", "Files"),
    ("org.gnome.Settings", "Settings"),
    ("com.mitchellh.ghostty", "Ghostty"),
    ("firefox", "Firefox"),
]
SYMBOLIC = [
    ("edit-find-symbolic", "search"),
    ("go-home-symbolic", "home"),
    ("list-add-symbolic", "add"),
    ("window-close-symbolic", "close"),
    ("open-menu-symbolic", "menu"),
    ("document-open-symbolic", "open"),
    ("network-wireless-signal-good-symbolic", "Wi-Fi"),
    ("audio-volume-high-symbolic", "volume"),
    ("battery-good-symbolic", "battery"),
    ("bluetooth-active-symbolic", "Bluetooth"),
    ("emblem-system-symbolic", "system"),
    ("user-trash-symbolic", "trash"),
]
# Lengths in logical pixels, from the tokens' scale: a 48 icon and a 16 one, each in a 64 cell.
CELL = 64
LABEL = 160
PAD = 24


class Theme:
    """An icon theme's directories and what it inherits, from its index.theme."""

    def __init__(self, roots: list[pathlib.Path], name: str):
        self.name = name
        self.base = next((r / name for r in roots if (r / name / "index.theme").exists()), None)
        self.dirs: list[tuple[pathlib.Path, dict]] = []
        self.inherits: list[str] = []
        if not self.base:
            return
        index = configparser.ConfigParser(interpolation=None, strict=False)
        index.read(self.base / "index.theme", encoding="utf-8")
        main = index["Icon Theme"]
        self.inherits = [t.strip() for t in main.get("Inherits", "").split(",") if t.strip()]
        for d in main.get("Directories", "").split(",") + main.get("ScaledDirectories", "").split(","):
            d = d.strip()
            if d and index.has_section(d):
                self.dirs.append((self.base / d, dict(index[d])))

    def find(self, icon: str, size: int) -> pathlib.Path | None:
        best, distance = None, None
        for path, info in self.dirs:
            for ext in ("svg", "png"):
                file = path / f"{icon}.{ext}"
                if not file.exists():
                    continue
                kind = info.get("type", "Threshold").lower()
                nominal = int(info.get("size", "0"))
                if kind == "scalable":
                    low, high = int(info.get("minsize", nominal)), int(info.get("maxsize", nominal))
                    d = 0 if low <= size <= high else min(abs(size - low), abs(size - high))
                else:
                    d = abs(nominal - size)
                if ext == "svg":
                    d -= 0.5
                if distance is None or d < distance:
                    best, distance = file, d
        return best


def lookup(roots, name, icon, size, seen=None):
    """`icon` in theme `name` or what it inherits: (file, whether the theme itself has it)."""
    seen = seen or set()
    chain = [name]
    found = None
    while chain:
        current = chain.pop(0)
        if current in seen:
            continue
        seen.add(current)
        theme = Theme(roots, current)
        file = theme.find(icon, size)
        if file:
            found = file, current == name
            break
        chain.extend(theme.inherits or (["hicolor"] if current != "hicolor" else []))
    return found


def raster(file: pathlib.Path, px: int, tmp: pathlib.Path) -> Image.Image:
    if file.suffix == ".svg":
        out = tmp / "icon.png"
        subprocess.run(["rsvg-convert", "-w", str(px), "-h", str(px), "-o", out, file], check=True)
        return Image.open(out).convert("RGBA")
    return Image.open(file).convert("RGBA").resize((px, px), Image.LANCZOS)


def tint(icon: Image.Image, rgb) -> Image.Image:
    solid = Image.new("RGBA", icon.size, rgb + (255,))
    solid.putalpha(icon.getchannel("A"))
    return solid


def hex_rgb(value: str):
    return tuple(int(value[i : i + 2], 16) for i in (1, 3, 5))


def sheet(repo: pathlib.Path, roots, mode: str, out: pathlib.Path) -> None:
    tokens = tomllib.loads((repo / "design/tokens.toml").read_text())
    palette = {k: hex_rgb(v) for k, v in tokens["color"][mode].items()}
    fonts = repo / "crates/kubaos-ui/assets/fonts"
    body = ImageFont.truetype(str(fonts / "Geist-Medium.ttf"), 14 * SCALE)
    caption = ImageFont.truetype(str(fonts / "Geist-Regular.ttf"), 11 * SCALE)
    columns = len(COLOUR) + len(SYMBOLIC)
    width = PAD * 2 + LABEL + columns * CELL + PAD
    height = PAD * 2 + 32 + len(THEMES) * (CELL + 8)
    image = Image.new("RGB", (width * SCALE, height * SCALE), palette["surface"])
    draw = ImageDraw.Draw(image)

    def x_of(column):
        gap = PAD if column >= len(COLOUR) else 0
        return PAD + LABEL + column * CELL + gap

    for column, (_, word) in enumerate(COLOUR + SYMBOLIC):
        x = (x_of(column) + CELL / 2) * SCALE
        draw.text((x, PAD * SCALE), word, font=caption, fill=palette["on_surface_variant"], anchor="mt")
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        for row, (label, dark, light) in enumerate(THEMES):
            theme = dark if mode == "dark" else light
            top = PAD + 32 + row * (CELL + 8)
            draw.text(
                (PAD * SCALE, (top + CELL / 2) * SCALE), label, font=body, fill=palette["on_surface"], anchor="lm"
            )
            for column, (icon, _) in enumerate(COLOUR + SYMBOLIC):
                symbolic = column >= len(COLOUR)
                size = 16 if symbolic else 48
                found = lookup(roots, theme, icon, size)
                cx, cy = x_of(column) + CELL / 2, top + CELL / 2
                if not found:
                    draw.text((cx * SCALE, cy * SCALE), "–", font=body, fill=palette["outline"], anchor="mm")
                    continue
                file, own = found
                picture = raster(file, size * SCALE, tmp)
                if symbolic:
                    picture = tint(picture, palette["on_surface"])
                at = (int((cx - size / 2) * SCALE), int((cy - size / 2) * SCALE))
                image.paste(picture, at, picture)
                if not own:
                    r = 2 * SCALE
                    y = (top + CELL - 4) * SCALE
                    draw.ellipse((cx * SCALE - r, y - r, cx * SCALE + r, y + r), fill=palette["outline"])
    image.save(out / f"icon-themes-{mode}.png", optimize=True)


def main() -> None:
    repo, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    roots = [pathlib.Path(r) for r in sys.argv[3:]]
    out.mkdir(parents=True, exist_ok=True)
    for mode in ("dark", "light"):
        sheet(repo, roots, mode, out)


if __name__ == "__main__":
    main()
