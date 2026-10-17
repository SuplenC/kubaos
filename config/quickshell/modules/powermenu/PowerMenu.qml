import QtQuick
import qs.theme
import qs.components
import qs.services

// Lock / suspend / log out / reboot / shut down, in a column that slides out
// of the right edge. The selected button morphs into a filled rounded square.
// Keys: j/k, h/l, arrows or Tab to move, Enter or Space to run, Esc to close.
Drawer {
    id: root

    readonly property var actions: [
        { icon: "lock", label: "Lock", run: () => Session.lock() },
        { icon: "bedtime", label: "Suspend", run: () => Session.suspend() },
        { icon: "logout", label: "Log out", run: () => Session.logout() },
        { icon: "restart_alt", label: "Reboot", run: () => Session.reboot() },
        { icon: "power_settings_new", label: "Shut down", run: () => Session.poweroff() }
    ]
    readonly property int button: 64
    property int selected: 0

    edge: "right"
    contentWidth: button + Theme.spacing.lg * 2
    contentHeight: column.implicitHeight + Theme.spacing.lg * 2
    onOpened: {
        selected = 0;
        column.forceActiveFocus();
    }

    function run(index: int): void {
        Panels.close();
        actions[index].run();
    }

    Column {
        id: column

        x: Theme.spacing.lg
        y: Theme.spacing.lg
        spacing: Theme.spacing.sm
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

        Repeater {
            model: root.actions

            delegate: Rectangle {
                id: action

                required property var modelData
                required property int index
                readonly property bool isSelected: root.selected === index
                readonly property bool danger: index === root.actions.length - 1

                width: root.button
                height: root.button
                radius: stateLayer.pressed ? Theme.rounding.md : isSelected ? Theme.rounding.xl : root.button / 2
                color: isSelected ? (danger ? Theme.m3error : Theme.m3primary) : Theme.m3surfaceContainerHigh

                Behavior on color {
                    CAnim {}
                }

                Behavior on radius {
                    Anim {
                        duration: Theme.anim.large
                        easing.bezierCurve: Theme.anim.spatial
                    }
                }

                Icon {
                    anchors.centerIn: parent
                    name: action.modelData.icon
                    size: 28
                    filled: action.isSelected
                    color: action.isSelected ? (action.danger ? Theme.m3onError : Theme.m3onPrimary) : Theme.m3onSurfaceVariant
                }

                StateLayer {
                    id: stateLayer

                    onClicked: root.run(action.index)
                    onContainsMouseChanged: if (containsMouse)
                        root.selected = action.index
                }
            }
        }

        StyledText {
            width: root.button
            topPadding: Theme.spacing.xs
            horizontalAlignment: Text.AlignHCenter
            text: root.actions[root.selected].label
            color: Theme.m3onSurface
            font.pixelSize: Theme.font.sm
            font.weight: Font.Medium
            elide: Text.ElideNone
        }
    }
}
