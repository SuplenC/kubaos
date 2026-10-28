import QtQuick

// Stand-in for a QsMenuEntry: one entry of an app's tray menu, or a submenu with `entries`.
QtObject {
    property string text
    property string icon
    property bool isSeparator: false
    property bool enabled: true
    property int buttonType: 0
    property int checkState: Qt.Unchecked
    property bool hasChildren: false
    property list<QtObject> entries

    function triggered(): void {
    }
}
