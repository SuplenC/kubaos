import QtQuick

// Stand-in for Process: it never runs anything. A command the shell reads state from gets a fixed
// answer; any other finishes at once with nothing on stdout.
QtObject {
    id: process

    property var command: []
    property var environment: ({})
    property string workingDirectory: ""
    property bool running: false
    property var stdout: null
    property var stderr: null
    property bool stdinEnabled: false
    readonly property int processId: 0

    signal started
    signal exited(int exitCode, int exitStatus)

    readonly property var answers: ({
            "kubaos keyboard show": "us\nde\n",
            "kubaos keyboard list": "us\tEnglish (US)\nde\tGerman\nfr\tFrench\njp\tJapanese\npl\tPolish\n",
            "kubaos keyboard accents": "on\n"
        })

    function answer(): void {
        const out = process.answers[(process.command ?? []).join(" ")] ?? "";
        if (process.stdout) {
            process.stdout.text = out;
            process.stdout.streamFinished();
        }
        if (process.stderr) {
            process.stderr.text = "";
            process.stderr.streamFinished();
        }
        process.running = false;
        process.exited(0, 0);
    }

    function exec(args: var): void {
        process.command = args?.command ?? args;
        process.running = true;
    }

    function signal(sig: int): void {
        process.running = false;
    }

    function write(text: string): void {
    }

    onRunningChanged: {
        if (running)
            Qt.callLater(process.answer);
    }
    Component.onCompleted: {
        if (running)
            Qt.callLater(process.answer);
    }
}
