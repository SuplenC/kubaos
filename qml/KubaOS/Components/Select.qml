import QtQuick
import KubaOS.Components as K

// One of a list too long for radio rows or tabs: `kubaos-ui`'s `select::select`. As tall as a
// field, the control radius, a hairline, the picked option in `body_large` and a chevron at its
// end. Enter, Space or a click opens its menu on a popover floating under it, as wide as it, the
// picked option checked; ↑ ↓ and Enter pick in it, Esc or a click elsewhere closes it. The owner
// keeps the state: bind `currentIndex`, handle `picked()`.
K.Pressable {
    id: root

    property list<string> options
    property int currentIndex: -1
    // What it shows while nothing is picked.
    property string placeholder: ""
    property string icon: ""
    // Whether it shows itself open: the accent hairline, the chevron up. Its menu floats only
    // once `show()` has placed it.
    property bool open: false
    property bool placed: false

    signal picked(int index)

    readonly property bool fillWidth: true
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property color ink: root.enabled ? Tokens.color.on_surface : K.Look.alpha(Tokens.color.on_surface, Tokens.opacity.disabled_content)
    readonly property color muted: root.enabled ? Tokens.color.on_surface_variant : root.ink
    readonly property var probe: ({
            role: "component",
            name: "Select",
            variant: root.icon === "" ? "" : "icon",
            fixed: {
                width: false,
                height: true
            }
        })

    function relayout(): void {
        line.relayout();
    }

    // Opens the menu under the select, the keyboard on the picked option.
    function show(): void {
        const window = root.Window.window;
        if (!window)
            return;
        const at = root.mapToItem(window.contentItem, 0, root.height + Tokens.space.s4);
        popover.x = at.x;
        popover.y = at.y;
        menu.currentIndex = Math.max(root.currentIndex, 0);
        root.placed = true;
        root.open = true;
        menu.forceActiveFocus();
        Qt.callLater(menu.reveal);
    }

    function hide(): void {
        root.open = false;
        root.placed = false;
        root.forceActiveFocus();
    }

    implicitHeight: Tokens.size.control_md
    Accessible.role: Accessible.ComboBox
    Accessible.name: root.options[root.currentIndex] ?? root.placeholder
    onActivated: root.open ? root.hide() : root.show()

    Rectangle {
        anchors.fill: parent
        radius: Tokens.radius.control
        color: root.open ? Tokens.color.surface_container_highest : Tokens.color.surface_container_high
        border.width: K.Look.hairline(root.dpr)
        border.color: root.open ? Tokens.color.primary : Tokens.color.outline_variant

        K.StateLayer {
            radius: parent.radius
            active: root.enabled
            hovered: root.hovered
            pressed: root.down
        }
    }

    K.Row {
        id: line

        x: Tokens.space.s12
        width: root.width - Tokens.space.s12 * 2
        fixedHeight: root.height
        gap: Tokens.space.s8
        align: "centre"

        K.Icon {
            visible: root.icon !== ""
            name: root.icon
            size: Tokens.type.body_large.icon
            color: root.muted
        }

        K.Stack {
            fill: true

            K.Text {
                text: root.options[root.currentIndex] ?? root.placeholder
                style: Tokens.type.body_large
                color: root.currentIndex >= 0 ? root.ink : root.muted
                maximumLines: 1
            }
        }

        K.Icon {
            name: root.open ? "expand_less" : "expand_more"
            size: Tokens.type.body_large.icon
            color: root.muted
        }
    }

    K.FocusRing {
        shown: root.focused && root.enabled && !root.open
        controlRadius: Tokens.radius.control
    }

    // A click anywhere else in the window closes the menu.
    MouseArea {
        readonly property bool overlay: true

        parent: root.Window.window ? root.Window.window.contentItem : root
        anchors.fill: parent
        z: 999
        visible: root.open && root.placed
        onClicked: root.hide()
    }

    K.Popover {
        id: popover

        parent: root.Window.window ? root.Window.window.contentItem : root
        floating: true
        fillWidth: true
        width: root.width
        visible: root.open && root.placed

        K.Menu {
            id: menu

            // Eight items at most; a longer list scrolls.
            maxHeight: Tokens.size.control_sm * 8
            entries: root.options.map((option, index) => ({
                        text: option,
                        checkable: true,
                        checked: index === root.currentIndex
                    }))
            onActivated: index => {
                root.picked(index);
                root.hide();
            }
            onDismissed: root.hide()
        }
    }
}
