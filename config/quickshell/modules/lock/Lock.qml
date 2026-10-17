import QtQuick
import Quickshell
import Quickshell.Wayland
import qs.theme
import qs.services

// Session lock (ext-session-lock). Locked through Session.lock(), which the
// `qs ipc call session lock` handler calls; unlocked only after PAM succeeded.
// On success the surfaces play a short exit animation, then the lock is released.
Scope {
    id: root

    property bool unlocking: false

    Auth {
        id: authenticator

        active: Session.locked
        onSucceeded: {
            root.unlocking = true;
            release.restart();
        }
    }

    Timer {
        id: release

        interval: Theme.anim.normal
        onTriggered: {
            Session.unlock();
            root.unlocking = false;
        }
    }

    WlSessionLock {
        locked: Session.locked

        LockSurface {
            auth: authenticator
            unlocking: root.unlocking
        }
    }
}
