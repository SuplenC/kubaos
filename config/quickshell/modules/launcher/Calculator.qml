import QtQuick
import Quickshell.Io

// Evaluates an expression with qalc (libqalculate), debounced while typing.
Item {
    id: root

    property string expression
    property string result: ""

    onExpressionChanged: {
        result = "";
        if (expression)
            debounce.restart();
    }

    Timer {
        id: debounce

        interval: 120
        // Leading space keeps expressions like "-2+3" from being read as flags.
        onTriggered: qalc.exec(["qalc", "-t", ` ${root.expression}`])
    }

    Process {
        id: qalc

        stdout: StdioCollector {
            onStreamFinished: root.result = text.trim()
        }
    }
}
