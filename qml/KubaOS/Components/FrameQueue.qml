pragma Singleton

import QtQuick

// The layouts that changed since their window's last frame, laid out as that window starts its
// next one, before it syncs and draws: a layout shown, resized or given new children is drawn
// laid out in its first frame, never one frame late. `Layout` queues itself here as well as with
// `Qt.callLater`, which runs before the next frame on some platforms and after it on others.
// `afterAnimating` comes after the frame's polish, so a Qt positioner or view a flush resizes
// polishes on the next frame; bind such an item to the layout's size, not to a polish of its own.
QtObject {
    id: queue

    // The layouts waiting, in the order they changed.
    property var waiting: []
    // The windows whose next frame lays them out.
    property var windows: []
    property bool flushing: false
    // How many times one flush lays one layout out at the most. One that keeps changing as it is
    // laid out is left to `Qt.callLater` past that, so a frame never waits on one that doesn't
    // settle.
    readonly property int rounds: 8

    // Lays `layout` out before `window`'s next frame.
    function add(layout: Item, window: Window): void {
        if (window === null || layout.laid.queued)
            return;
        layout.laid.queued = true;
        queue.waiting.push(layout);
        if (queue.flushing || queue.windows.includes(window))
            return;
        queue.windows.push(window);
        window.afterAnimating.connect(queue.flush);
    }

    // Lays out every layout waiting, and those that laying them out makes wait, until none does.
    function flush(): void {
        const windows = queue.windows;
        queue.windows = [];
        // A window can be gone by now: Quickshell deletes a layer window each time it hides.
        for (const window of windows) {
            if (window && window.afterAnimating)
                window.afterAnimating.disconnect(queue.flush);
        }
        queue.flushing = true;
        const times = new Map();
        while (queue.waiting.length > 0) {
            const layout = queue.waiting.shift();
            // A layout destroyed while it waited has no `laid` left.
            if (!layout || !layout.laid)
                continue;
            layout.laid.queued = false;
            const n = (times.get(layout) ?? 0) + 1;
            times.set(layout, n);
            if (n <= queue.rounds)
                layout.relayout();
        }
        queue.flushing = false;
    }
}
