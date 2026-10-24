# KubaOS's right-click actions in Files (Nautilus), loaded by nautilus-python from
# ~/.local/share/nautilus-python/extensions (home/files.nix links it there):
#
# - Copy path: the selected files' paths, a line each, through wl-copy.
# - Convert image: to JPG, PNG or WebP, next to the original, through ImageMagick.
# - Resize image: to half, a quarter, or at most 1920 px, next to the original.
# - Open as administrator: the folder at admin://, which gvfs opens after a polkit prompt.
# - Import photos: a DCIM folder's photos into ~/Pictures/<year>/<month>/, through
#   `kubaos-automount import`, for cards, cameras and phones alike.
#
# "Open in Ghostty" is Ghostty's own extension, linked next to this one.
#
# Every command runs without a shell, and a failure says so in a notification, since Nautilus
# has no place to show it.

from pathlib import Path

from gi.repository import GObject
from gi.repository import Gio
from gi.repository import Nautilus

FORMATS = (("jpg", "JPG"), ("png", "PNG"), ("webp", "WebP"))
SIZES = (("50%", "Half size"), ("25%", "Quarter size"), ("1920x1920>", "At most 1920 px"))
# What a resized copy's name says, by geometry: `photo (50%).jpg`.
SIZE_TAGS = {"50%": "50%", "25%": "25%", "1920x1920>": "1920px"}
SAME_FORMAT = {"jpg": ("jpg", "jpeg"), "png": ("png",), "webp": ("webp",)}


def run(argv, failure):
    """Starts `argv`, and notifies `failure` when it can't start or exits non-zero."""
    try:
        process = Gio.Subprocess.new(argv, Gio.SubprocessFlags.NONE)
    except Exception:
        notify(failure, f"{argv[0]} couldn't start.")
        return

    def done(process, result):
        try:
            ok = process.wait_check_finish(result)
        except Exception:
            ok = False
        if not ok:
            notify(failure, f"{argv[0]} failed.")

    process.wait_check_async(None, done)


def notify(summary, body):
    try:
        Gio.Subprocess.new(
            ["notify-send", "--app-name=Files", "--icon=dialog-error", summary, body],
            Gio.SubprocessFlags.NONE,
        )
    except Exception:
        pass


def local_paths(files):
    paths = []
    for file in files:
        location = file.get_location()
        path = location.get_path() if location else None
        if path:
            paths.append(Path(path))
    return paths


def images(files):
    """The local image files among `files`; none unless every one of them is an image."""
    if not files or not all(f.get_mime_type().startswith("image/") for f in files):
        return []
    paths = local_paths(files)
    return paths if len(paths) == len(files) else []


def free_path(folder, stem, suffix):
    """`stem.suffix` in `folder`, or `stem (2).suffix` and on, so no file is overwritten."""
    candidate = folder / f"{stem}{suffix}"
    n = 2
    while candidate.exists():
        candidate = folder / f"{stem} ({n}){suffix}"
        n += 1
    return candidate


def convert(_item, paths, extension):
    for path in paths:
        if path.suffix.lower().lstrip(".") in SAME_FORMAT[extension]:
            continue
        target = free_path(path.parent, path.stem, f".{extension}")
        run(["magick", str(path), str(target)], f"Couldn't convert {path.name}")


def resize(_item, paths, geometry):
    for path in paths:
        target = free_path(path.parent, f"{path.stem} ({SIZE_TAGS[geometry]})", path.suffix)
        # `-auto-orient` first, so a phone photo keeps its rotation once the EXIF tag is
        # resized away with the rest of the metadata.
        run(
            ["magick", str(path), "-auto-orient", "-resize", geometry, str(target)],
            f"Couldn't resize {path.name}",
        )


def copy_paths(_item, paths):
    text = "\n".join(str(path) for path in paths)
    run(["wl-copy", "--", text], "Couldn't copy the path")


def open_as_admin(_item, folder):
    run(["nautilus", "--new-window", f"admin://{folder}"], "Couldn't open the folder as administrator")


def import_photos(_item, folder):
    run(["kubaos-automount", "import", str(folder)], "Couldn't import the photos")


def dcim_of(folder):
    """`folder` when it is a DCIM folder, or the DCIM folder inside it."""
    if folder.name.lower() == "dcim":
        return folder
    try:
        children = list(folder.iterdir())
    except OSError:
        return None
    return next((child for child in children if child.name.lower() == "dcim" and child.is_dir()), None)


def submenu(name, label, entries, callback, paths):
    item = Nautilus.MenuItem(name=name, label=label)
    menu = Nautilus.Menu()
    item.set_submenu(menu)
    for value, entry_label in entries:
        entry = Nautilus.MenuItem(name=f"{name}::{value}", label=entry_label)
        entry.connect("activate", callback, paths, value)
        menu.append_item(entry)
    return item


def folder_items(prefix, folder):
    items = []
    admin = Nautilus.MenuItem(name=f"{prefix}::admin", label="Open as Administrator")
    admin.connect("activate", open_as_admin, folder)
    items.append(admin)
    dcim = dcim_of(folder)
    if dcim is not None:
        photos = Nautilus.MenuItem(name=f"{prefix}::import", label="Import Photos")
        photos.connect("activate", import_photos, dcim)
        items.append(photos)
    return items


class KubaOSMenuProvider(GObject.GObject, Nautilus.MenuProvider):
    def get_file_items(self, files):
        paths = local_paths(files)
        if not paths:
            return []
        copy = Nautilus.MenuItem(name="KubaOS::copy_path", label="Copy Path")
        copy.connect("activate", copy_paths, paths)
        items = [copy]
        pictures = images(files)
        if pictures:
            items.append(submenu("KubaOS::convert", "Convert Image", FORMATS, convert, pictures))
            items.append(submenu("KubaOS::resize", "Resize Image", SIZES, resize, pictures))
        if len(files) == 1 and files[0].is_directory():
            items.extend(folder_items("KubaOS::folder", paths[0]))
        return items

    def get_background_items(self, folder):
        paths = local_paths([folder])
        return folder_items("KubaOS::background", paths[0]) if paths else []
