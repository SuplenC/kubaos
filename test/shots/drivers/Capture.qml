import QtQuick

// Saves `window`'s content as `path` once it is still, then quits. `start()` begins the wait:
// at least `interval` ms, then until the window has presented no frame for `quiet` ms, so a
// panel still opening or a fill still animating is never caught mid-way, however busy the
// machine. A caret blinks every 500 ms, so it doesn't hold the wait open. A window still drawing
// after `longest` ms fails the shot rather than save it half-way. The runner keeps the file only
// when the driver exits 0 after its `shot` line.
Item {
    id: capture

    property var window: null
    property string path
    property int interval: 1200
    property int quiet: 300
    property int longest: 20000
    property real started: 0
    property real lastFrame: 0

    function start(): void {
        capture.started = Date.now();
        capture.lastFrame = Date.now();
        poll.start();
    }

    function grab(): void {
        poll.stop();
        if (!capture.window || capture.window.contentItem.width < 1) {
            console.error(`shot: no window to grab (${capture.window})`);
            Qt.exit(1);
            return;
        }
        capture.window.contentItem.grabToImage(result => {
            if (!result.saveToFile(capture.path)) {
                console.error(`shot: could not write ${capture.path}`);
                Qt.exit(1);
                return;
            }
            console.log(`shot ${capture.path}`);
            Qt.quit();
        });
    }

    Connections {
        target: capture.window

        function onFrameSwapped(): void {
            capture.lastFrame = Date.now();
        }
    }

    Timer {
        id: poll

        interval: 50
        repeat: true
        onTriggered: {
            const now = Date.now();
            if (now - capture.started < capture.interval)
                return;
            if (now - capture.lastFrame >= capture.quiet) {
                capture.grab();
            } else if (now - capture.started > capture.longest) {
                poll.stop();
                console.error(`shot: still drawing after ${capture.longest} ms`);
                Qt.exit(1);
            }
        }
    }
}
