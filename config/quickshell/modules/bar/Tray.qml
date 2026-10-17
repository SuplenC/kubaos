import QtQuick
import Quickshell
import Quickshell.Widgets
import Quickshell.Services.SystemTray
import qs.theme
import qs.components

// StatusNotifierItem tray. Left click activates, middle click secondary action,
// right click (or left on menu-only items) opens the app's menu.
Pill {
    id: root

    required property QsWindow window

    visible: SystemTray.items.values.length > 0
    spacing: Theme.spacing.sm

    Repeater {
        model: SystemTray.items

        delegate: Item {
            id: entry

            required property SystemTrayItem modelData

            anchors.verticalCenter: parent?.verticalCenter
            width: Theme.bar.icon
            height: Theme.bar.icon
            scale: mouse.pressed ? 0.85 : mouse.containsMouse ? 1.1 : 1

            Behavior on scale {
                Anim {
                    duration: Theme.anim.small
                    easing.bezierCurve: Theme.anim.spatialFast
                }
            }

            IconImage {
                anchors.fill: parent
                source: entry.modelData.icon
                asynchronous: true
            }

            MouseArea {
                id: mouse

                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
                cursorShape: Qt.PointingHandCursor
                onWheel: wheel => entry.modelData.scroll(wheel.angleDelta.y / 120, false)
                onClicked: mouse => {
                    const item = entry.modelData;
                    if (mouse.button === Qt.MiddleButton) {
                        item.secondaryActivate();
                    } else if (mouse.button === Qt.LeftButton && !item.onlyMenu) {
                        item.activate();
                    } else if (item.hasMenu) {
                        const pos = entry.mapToItem(null, 0, entry.height + Theme.spacing.md);
                        item.display(root.window, pos.x, pos.y);
                    }
                }
            }

            Tooltip {
                text: entry.modelData.tooltipTitle || entry.modelData.title
                shown: text !== "" && mouse.containsMouse
            }
        }
    }
}
