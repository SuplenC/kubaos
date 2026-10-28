import QtQuick

// Stand-in for the lock's Auth: never reaches PAM. The lock driver sets the state it shows.
QtObject {
    property bool active: true
    property string message: ""
    property string fingerprintMessage: ""
    property string keyMessage: ""
    property string keyState: ""
    property bool keyEnrolled: true
    property bool keyBroken: false
    property bool busy: false
    property bool fingerprintActive: false
    readonly property bool keyAvailable: keyEnrolled && !keyBroken
    readonly property string keyHint: keyState === "waiting" ? "Touch your security key" : keyState === "failed" ? "That key isn't yours. Try another, or type your password" : "Or use your security key"
    readonly property bool keyHintIsError: keyState === "failed" || keyState === "blocked" || keyState === "pin"
    property string pendingPassword: ""

    signal succeeded
    signal failed

    function submit(text: string): void {
    }

    function fail(text: string): void {
    }

    function startOthers(): void {
    }

    function startFingerprint(): void {
    }

    function startKey(): void {
    }
}
