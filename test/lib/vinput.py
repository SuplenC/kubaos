"""A keyboard and a pointer for the harness's headless sway, which has no input devices.

  python3 -I vinput.py serve SOCKET WAYLAND_XML KEYBOARD_XML POINTER_XML WIDTH HEIGHT
  python3 -I vinput.py send SOCKET [--delay MS] STEP...

`serve` makes one virtual keyboard and one virtual pointer and keeps them until
it is killed, taking steps from `send` on the Unix socket SOCKET. `send`
returns once the compositor has taken every step's events. The XML files are
wayland.xml, virtual-keyboard-unstable-v1.xml and
wlr-virtual-pointer-unstable-v1.xml, which pywayland's scanner turns into
Python first. WIDTH and HEIGHT are the output's, in logical pixels.

A STEP is one of:

  type:TEXT     each character of TEXT, pressed and released
  key:NAME      a key by its keysym's name (Escape, Return, BackSpace, Down, Prior)
  down:MOD      hold a modifier (ctrl, shift, alt, logo); up:MOD lets it go
  move:X,Y      the pointer to a point on the output
  click         the left button; press and release hold it for a drag
  wait:SECONDS  a pause, so a drag's moves come as a hand makes them

--delay MS waits between the steps, and between the keys of a `type`, as
someone typing does.

Start `serve` before the shell. The devices then exist when it binds the seat:
it gets the pointer and the keyboard's one keymap at once, and no event can
reach it before it is ready for them. A device made per command makes the
seat change under the shell, which loses what comes before it catches up: a
new keyboard's first key, a new pointer's first click.
"""

import os
import shutil
import socket
import subprocess
import sys
import tempfile
import time

BTN_LEFT = 0x110
PRESSED = 1
RELEASED = 0
KEYMAP_FORMAT_XKB_V1 = 1
# The modifiers' masks in xkb's "complete" compatibility map.
MODIFIERS = {"shift": 1, "ctrl": 4, "alt": 8, "logo": 64}
# What `type` can type, as keysym names: each has a keycode of its own, so no
# modifier is needed for any of them.
CHARACTERS = {
    **{c: c for c in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"},
    " ": "space", "'": "apostrophe", "-": "minus", ".": "period", ",": "comma", "/": "slash",
    ":": "colon", "?": "question", "!": "exclam", "@": "at", "_": "underscore",
}
KEYS = ["Escape", "Return", "BackSpace", "Tab", "Delete", "Up", "Down", "Left", "Right", "Prior", "Next",
        "Home", "End"]
KEYSYMS = sorted(set(CHARACTERS.values())) + KEYS
# The evdev code of each keysym: xkb's keycode less 8.
CODES = {name: i + 1 for i, name in enumerate(KEYSYMS)}


def keymap() -> bytes:
    """A keymap with a keycode per keysym of KEYSYMS, as wtype writes one."""
    lines = ["xkb_keymap {", 'xkb_keycodes "(unnamed)" {', "minimum = 8;", f"maximum = {len(KEYSYMS) + 9};"]
    lines += [f"<K{code}> = {code + 8};" for code in CODES.values()]
    lines += ["};", 'xkb_types "(unnamed)" { include "complete" };',
              'xkb_compatibility "(unnamed)" { include "complete" };', 'xkb_symbols "(unnamed)" {']
    lines += [f"key <K{code}> {{[{name}]}};" for name, code in CODES.items()]
    lines += ["};", "};"]
    return ("\n".join(lines) + "\n").encode() + b"\0"


def protocols(xml: list[str]):
    """The generated protocol modules, imported."""
    out = tempfile.mkdtemp(prefix="vinput-")
    # The generated modules import each other relatively, so they go in a package.
    package = os.path.join(out, "protocols")
    subprocess.run([sys.executable, "-m", "pywayland.scanner", "-i", *xml, "-o", package],
                   check=True, capture_output=True)
    open(os.path.join(package, "__init__.py"), "w").close()
    sys.path.insert(0, out)
    from protocols.virtual_keyboard_unstable_v1 import ZwpVirtualKeyboardManagerV1
    from protocols.wayland import WlSeat
    from protocols.wlr_virtual_pointer_unstable_v1 import ZwlrVirtualPointerManagerV1
    shutil.rmtree(out)
    return WlSeat, ZwpVirtualKeyboardManagerV1, ZwlrVirtualPointerManagerV1


class Devices:
    def __init__(self, xml: list[str], width: int, height: int):
        from pywayland.client import Display
        seat_type, keyboards_type, pointers_type = protocols(xml)
        self.width, self.height = width, height
        self.display = Display()
        self.display.connect()
        found = {}

        def global_added(registry, name, interface, version):
            if interface == "wl_seat" and "seat" not in found:
                found["seat"] = registry.bind(name, seat_type, 1)
            elif interface == "zwp_virtual_keyboard_manager_v1":
                found["keyboards"] = registry.bind(name, keyboards_type, 1)
            elif interface == "zwlr_virtual_pointer_manager_v1":
                found["pointers"] = registry.bind(name, pointers_type, 1)

        registry = self.display.get_registry()
        registry.dispatcher["global"] = global_added
        self.display.roundtrip()
        missing = {"seat", "keyboards", "pointers"} - found.keys()
        if missing:
            raise SystemExit(f"vinput: the compositor lacks {', '.join(sorted(missing))}")
        self.keyboard = found["keyboards"].create_virtual_keyboard(found["seat"])
        data = keymap()
        fd = os.memfd_create("vinput-keymap")
        os.write(fd, data)
        self.keyboard.keymap(KEYMAP_FORMAT_XKB_V1, fd, len(data))
        self.display.roundtrip()
        os.close(fd)
        self.pointer = found["pointers"].create_virtual_pointer(found["seat"])
        self.mods = 0
        self.display.roundtrip()

    def release_modifiers(self) -> None:
        self.mods = 0
        self.keyboard.modifiers(0, 0, 0, 0)
        self.display.roundtrip()

    @staticmethod
    def stamp() -> int:
        return int(time.monotonic() * 1000) & 0xFFFFFFFF

    def tap(self, code: int) -> None:
        for state in (PRESSED, RELEASED):
            self.keyboard.key(self.stamp(), code, state)
            self.display.roundtrip()

    def button(self, state: int) -> None:
        self.pointer.button(self.stamp(), BTN_LEFT, state)
        self.pointer.frame()
        self.display.roundtrip()

    def step(self, step: str, delay: float) -> None:
        verb, _, arg = step.partition(":")
        if verb == "type":
            for i, char in enumerate(arg):
                if char not in CHARACTERS:
                    raise ValueError(f"vinput: no key for {char!r}")
                if i:
                    time.sleep(delay)
                self.tap(CODES[CHARACTERS[char]])
        elif verb == "key":
            if arg not in CODES:
                raise ValueError(f"vinput: no key {arg}")
            self.tap(CODES[arg])
        elif verb in ("down", "up"):
            if arg not in MODIFIERS:
                raise ValueError(f"vinput: no modifier {arg}")
            self.mods = self.mods | MODIFIERS[arg] if verb == "down" else self.mods & ~MODIFIERS[arg]
            self.keyboard.modifiers(self.mods, 0, 0, 0)
            self.display.roundtrip()
        elif verb == "move":
            x, y = (int(v) for v in arg.split(","))
            self.pointer.motion_absolute(self.stamp(), x, y, self.width, self.height)
            self.pointer.frame()
            self.display.roundtrip()
        elif verb == "click":
            self.button(PRESSED)
            self.button(RELEASED)
        elif verb == "press":
            self.button(PRESSED)
        elif verb == "release":
            self.button(RELEASED)
        elif verb == "wait":
            time.sleep(float(arg))
        else:
            raise ValueError(f"vinput: unknown step {step}")


def serve(path: str, xml: list[str], width: int, height: int) -> int:
    devices = Devices(xml, width, height)
    if os.path.exists(path):
        os.unlink(path)
    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    # Listening before the socket shows, so a sender that sees it can connect.
    tmp = f"{path}.new"
    if os.path.exists(tmp):
        os.unlink(tmp)
    server.bind(tmp)
    server.listen()
    os.rename(tmp, path)
    while True:
        conn, _ = server.accept()
        with conn, conn.makefile("rw") as stream:
            delay, *steps = stream.readline().rstrip("\n").split("\0")
            try:
                for i, step in enumerate(steps):
                    if i:
                        time.sleep(float(delay) / 1000)
                    devices.step(step, float(delay) / 1000)
                stream.write("ok\n")
            except ValueError as e:
                devices.release_modifiers()
                stream.write(f"{e}\n")
            except OSError:
                # The sender went away mid-way: the next one starts from no modifier held.
                devices.release_modifiers()


def send(path: str, args: list[str]) -> int:
    delay = "0"
    if args[:1] == ["--delay"]:
        delay, args = args[1], args[2:]
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as conn:
        conn.connect(path)
        with conn.makefile("rw") as stream:
            stream.write("\0".join([delay, *args]) + "\n")
            stream.flush()
            reply = stream.readline().strip()
    if reply != "ok":
        print(reply or "vinput: the server closed without an answer", file=sys.stderr)
        return 1
    return 0


def main(argv: list[str]) -> int:
    if len(argv) >= 7 and argv[0] == "serve":
        return serve(argv[1], argv[2:5], int(argv[5]), int(argv[6]))
    if len(argv) >= 3 and argv[0] == "send":
        return send(argv[1], argv[2:])
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
