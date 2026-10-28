import QtQuick

// Stand-in for the polkit agent: one request, waiting for a password.
QtObject {
    property string path
    property bool isActive: true
    property bool isRegistered: true
    property AuthFlow flow: AuthFlow {}

    signal authenticationRequestStarted
}
