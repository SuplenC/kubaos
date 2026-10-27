import QtQuick
import KubaOS.Components as K

// What `Row`, `Stack` and `Grid` share: the children they lay out, laying them out again when one
// changes, and filling. Every child's box is rounded up to whole device pixels and placed on one.
//
// Sizes flow both ways, as in iced: a layout offers each child the room it may take (`room`, read
// by text that wraps and by layouts inside it) and sets its width, then reads back its height.
// A child that has `fillWidth` takes the room it is offered.
//
// A pass lays each layout out at most once: a child is given its room and width before it lays
// out, and a layout whose inputs haven't changed since it last arranged its children skips the
// work. On the QML specimen's components page a resize arranges 163 of its 293 layouts, once each,
// in about 15 ms; laying each child out again after setting its width took 48,955 arrangements
// and about 850 ms, doubling at every level. `qml-specimen` fails when a pass arranges more than
// twice per layout.
//
// A layout that changes lays out again before its window draws the next frame (`FrameQueue`), so
// a page shown, pushed or changed is drawn laid out from its first frame.
Item {
    id: root

    // Takes all the room it is given, whatever its children need.
    property bool fill: false
    // Whether a child fills, which makes the layout fill too, as iced's do.
    property bool childFills: false
    readonly property bool fillWidth: root.fill || root.childFills
    // The widest the layout may grow, offered by its parent.
    property real room: Infinity
    property real leftPadding: 0
    property real rightPadding: 0
    property real topPadding: 0
    property real bottomPadding: 0
    // What the geometry check reads this box as; each layout sets its own.
    property var probe: null

    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    // What the layout last arranged, to skip arranging again when nothing it reads has changed.
    // HAZARD: keep these in this object. Declared on the layout itself, they make Qt 6.11 evaluate
    // the bindings of the grids in the specimen's Repeater delegates with no context (`root` and
    // `console` undefined), and those grids lose their columns.
    readonly property QtObject laid: QtObject {
        // Whether a child, the children or the padding changed since. The width, room and scale
        // it arranged at are kept below and compared instead.
        property bool dirty: true
        property real width: NaN
        property real room: NaN
        property real dpr: NaN
        // Which children it arranged, as their indices, so a child shown or hidden that changes
        // nothing costs nothing.
        property string kids: ""
        // Whether it waits in `FrameQueue` for its window's next frame.
        property bool queued: false
    }

    // Places `kids`, sets their sizes, and the layout's implicit size.
    function arrange(kids: list<Item>): void {
    }

    // Which of the children `kids` are, as `laid.kids` holds them.
    function indices(kids: list<Item>): string {
        const all = Array.from(root.children);
        return kids.map(kid => all.indexOf(kid)).join(",");
    }

    // The children laid out: the visible ones, less a `Repeater`'s own item. Read only while the
    // layout itself is visible, when a child's `visible` is its own.
    function managed(): list<Item> {
        const kids = [];
        for (const kid of root.children) {
            if (kid.visible && typeof kid.itemAt !== "function")
                kids.push(kid);
        }
        return kids;
    }

    // Offers `kid` `room`, sets its width and lays it out once, so its size is read fresh: `stretch`
    // wide when it fills and `stretch` is a number, else as wide as it needs. A child knows whether
    // it fills only once it has laid out its own children, so one that turns out otherwise than it
    // said before is laid out a second time.
    function fit(kid: Item, room: real, stretch: real): void {
        if (kid.room !== undefined)
            kid.room = room;
        const stretches = () => !isNaN(stretch) && kid.fillWidth === true;
        const before = stretches();
        if (before)
            kid.width = stretch;
        root.settle(kid);
        const after = stretches();
        if (after && !before) {
            kid.width = stretch;
            root.settle(kid);
        } else if (!after) {
            kid.width = K.Look.snapUp(kid.implicitWidth, root.dpr);
        }
    }

    // Lays `kid` out at once when it is laid out at all, so its size is read fresh.
    function settle(kid: Item): void {
        if (typeof kid.relayout === "function")
            kid.relayout();
    }

    // Arranges the children, unless the layout is hidden (it is arranged when shown) or nothing it
    // reads has changed since it last did. Its width counts only when it fills: otherwise its
    // children are measured in `room` alone.
    function relayout(): void {
        if (!root.visible)
            return;
        const sameWidth = !root.fillWidth || root.width === root.laid.width;
        if (!root.laid.dirty && sameWidth && root.room === root.laid.room && root.dpr === root.laid.dpr)
            return;
        K.LayoutPasses.count += 1;
        const kids = root.managed();
        root.childFills = kids.some(kid => kid.fillWidth === true);
        root.arrange(kids);
        root.laid.width = root.width;
        root.laid.room = root.room;
        root.laid.dpr = root.dpr;
        root.laid.kids = root.indices(kids);
        // HAZARD: clear this last. Arranging changes the children's sizes, which mark the layout
        // dirty again, and it has already read them.
        root.laid.dirty = false;
    }

    function schedule(): void {
        root.laid.dirty = true;
        Qt.callLater(root.relayout);
        K.FrameQueue.add(root, root.Window.window);
    }

    // A child shown or hidden: laid out again only when that changes which children are laid out.
    // Hiding an ancestor hides every child at once and changes nothing here.
    function kidShownOrHidden(): void {
        if (!root.visible)
            return;
        if (root.indices(root.managed()) !== root.laid.kids)
            root.schedule();
    }

    // The children whose size changes lay this out again.
    property var watched: []

    // Connects or disconnects `kid`'s changes of size to laying this out again.
    function watch(kid: Item, on: bool): void {
        const signals = [kid.implicitWidthChanged, kid.implicitHeightChanged];
        if (kid.fillWidthChanged)
            signals.push(kid.fillWidthChanged);
        for (const changed of signals) {
            if (on)
                changed.connect(root.schedule);
            else
                changed.disconnect(root.schedule);
        }
        if (on)
            kid.visibleChanged.connect(root.kidShownOrHidden);
        else
            kid.visibleChanged.disconnect(root.kidShownOrHidden);
    }

    onChildrenChanged: {
        const kids = Array.from(root.children);
        for (const kid of root.watched.filter(kid => !kids.includes(kid)))
            root.watch(kid, false);
        for (const kid of kids.filter(kid => !root.watched.includes(kid)))
            root.watch(kid, true);
        root.watched = kids;
        root.schedule();
    }
    Component.onDestruction: {
        for (const kid of root.watched)
            root.watch(kid, false);
    }
    onWidthChanged: {
        if (root.fillWidth)
            root.schedule();
    }
    onVisibleChanged: {
        if (root.visible && root.laid.dirty)
            root.schedule();
    }
    onRoomChanged: root.schedule()
    onDprChanged: root.schedule()
    onFillChanged: root.schedule()
    onLeftPaddingChanged: root.schedule()
    onRightPaddingChanged: root.schedule()
    onTopPaddingChanged: root.schedule()
    onBottomPaddingChanged: root.schedule()
    Component.onCompleted: root.schedule()
}
