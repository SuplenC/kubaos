import QtQuick
import Quickshell
import Quickshell.Hyprland
import qs.theme
import qs.settings
import qs.components
import qs.services

// The workspace group of this monitor (1-3, 4-6 or 7-9), chosen from the
// monitor's active workspace. If an unplugged monitor's workspaces land here,
// the group follows whichever one is active. Scroll to move within the group.
//
// The active indicator stretches toward the new workspace: its leading edge
// moves fast, the trailing edge follows a little later.
Pill {
    id: root

    required property ShellScreen screen
    readonly property HyprlandMonitor monitor: Hyprland.monitorFor(screen)
    readonly property int size: Settings.workspacesPerMonitor
    readonly property int activeId: Math.max(1, monitor?.activeWorkspace?.id ?? 1)
    readonly property int first: Math.floor((activeId - 1) / size) * size + 1
    readonly property int slot: activeId - first
    readonly property int cell: Theme.bar.pillHeight - 6
    readonly property var kanji: ["一", "二", "三", "四", "五", "六", "七", "八", "九"]

    padding: 3
    spacing: 0
    interactive: true
    onScrolled: steps => Hypr.focusWorkspace(first + (slot - steps + size) % size)

    Item {
        implicitWidth: root.size * root.cell
        implicitHeight: root.cell

        Rectangle {
            id: indicator

            // Plain values, set in onSlotChanged, so the durations are chosen
            // before the Behaviors start.
            property real start: 0
            property real end: root.cell
            property bool ready: false
            property int lead: Theme.anim.normal
            property int trail: Theme.anim.large

            Component.onCompleted: {
                start = root.slot * root.cell;
                end = start + root.cell;
                ready = true;
            }

            x: start
            width: end - start
            height: root.cell
            radius: height / 2
            color: Theme.m3primary

            Connections {
                target: root

                function onSlotChanged() {
                    const right = root.slot * root.cell > indicator.start;
                    startAnim.duration = right ? indicator.trail : indicator.lead;
                    endAnim.duration = right ? indicator.lead : indicator.trail;
                    indicator.start = root.slot * root.cell;
                    indicator.end = indicator.start + root.cell;
                }
            }

            Behavior on start {
                enabled: indicator.ready

                Anim {
                    id: startAnim

                    easing.bezierCurve: Theme.anim.emphasizedDecel
                }
            }

            Behavior on end {
                enabled: indicator.ready

                Anim {
                    id: endAnim

                    easing.bezierCurve: Theme.anim.emphasizedDecel
                }
            }
        }

        Row {
            Repeater {
                model: root.size

                delegate: Item {
                    id: cell

                    required property int index
                    readonly property int workspaceId: root.first + index
                    readonly property bool isActive: workspaceId === root.activeId
                    readonly property bool occupied: Hyprland.workspaces.values.some(w => w.id === workspaceId && w.toplevels.values.length > 0)

                    width: root.cell
                    height: root.cell

                    Text {
                        anchors.centerIn: parent
                        text: Settings.workspaceKanji ? root.kanji[cell.workspaceId - 1] ?? cell.workspaceId : cell.workspaceId
                        font.family: Settings.workspaceKanji ? Theme.font.jp : Theme.font.sans
                        font.pixelSize: Theme.font.sm + 1
                        font.weight: cell.isActive || cell.occupied ? Font.DemiBold : Font.Medium
                        font.features: Theme.font.tabular
                        renderType: Text.NativeRendering
                        color: cell.isActive ? Theme.m3onPrimary : cell.occupied ? Theme.m3onSurface : Theme.m3outline

                        Behavior on color {
                            CAnim {}
                        }
                    }

                    // Occupied but not active: a small dot under the number.
                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: 2
                        width: 3
                        height: 3
                        radius: 1.5
                        color: Theme.m3tertiary
                        opacity: cell.occupied && !cell.isActive ? 1 : 0

                        Behavior on opacity {
                            Anim {
                                duration: Theme.anim.fast
                            }
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Hypr.focusWorkspace(cell.workspaceId)
                    }
                }
            }
        }
    }
}
