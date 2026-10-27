import QtQuick
import KubaOS.Components as K
import qs.components
import qs.services

// Lock / suspend / log out / reboot / shut down, as list rows in a column that slides out of the
// right edge. The panel is the rows' well: they sit 12 inside its surface, so their corners are
// concentric with it, as in a list. The selected row is where the keyboard is.
// Keys: j/k, h/l, arrows or Tab to move, Enter or Space to run, Esc to close.
Drawer {
    id: root

    readonly property var actions: [
        { icon: "lock", label: "Lock", run: () => Session.lock() },
        { icon: "bedtime", label: "Suspend", run: () => Session.suspend() },
        { icon: "logout", label: "Log out", run: () => Session.logout() },
        { icon: "restart_alt", label: "Reboot", run: () => Session.reboot() },
        { icon: "power_settings_new", label: "Shut down", danger: true, run: () => Session.poweroff() }
    ]
    property int selected: 0

    edge: "right"
    // A side column: Kōbō's step rail is the same narrow list of rows.
    contentWidth: K.Tokens.size.rail
    contentHeight: rows.implicitHeight
    // HAZARD: keep the size unanimated. A layout arranges itself only once it is shown, so the
    // first opening learns the rows' height as it starts, and animating it grows the panel along
    // the edge while it slides out.
    animateSize: false
    onOpened: {
        selected = 0;
        keys.forceActiveFocus();
    }

    function run(index: int): void {
        Panels.close();
        actions[index].run();
    }

    Item {
        id: keys

        anchors.fill: parent
        focus: true

        Keys.onPressed: event => {
            const k = event.key;
            const n = root.actions.length;
            if (k === Qt.Key_Up || k === Qt.Key_K || k === Qt.Key_H || k === Qt.Key_Left || k === Qt.Key_Backtab)
                root.selected = (root.selected + n - 1) % n;
            else if (k === Qt.Key_Down || k === Qt.Key_J || k === Qt.Key_L || k === Qt.Key_Right || k === Qt.Key_Tab)
                root.selected = (root.selected + 1) % n;
            else if (k === Qt.Key_Return || k === Qt.Key_Enter || k === Qt.Key_Space)
                root.run(root.selected);
            else
                return;
            event.accepted = true;
        }
    }

    K.Stack {
        id: rows

        width: parent.width
        fill: true
        gap: K.Tokens.space.s4
        leftPadding: K.Tokens.space.s12
        rightPadding: K.Tokens.space.s12
        topPadding: K.Tokens.space.s12
        bottomPadding: K.Tokens.space.s12

        Repeater {
            model: root.actions

            delegate: K.ListRow {
                id: action

                required property var modelData
                required property int index

                title: action.modelData.label
                icon: action.modelData.icon
                danger: action.modelData.danger === true
                selected: root.selected === action.index
                onActivated: root.run(action.index)
                onPointerOverChanged: if (pointerOver)
                    root.selected = action.index
            }
        }
    }
}
