import QtQuick
import Quickshell
import KubaOS.Components as K
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
    // A narrow strip, as wide as the side column the power menu takes.
    contentWidth: K.Tokens.size.rail
    contentHeight: row.implicitHeight

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

    K.Row {
        id: row

        width: parent.width
        fill: true
        gap: K.Tokens.space.s12
        align: "centre"
        leftPadding: K.Tokens.space.s16
        rightPadding: K.Tokens.space.s16
        topPadding: K.Tokens.space.s16
        bottomPadding: K.Tokens.space.s16

        K.Icon {
            name: root.icon
            size: K.Tokens.icon.s24
            color: K.Tokens.color.primary
        }

        K.ProgressBar {
            value: root.value
        }

        // As wide as "100", the number right-aligned in it, so the bar keeps one length and the
        // number never runs past the panel.
        Item {
            implicitWidth: widest.implicitWidth
            implicitHeight: percent.implicitHeight

            K.Text {
                id: widest

                visible: false
                text: "100"
                style: K.Tokens.type.body_strong
            }

            K.Text {
                id: percent

                x: parent.width - percent.width
                text: `${Math.round(root.value * 100)}`
                style: K.Tokens.type.body_strong
                color: K.Tokens.color.on_surface
            }
        }
    }
}
