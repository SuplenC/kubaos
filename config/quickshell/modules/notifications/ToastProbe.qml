import QtQuick
import qs.components

// Logs where the toast stacks and the open panels are, as `toast probe: ...`
// lines in Quickshell's log, each time that changes (polled every frame or
// so, to catch a glide). Frame.qml loads it only when Quickshell runs with
// KUBAOS_TOAST_PROBE=1; `c_notifications` (test/lib/e2e.sh) reads the lines.
//
//   toast probe: panel=X,Y,W,H ... stack=EDGE,CARDS,X,Y,W,H ...
//
// A panel is listed while any of it is on screen; CARDS counts the cards a
// stack draws now (0 while it or its cards are faded out).
Timer {
    id: probe

    required property var panels
    required property var stacks
    property string last: ""

    function rect(d: Drawer): string {
        return [d.x, d.y, d.width, d.height].map(Math.round).join(",");
    }

    interval: 16
    repeat: true
    running: true
    onTriggered: {
        const parts = probe.panels.filter(d => d.active).map(d => `panel=${rect(d)}`);
        for (const s of probe.stacks)
            parts.push(`stack=${s.edge},${s.drawn},${rect(s)}`);
        const line = parts.join(" ");
        if (line === probe.last)
            return;
        probe.last = line;
        console.info(`toast probe: ${line}`);
    }
}
