pragma Singleton
import QtQuick

// Stand-in for Recorder: recording the screen, 1:23 in.
QtObject {
    readonly property string state: "recording"
    readonly property bool recording: true
    readonly property bool replay: false
    readonly property bool active: true
    readonly property string mode: "screen"
    readonly property int started: 0
    readonly property int elapsed: 83
    readonly property string elapsedText: "1:23"

    function run(args: var, delayed: bool): void {
    }

    function stop(): void {
    }

    function toggle(): void {
    }

    function record(mode: string): void {
    }

    function toggleReplay(): void {
    }

    function saveReplay(): void {
    }
}
