pragma Singleton
import QtQuick

// Stand-in for Preview: the release, until a driver puts the home on a live preview.
QtObject {
    readonly property string stateFile: "/nonexistent/state/kubaos/preview.json"
    property var record: ({})
    property bool active: false
    property string branch: ""
    property string commit: ""
    readonly property string name: branch !== "" ? branch : commit
    property bool dirty: false
    property real since: 0

    function parse(text: string): var {
        return ({});
    }
}
