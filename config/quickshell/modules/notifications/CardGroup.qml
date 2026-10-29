import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// A notification group, the notifications of one app as one thing, in the notifications panel, the
// bell's peek and the popups alike. It has three states:
//
// - At rest, a stack: the newest card, the others tucked under it. One narrower edge peeks out
//   below it for a pair, two for more, each card's shadow cast onto the edge under it. The edges'
//   room is part of the group's height, so nothing overlaps.
// - With the pointer resting on it a beat, a preview: up to three cards spread down into whole
//   cards the list's gap apart, lifted over the list behind, which its owner fades toward the
//   surface by `recede`; nothing in the list moves. "+N more · click to open" sits under them when there are more. Leaving tucks
//   them back. The keyboard never previews.
// - Opened (`open`, which its owner keeps): every card takes its place under a heading with the
//   app, the count and Show less, and the group grows, so only what is below it moves. Each card
//   is then on its own.
//
// The cards are the owner's `delegate`, made as they are needed: the newest always, the preview's
// when it shows, every one while open. A delegate is a `CardFace` with a `required property int
// index`, showing `items[index]` and standing for `standing[index]`; the group places it and sets
// its `opens` and `selected`. A single notification is the card alone.
Item {
    id: root

    // The notifications, newest first.
    property var items: []
    // Whether it is opened; the owner keeps it, and `openRequested` asks for a change.
    property bool open: false
    property Component delegate
    // The list's gap between two cards, which the preview and the opened group keep.
    property int gap: K.Tokens.space.s8
    // Whether the keyboard is on the group: on its newest while it is a stack, on `current` open.
    property bool selected: false
    property int current: 0

    readonly property int count: root.items.length
    readonly property bool stacked: root.count > 1
    readonly property bool isOpen: root.open && root.stacked
    // How many cards the preview shows, the newest among them, and how many it leaves out.
    readonly property int fanCount: Math.min(root.count, 3)
    readonly property int more: root.count - root.fanCount
    // How many edges peek out under the stack.
    readonly property int edges: Math.max(0, Math.min(root.count - 1, 2))
    // How far each edge peeks out below the one above it, and how much narrower it is each side.
    readonly property int peek: K.Tokens.space.s8
    readonly property int narrower: K.Tokens.space.s8
    readonly property int headingHeight: K.Tokens.size.control_sm
    readonly property real radius: K.Look.nested(K.Tokens.radius.surface, K.Tokens.space.s8)
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1

    // The pointer on the stack, its newest card or what its preview covers.
    readonly property bool pointed: (root.head?.pointed ?? false) || catcherHover.hovered || guardHover.hovered || headingHover.hovered
    // The pointer has rested on it long enough to preview, and is still on it.
    property bool armed: false
    // Closed under the pointer: no preview and no click on the newest card until the pointer has
    // left, so Show less doesn't fan the cards it just tucked in, and a second click on it doesn't
    // land on the dismiss button that comes up where it was.
    property bool shut: false
    readonly property bool fanning: root.armed && !root.shut && root.stacked
    // How far the preview has spread (0 -> 1), and how far the group has opened.
    property real fan: root.fanning ? 1 : 0
    property real opening: root.isOpen ? 1 : 0
    // The preview is drawn over the list: the owner raises the group above the rows after it.
    readonly property bool raised: root.fan > 0
    // How far the rest of the list sits back behind the preview (0 -> 1): the owner fades every
    // other row by it, at `opacity.recede`, toward the panel's own surface.
    readonly property real recede: root.fan * (1 - root.opening)
    // Moving between two states; an owner sized by the group follows it frame by frame.
    readonly property bool settling: root.fan !== (root.fanning ? 1 : 0) || root.opening !== (root.isOpen ? 1 : 0)
    readonly property bool live: root.armed || root.isOpen || root.fan > 0 || root.opening > 0
    // How many cards are made.
    readonly property int made: !root.live ? Math.min(root.count, 1) : root.isOpen || root.opening > 0 ? root.count : root.fanCount

    // The cards made, by index, and the newest.
    property var cards: []
    readonly property Item head: root.cards[0] ?? null
    readonly property real headHeight: root.head ? root.head.implicitHeight : 0
    readonly property var heights: {
        const out = [];
        for (let k = 0; k < Math.max(root.made, root.edges + 1); k++)
            out.push(root.cards[k] ? root.cards[k].implicitHeight : root.headHeight);
        return out;
    }
    // Each card's top in the preview and opened, under the heading's room.
    readonly property var offsets: {
        const out = [];
        let y = 0;
        for (const h of root.heights) {
            out.push(y);
            y += h + root.gap;
        }
        return out;
    }
    readonly property real underHeading: root.headingHeight + root.gap
    readonly property real restHeight: root.headHeight + root.edges * root.peek
    readonly property int last: root.count - 1
    readonly property real openHeight: root.heights.length === root.count && root.stacked ? root.underHeading + root.offsets[root.last] + root.heights[root.last] : root.restHeight
    // Where each card is now, `{ x, y, w, h }`: at rest the newest, then the edges, narrower and
    // their newest's height, the rest tucked under the last; spread in the preview; in their places
    // opened.
    readonly property var boxes: {
        const out = [];
        const f = root.fan;
        const o = root.opening;
        const mix = (a, b, t) => a + (b - a) * t;
        for (let k = 0; k < root.heights.length; k++) {
            const edge = Math.min(k, root.edges);
            const rest = {
                x: edge * root.narrower,
                y: edge * root.peek,
                h: root.headHeight
            };
            const spread = k < root.fanCount ? f : 0;
            const fanned = {
                x: mix(rest.x, 0, spread),
                y: mix(rest.y, root.offsets[k], spread),
                h: mix(rest.h, root.heights[k], spread)
            };
            const x = K.Look.snap(mix(fanned.x, 0, o), root.dpr);
            out.push({
                x: x,
                y: K.Look.snap(mix(fanned.y, root.underHeading + root.offsets[k], o), root.dpr),
                w: root.width - x * 2,
                h: K.Look.snap(mix(fanned.h, root.heights[k], o), root.dpr)
            });
        }
        return out;
    }
    // How much of each card's content shows, by index: it fades in just after its edge has spread.
    // Values, not a function, as everything the cards bind to: a group going away takes its
    // functions' context with it while their bindings still run.
    readonly property var reveals: root.heights.map((h, k) => {
        const spread = Math.max(k < root.fanCount ? root.fan : 0, root.opening);
        return Math.max(0, Math.min(1, 1 - (1 - spread) * 5));
    })
    // The notifications each card stands for, by index: the whole group on a stack's newest card,
    // so its dismiss and its actions take them all.
    readonly property var standing: root.items.map((one, index) => index === 0 && !root.isOpen ? root.items : [one])
    // A box for a card the group no longer has, for the moment before what draws it goes.
    readonly property var nowhere: ({
            x: 0,
            y: 0,
            w: 0,
            h: 0
        })
    // The bottom of the preview's last card, and of the line under it.
    readonly property real fanBottom: {
        const box = root.boxes[root.fanCount - 1];
        return box ? box.y + box.h : 0;
    }
    readonly property real moreBottom: root.more > 0 ? root.fanBottom + root.gap + K.Tokens.type.small.line : root.fanBottom
    // How far down the group draws, its preview included: past its height while that shows.
    readonly property real extent: root.raised ? Math.max(root.height, root.moreBottom) : root.height

    signal openRequested(bool open)


    // Where card `k` sits once the group has opened, for an owner keeping it in view.
    function cardRect(k: int): rect {
        return Qt.rect(0, root.underHeading + (root.offsets[k] ?? 0), root.width, root.heights[k] ?? 0);
    }

    // What a click, Enter or Space on the group does: a stack opens, otherwise the card the
    // keyboard is on unfolds.
    function press(): void {
        if (root.stacked && !root.isOpen) {
            root.openRequested(true);
            return;
        }
        root.cards[root.isOpen ? Math.max(0, root.current) : 0]?.press();
    }

    function place(index: int, card: Item): void {
        const next = root.cards.slice();
        next[index] = card;
        root.cards = next;
        card.x = 0;
        card.width = Qt.binding(() => root.width);
        card.y = Qt.binding(() => root.boxes[index]?.y ?? 0);
        card.z = Qt.binding(() => (root.heights.length - index) * 3 + 1);
        card.opacity = Qt.binding(() => index === 0 ? 1 : root.reveals[index] ?? 0);
        card.visible = Qt.binding(() => card.opacity > 0);
        card.opens = Qt.binding(() => index === 0 && !root.isOpen);
        card.selected = Qt.binding(() => root.selected && (root.isOpen ? root.current === index : index === 0));
        if (index === 0)
            card.expandRequested.connect(() => root.openRequested(true));
    }

    // As many cards as `made`, adding or taking only the ones past what there is.
    function make(): void {
        while (slots.count < root.made)
            slots.append({});
        while (slots.count > root.made)
            slots.remove(slots.count - 1);
    }

    // The pointer has left: the preview tucks in, and the next rest previews again.
    function letGo(): void {
        if (root.pointed)
            return;
        root.armed = false;
        root.shut = false;
    }

    function unplace(index: int, card: Item): void {
        if (root.cards[index] !== card)
            return;
        const next = root.cards.slice();
        next[index] = null;
        while (next.length > 0 && !next[next.length - 1])
            next.pop();
        root.cards = next;
    }

    implicitHeight: K.Look.snap(root.restHeight + (root.openHeight - root.restHeight) * root.opening, root.dpr)
    // HAZARD: let go once the hover has settled, not as it changes. The cards the preview makes and
    // the catcher growing under the pointer move the hover between them on the way, and letting go
    // there would tuck the preview the moment it spreads.
    onPointedChanged: settle.restart()
    onIsOpenChanged: {
        if (!root.isOpen && root.pointed)
            root.shut = true;
    }
    onMadeChanged: root.make()
    Component.onCompleted: root.make()
    onStackedChanged: {
        if (!root.stacked && root.open)
            root.openRequested(false);
    }

    Behavior on fan {
        NumberAnimation {
            duration: K.Motion.reduced ? K.Tokens.duration.instant : root.fanning ? K.Tokens.duration.medium : K.Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: root.fanning ? K.Tokens.easing.standard_decel : K.Tokens.easing.standard_accel
        }
    }

    Behavior on opening {
        NumberAnimation {
            duration: K.Motion.reduced ? K.Tokens.duration.instant : root.isOpen ? K.Tokens.duration.medium : K.Tokens.duration.short
            easing.type: Easing.BezierSpline
            easing.bezierCurve: root.isOpen ? K.Tokens.easing.standard_decel : K.Tokens.easing.standard_accel
        }
    }

    // The hover settled, a turn of the event loop after it changed; a timer of the group's own, so
    // nothing calls into a group that has gone.
    Timer {
        id: settle

        interval: K.Tokens.duration.instant
        onTriggered: root.letGo()
    }

    // The beat the pointer rests before the preview spreads, about as long as it takes to spread.
    Timer {
        id: dwell

        interval: K.Tokens.duration.medium
        running: root.pointed && root.stacked && !root.armed && !root.isOpen && !root.shut
        onTriggered: root.armed = true
    }

    // Each previewed card's own shadow, straight under it on the list, under every card.
    Repeater {
        model: root.fan > 0 ? root.fanCount : 0

        delegate: RectangularShadow {
            required property int index
            readonly property var box: root.boxes[index] ?? root.nowhere

            x: box.x
            y: box.y
            z: 1
            width: box.w
            height: box.h
            radius: root.radius
            offset.y: K.Tokens.shadow.tucked.y + (K.Tokens.shadow.lifted.y - K.Tokens.shadow.tucked.y) * root.fan
            blur: K.Tokens.shadow.tucked.blur + (K.Tokens.shadow.lifted.blur - K.Tokens.shadow.tucked.blur) * root.fan
            color: K.Look.alpha(K.Tokens.color.shadow, K.Tokens.shadow.lifted.opacity * root.fan * (1 - root.opening))
        }
    }

    // The tucked cards' edges. Each card's face fades in over its edge once it has spread.
    Repeater {
        model: Math.max(0, root.heights.length - 1)

        delegate: Rectangle {
            required property int index
            readonly property int k: index + 1
            readonly property var box: root.boxes[k] ?? root.nowhere

            x: box.x
            y: box.y
            z: (root.heights.length - k) * 3
            width: box.w
            height: box.h
            radius: root.radius
            color: K.Tokens.color.surface_container
            visible: (root.reveals[k] ?? 0) < 1
        }
    }

    // Each card's shadow cast onto the edges tucked under it, cut to them. As the cards spread it
    // eases toward the shadow each casts on the list, and fades as that one takes over.
    Repeater {
        model: root.edges

        delegate: Item {
            id: cast

            required property int index
            readonly property var box: root.boxes[index] ?? root.nowhere
            readonly property var below: root.boxes.length > index + 1 ? root.boxes.slice(index + 1, root.edges + 1) : [root.nowhere]
            readonly property real fromX: Math.min(...cast.below.map(b => b.x))
            readonly property real toX: Math.max(...cast.below.map(b => b.x + b.w))
            readonly property real fromY: Math.min(...cast.below.map(b => b.y))
            readonly property real toY: Math.max(...cast.below.map(b => b.y + b.h))

            x: cast.fromX
            y: cast.fromY
            z: (root.heights.length - cast.index) * 3 - 1
            width: cast.toX - cast.fromX
            height: cast.toY - cast.fromY
            clip: true
            visible: root.fan < 1 && root.opening < 1

            RectangularShadow {
                x: cast.box.x - cast.x
                y: cast.box.y - cast.y
                width: cast.box.w
                height: cast.box.h
                radius: root.radius
                offset.y: K.Tokens.shadow.tucked.y + (K.Tokens.shadow.lifted.y - K.Tokens.shadow.tucked.y) * root.fan
                blur: K.Tokens.shadow.tucked.blur + (K.Tokens.shadow.lifted.blur - K.Tokens.shadow.tucked.blur) * root.fan
                color: K.Look.alpha(K.Tokens.color.shadow, K.Tokens.shadow.tucked.opacity * (1 - root.fan) * (1 - root.opening))
            }
        }
    }

    // HAZARD: a model the cards are added to and taken from one at a time, never a number. A
    // repeater given a new number makes every card again, the newest under the pointer with them,
    // which lets go of the hover the preview is waiting on.
    ListModel {
        id: slots
    }

    Repeater {
        model: slots
        delegate: root.delegate
        onItemAdded: (index, item) => root.place(index, item)
        onItemRemoved: (index, item) => root.unplace(index, item)
    }

    // Under the preview, when it leaves cards out.
    K.Text {
        objectName: "more"
        x: K.Look.centre(root.width - width, root.dpr)
        y: root.fanBottom + root.gap
        z: (root.heights.length + 1) * 3
        visible: root.more > 0 && opacity > 0
        opacity: (root.reveals[root.fanCount - 1] ?? 0) * (1 - root.opening)
        text: `+${root.more} more · click to open`
        style: K.Tokens.type.small
        color: K.Tokens.color.primary
    }

    // The tucked edges and the preview under the newest card are the group's: the pointer on them
    // keeps the preview and a click opens the group, whatever card it lands on.
    MouseArea {
        id: catcher

        y: root.headHeight
        z: (root.heights.length + 2) * 3
        width: root.width
        height: Math.max(0, root.extent - root.headHeight)
        visible: root.stacked && !root.isOpen
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.openRequested(true)

        HoverHandler {
            id: catcherHover

            // The cards under the preview take no hover while it covers them.
            blocking: true
        }
    }

    // Over the newest card while the group is shut, taking its hover and its clicks.
    MouseArea {
        readonly property var box: root.boxes[0] ?? root.nowhere

        y: box.y
        z: (root.heights.length + 2) * 3
        width: root.width
        height: box.h
        visible: root.shut
        hoverEnabled: true
        acceptedButtons: Qt.AllButtons

        HoverHandler {
            id: guardHover

            blocking: true
        }
    }

    // In the group's place while it is open: the app, how many, and Show less.
    K.Row {
        width: root.width
        height: root.headingHeight
        z: (root.heights.length + 2) * 3
        visible: root.opening > 0
        opacity: Math.max(0, Math.min(1, (root.opening - 0.5) * 2))
        gap: K.Tokens.space.s8
        align: "centre"

        HoverHandler {
            id: headingHover
        }

        K.Text {
            text: root.head?.appName || "Notification"
            style: K.Tokens.type.label
            maximumLines: 1
        }

        K.Chip {
            text: String(root.count)
        }

        K.Fill {}

        K.Button {
            objectName: "showLess"
            text: "Show less"
            kind: "Text"
            size: "Sm"
            activeFocusOnTab: false
            onActivated: root.openRequested(false)
        }
    }
}
