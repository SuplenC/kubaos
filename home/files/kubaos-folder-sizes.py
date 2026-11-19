# The Total Size column in Files' (Nautilus's) list view: a file's size, and for a folder the
# size of everything inside it, where Nautilus's own Size column only counts a folder's items.
# nautilus-python loads it from ~/.local/share/nautilus-python/extensions (home/files.nix links
# it there, and puts the column in place of Size).
#
# `du` adds a folder up, in a process of its own at idle CPU and disk priority, so a big tree
# neither holds Python's lock against Nautilus's window nor slows the desktop. Two run at a time;
# leaving the folder stops its walks. A total is kept while the folder's own modification time
# stays the same, for a while, so going back to a folder costs nothing.
#
# Only folders on this machine are added up: a network share or a phone would be walked over the
# wire. `du` stays on the folder's own filesystem, as a mount inside it isn't part of its size.
#
# `kubaos-files folder-sizes off` writes ~/.config/kubaos/desktop/files/no-folder-sizes; while it
# exists, a folder shows its item count, as Nautilus does.

import os
import time
from collections import OrderedDict
from pathlib import Path

from gi.repository import GLib
from gi.repository import GObject
from gi.repository import Gio
from gi.repository import Nautilus

ATTRIBUTE = "kubaos_total_size"
CONFIG = Path(os.environ.get("XDG_CONFIG_HOME") or Path.home() / ".config")
OFF = CONFIG / "kubaos" / "desktop" / "files" / "no-folder-sizes"
RUNNING_AT_ONCE = 2
# Long enough that going back and forth costs nothing, short enough that a change deep inside a
# folder, which leaves the folder's own modification time alone, shows before long.
KEPT_FOR = 600
KEPT_AT_MOST = 4096
REMOTE = ("/run/user/", "/proc/", "/sys/")


def local_path(file):
    if file.get_uri_scheme() != "file":
        return None
    location = file.get_location()
    path = location.get_path() if location else None
    if not path or path.startswith(REMOTE):
        return None
    return path


def item_count(path):
    try:
        count = sum(1 for _ in os.scandir(path))
    except OSError:
        return ""
    return "1 item" if count == 1 else f"{count} items"


class Walk:
    """One `du` over a folder, for the request `handle`."""

    def __init__(self, provider, handle, closure, file, path, modified):
        self.provider = provider
        self.handle = handle
        self.closure = closure
        self.file = file
        self.path = path
        self.modified = modified
        self.process = None
        self.cancelled = False


class KubaOSFolderSizes(GObject.GObject, Nautilus.ColumnProvider, Nautilus.InfoProvider):
    def __init__(self):
        super().__init__()
        self.waiting = []
        self.running = []
        # Path to (modification time, size, when it was added up), oldest first.
        self.sizes = OrderedDict()

    def get_columns(self):
        return [
            Nautilus.Column(
                name="KubaOS::total_size",
                attribute=ATTRIBUTE,
                label="Total Size",
                description="The size of a file, or of everything inside a folder",
                xalign=1.0,
            )
        ]

    def update_file_info_full(self, provider, handle, closure, file):
        path = local_path(file)
        if path is None:
            return Nautilus.OperationResult.COMPLETE
        try:
            stat = os.stat(path) if file.is_directory() else os.lstat(path)
        except OSError:
            return Nautilus.OperationResult.COMPLETE
        if not file.is_directory():
            file.add_string_attribute(ATTRIBUTE, GLib.format_size(stat.st_size))
            return Nautilus.OperationResult.COMPLETE
        if OFF.exists():
            file.add_string_attribute(ATTRIBUTE, item_count(path))
            return Nautilus.OperationResult.COMPLETE
        known = self.sizes.get(path)
        if known and known[0] == stat.st_mtime_ns and time.monotonic() - known[2] < KEPT_FOR:
            file.add_string_attribute(ATTRIBUTE, GLib.format_size(known[1]))
            return Nautilus.OperationResult.COMPLETE
        self.waiting.append(Walk(provider, handle, closure, file, path, stat.st_mtime_ns))
        self.start_next()
        return Nautilus.OperationResult.IN_PROGRESS

    def cancel_update(self, provider, handle):
        for walk in self.waiting + self.running:
            if walk.handle == handle:
                walk.cancelled = True
                if walk.process is not None:
                    walk.process.force_exit()
        self.waiting = [walk for walk in self.waiting if not walk.cancelled]

    def start_next(self):
        while self.waiting and len(self.running) < RUNNING_AT_ONCE:
            walk = self.waiting.pop(0)
            argv = ["nice", "-n", "19", "ionice", "-c", "3", "du", "-sbx", "--", walk.path]
            try:
                walk.process = Gio.Subprocess.new(
                    argv, Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_SILENCE
                )
            except GLib.Error:
                self.finish(walk, None)
                continue
            self.running.append(walk)
            walk.process.communicate_utf8_async(None, None, self.walked, walk)

    def walked(self, process, result, walk):
        try:
            _, out, _ = process.communicate_utf8_finish(result)
        except GLib.Error:
            out = None
        self.running.remove(walk)
        # `du` exits 1 when a part of the tree can't be read, and still prints the total of
        # the rest, which is the most the user can see of the folder.
        size = None
        if out and not walk.cancelled:
            try:
                size = int(out.split("\t", 1)[0])
            except ValueError:
                size = None
        self.finish(walk, size)
        self.start_next()

    def finish(self, walk, size):
        if walk.cancelled:
            return
        if size is not None:
            self.remember(walk.path, walk.modified, size)
            walk.file.add_string_attribute(ATTRIBUTE, GLib.format_size(size))
        outcome = Nautilus.OperationResult.FAILED if size is None else Nautilus.OperationResult.COMPLETE
        Nautilus.info_provider_update_complete_invoke(walk.closure, walk.provider, walk.handle, outcome)

    def remember(self, path, modified, size):
        self.sizes.pop(path, None)
        self.sizes[path] = (modified, size, time.monotonic())
        while len(self.sizes) > KEPT_AT_MOST:
            self.sizes.popitem(last=False)
