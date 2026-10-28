import QtQuick

QtObject {
    property string message: "Authentication is needed to run `/usr/bin/apt-get install btop' as the super user"
    property string iconName: ""
    property string actionId: "org.freedesktop.policykit.exec"
    property var identities: [selectedIdentity]
    property Identity selectedIdentity: Identity {}
    property bool responseVisible: false
    property string inputPrompt: "Password: "
    property bool isResponseRequired: true
    property bool isCompleted: false
    property bool isSuccessful: false
    property bool isCancelled: false
    property bool failed: false
    property string supplementaryMessage: ""
    property bool supplementaryIsError: false

    signal authenticationSucceeded
    signal authenticationFailed
    signal authenticationRequestCancelled

    function cancelAuthenticationRequest(): void {
        isCancelled = true;
    }

    function submit(text: string): void {
        console.log("polkit: submit");
    }
}
