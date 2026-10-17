"""What the engine needs in the store to build the home-manager generation
for another username offline (assemble.sh, "build inputs").

  build-inputs.py PAYLOAD_DRVS DIFFERING < `nix derivation show DIFFERING...`

PAYLOAD_DRVS: the payload generation's derivation closure (nix-store -qR on
its .drv), one path per line. DIFFERING: the derivations a probe user's
generation has and the payload's doesn't, the ones that name the user.

Prints, per input derivation that isn't itself one of DIFFERING, a line
"<drv> <output>...", then one line per input source that the payload's
closure has too. The probe's own sources (its flake checkout) are left out:
on the machine they come from the user's checkout.
"""

import json
import sys

STORE = "/nix/store/"


def full(p):
    return p if p.startswith("/") else STORE + p


payload = set(open(sys.argv[1]).read().split())
differing = set(open(sys.argv[2]).read().split())
shown = json.load(sys.stdin)
# Nix 2.35 nests them under "derivations" and drops the store dir.
drvs = shown.get("derivations", shown)

outputs = {}
srcs = set()
for v in drvs.values():
    inputs = v.get("inputs", {})
    for drv, o in inputs.get("drvs", v.get("inputDrvs", {})).items():
        drv = full(drv)
        if drv not in differing:
            outputs.setdefault(drv, set()).update(o["outputs"] if isinstance(o, dict) else o)
    for s in inputs.get("srcs", v.get("inputSrcs", [])):
        if full(s) in payload:
            srcs.add(full(s))

for drv, outs in sorted(outputs.items()):
    print(drv, *sorted(outs))
for s in sorted(srcs):
    print(s)
