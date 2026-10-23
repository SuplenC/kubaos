import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.Pam
import qs.settings

// PAM for the lock screen. Conversations run side by side and any one unlocks:
//   qs-lock              password, started on Enter
//   qs-lock-u2f          FIDO2 security key (Settings.u2f), only when
//                        /etc/u2f_mappings has a key for this user. It
//                        restarts after every attempt, so a key plugged in
//                        later is picked up within a few seconds. The PAM
//                        file demands user verification: the YubiKey C Bio
//                        wants an enrolled finger. No PIN here (sudo has it).
//   qs-lock-fingerprint  fprintd reader (Settings.fingerprint, off by default),
//                        restarted after each failed scan
// The PAM files come from system/ (50-hyprland.sh, 70-bitwarden.sh).
Scope {
    id: root

    property bool active: false
    property string message: ""
    property string fingerprintMessage: ""
    // pam_u2f's cue: only sent once it found a key, so non-empty means
    // "the key is plugged in and waiting for a finger".
    property string keyMessage: ""
    // What the key half is doing, for the hint under the field:
    //   ""         no key plugged in (or nothing tried yet)
    //   "waiting"  cue received: put a finger on the sensor
    //   "failed"   it was waiting, then failed: wrong finger or timeout
    //   "blocked"  fails right after the cue, twice: the Bio wants its PIN
    //              (3 bad scans), which only sudo/polkit can take
    //   "pin"      pam_u2f asked for a PIN (a key enrolled with --pin)
    property string keyState: ""
    property double keyCueAt: 0
    property int keyQuickFails: 0
    readonly property string keyHint: {
        switch (keyState) {
        case "waiting":
            return "Touch the fingerprint sensor";
        case "failed":
            return "Fingerprint not recognised. Wrong finger? Try again or type your password";
        case "blocked":
            return "Fingerprint locked: type your password, then use the key's PIN once (sudo)";
        case "pin":
            return "This key needs its PIN: type your password";
        default:
            return "Or use your security key";
        }
    }
    readonly property bool keyHintIsError: keyState === "failed" || keyState === "blocked" || keyState === "pin"
    property bool keyEnrolled: false
    // Set when qs-lock-u2f itself is broken (module missing), to stop retrying.
    property bool keyBroken: false
    readonly property bool busy: password.active
    readonly property bool fingerprintActive: fingerprint.active
    readonly property bool keyAvailable: Settings.u2f && keyEnrolled && !keyBroken
    property string pendingPassword: ""

    signal succeeded
    signal failed

    // Locked state survives a config reload, so this can start out active.
    Component.onCompleted: startOthers()
    onActiveChanged: {
        message = "";
        keyBroken = false;
        keyState = "";
        keyQuickFails = 0;
        if (active) {
            startOthers();
        } else {
            password.abort();
            fingerprint.abort();
            key.abort();
            keyRetry.stop();
            retry.stop();
        }
    }
    onKeyAvailableChanged: startKey()

    function submit(text: string): void {
        if (password.active || text === "")
            return;
        pendingPassword = text;
        message = "";
        if (!password.start())
            fail("Fuck. Couldn't start PAM: is /etc/pam.d/qs-lock installed?");
    }

    function fail(text: string): void {
        message = text;
        failed();
    }

    function startOthers(): void {
        startFingerprint();
        startKey();
    }

    function startFingerprint(): void {
        if (Settings.fingerprint && active && !fingerprint.active)
            fingerprint.start();
    }

    function startKey(): void {
        if (keyAvailable && active && keyState !== "pin" && !key.active && !key.start())
            keyBroken = true;
    }

    PamContext {
        id: password

        config: "qs-lock"
        onResponseRequiredChanged: {
            if (responseRequired) {
                respond(root.pendingPassword);
                root.pendingPassword = "";
            }
        }
        onCompleted: result => {
            if (result === PamResult.Success)
                root.succeeded();
            else
                root.fail(result === PamResult.Failed ? "Wrong password. Fat fingers? Try again." : `Authentication error: ${PamResult.toString(result)}`);
        }
    }

    PamContext {
        id: fingerprint

        config: "qs-lock-fingerprint"
        onPamMessage: root.fingerprintMessage = message
        onCompleted: result => {
            if (result === PamResult.Success) {
                root.succeeded();
            } else if (result === PamResult.Failed || result === PamResult.MaxTries) {
                retry.restart();
            } else {
                // No reader or no enrolled finger: stay password-only.
                root.fingerprintMessage = "";
            }
        }
    }

    PamContext {
        id: key

        config: "qs-lock-u2f"
        onPamMessage: {
            if (responseRequired) {
                // A prompt (the PIN), not the cue.
                root.keyState = "pin";
                abort();
                return;
            }
            root.keyMessage = message;
            if (!messageIsError && root.keyState !== "pin") {
                root.keyState = "waiting";
                root.keyCueAt = Date.now();
            }
        }
        // pam_u2f only asks for something when a key wants its PIN, which a
        // lock screen can't take: say so and stop trying until the next lock.
        onResponseRequiredChanged: {
            if (responseRequired) {
                root.keyState = "pin";
                abort();
            }
        }
        onCompleted: result => {
            root.keyMessage = "";
            if (result === PamResult.Success) {
                root.keyState = "";
                root.succeeded();
            } else if (root.keyState === "pin") {
                // Stays until the next lock; the password works.
            } else if (result === PamResult.Failed || result === PamResult.MaxTries) {
                if (root.keyState === "waiting") {
                    // A scan fails after the finger was on the sensor (or a
                    // timeout); a blocked fingerprint fails at once.
                    root.keyQuickFails = Date.now() - root.keyCueAt < 1500 ? root.keyQuickFails + 1 : 0;
                    root.keyState = root.keyQuickFails >= 2 ? "blocked" : "failed";
                } else if (root.keyState !== "blocked") {
                    // No cue: no enrolled key plugged in.
                    root.keyState = "";
                    root.keyQuickFails = 0;
                }
                keyRetry.interval = root.keyState === "blocked" ? 15000 : root.keyState === "failed" ? 2500 : 4000;
                keyRetry.restart();
            } else {
                // Module or PAM file missing: hide the hint, don't spin.
                root.keyBroken = true;
            }
        }
    }

    Timer {
        id: retry

        interval: 800
        onTriggered: root.startFingerprint()
    }

    Timer {
        id: keyRetry

        interval: 4000
        onTriggered: root.startKey()
    }

    // Is a key enrolled for us? Same file pam_u2f reads (root-owned, 0644).
    FileView {
        path: "/etc/u2f_mappings"
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: {
            const user = Quickshell.env("USER");
            root.keyEnrolled = text().split("\n").some(line => line.startsWith(`${user}:`) && line.length > user.length + 1);
        }
        onLoadFailed: root.keyEnrolled = false
    }
}
