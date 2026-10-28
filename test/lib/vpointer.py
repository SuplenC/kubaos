"""A pointer for the harness's headless sway, which has no input devices.

  python3 -I vpointer.py WAYLAND_XML VIRTUAL_POINTER_XML WIDTH HEIGHT STEP...

The XML files are wayland.xml and wlr-virtual-pointer-unstable-v1.xml, which
pywayland's scanner turns into Python first. WIDTH and HEIGHT are the output's,
in logical pixels. Each STEP is `move:X,Y` (to a point on the output), `click`
(the left button), `press` and `release` (it held down for a drag, the moves
between them dragging) or `wait:SECONDS`. One virtual pointer lasts for all the
steps: sway gives the seat a pointer only while a pointer device exists, and
the shell binds it a moment after, so a pointer made per click would miss its
own clicks.
"""

import os
import shutil
import subprocess
import sys
import tempfile
import time

BTN_LEFT = 0x110
PRESSED = 1
RELEASED = 0
# Time for the seat to gain its pointer and Quickshell to bind it before the
# first step. wlrctl, which makes a pointer per command and clicks at once,
# lost every click.
SETTLE_S = 0.5


def main(argv: list[str]) -> int:
    if len(argv) < 5:
        print(__doc__, file=sys.stderr)
        return 2
    out = tempfile.mkdtemp(prefix="vpointer-")
    # The generated modules import each other relatively, so they go in a package.
    package = os.path.join(out, "protocols")
    subprocess.run([sys.executable, "-m", "pywayland.scanner", "-i", argv[0], argv[1], "-o", package],
                   check=True, capture_output=True)
    open(os.path.join(package, "__init__.py"), "w").close()
    sys.path.insert(0, out)
    from pywayland.client import Display
    from protocols.wayland import WlSeat
    from protocols.wlr_virtual_pointer_unstable_v1 import ZwlrVirtualPointerManagerV1
    shutil.rmtree(out)

    width, height = int(argv[2]), int(argv[3])
    display = Display()
    display.connect()
    found = {}

    def global_added(registry, name, interface, version):
        if interface == "wl_seat" and "seat" not in found:
            found["seat"] = registry.bind(name, WlSeat, 1)
        elif interface == "zwlr_virtual_pointer_manager_v1":
            found["manager"] = registry.bind(name, ZwlrVirtualPointerManagerV1, 1)

    registry = display.get_registry()
    registry.dispatcher["global"] = global_added
    display.roundtrip()
    if "manager" not in found:
        print("vpointer: the compositor has no zwlr_virtual_pointer_manager_v1", file=sys.stderr)
        return 1
    pointer = found["manager"].create_virtual_pointer(found.get("seat"))
    display.roundtrip()
    time.sleep(SETTLE_S)

    def stamp() -> int:
        return int(time.monotonic() * 1000) & 0xFFFFFFFF

    def button(state: int) -> None:
        pointer.button(stamp(), BTN_LEFT, state)
        pointer.frame()
        display.flush()

    for step in argv[4:]:
        verb, _, arg = step.partition(":")
        if verb == "move":
            x, y = (int(v) for v in arg.split(","))
            pointer.motion_absolute(stamp(), x, y, width, height)
            pointer.frame()
            display.flush()
        elif verb == "click":
            button(PRESSED)
            time.sleep(0.03)
            button(RELEASED)
        elif verb == "press":
            button(PRESSED)
        elif verb == "release":
            button(RELEASED)
        elif verb == "wait":
            display.roundtrip()
            time.sleep(float(arg))
        else:
            print(f"vpointer: unknown step {step}", file=sys.stderr)
            return 2
    display.roundtrip()
    pointer.destroy()
    display.roundtrip()
    display.disconnect()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
