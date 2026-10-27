import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services

// Screenshot, annotate, record and replay. Buttons that need a region close
// the panel first; the command starts once it has slid away, so slurp and
// the screenshot see the desktop, not the panel.
K.Card {
    id: root

    function later(cmd: list<string>): void {
        Panels.close();
        Quickshell.execDetached(["sh", "-c", 'sleep 0.4; exec "$@"', "sh", ...cmd]);
    }

    K.Row {
        fill: true
        gap: K.Tokens.space.s4
        align: "centre"

        // Takes the room the buttons leave, so its lines end before them.
        K.Stack {
            fill: true

            K.Text {
                text: "Capture"
                style: K.Tokens.type.body_strong
                maximumLines: 1
            }

            // The elapsed time ticks, so it is mono and keeps its width.
            K.Text {
                text: Recorder.recording ? `Recording · ${Recorder.elapsedText}` : Recorder.replay ? "Replay buffer on" : "Screenshot or record"
                style: Recorder.recording ? K.Tokens.type.mono : K.Tokens.type.small
                color: Recorder.active ? K.Tokens.color.error : K.Tokens.color.on_surface_variant
                maximumLines: 1
            }
        }

        K.IconButton {
            icon: "screenshot_region"
            label: "Screenshot region"
            kind: "Tonal"
            size: "Sm"
            onActivated: root.later(["dot-hypr-screenshot", "region"])
        }

        K.IconButton {
            icon: "draw"
            label: "Screenshot and annotate"
            kind: "Tonal"
            size: "Sm"
            onActivated: root.later(["dot-hypr-screenshot", "region", "--edit"])
        }

        K.IconButton {
            icon: Recorder.recording ? "stop" : "screen_record"
            label: Recorder.recording ? "Stop recording" : "Record region"
            kind: Recorder.recording ? "Danger" : "Tonal"
            size: "Sm"
            onActivated: {
                if (Recorder.recording) {
                    Recorder.stop();
                } else {
                    Panels.close();
                    Recorder.record("region");
                }
            }
        }

        K.IconButton {
            visible: !Recorder.recording
            icon: "desktop_windows"
            label: "Record screen"
            kind: "Tonal"
            size: "Sm"
            onActivated: {
                Panels.close();
                Recorder.record("output");
            }
        }

        K.IconButton {
            icon: "replay_30"
            label: Recorder.replay ? "Replay buffer off" : "Replay buffer: keep the last 30 s"
            kind: Recorder.replay ? "Filled" : "Tonal"
            size: "Sm"
            onActivated: Recorder.toggleReplay()
        }
    }
}
