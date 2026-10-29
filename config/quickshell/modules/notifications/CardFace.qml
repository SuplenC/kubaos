import QtQuick
import KubaOS.Components as K

// What a notification card draws and how it answers the pointer and the keys, from plain values:
// `NotificationCard` feeds it a notification, and `test/qml` drives it without a server.
//
// A header band says who it is from and when: the app's icon (or the notification's image) on a
// small icon tile, an accent dot on its corner while unread, its name, the stack's count, and the
// day and time it came; at the band's far corner the dismiss button, shown while the pointer is on
// the card or the keyboard is, and beside it a chevron while the text is cut off. Under it, across
// the whole card, the summary, two lines of the body, then the buttons: the main action filled,
// the others as text after it, and Mark read while it is unread, unless the sender offers its own.
//
// A click, Enter or Space unfolds cut-off text to the whole of it (`unfoldRequested`), or asks for
// a stack to open where a stack opens (`expandRequested`); Esc folds it again. Its owner keeps
// `unfolded`, one card at a time. A card with nothing to unfold takes no click, and a click never
// runs an action: the buttons do, and Shift+Enter runs the main one (`runMain`).
Item {
    id: root

    property string appName
    // The tile's symbol, and the app's icon and the notification's image drawn over it.
    property string icon: "notifications"
    property string appIcon
    property string photo
    // The day and time it came.
    property string when
    // How many notifications the card stands for; past one it is a stack.
    property int count
    readonly property bool stacked: root.count > 1
    property bool unread: false
    property bool critical: false
    property bool selected: false
    property bool popup: false
    // Time left before the popup hides (1 -> 0).
    property real remaining: 1
    property string summary
    // The body as plain words.
    property string body
    // The main action's label, "" for none, and the other actions' labels, in order.
    property string mainAction
    property var otherActions: []
    // Whether a click asks for the stack to open rather than unfolding its newest.
    property bool opens: false
    // Whether the whole text shows; the owner keeps it.
    property bool unfolded: false

    // Whether it has a Mark read button of its own: while unread, unless the sender offers one,
    // as a chat app does, whose action tells the app too, so two never sit side by side.
    readonly property bool marksRead: root.unread && ![root.mainAction, ...root.otherActions].some(text => /^mark (as )?read$/i.test(text.trim()))

    // The lines of the summary and of the body shown while folded.
    readonly property int foldedLines: 2
    readonly property bool foldable: words.fullHeight > words.foldedHeight
    readonly property bool clickable: (root.opens && root.stacked) || root.foldable
    // The pointer is on the card, its buttons included, which take their own hover.
    readonly property bool pointed: hover.hovered
    readonly property color fg: root.critical ? K.Tokens.color.on_error_container : K.Tokens.color.on_surface
    readonly property color fgMuted: root.critical ? K.Look.alpha(K.Tokens.color.on_error_container, K.Tokens.opacity.subdued_text) : K.Tokens.color.on_surface_variant
    readonly property real radius: K.Look.nested(K.Tokens.radius.surface, K.Tokens.space.s8)
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // The band's inset from the card's top and from its corner: the dismiss button sits as close
    // to the corner as the curve lets it, so the words below keep the room.
    readonly property int band: K.Tokens.space.s8

    // A right-click on the card asks for its menu, at the pointer in the window's coordinates.
    signal menuRequested(point at)
    signal expandRequested
    signal unfoldRequested(bool open)
    signal dismissRequested
    signal mainPicked
    signal otherPicked(int index)
    signal markReadPicked

    // What a click, Enter or Space does.
    function press(): void {
        if (root.opens && root.stacked)
            root.expandRequested();
        else if (root.foldable)
            root.unfoldRequested(!root.unfolded);
    }

    // What Shift+Enter does: runs the main action. Whether there was one.
    function runMain(): bool {
        if (root.mainAction === "")
            return false;
        root.mainPicked();
        return true;
    }

    // What Esc does: folds the card if it is unfolded. Whether it did, so Esc goes on to close what
    // holds the card otherwise.
    function foldBack(): bool {
        if (!root.unfolded)
            return false;
        root.unfoldRequested(false);
        return true;
    }

    // Every height here is a binding, never a layout pass: a list places the cards after this one
    // by its height as it arrives, and doesn't move them for a change while they slide in.
    implicitHeight: K.Look.snapUp(words.y + words.height + K.Tokens.space.s12, root.dpr)
    Accessible.role: root.clickable ? Accessible.Button : Accessible.Grouping
    Accessible.name: root.summary

    Keys.onPressed: event => {
        const enter = event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space;
        if (enter && event.key !== Qt.Key_Space && (event.modifiers & Qt.ShiftModifier)) {
            event.accepted = true;
            if (!event.isAutoRepeat)
                root.runMain();
        } else if (enter) {
            event.accepted = true;
            if (!event.isAutoRepeat)
                root.press();
        } else if (event.key === Qt.Key_Escape) {
            event.accepted = root.foldBack();
        }
    }

    HoverHandler {
        id: hover

        cursorShape: root.clickable ? Qt.PointingHandCursor : Qt.ArrowCursor
    }

    // Under the card's buttons, which take their own clicks.
    MouseArea {
        id: pointer

        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: mouse => {
            if (mouse.button === Qt.RightButton)
                root.menuRequested(mapToItem(null, mouse.x, mouse.y));
            else
                root.press();
        }
    }

    Rectangle {
        anchors.fill: parent
        radius: root.radius
        color: root.critical ? K.Tokens.color.error_container : K.Tokens.color.surface_container

        K.StateLayer {
            radius: parent.radius
            content: root.fg
            hovered: root.clickable && root.pointed && !dismiss.hovered && !buttonsHover.hovered
            pressed: root.clickable && pointer.pressed && (pointer.pressedButtons & Qt.LeftButton) !== 0
        }
    }

    // The header band, a small control tall: who it is from and when.
    K.IconTile {
        id: tile

        x: K.Tokens.space.s12
        y: root.band
        size: "Sm"
        icon: root.icon
        source: root.appIcon
        photo: root.photo
        danger: root.critical
    }

    K.Badge {
        x: K.Look.snap(tile.x + tile.width - width / 2, root.dpr)
        y: K.Look.snap(tile.y - height / 2, root.dpr)
        visible: root.unread
        accent: true
    }

    K.Text {
        id: app

        readonly property real end: chevron.visible ? chevron.x : dismiss.x

        x: tile.x + tile.width + K.Tokens.space.s8
        y: root.band + K.Look.snap((K.Tokens.size.control_sm - height) / 2, root.dpr)
        room: Math.max(0, app.end - K.Tokens.space.s8 - x - (count.visible ? count.width + K.Tokens.space.s8 : 0) - (time.visible ? time.width + K.Tokens.space.s8 : 0))
        // One line, cut at the band's room: an app's name past it breaks at a space onto a line
        // that isn't shown.
        height: K.Tokens.type.label.line
        clip: true
        text: root.appName || "Notification"
        style: K.Tokens.type.label
        color: root.fg
    }

    // How many the stack holds.
    K.Chip {
        id: count

        x: app.x + app.width + K.Tokens.space.s8
        y: root.band + K.Look.snap((K.Tokens.size.control_sm - height) / 2, root.dpr)
        visible: root.stacked
        text: String(root.count)
    }

    K.Text {
        id: time

        x: (count.visible ? count.x + count.width : app.x + app.width) + K.Tokens.space.s8
        y: root.band + K.Look.snap((K.Tokens.size.control_sm - height) / 2, root.dpr)
        visible: text !== ""
        text: root.when
        style: K.Tokens.type.small
        color: root.fgMuted
    }

    // Down while folded, up while unfolded, in a box as big as the dismiss button's beside it. It
    // says the card unfolds; the click is the card's.
    Item {
        id: chevron

        objectName: "chevron"
        x: dismiss.x - width
        y: root.band
        width: K.Tokens.size.control_sm
        height: K.Tokens.size.control_sm
        visible: root.foldable

        K.Icon {
            x: K.Look.snap((chevron.width - width) / 2, root.dpr)
            y: K.Look.snap((chevron.height - height) / 2, root.dpr)
            name: root.unfolded ? "expand_less" : "expand_more"
            size: K.ButtonMetrics.of("Sm").icon
            color: root.fgMuted
        }
    }

    // Shown while the pointer is on the card or the keyboard is, its room kept at rest so the band
    // never moves.
    K.IconButton {
        id: dismiss

        x: root.width - root.band - width
        y: root.band
        visible: root.pointed || root.selected
        icon: "close"
        label: root.stacked ? qsTr("Dismiss all %1").arg(root.count) : qsTr("Dismiss")
        kind: "Text"
        size: "Sm"
        activeFocusOnTab: false
        onActivated: root.dismissRequested()
    }

    // Under the band, across the card: the summary, the body, then the buttons.
    Item {
        id: words

        // The text's height with each part cut at `lines`.
        function textHeight(lines: real): real {
            const summaryHeight = summary.text !== "" ? Math.min(summary.laid.length, lines) * summary.style.line : 0;
            const bodyHeight = body.text !== "" ? K.Tokens.space.s4 + Math.min(body.laid.length, lines) * body.style.line : 0;
            return summaryHeight + bodyHeight;
        }

        readonly property real foldedHeight: words.textHeight(root.foldedLines)
        readonly property real fullHeight: words.textHeight(Infinity)
        // How far it is unfolded (0 -> 1). Only a fold or an unfold moves it, so a card that
        // arrives, or whose words change, takes its height at once.
        property real progress: root.unfolded ? 1 : 0
        // The whole text is laid out from the moment it starts unfolding until it has folded.
        readonly property bool open: words.progress > 0
        readonly property real textShown: words.foldedHeight + (words.fullHeight - words.foldedHeight) * words.progress
        readonly property bool hasButtons: root.mainAction !== "" || root.otherActions.length > 0 || root.marksRead
        readonly property real buttonsHeight: words.hasButtons ? K.Tokens.space.s12 + K.Tokens.size.control_sm : 0

        x: K.Tokens.space.s12
        y: root.band + K.Tokens.size.control_sm + K.Tokens.space.s8
        width: Math.max(0, root.width - K.Tokens.space.s12 * 2)
        height: words.textShown + words.buttonsHeight

        Behavior on progress {
            NumberAnimation {
                duration: K.Motion.reduced ? K.Tokens.duration.instant : root.unfolded ? K.Tokens.duration.medium : K.Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: root.unfolded ? K.Tokens.easing.standard_decel : K.Tokens.easing.standard_accel
            }
        }

        // The text, cut to what has unfolded so far while it moves.
        Item {
            width: words.width
            height: words.textShown
            clip: words.progress > 0 && words.progress < 1

            K.Text {
                id: summary

                visible: text !== ""
                room: words.width
                text: root.summary
                style: K.Tokens.type.body_strong
                color: root.fg
                maximumLines: words.open ? Math.max(summary.laid.length, 1) : root.foldedLines
            }

            K.Text {
                id: body

                objectName: "body"
                y: (summary.text !== "" ? summary.height : 0) + K.Tokens.space.s4
                visible: text !== ""
                room: words.width
                text: root.body
                style: K.Tokens.type.body
                color: root.fgMuted
                maximumLines: words.open ? Math.max(body.laid.length, 1) : root.foldedLines
            }
        }

        // A filled main action's box starts on the text's edge; a row of text buttons starts its
        // first label there instead, its box reaching out into the padding as the peek's footer's.
        K.Row {
            id: buttons

            x: root.mainAction !== "" ? 0 : -K.ButtonMetrics.of("Sm").textPadding
            y: words.textShown + K.Tokens.space.s12
            visible: words.hasButtons
            gap: K.Tokens.space.s8

            HoverHandler {
                id: buttonsHover
            }

            K.Button {
                objectName: "main"
                visible: root.mainAction !== ""
                text: root.mainAction
                kind: root.critical ? "Danger" : "Filled"
                size: "Sm"
                activeFocusOnTab: false
                onActivated: root.mainPicked()
            }

            Repeater {
                model: root.otherActions

                delegate: K.Button {
                    required property string modelData
                    required property int index

                    objectName: `other${index}`
                    text: modelData
                    kind: "Text"
                    size: "Sm"
                    activeFocusOnTab: false
                    onActivated: root.otherPicked(index)
                }
            }

            K.Button {
                objectName: "markRead"
                visible: root.marksRead
                text: "Mark read"
                kind: "Text"
                size: "Sm"
                activeFocusOnTab: false
                onActivated: root.markReadPicked()
            }
        }
    }

    // The time left, a thin line along the bottom inside the padding, on whole device pixels so it
    // repaints only when it loses one.
    Rectangle {
        x: K.Tokens.space.s12
        y: root.height - height
        width: K.Look.snap((root.width - K.Tokens.space.s12 * 2) * root.remaining, root.dpr)
        height: K.Look.thin(root.dpr)
        radius: K.Tokens.radius.full
        color: K.Tokens.color.primary
        visible: root.popup && !root.critical
    }

    K.FocusRing {
        shown: root.selected
        controlRadius: root.radius
    }
}
