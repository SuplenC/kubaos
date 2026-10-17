import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.theme
import qs.components
import qs.services

// Screenshot, annotate, record and replay. Buttons that need a region close
// the panel first; the command starts once it has slid away, so slurp and
// the screenshot see the desktop, not the panel.
Rectangle {
    id: root

    implicitHeight: row.implicitHeight + Theme.spacing.sm * 2
    radius: Theme.rounding.xl
    color: Theme.m3surfaceContainer

    function later(cmd: list<string>): void {
        Panels.close();
        Quickshell.execDetached(["sh", "-c", 'sleep 0.4; exec "$@"', "sh", ...cmd]);
    }

    RowLayout {
        id: row

        anchors.fill: parent
        anchors.margins: Theme.spacing.sm
        anchors.leftMargin: Theme.spacing.lg
        spacing: Theme.spacing.sm

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 0

            StyledText {
                Layout.fillWidth: true
                text: "Capture"
                font.weight: Font.Medium
            }

            StyledText {
                Layout.fillWidth: true
                text: Recorder.recording ? `Recording  ·  ${Recorder.elapsedText}` : Recorder.replay ? "Replay buffer on" : "Screenshot or record"
                color: Recorder.active ? Theme.m3error : Theme.m3onSurfaceVariant
                font.pixelSize: Theme.font.sm
                font.features: Theme.font.tabular
            }
        }

        IconButton {
            icon: "screenshot_region"
            tooltip: "Screenshot region"
            baseColor: Theme.m3surfaceContainerHigh
            onClicked: root.later(["dot-hypr-screenshot", "region"])
        }

        IconButton {
            icon: "draw"
            tooltip: "Screenshot and annotate"
            baseColor: Theme.m3surfaceContainerHigh
            onClicked: root.later(["dot-hypr-screenshot", "region", "--edit"])
        }

        IconButton {
            icon: Recorder.recording ? "stop" : "screen_record"
            tooltip: Recorder.recording ? "Stop recording" : "Record region"
            baseColor: Theme.m3surfaceContainerHigh
            tint: Theme.m3error
            toggled: Recorder.recording
            onClicked: {
                if (Recorder.recording) {
                    Recorder.stop();
                } else {
                    Panels.close();
                    Recorder.record("region");
                }
            }
        }

        IconButton {
            icon: "desktop_windows"
            tooltip: "Record screen"
            baseColor: Theme.m3surfaceContainerHigh
            visible: !Recorder.recording
            onClicked: {
                Panels.close();
                Recorder.record("output");
            }
        }

        IconButton {
            icon: "replay_30"
            tooltip: Recorder.replay ? "Replay buffer off" : "Replay buffer: keep the last 30 s"
            baseColor: Theme.m3surfaceContainerHigh
            toggled: Recorder.replay
            onClicked: Recorder.toggleReplay()
        }
    }
}
