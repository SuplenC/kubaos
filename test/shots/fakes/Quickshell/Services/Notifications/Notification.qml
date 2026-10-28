import QtQuick

QtObject {
    property string summary
    property string body
    property string appName
    property string appIcon: ""
    property string image: ""
    property string desktopEntry: ""
    property int urgency: 1
    property real expireTimeout: -1
    property bool resident: false
    property bool tracked: true
    property bool lastGeneration: false
    property var hints: ({})
    property list<QtObject> actions

    signal closed(int reason)

    function dismiss(): void {
        console.log(`notification: dismiss ${summary}`);
    }

    function expire(): void {
    }
}
