"""QMP helper for test/vm/run (stdlib only; run with python3 -I).

  qmp.py SOCK raw JSON...            send each command, print each reply
  qmp.py SOCK hmp COMMAND            human-monitor-command, print its text
  qmp.py SOCK snapshot save|load|delete TAG
                                     snapshot-* job: VM state in the target disk,
                                     target + EFI vars snapshotted together
  qmp.py SOCK key COMBO...           e.g. ctrl-alt-f2 ret esc
  qmp.py SOCK type TEXT              type TEXT ("\\n" = Enter)
"""

import json
import socket
import sys
import time


class Qmp:
    def __init__(self, path):
        self.sock = socket.socket(socket.AF_UNIX)
        self.sock.connect(path)
        self.f = self.sock.makefile("rw")
        self._recv()  # greeting
        self.call("qmp_capabilities")

    def _recv(self):
        while True:
            line = self.f.readline()
            if not line:
                sys.exit("qmp: connection closed (did QEMU exit?)")
            m = json.loads(line)
            if "event" not in m:
                return m

    def send(self, obj):
        self.f.write(json.dumps(obj) + "\n")
        self.f.flush()
        return self._recv()

    def call(self, execute, **args):
        msg = {"execute": execute}
        if args:
            msg["arguments"] = args
        r = self.send(msg)
        if "error" in r:
            sys.exit(f"qmp: {execute}: {r['error']['desc']}")
        return r.get("return")


SPECIAL = {" ": "spc", "\n": "ret", "\t": "tab", "-": "minus", "=": "equal", "/": "slash",
           ".": "dot", ",": "comma", ";": "semicolon", "'": "apostrophe", "\\": "backslash",
           "[": "bracket_left", "]": "bracket_right", "`": "grave_accent"}
SHIFTED = {"_": "minus", "+": "equal", "?": "slash", ">": "dot", "<": "comma", ":": "semicolon",
           '"': "apostrophe", "|": "backslash", "{": "bracket_left", "}": "bracket_right",
           "~": "grave_accent", "!": "1", "@": "2", "#": "3", "$": "4", "%": "5", "^": "6",
           "&": "7", "*": "8", "(": "9", ")": "0"}


def combo(q, keys):
    q.call("send-key", keys=[{"type": "qcode", "data": k} for k in keys])
    time.sleep(0.03)


def main():
    path, cmd, *args = sys.argv[1:]
    q = Qmp(path)
    if cmd == "raw":
        for a in args:
            r = q.send(json.loads(a))
            print(json.dumps(r))
            if "error" in r:
                sys.exit(1)
    elif cmd == "hmp":
        text = q.call("human-monitor-command", **{"command-line": args[0]})
        if text:
            print(text, end="")
        if "rror" in (text or ""):
            sys.exit(1)
    elif cmd == "snapshot":
        op, tag = args
        job = {"job-id": "snap", "tag": tag, "devices": ["target", "vars"]}
        if op != "delete":
            job["vmstate"] = "target"
        q.call(f"snapshot-{op}", **job)
        while True:
            jobs = [j for j in q.call("query-jobs") if j["id"] == "snap"]
            if not jobs or jobs[0]["status"] == "concluded":
                break
            time.sleep(0.2)
        err = jobs[0].get("error") if jobs else None
        if jobs:
            q.call("job-dismiss", id="snap")
        if err:
            sys.exit(f"snapshot {op} {tag}: {err}")
        if op == "load" and not q.call("query-status")["running"]:
            q.call("cont")
    elif cmd == "key":
        for a in args:
            combo(q, a.split("-") if a != "minus" else [a])
    elif cmd == "type":
        for c in args[0].replace("\\n", "\n"):
            if c in SPECIAL:
                combo(q, [SPECIAL[c]])
            elif c in SHIFTED:
                combo(q, ["shift", SHIFTED[c]])
            elif c.isupper():
                combo(q, ["shift", c.lower()])
            else:
                combo(q, [c])
    else:
        sys.exit(f"qmp.py: unknown command {cmd}")


if __name__ == "__main__":
    main()
