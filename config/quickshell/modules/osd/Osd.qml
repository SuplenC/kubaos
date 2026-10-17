import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services

// Volume / brightness indicator, rising out of the bottom edge of the
// focused screen. Hyprland binds the keys to wpctl and brightnessctl; this
// only watches the resulting values.
Drawer {
    id: root

    required property ShellScreen screen
    property string kind: "volume"
    property bool visibleNow: false
    // Ignore the values services report while starting up.
    property bool armed: false
    readonly property real value: kind === "volume" ? (Audio.muted ? 0 : Audio.volume) : Brightness.value
    readonly property string icon: kind === "volume" ? Audio.icon : Brightness.icon

    function show(newKind: string): void {
        if (!armed)
            return;
        kind = newKind;
        visibleNow = true;
        hideTimer.restart();
    }

    edge: "bottom"
    takesFocus: false
    animateSize: false
    shown: visibleNow && Hypr.focusedScreen === screen
    contentWidth: 320
    contentHeight: 60

    Timer {
        interval: 2000
        running: true
        onTriggered: root.armed = true
    }

    Timer {
        id: hideTimer

        interval: 1600
        onTriggered: root.visibleNow = false
    }

    Connections {
        target: Audio

        function onVolumeChanged() {
            root.show("volume");
        }

        function onMutedChanged() {
            root.show("volume");
        }
    }

    Connections {
        target: Brightness

        function onValueChanged() {
            root.show("brightness");
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.spacing.xl
        anchors.rightMargin: Theme.spacing.xl
        spacing: Theme.spacing.md

        Icon {
            name: root.icon
            size: 22
            filled: true
            color: Theme.m3primary
        }

        Rectangle {
            Layout.fillWidth: true
            implicitHeight: 8
            radius: 4
            color: Theme.m3surfaceContainerHighest

            Rectangle {
                width: Math.max(parent.height, parent.width * Math.min(1, root.value))
                height: parent.height
                radius: 4
                color: Theme.m3primary

                Behavior on width {
                    Anim {
                        duration: Theme.anim.small
                        easing.bezierCurve: Theme.anim.emphasizedDecel
                    }
                }
            }
        }

        StyledText {
            Layout.preferredWidth: 30
            horizontalAlignment: Text.AlignRight
            text: Math.round(root.value * 100)
            font.weight: Font.DemiBold
            font.features: Theme.font.tabular
        }
    }
}
