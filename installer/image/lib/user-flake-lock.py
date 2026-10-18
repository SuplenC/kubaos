"""The user flake's lock at payload build (docs/INSTALLER.md, "The user flake").

  user-flake-lock.py override KUBAOS_LOCK
      prints the --override-input URL that locks the user's nixpkgs to
      KubaOS's revision
  user-flake-lock.py pin KUBAOS_LOCK USER_LOCK
      rewrites USER_LOCK's nixpkgs node to KubaOS's, so its `original` is
      the branch in user-flake/flake.nix.in again and Nix takes the lock as
      it is. On day one both flakes use one nixpkgs; they part when the user
      updates theirs.
"""

import json
import sys


def nixpkgs_node(lock):
    return lock["nodes"][lock["nodes"]["root"]["inputs"]["nixpkgs"]]


def main(argv):
    if len(argv) == 3 and argv[1] == "override":
        with open(argv[2]) as f:
            locked = nixpkgs_node(json.load(f))["locked"]
        if locked.get("type") != "github":
            sys.exit(f"KubaOS's nixpkgs is a {locked.get('type')} input; expected github")
        print(f"github:{locked['owner']}/{locked['repo']}/{locked['rev']}")
    elif len(argv) == 4 and argv[1] == "pin":
        with open(argv[2]) as f:
            kubaos = nixpkgs_node(json.load(f))
        with open(argv[3]) as f:
            user = json.load(f)
        node = nixpkgs_node(user)
        if node["locked"] != kubaos["locked"]:
            sys.exit("the user flake's nixpkgs isn't locked to KubaOS's revision")
        node["original"] = kubaos["original"]
        with open(argv[3], "w") as f:
            json.dump(user, f, indent=2)
            f.write("\n")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv)
