import QtQuick
import KubaOS.Components as K

// One value asked for in a popover: `kubaos-ui`'s `prompt::prompt`. Its label in `label` type with
// an Enter keycap at the label row's end, the Field under it, and the Field's Note under that when
// there is something to say: the library's `Labelled`, `Field`, `Keycap` and `Note`, nothing of its
// own. It is `size.menu` wide.
//
// It floats on a Popover of its own (`show`), `space.s8` under the text it changes, its label and
// Field lined up with that text's start, or its end for text at a panel's end; or it sits inside a
// menu's popover between its separators (`inMenu`), with no popover of its own, so its Field's
// edges land where a floating one's do. It opens with the keyboard in its Field and the whole value
// selected. Enter takes the value (`saved`); Esc, or a click outside a floating one, cancels
// (`cancelled`). A value it can't take (`check` gives the reason) turns the Field invalid with the
// reason in its Note, and it stays open until the value is one it takes.
//
// Floating, it fades in over `short` on `emphasized_decel` and out over `micro` on
// `emphasized_accel`, and closes itself once saved or cancelled; with reduce motion it shows and
// goes at once. Its owner puts the keyboard back on what opened it.
Item {
    id: root

    property string label
    // The value: what it opens with, then what is typed.
    property alias text: field.text
    // Why `typed` can't be taken, in a few words, or "" when it can.
    property var check: typed => ""
    // What the Note says while the value is one it takes, if anything: what a rule reads as.
    property var note: typed => ""
    // Inside a menu's popover: no popover of its own.
    property bool inMenu: false
    // Over the window, under its anchor, while `open`; laid out in place otherwise (the specimen).
    property bool floating: false
    property bool open: false
    // Enter was pressed on a value it can't take, and the value hasn't become one since.
    property bool invalid: false
    // Whether its Field shows the keyboard in it, whatever Qt's focus: the specimen's view of it.
    property alias focused: field.focused

    // Its own popover: floating, it is on the window's content item, not under the prompt.
    readonly property alias popover: card

    signal saved(string text)
    signal cancelled

    readonly property bool fillWidth: root.inMenu
    readonly property real dpr: Window.window ? Window.window.devicePixelRatio : 1
    // A floating prompt is no part of the page the geometry check reads.
    readonly property bool overlay: root.floating
    readonly property var probe: ({
            role: "component",
            name: "Prompt",
            variant: root.inMenu ? "menu" : "",
            fixed: {
                width: true,
                height: false
            }
        })

    // Opens it `space.s8` under `anchor`, its label and Field lined up with the anchor's start, or
    // with its end when `held` is "end", the keyboard in its Field with all of the value selected.
    function show(anchor: Item, held: string): void {
        const window = root.Window.window;
        if (!window)
            return;
        const at = anchor.mapToItem(window.contentItem, 0, anchor.height + Tokens.space.s8);
        const start = held === "end" ? at.x + anchor.width + Tokens.space.s12 - card.width : at.x - Tokens.space.s12;
        card.x = K.Look.snap(start, root.dpr);
        card.y = K.Look.snap(at.y, root.dpr);
        root.invalid = false;
        root.open = true;
        root.focusField();
    }

    // Closes a floating prompt; the owner puts the keyboard back.
    function hide(): void {
        root.open = false;
    }

    // The keyboard into its Field, all of the value selected.
    function focusField(): void {
        field.forceActiveFocus();
        field.selectAll();
    }

    // Enter: takes the value, or turns the Field invalid with the reason.
    function take(): void {
        if (root.check(field.text) !== "") {
            root.invalid = true;
            return;
        }
        root.saved(field.text);
        if (root.floating)
            root.hide();
    }

    function cancel(): void {
        root.cancelled();
        if (root.floating)
            root.hide();
    }

    function relayout(): void {
        if (root.inMenu)
            labelled.relayout();
        else
            card.relayout();
    }

    // Floating, it takes no room where it is declared.
    implicitWidth: root.inMenu || root.floating ? 0 : Tokens.size.menu
    implicitHeight: root.floating ? 0 : root.inMenu ? labelled.implicitHeight : card.implicitHeight

    K.Popover {
        id: card

        parent: root.floating && root.Window.window ? root.Window.window.contentItem : root
        floating: root.floating
        fixedWidth: Tokens.size.menu
        width: Tokens.size.menu
        visible: !root.inMenu && opacity > 0
        opacity: !root.floating || root.open ? 1 : 0

        Behavior on opacity {
            NumberAnimation {
                duration: K.Motion.reduced ? Tokens.duration.instant : root.open ? Tokens.duration.short : Tokens.duration.micro
                easing.type: Easing.BezierSpline
                easing.bezierCurve: root.open ? Tokens.easing.emphasized_decel : Tokens.easing.emphasized_accel
            }
        }

        // The Labelled's room when the prompt has a popover of its own.
        K.Slot {
            id: inCard

            fill: true
        }
    }

    K.Labelled {
        id: labelled

        parent: root.inMenu ? root : inCard
        width: root.inMenu ? root.width : inCard.width
        label: root.label
        action: enter
        note: root.invalid ? root.check(field.text) : root.note(field.text)
        invalid: root.invalid

        K.Field {
            id: field

            status: root.invalid ? "Invalid" : "Normal"
            onAccepted: root.take()
            // Once the value is one it takes, the Field is no longer invalid.
            onTextChanged: {
                if (root.invalid && root.check(field.text) === "")
                    root.invalid = false;
            }
            onKeyPressed: event => {
                if (event.key === Qt.Key_Escape) {
                    root.cancel();
                    event.accepted = true;
                } else if (event.key === Qt.Key_Tab || event.key === Qt.Key_Backtab) {
                    // Floating, its Field is its one stop; in a menu, Tab goes on through the menu.
                    event.accepted = root.floating;
                }
            }
        }
    }

    Component {
        id: enter

        K.Keycap {
            key: "Enter"
        }
    }

    // A click anywhere else in the window cancels a floating prompt.
    MouseArea {
        readonly property bool overlay: true

        parent: root.Window.window ? root.Window.window.contentItem : root
        anchors.fill: parent
        z: 999
        visible: root.floating && root.open
        onClicked: root.cancel()
    }
}
