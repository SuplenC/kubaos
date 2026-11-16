#!/usr/bin/env bash
# apps-adopt: the Apps catalog keeps a journal of what each app's install did,
# and an app is installed when its journal says so. The apps this machine had
# before (base apps handed over by base-apps-picked, apps added with `kubaos
# app add`, an app set up by hand) are taken in: `kubaos app adopt` runs each
# app's checks and records what they find, as KubaOS's own for a picked app,
# so its removal undoes it, else as found, which a removal leaves. Nothing is
# installed, removed or run but the checks, and no password is asked: root's
# steps are recorded on the first run that starts root.
# kubaos update runs it once, as the user, after the home-manager switch; KUBAOS_USER_FLAKE is their flake,
# under bash -euo pipefail from the new release's checkout (KUBAOS_DIR). Exiting
# non-zero stops the update; it runs again on the next one, so make it safe to rerun.
# shellcheck source=migrations/lib.sh
source "$KUBAOS_DIR/migrations/lib.sh"

kubaos app adopt
