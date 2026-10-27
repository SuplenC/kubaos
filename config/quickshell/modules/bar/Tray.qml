import QtQuick
import Quickshell
import Quickshell.Widgets
import Quickshell.Services.SystemTray
import KubaOS.Components as K
import qs.components

// StatusNotifierItem tray. Left click activates, middle click secondary action, right click (or
// left on menu-only items) opens the app's menu. Each app's icon sits in a circle as tall as a
// chip, which takes the state layer under the pointer, 4 in from the pill's curve.
Pill {
    id: root

    required property QsWindow window

    visible: SystemTray.items.values.length > 0
    padding: K.Tokens.space.s4
    spacing: 0

    Repeater {
        model: SystemTray.items

        delegate: Item {
            id: entry

            required property SystemTrayItem modelData

            implicitWidth: K.Tokens.size.chip
            implicitHeight: K.Tokens.size.chip

            K.StateLayer {
                radius: K.Tokens.radius.full
                hovered: mouse.containsMouse
                pressed: mouse.pressed
            }

            IconImage {
                x: (K.Tokens.size.chip - K.Tokens.icon.s16) / 2
                y: (K.Tokens.size.chip - K.Tokens.icon.s16) / 2
                implicitSize: K.Tokens.icon.s16
                source: entry.modelData.icon
                asynchronous: true
            }

            MouseArea {
                id: mouse

                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
                cursorShape: Qt.PointingHandCursor
                // A wheel's notch is 120 (Qt's QWheelEvent::angleDelta).
                onWheel: wheel => entry.modelData.scroll(wheel.angleDelta.y / 120, false)
                onClicked: mouse => {
                    const item = entry.modelData;
                    if (mouse.button === Qt.MiddleButton) {
                        item.secondaryActivate();
                    } else if (mouse.button === Qt.LeftButton && !item.onlyMenu) {
                        item.activate();
                    } else if (item.hasMenu) {
                        const pos = entry.mapToItem(null, 0, entry.height + K.Tokens.space.s12);
                        item.display(root.window, pos.x, pos.y);
                    }
                }
            }

            K.Hint {
                text: entry.modelData.tooltipTitle || entry.modelData.title
                shown: text !== "" && mouse.containsMouse
            }
        }
    }
}
