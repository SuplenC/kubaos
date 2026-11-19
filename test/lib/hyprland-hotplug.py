"""A stand-in for Hyprland's two IPC sockets that plays screens plugged and unplugged.

  python3 -I hyprland-hotplug.py RUNTIME_DIR SIGNATURE STATE_DIR QS_LOG SWAYSOCK

It serves `.socket.sock` (requests) and `.socket2.sock` (events) under
RUNTIME_DIR/hypr/SIGNATURE for a Quickshell started with
HYPRLAND_INSTANCE_SIGNATURE=SIGNATURE and KUBAOS_WORKSPACES_PROBE=1, and writes
the per-screen workspace state file in STATE_DIR/kubaos/hypr. QS_LOG is
Quickshell's log, where the bar's probe lines (`workspaces probe: SCREEN
GROUPS`, `active workspace probe: SCREEN ID`, `indicator probe: SCREEN ID X`,
X `none` while the indicator isn't drawn, and `active window probe: SCREEN
"TITLE"`) tell it the bar has caught up. SWAYSOCK is the headless sway the
session runs in, whose screens it adds and takes away.

Each screen comes and goes with Hyprland's events in Hyprland's order
(`CMonitor::onConnect` and `onDisconnect`, v0.56). Going: each of its
workspaces moves to the first other screen, the one it shows first switching
it to another of its own, or to a new one when it has none left; the focus
moves to that screen if it was on the one going; then the screen is removed,
and the workspace made to fill the gap is destroyed. Coming: its default
workspace is created, the workspaces it had move back, and it is added, with
a new record, showing the workspace it showed; then the default workspace,
left empty and hidden, is destroyed.

1. The session's one screen, HEADLESS-1, is the primary: the laptop panel's
   block 0, with workspaces 1 and 2, holding the waiting groups of two desk
   positions no screen holds, 33-34 and 41, and the focused window on 33.
   DP-8 and DP-9, screens Hyprland has and sway doesn't, are plugged in and
   `config/hypr/workspaces.lua` sends each group to its screen: Quickshell
   asks for the workspace list as DP-8 arrives, the reconcile moves 33, 34
   and 41 off the primary, and the answer, from before the moves, is read
   only after the bar has shown them. The new screens' default workspaces, 4
   and 5, are of the primary's block, so its bar shows them wherever they
   are.
2. Hyprland drops the primary and lists it again, as it does a laptop panel
   whose lid closes and opens, while the session's screen stays: the bar
   shows no workspace, then workspace 1 again.
3. Sway adds HEADLESS-2 before Hyprland lists it, as a screen usually
   arrives: its bar shows no workspace, then, once Hyprland lists it, its
   workspace 11. Sway takes it away and Hyprland drops it, then both bring it
   back, and its bar shows 11 again.
4. HEADLESS-2, not focused, switches to its new workspace 12, as the
   reconcile's `monitor:set_workspace` does: its bar shows 12, and the
   primary's still shows 1.

It exits 0 once the bar has, and 1, with the reason on stderr, when a step
doesn't happen in time. The bar's state after it is the last probe line of
each kind in QS_LOG.
"""

import json
import os
import pathlib
import re
import socket
import subprocess
import sys
import threading
import time

PRIMARY = "HEADLESS-1"
SECOND = "HEADLESS-2"
# Each step waits this long for Quickshell before the run fails.
STEP_TIMEOUT_S = 30
# The bar's cell, `size.chip` in design/tokens.toml: the indicator's place is
# a whole number of them in a group of cells alone.
CELL = int(
    re.search(
        r"^chip = (\d+)$",
        (pathlib.Path(__file__).resolve().parents[2] / "design" / "tokens.toml").read_text(encoding="utf-8"),
        re.M,
    ).group(1)
)


# The compositor's state, and the request and event sockets serving it.
class Hyprland:
    def __init__(self, directory):
        self.lock = threading.Lock()
        self.next_monitor_id = 0
        self.monitors = []
        self.plug_record(PRIMARY, "Headless panel", 1)
        self.focused = PRIMARY
        # Id -> the connector of the screen the workspace is on.
        self.workspaces = {1: PRIMARY, 2: PRIMARY, 33: PRIMARY, 34: PRIMARY, 41: PRIMARY}
        self.windows = {1: 0x1001, 2: 0x1002, 33: 0x1033, 34: 0x1034, 41: 0x1041}
        # Connector -> the workspace a screen showed as it went, and the
        # workspaces it had then, which come back with it.
        self.remembered = {}
        self.returning = {}
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

    # Adds a screen's record, under a monitor id no screen had before, as
    # Hyprland gives a screen that comes back.
    def plug_record(self, name, description, active):
        monitor_id = self.next_monitor_id
        self.next_monitor_id += 1
        self.monitors.append({
            "id": monitor_id,
            "name": name,
            "description": description,
            "x": 1920 * monitor_id,
            "y": 0,
            "width": 1920,
            "height": 1080,
            "scale": 1.0,
            "activeWorkspace": {"id": active, "name": str(active)},
        })
        return monitor_id

    def record(self, name):
        return next((m for m in self.monitors if m["name"] == name), None)

    # A workspace whose screen is gone names none, so -1, as Hyprland's
    # MONITOR_INVALID.
    def monitor_id(self, name):
        record = self.record(name)
        return record["id"] if record else -1

    def active(self, name):
        return self.record(name)["activeWorkspace"]["id"]

    def set_active(self, name, workspace):
        self.record(name)["activeWorkspace"] = {"id": workspace, "name": str(workspace)}

    def monitor_list(self):
        return [dict(m, focused=m["name"] == self.focused) for m in self.monitors]

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
                return json.dumps(self.monitor_list())
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

    # The lowest id no workspace has, as Hyprland picks one to fill a gap or
    # for a screen's default.
    def free_id(self):
        workspace = 1
        while workspace in self.workspaces:
            workspace += 1
        return workspace

    def create(self, workspace, name):
        with self.lock:
            self.workspaces[workspace] = name
        self.emit(f"createworkspace>>{workspace}", f"createworkspacev2>>{workspace},{workspace}")

    def destroy(self, workspace):
        with self.lock:
            del self.workspaces[workspace]
        self.emit(f"destroyworkspace>>{workspace}", f"destroyworkspacev2>>{workspace},{workspace}")

    # A screen switching to a workspace, as `CMonitor::changeWorkspace` does
    # whichever screen has the focus.
    def switch(self, name, workspace):
        with self.lock:
            self.set_active(name, workspace)
        self.emit(f"workspace>>{workspace}", f"workspacev2>>{workspace},{workspace}")

    # A workspace moving to screen `name`. Hyprland switches the screen it
    # leaves, when it shows it, to another of that screen's, or to a new one.
    def move(self, workspace, name):
        source = self.workspaces[workspace]
        if self.record(source) and self.active(source) == workspace:
            others = sorted(ws for ws, on in self.workspaces.items() if on == source and ws != workspace)
            if others:
                self.switch(source, others[0])
            else:
                gap = self.free_id()
                self.create(gap, source)
                self.switch(source, gap)
        with self.lock:
            self.workspaces[workspace] = name
        self.emit(f"moveworkspace>>{workspace},{name}", f"moveworkspacev2>>{workspace},{workspace},{name}")

    def focus(self, name):
        with self.lock:
            self.focused = name
        workspace = self.active(name)
        self.emit(f"focusedmon>>{name},{workspace}", f"focusedmonv2>>{name},{workspace}")

    # A screen Hyprland didn't have, showing its new default workspace.
    def plug(self, name, description, default_workspace):
        with self.lock:
            monitor_id = self.plug_record(name, description, default_workspace)
        self.create(default_workspace, name)
        self.emit(f"monitoradded>>{name}", f"monitoraddedv2>>{monitor_id},{name},{description}")

    def unplug(self, name):
        backup = next(m["name"] for m in self.monitors if m["name"] != name)
        self.remembered[name] = self.active(name)
        self.returning[name] = sorted(ws for ws, on in self.workspaces.items() if on == name)
        for workspace in self.returning[name]:
            self.move(workspace, backup)
        if self.focused == name:
            self.focus(backup)
        gaps = sorted(ws for ws, on in self.workspaces.items() if on == name)
        record = self.record(name)
        with self.lock:
            self.monitors.remove(record)
        self.emit(
            f"monitorremoved>>{name}",
            f"monitorremovedv2>>{record['id']},{name},{record['description']}",
        )
        for gap in gaps:
            self.destroy(gap)

    def replug(self, name, description):
        default = self.free_id()
        with self.lock:
            monitor_id = self.plug_record(name, description, default)
        self.create(default, name)
        for workspace in self.returning.pop(name):
            self.move(workspace, name)
        with self.lock:
            self.set_active(name, self.remembered.pop(name))
        self.emit(f"monitoradded>>{name}", f"monitoraddedv2>>{monitor_id},{name},{description}")
        self.destroy(default)


def fail(reason):
    print(f"hyprland-hotplug: {reason}", file=sys.stderr)
    sys.exit(1)


def wait(event, what):
    if not event.wait(STEP_TIMEOUT_S):
        fail(f"{what}: not within {STEP_TIMEOUT_S}s")


def log_end(log):
    return os.path.getsize(log)


# Waits for the whole probe line `probe` in QS_LOG past the offset `since`:
# while Quickshell reads a workspace list, the bar passes through the groups it
# reads one at a time, and a line still being written is not yet the bar's.
def wait_for_probe(log, since, probe, what):
    line = probe.encode() + b"\n"
    end = time.monotonic() + STEP_TIMEOUT_S
    while time.monotonic() < end:
        with open(log, "rb") as text:
            text.seek(since)
            if any(row.endswith(line) for row in text):
                return
        time.sleep(0.1)
    fail(f"{what}: no `{probe}`")


# Waits until the last line of the probe `probe`'s kind for its screen in
# QS_LOG is `probe`: what the bar holds now, which the step need not have
# changed.
def wait_for_last(log, probe, what):
    head, rest = probe.split(" probe: ", 1)
    kind = f"{head} probe: {rest.split(' ', 1)[0]} ".encode()
    line = probe.encode() + b"\n"
    end = time.monotonic() + STEP_TIMEOUT_S
    last = None
    while time.monotonic() < end:
        with open(log, "rb") as text:
            rows = [row for row in text if row.endswith(b"\n") and kind in row]
        last = rows[-1] if rows else None
        if last is not None and last.endswith(line):
            return
        time.sleep(0.1)
    fail(f"{what}: the last is `{last.decode().strip() if last else None}`, not `{probe}`")


def groups_probe(screen, groups):
    return f"workspaces probe: {screen} {groups}"


def active_probe(screen, workspace):
    return f"active workspace probe: {screen} {workspace}"


# The indicator on the cell `cells` cells from the pill's start, or not drawn.
def indicator_probe(screen, workspace, cells=None):
    return f"indicator probe: {screen} {workspace} {'none' if cells is None else cells * CELL}"


def title_probe(screen, title):
    return f"active window probe: {screen} {json.dumps(title)}"


def write_screens(state_dir, rows):
    directory = os.path.join(state_dir, "kubaos", "hypr")
    os.makedirs(directory, exist_ok=True)
    path = os.path.join(directory, "workspace-screens")
    with open(f"{path}.tmp", "w", encoding="utf-8") as screens:
        screens.write("".join("\t".join(row) + "\n" for row in rows))
    os.replace(f"{path}.tmp", path)


def sway(swaysock, *command):
    result = subprocess.run(
        ["swaymsg", *command],
        env={**os.environ, "SWAYSOCK": swaysock},
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        fail(f"swaymsg {' '.join(command)}: {result.stdout}{result.stderr}".strip())


# 1: the groups waiting on the primary go to DP-8 and DP-9 as they arrive.
def groups_leave_the_primary(hyprland, state_dir, log):
    wait(hyprland.listener_ready, "Quickshell connecting to the event socket")
    wait(hyprland.served_workspaces, "Quickshell reading the workspaces")
    wait_for_probe(log, 0, groups_probe(PRIMARY, "[[1,2],[33,34],[41]]"), "the waiting groups on the primary")
    wait_for_probe(log, 0, indicator_probe(PRIMARY, 1, 0), "the indicator on the primary's workspace 1")
    hyprland.emit("activewindow>>foot,window 33", f"activewindowv2>>{hyprland.windows[33]:x}")
    wait_for_probe(log, 0, title_probe(PRIMARY, "window 33"), "the focused window's title on the primary")

    since = log_end(log)
    hyprland.hold = True
    hyprland.plug("DP-8", "LG", 4)
    wait(hyprland.held, "Quickshell asking for the workspaces after DP-8's arrival")
    hyprland.plug("DP-9", "BenQ", 5)
    hyprland.move(33, "DP-8")
    hyprland.move(34, "DP-8")
    hyprland.move(41, "DP-9")
    write_screens(state_dir, [
        ["0", PRIMARY, "panel", "Headless panel"],
        ["3", "DP-8", "2", "LG", "Centre"],
        ["4", "DP-9", "1", "BenQ", "Left"],
    ])
    wait_for_probe(log, since, groups_probe(PRIMARY, "[[1,2,4,5]]"), "the groups leaving the primary as they move")
    wait_for_probe(log, since, title_probe(PRIMARY, ""), "the focused window's title leaving the primary with 33")

    hyprland.release.set()
    wait(hyprland.held_read, "Quickshell reading the answer sent before the moves")
    wait_for_last(log, groups_probe(PRIMARY, "[[1,2,4,5]]"), "the primary's own group once the late answer is read")
    wait_for_last(log, title_probe(PRIMARY, ""), "no title on the primary once the late answer is read")


# 2: Hyprland drops the primary and lists it again; the session's screen stays.
def primary_comes_back(hyprland, log):
    since = log_end(log)
    hyprland.unplug(PRIMARY)
    wait_for_probe(log, since, active_probe(PRIMARY, 0), "the bar showing no workspace while Hyprland lists no primary")
    wait_for_probe(log, since, indicator_probe(PRIMARY, 0), "the indicator gone while Hyprland lists no primary")

    since = log_end(log)
    hyprland.replug(PRIMARY, "Headless panel")
    wait_for_probe(log, since, active_probe(PRIMARY, 1), "the bar showing workspace 1 once Hyprland lists the primary again")
    wait_for_probe(log, since, indicator_probe(PRIMARY, 1, 0), "the indicator on workspace 1 once the primary is back")
    wait_for_probe(log, since, groups_probe(PRIMARY, "[[1,2,4,5]]"), "the primary's group once it is back")
    hyprland.focus(PRIMARY)


# 3: a second screen sway adds before Hyprland lists it, taken away and back.
def second_screen_arrives(hyprland, state_dir, log, swaysock):
    since = log_end(log)
    sway(swaysock, "create_output")
    wait_for_probe(log, since, active_probe(SECOND, 0), "HEADLESS-2's bar showing no workspace before Hyprland lists it")
    hyprland.plug(SECOND, "Headless 2", 11)
    write_screens(state_dir, [
        ["0", PRIMARY, "panel", "Headless panel"],
        ["1", SECOND, "3", "Headless 2"],
        ["3", "DP-8", "2", "LG", "Centre"],
        ["4", "DP-9", "1", "BenQ", "Left"],
    ])
    shows_its_workspace(log, since, "once Hyprland lists it")

    since = log_end(log)
    sway(swaysock, "output", SECOND, "disable")
    hyprland.unplug(SECOND)
    wait_for_probe(log, since, groups_probe(PRIMARY, "[[1,2,4,5]]"), "the primary's group once HEADLESS-2 is gone")
    wait_for_last(log, active_probe(PRIMARY, 1), "the primary showing workspace 1 once HEADLESS-2 is gone")
    wait_for_last(log, indicator_probe(PRIMARY, 1, 0), "the primary's indicator on workspace 1 once HEADLESS-2 is gone")

    since = log_end(log)
    sway(swaysock, "output", SECOND, "enable")
    wait_for_probe(log, since, active_probe(SECOND, 0), "HEADLESS-2's bar showing no workspace before Hyprland lists it again")
    hyprland.replug(SECOND, "Headless 2")
    shows_its_workspace(log, since, "once Hyprland lists it again")


def shows_its_workspace(log, since, when):
    wait_for_probe(log, since, active_probe(SECOND, 11), f"HEADLESS-2's bar showing workspace 11 {when}")
    wait_for_probe(log, since, groups_probe(SECOND, "[[11]]"), f"HEADLESS-2's group {when}")
    wait_for_probe(log, since, indicator_probe(SECOND, 11, 0), f"HEADLESS-2's indicator on workspace 11 {when}")


# 4: the screen without the focus switches workspace.
def unfocused_screen_switches(hyprland, log):
    since = log_end(log)
    hyprland.create(12, SECOND)
    wait_for_probe(log, since, groups_probe(SECOND, "[[11,12]]"), "HEADLESS-2's new workspace 12 in its group")

    since = log_end(log)
    hyprland.switch(SECOND, 12)
    wait_for_probe(log, since, active_probe(SECOND, 12), "HEADLESS-2's bar showing the workspace it switched to")
    wait_for_probe(log, since, indicator_probe(SECOND, 12, 1), "HEADLESS-2's indicator on workspace 12")
    wait_for_last(log, active_probe(PRIMARY, 1), "the primary still showing workspace 1")
    wait_for_last(log, indicator_probe(PRIMARY, 1, 0), "the primary's indicator still on workspace 1")
    wait_for_last(log, groups_probe(PRIMARY, "[[1,2,4,5]]"), "the primary's group at the end")


def main():
    if len(sys.argv) != 6:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    runtime_dir, signature, state_dir, log, swaysock = sys.argv[1:]
    directory = os.path.join(runtime_dir, "hypr", signature)
    os.makedirs(directory, exist_ok=True)
    write_screens(state_dir, [
        ["0", PRIMARY, "panel", "Headless panel"],
        ["3", "", "2", "LG", "Centre"],
        ["4", "", "1", "BenQ", "Left"],
    ])
    hyprland = Hyprland(directory)
    print("ready", flush=True)

    groups_leave_the_primary(hyprland, state_dir, log)
    primary_comes_back(hyprland, log)
    second_screen_arrives(hyprland, state_dir, log, swaysock)
    unfocused_screen_switches(hyprland, log)


if __name__ == "__main__":
    main()
