"""menu-query.py SOCKET TEXT...: what the Command menu finds for each TEXT.

A Front end of its own on the Daemon's socket (Kotodama protocol 1.0, JSON):
for each TEXT it sends a `query` and prints one line, the TEXT, a tab, then
`<extension>/<item id>` for every Result, best first. The `kotodama` command
has no query of its own. Run with `python3 -I`.
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


def main(path, texts):
    daemon = Daemon(path)
    daemon.send({"version": 1, "minor": 0, "formats": ["json"]})
    hello = daemon.recv(WAIT)
    if not hello or hello.get("version") != 1:
        sys.exit(f"the Daemon's hello: {hello}")
    for text in texts:
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
        found = [f"{r['extension']}/{r['item']['id']}" for r in results["results"]]
        print(text + "\t" + " ".join(found))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2:])
