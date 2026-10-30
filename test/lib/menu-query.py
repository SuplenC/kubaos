"""menu-query.py SOCKET [--top | --primary | --app WORD] TEXT...: what the Command menu finds for each TEXT.

A Front end of its own on the Daemon's socket (Kotodama protocol 1.0, JSON):
for each TEXT it sends a `query` and prints one line, the TEXT, a tab, then
`<extension>/<item id>` for every Result, best first. With --primary, each
Result is followed by `><extension>/<action id>`, the Action the Daemon marks
for Enter: the one the query chose by its verb, else the Item's first. The
shell's menu runs the Item's first all the same (its `enter.js`). With --top, the line is
the TEXT, then the top Result's `<extension>/<item id>`, title and subtitle,
tab-separated: what an answer in the top row says. With --app, it first
enters the app of WORD from its entry Item, as a user does, and each TEXT is
searched in that app. The `kotodama` command has no query of its own. Run
with `python3 -I`.
"""

import json
import select
import socket
import sys
import time

WAIT = 10
QUIET = 0.5


class Daemon:
    def __init__(self, path):
        self.conn = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.conn.connect(path)
        self.buf = b""

    def send(self, message):
        data = json.dumps(message).encode()
        self.conn.sendall(str(len(data)).encode() + b"\n" + data)

    def recv(self, wait):
        """The next message, or None when none comes within `wait` seconds."""
        deadline = time.monotonic() + wait
        while True:
            head, newline, rest = self.buf.partition(b"\n")
            if newline and len(rest) >= int(head):
                self.buf = rest[int(head) :]
                return json.loads(rest[: int(head)])
            left = deadline - time.monotonic()
            if left <= 0 or not select.select([self.conn], [], [], left)[0]:
                return None
            data = self.conn.recv(65536)
            if not data:
                sys.exit("the Daemon hung up")
            self.buf += data


def ask(daemon, text):
    """The Results the Daemon settles on for `text`."""
    daemon.send({"query": {"text": text}})
    results = None
    while results is None:
        message = daemon.recv(WAIT)
        if message is None:
            sys.exit(f"no results for {text!r}")
        results = message.get("results")
    # The list is sent again as slower Extensions answer: take the last
    # one before the Daemon goes quiet.
    while (message := daemon.recv(QUIET)) is not None:
        results = message.get("results", results)
    return results


def enter(daemon, word):
    """Runs the entry Item of the app of `word`, as a click on it does."""
    results = ask(daemon, word + " ")
    entry = next((r for r in results["results"] if r["item"].get("keyword") == word), None)
    if entry is None:
        sys.exit(f"no entry Item for the app of {word!r}")
    item = entry["item"]
    daemon.send({"run_action": {
        "result": {"extension": entry["extension"], "id": item["id"]},
        "shown": results["version"],
        "action": {"extension": entry["extension"], "id": item["actions"][0]["id"]},
    }})
    while (message := daemon.recv(WAIT)) is not None:
        if (message.get("results") or {}).get("scope"):
            return
    sys.exit(f"running {item['id']} entered no app")


def primary(result):
    """`><extension>/<action id>` of the Action Enter runs on `result`."""
    chosen = result.get("primary")
    if chosen:
        return f">{chosen['extension']}/{chosen['id']}"
    actions = result["item"].get("actions") or [{"id": ""}]
    return f">{result['extension']}/{actions[0]['id']}"


def main(path, texts, top=False, app=None, verbs=False):
    daemon = Daemon(path)
    daemon.send({"version": 1, "minor": 0, "formats": ["json"]})
    hello = daemon.recv(WAIT)
    if not hello or hello.get("version") != 1:
        sys.exit(f"the Daemon's hello: {hello}")
    if app:
        enter(daemon, app)
    for text in texts:
        shown = ask(daemon, f"{app} {text}" if app else text)["results"]
        if top:
            first = shown[0] if shown else {"extension": "", "item": {"id": "", "title": ""}}
            item = first["item"]
            fields = [text, f"{first['extension']}/{item['id']}", item.get("title", ""), item.get("subtitle") or ""]
            print("\t".join(fields))
            continue
        found = [f"{r['extension']}/{r['item']['id']}" + (primary(r) if verbs else "") for r in shown]
        print(text + "\t" + " ".join(found))


if __name__ == "__main__":
    args = sys.argv[2:]
    top = bool(args) and args[0] == "--top"
    if top:
        args = args[1:]
    verbs = bool(args) and args[0] == "--primary"
    if verbs:
        args = args[1:]
    app = None
    if len(args) > 1 and args[0] == "--app":
        app, args = args[1], args[2:]
    if not args:
        sys.exit(__doc__)
    main(sys.argv[1], args, top, app, verbs)
