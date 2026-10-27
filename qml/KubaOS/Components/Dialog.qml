import QtQuick
import QtQuick.Effects
import KubaOS.Components as K

// A question over a dimmed screen: `kubaos-ui`'s `dialog::dialog`. One width, 32 inside, the
// surface radius, the dialog shadow. A 24 icon and a `headline` title, the `body` 12 under them,
// the buttons 32 under that: the dismiss button (Cancel) on the left, the actions on the right,
// the one Enter presses last. A `Danger` dialog shows the warning sign and the error colour.
//
// Opening it (making it visible, or loading it visible) moves focus into it, onto `initialFocus`;
// closing it puts focus back where it was. Tab and Shift+Tab go round its buttons and never leave
// it. Esc dismisses it, as its dismiss button does.
FocusScope {
    id: root

    // Esc, or the dismiss button: back out.
    signal dismissed

    // `Normal` or `Danger`.
    property string tone: "Normal"
    property string icon: "info"
    property string title
    property string body
    // The button on the left that backs out, or none.
    property Item dismiss: null
    // The buttons on the right, the one Enter presses last.
    property list<Item> actions
    // What the keyboard starts on: the last action, unless set.
    property Item initialFocus: root.actions.length > 0 ? root.actions[root.actions.length - 1] : root.dismiss
    // What had the keyboard before it opened, given back when it closes.
    property Item returnFocus: null
    // Whether opening it takes the keyboard. Off only for a picture of a dialog, as in the
    // specimen, which leaves the keyboard where it is.
    property bool takesFocus: true

    readonly property bool danger: root.tone === "Danger"
    readonly property color tint: root.danger ? Tokens.color.error : Tokens.color.primary
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    readonly property var probe: ({
            role: "component",
            name: "Dialog",
            variant: root.tone,
            fixed: {
                width: true,
                height: false
            }
        })

    function relayout(): void {
        content.relayout();
    }

    implicitWidth: Tokens.size.dialog
    implicitHeight: content.implicitHeight

    function takeFocus(): void {
        if (!root.takesFocus)
            return;
        root.returnFocus = root.Window.activeFocusItem;
        if (root.initialFocus)
            root.initialFocus.forceActiveFocus();
        else
            root.forceActiveFocus();
    }

    // Moves the keyboard `step` buttons along, round from the last to the first and back.
    function cycle(step: int): void {
        const buttons = [root.dismiss, ...root.actions].filter(item => item && item.visible && item.enabled);
        if (buttons.length === 0)
            return;
        const at = buttons.findIndex(item => item.activeFocus);
        const next = at < 0 ? 0 : (at + step + buttons.length) % buttons.length;
        buttons[next].forceActiveFocus(step > 0 ? Qt.TabFocusReason : Qt.BacktabFocusReason);
    }

    function giveFocusBack(): void {
        if (root.returnFocus)
            root.returnFocus.forceActiveFocus();
        root.returnFocus = null;
    }

    onVisibleChanged: root.visible ? root.takeFocus() : root.giveFocusBack()
    Keys.onEscapePressed: root.dismissed()
    Keys.onTabPressed: event => {
        root.cycle(1);
        event.accepted = true;
    }
    Keys.onBacktabPressed: event => {
        root.cycle(-1);
        event.accepted = true;
    }
    Component.onCompleted: {
        for (const item of [root.dismiss, spacer, ...root.actions]) {
            if (item)
                item.parent = buttons;
        }
        // A dialog loaded already visible never sees `visible` change. Its loader puts it in the
        // window after this, so it takes focus once that is done.
        if (root.visible)
            Qt.callLater(root.takeFocus);
    }

    RectangularShadow {
        anchors.fill: surface
        radius: surface.radius
        offset.y: Tokens.shadow.dialog.y
        blur: Tokens.shadow.dialog.blur
        color: K.Look.alpha(Tokens.color.shadow, Tokens.shadow.dialog.opacity)
    }

    Rectangle {
        id: surface

        anchors.fill: parent
        radius: Tokens.radius.surface
        color: Tokens.color.surface_container_high
        border.width: K.Look.hairline(root.dpr)
        border.color: K.Look.alpha(root.tint, root.danger ? Tokens.opacity.danger_edge : Tokens.opacity.dialog_edge)
    }

    K.Slot {
        id: content

        width: root.width
        fill: true
        leftPadding: Tokens.space.s32
        rightPadding: Tokens.space.s32
        topPadding: Tokens.space.s32
        bottomPadding: Tokens.space.s32

        K.Stack {
            fill: true

            K.Row {
                gap: Tokens.space.s12
                align: "centre"

                K.Icon {
                    name: root.danger ? "warning" : root.icon
                    size: Tokens.icon.s24
                    color: root.tint
                }

                K.Text {
                    text: root.title
                    style: Tokens.type.headline
                    color: Tokens.color.on_surface
                }
            }

            K.Gap {
                space: Tokens.space.s12
            }

            K.Text {
                text: root.body
                style: Tokens.type.body
                color: Tokens.color.on_surface_variant
            }

            K.Gap {
                space: Tokens.space.s32
            }

            K.Row {
                id: buttons

                gap: Tokens.space.s8
                align: "centre"
            }
        }
    }

    // Between the dismiss button and the actions, pushing the actions to the right.
    property Item spacer: K.Fill {}
}
