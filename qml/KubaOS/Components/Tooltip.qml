import QtQuick
import KubaOS.Components as K

// What a control does, on a card over it while the pointer is over it or the keyboard is on it:
// `kubaos-ui`'s `tooltip::tooltip`. Where a `Hint` names a control in one line, a tooltip
// explains one: a title in `body_strong`, the body in `small` under it, and what it holds 8 below
// (its keys, as `KeyHints`). The card is a `Popover` as wide as a menu (240), 8 above the control,
// or 8 below where there is no room above (a header's button), starting where it starts and held
// inside the window, or inside `within` (a panel), on whole device pixels: wider than a chip at a
// toolbar's start, a centred card would hang past the panel.
//
// Floating (`floating`, the default), the card is drawn on the window's own content item, as a
// `Hint`'s bubble is, so neither a clipping panel nor what is laid out after the control covers
// it, and it is no part of the page's layout: the tooltip fills its control, its parent. In place
// (the specimen), it is laid out where it is, as tall as the card.
Item {
    id: root

    default property alias content: extra.data
    property string title
    property string body
    property bool shown: false
    property bool floating: true
    // The box the card keeps inside, a panel, rather than the whole window; it opens below where
    // the box leaves no room above.
    property Item within: null

    // What a geometry dump leaves out: a floating card is no part of the page.
    readonly property bool overlay: root.floating
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // Where the card floats: the window's content item, or the control when there is no window.
    readonly property Item host: Window.window ? Window.window.contentItem : null

    // Puts the floating card over the control, on whole device pixels. A place in the window isn't
    // a binding, so it is worked out after each frame drawn while the card shows: it follows a
    // panel sliding in, and costs nothing while nothing moves.
    function place(): void {
        if (!root.floating || !root.host || !root.shown)
            return;
        const at = root.mapToItem(root.host, 0, 0);
        const box = root.within ? root.within.mapToItem(root.host, 0, 0) : Qt.point(0, 0);
        const left = box.x;
        const right = root.within ? box.x + root.within.width : root.host.width;
        const above = at.y - Tokens.space.s8 - card.height;
        const below = at.y + root.height + Tokens.space.s8;
        floater.x = K.Look.snap(Math.max(left, Math.min(at.x, right - card.width)), root.dpr);
        floater.y = K.Look.snap(above < box.y ? below : above, root.dpr);
    }

    anchors.fill: root.floating ? root.parent : undefined
    implicitWidth: root.floating ? 0 : card.implicitWidth
    implicitHeight: root.floating ? 0 : card.implicitHeight
    z: 1
    onShownChanged: root.place()
    Component.onCompleted: Qt.callLater(root.place)

    Connections {
        target: root.floating && root.shown ? root.Window.window : null

        function onFrameSwapped() {
            root.place();
        }
    }

    Item {
        id: floater

        readonly property bool overlay: root.floating

        parent: root.floating ? (root.host ?? root) : root
        z: 1
        width: card.width
        height: card.height
        // Its control's visibility, which it no longer inherits on the window's content item.
        visible: floater.opacity > 0 && root.visible
        opacity: !root.floating || root.shown ? 1 : 0

        Behavior on opacity {
            NumberAnimation {
                duration: Tokens.duration.short
                easing.type: Easing.BezierSpline
                easing.bezierCurve: Tokens.easing.standard
            }
        }

        Item {
            id: card

            readonly property var probe: ({
                    role: "component",
                    name: "Tooltip",
                    variant: "",
                    fixed: {
                        width: true,
                        height: false
                    }
                })

            implicitWidth: surface.implicitWidth
            implicitHeight: surface.implicitHeight
            width: implicitWidth
            height: implicitHeight

            K.Popover {
                id: surface

                width: implicitWidth
                height: implicitHeight
                fixedWidth: Tokens.size.menu
                room: Tokens.size.menu

                K.Stack {
                    gap: Tokens.space.s8

                    K.Stack {
                        gap: Tokens.space.s4

                        K.Text {
                            text: root.title
                            style: Tokens.type.body_strong
                        }

                        K.Text {
                            visible: root.body !== ""
                            text: root.body
                            style: Tokens.type.small
                            color: Tokens.color.on_surface_variant
                        }
                    }

                    K.Slot {
                        id: extra

                        visible: children.length > 0
                    }
                }
            }
        }
    }
}
