import QtQuick

QtObject {
    property string title
    property string address
    property var workspace: null
    property var monitor: null
    property bool activated: false
    property bool urgent: false
    property var lastIpcObject: ({})
}
