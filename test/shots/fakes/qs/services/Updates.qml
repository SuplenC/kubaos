pragma Singleton
import QtQuick

// Stand-in for Updates: a KubaOS update waiting.
QtObject {
    readonly property string stateFile: "/nonexistent/run/kubaos-updates.json"
    readonly property string update: "26.10.2"
    readonly property string upgrade: ""
    readonly property string upgradeCodename: ""
    readonly property int aptSecurity: 0
    readonly property bool available: true

    function run(command: string): void {
    }

    function parse(text: string): var {
        return ({});
    }
}
