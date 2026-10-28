"""A stand-in for Hyprland's two IPC sockets that plays two screens plugged in.

  python3 -I hyprland-hotplug.py RUNTIME_DIR SIGNATURE STATE_DIR QS_LOG

It serves `.socket.sock` (requests) and `.socket2.sock` (events) under
RUNTIME_DIR/hypr/SIGNATURE for a Quickshell started with
HYPRLAND_INSTANCE_SIGNATURE=SIGNATURE and KUBAOS_WORKSPACES_PROBE=1, and writes
the per-screen workspace state file in STATE_DIR/kubaos/hypr. QS_LOG is
Quickshell's log, where the bar's probe lines (`workspaces probe: SCREEN
GROUPS`) tell it the bar has caught up.

The session's one screen, HEADLESS-1, is the primary: the laptop panel's
block 0, with workspaces 1 and 2, holding the waiting groups of two desk
positions no screen holds, 33-34 and 41. Then DP-8 and DP-9 are plugged in
and `config/hypr/workspaces.lua` sends each group to its screen, as Hyprland
does it:

1. The new screen's default workspace is created; Quickshell asks for the
   workspace list.
2. Hyprland answers that request; then the reconcile moves 33, 34 and 41 off
   the primary, each move an event. The new screens' default workspaces, 4
   and 5, are of the primary's block, so its bar shows them wherever they are.
3. Quickshell reads the move events before the answer, which still has every
   group on the primary. The answer is held back until the bar has shown the
   moves, then sent.

It exits 0 once Quickshell has read the answer, and 1, with the reason on
stderr, when a step doesn't happen in time. The bar's state after it is the
last probe line in QS_LOG.
"""

import json
import os
import socket
import sys
import threading
import time

PRIMARY = "HEADLESS-1"
# Each step waits this long for Quickshell before the run fails.
STEP_TIMEOUT_S = 30
# How long the bar has, after the held answer is read, to draw what it makes
# of it before the run ends.
SETTLE_S = 1.0


# The compositor's state, and the request and event sockets serving it.
class Hyprland:
    def __init__(self, directory):
        self.lock = threading.Lock()
        self.monitors = [self.monitor(0, PRIMARY, "Headless panel", 1)]
        # Id -> the connector of the screen the workspace is on.
        self.workspaces = {1: PRIMARY, 2: PRIMARY, 33: PRIMARY, 34: PRIMARY, 41: PRIMARY}
        self.windows = {1: 0x1001, 2: 0x1002, 33: 0x1033, 34: 0x1034, 41: 0x1041}
        self.events = []
        self.listener = None
        self.listener_ready = threading.Event()
        self.served_workspaces = threading.Event()
        # While set, the next workspace list is read at once and sent only
        # once `release` is set.
        self.hold = False
        self.held = threading.Event()
        self.release = threading.Event()
        self.held_read = threading.Event()

        self.requests = self.bind(os.path.join(directory, ".socket.sock"))
        self.event_socket = self.bind(os.path.join(directory, ".socket2.sock"))
        threading.Thread(target=self.serve_requests, daemon=True).start()
        threading.Thread(target=self.serve_events, daemon=True).start()

    @staticmethod
    def bind(path):
        server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        server.bind(path)
        server.listen(16)
        return server

    @staticmethod
    def monitor(monitor_id, name, description, active):
        return {
            "id": monitor_id,
            "name": name,
            "description": description,
            "x": 1920 * monitor_id,
            "y": 0,
            "width": 1920,
            "height": 1080,
            "scale": 1.0,
            "focused": monitor_id == 0,
            "activeWorkspace": {"id": active, "name": str(active)},
        }

    def monitor_id(self, name):
        return next(m["id"] for m in self.monitors if m["name"] == name)

    def workspace_list(self):
        return [
            {
                "id": ws,
                "name": str(ws),
                "monitor": name,
                "monitorID": self.monitor_id(name),
                "windows": 1 if ws in self.windows else 0,
                "hasfullscreen": False,
            }
            for ws, name in sorted(self.workspaces.items())
        ]

    def client_list(self):
        return [
            {
                "address": hex(address),
                "title": f"window {ws}",
                "class": "foot",
                "workspace": {"id": ws, "name": str(ws)},
            }
            for ws, address in sorted(self.windows.items())
        ]

    def answer(self, request):
        with self.lock:
            if request == "j/monitors":
                return json.dumps(self.monitors)
            if request == "j/workspaces":
                return json.dumps(self.workspace_list())
            if request == "j/clients":
                return json.dumps(self.client_list())
            if request == "j/status":
                return "{}"
            return "ok"

    def serve_requests(self):
        while True:
            conn, _ = self.requests.accept()
            threading.Thread(target=self.serve_request, args=(conn,), daemon=True).start()

    def serve_request(self, conn):
        with conn:
            request = conn.recv(65536).decode()
            reply = self.answer(request)
            if request != "j/workspaces":
                conn.sendall(reply.encode())
                return
            if not self.hold:
                conn.sendall(reply.encode())
                self.served_workspaces.set()
                return
            self.hold = False
            self.held.set()
            self.release.wait()
            conn.sendall(reply.encode())
            # Quickshell closes the socket once it has applied the answer.
            conn.recv(1)
            self.held_read.set()

    def serve_events(self):
        conn, _ = self.event_socket.accept()
        self.listener = conn
        self.listener_ready.set()

    def emit(self, *lines):
        self.listener.sendall("".join(f"{line}\n" for line in lines).encode())

    def plug(self, monitor_id, name, description, default_workspace):
        with self.lock:
            self.monitors.append(self.monitor(monitor_id, name, description, default_workspace))
            self.workspaces[default_workspace] = name
        self.emit(
            f"createworkspace>>{default_workspace}",
            f"createworkspacev2>>{default_workspace},{default_workspace}",
            f"monitoradded>>{name}",
            f"monitoraddedv2>>{monitor_id},{name},{description}",
        )

    def move(self, workspace, name):
        with self.lock:
            self.workspaces[workspace] = name
        self.emit(f"moveworkspace>>{workspace},{name}", f"moveworkspacev2>>{workspace},{workspace},{name}")


def fail(reason):
    print(f"hyprland-hotplug: {reason}", file=sys.stderr)
    sys.exit(1)


def wait(event, what):
    if not event.wait(STEP_TIMEOUT_S):
        fail(f"{what}: not within {STEP_TIMEOUT_S}s")


def log_end(log):
    return os.path.getsize(log)


# Waits for the bar to show `groups` on the primary in a line of QS_LOG past the
# offset `since`: while Quickshell reads the first workspace list, the bar
# passes through the groups it reads one at a time.
def wait_for_probe(log, since, groups, what):
    line = f"workspaces probe: {PRIMARY} {groups}".encode()
    end = time.monotonic() + STEP_TIMEOUT_S
    while time.monotonic() < end:
        with open(log, "rb") as text:
            text.seek(since)
            if any(line in row for row in text):
                return
        time.sleep(0.1)
    fail(f"{what}: the bar never showed {groups}")


def write_screens(state_dir, rows):
    directory = os.path.join(state_dir, "kubaos", "hypr")
    os.makedirs(directory, exist_ok=True)
    path = os.path.join(directory, "workspace-screens")
    with open(f"{path}.tmp", "w", encoding="utf-8") as screens:
        screens.write("".join("\t".join(row) + "\n" for row in rows))
    os.replace(f"{path}.tmp", path)


def main():
    if len(sys.argv) != 5:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    runtime_dir, signature, state_dir, log = sys.argv[1:]
    directory = os.path.join(runtime_dir, "hypr", signature)
    os.makedirs(directory, exist_ok=True)
    write_screens(state_dir, [
        ["0", PRIMARY, "panel", "Headless panel"],
        ["3", "", "2", "LG", "Centre"],
        ["4", "", "1", "BenQ", "Left"],
    ])
    hyprland = Hyprland(directory)
    print("ready", flush=True)

    wait(hyprland.listener_ready, "Quickshell connecting to the event socket")
    wait(hyprland.served_workspaces, "Quickshell reading the workspaces")
    wait_for_probe(log, 0, "[[1,2],[33,34],[41]]", "the waiting groups on the primary")

    since = log_end(log)
    hyprland.hold = True
    hyprland.plug(1, "DP-8", "LG", 4)
    wait(hyprland.held, "Quickshell asking for the workspaces after DP-8's arrival")
    hyprland.plug(2, "DP-9", "BenQ", 5)
    hyprland.move(33, "DP-8")
    hyprland.move(34, "DP-8")
    hyprland.move(41, "DP-9")
    write_screens(state_dir, [
        ["0", PRIMARY, "panel", "Headless panel"],
        ["3", "DP-8", "2", "LG", "Centre"],
        ["4", "DP-9", "1", "BenQ", "Left"],
    ])
    wait_for_probe(log, since, "[[1,2,4,5]]", "the groups leaving the primary as they move")

    hyprland.release.set()
    wait(hyprland.held_read, "Quickshell reading the answer sent before the moves")
    time.sleep(SETTLE_S)


if __name__ == "__main__":
    main()
