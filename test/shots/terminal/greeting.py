"""The greeting's fastfetch config with fixed facts, for the terminals' shots.

    greeting.py CONFIG

Prints CONFIG (JSON, `//` comments allowed) with its modules made `custom` lines of the same
keys, so a shot shows the same facts on every machine. The logo, its colours and the keys' colour
stay the config's.
"""

import json
import re
import sys

FACTS = [
    ("OS", "KubaOS 0.1 (Shinjuku) x86_64"),
    ("Host", "InfinityBook Pro 14 Gen10"),
    ("Kernel", "Linux 7.0.0-38-generic"),
    ("Uptime", "2 hours, 14 mins"),
    ("Terminal", "ghostty 1.3.1"),
    ("CPU", "AMD Ryzen AI 9 HX 370 (24) @ 5.16 GHz"),
    ("Memory", "11.42 GiB / 62.02 GiB (18%)"),
    ("Disk (/)", "214.80 GiB / 1.82 TiB (12%)"),
    ("Battery", "86% [Discharging]"),
]


def main() -> None:
    text = open(sys.argv[1], encoding="utf-8").read()
    config = json.loads(re.sub(r"^\s*//.*$", "", text, flags=re.M))
    modules = [{"type": "custom", "format": "{#keys}kuba{#}@{#keys}tuxedo"}, "separator"]
    modules += [{"type": "custom", "key": key, "format": value} for key, value in FACTS]
    config["modules"] = modules + ["break", "colors"]
    json.dump(config, sys.stdout, indent=2, ensure_ascii=False)


if __name__ == "__main__":
    main()
