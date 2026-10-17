import QtQuick
import Quickshell
import Quickshell.Io
import qs.services

// Entry points for `qs ipc call <target> <function>`, bound in Hyprland.
// The table in docs/CONTRACTS.md is the source of truth; keep them in sync.
Scope {
    IpcHandler {
        target: "launcher"

        function toggle(): void {
            Panels.toggle("launcher");
        }
    }

    IpcHandler {
        target: "controlcenter"

        function toggle(): void {
            Panels.toggle("controlcenter");
        }
    }

    IpcHandler {
        target: "powermenu"

        function toggle(): void {
            Panels.toggle("powermenu");
        }
    }

    IpcHandler {
        target: "wallpaper"

        function toggle(): void {
            Panels.toggle("wallpaper");
        }

        // The picker on its settings tab: Motion and every ukiyo option.
        function settings(): void {
            Wallpapers.openSettings = true;
            Panels.open("wallpaper");
        }

        // Wallpaper motion: always, ac or never. Remembered.
        function animate(mode: string): string {
            return Motion.setMode(mode) ? Motion.mode : `unknown mode '${mode}': always, ac or never`;
        }

        // JSON: what's shown, the motion mode, whether motion is allowed
        // right now, which screens are playing which video, and the ukiyo
        // scene (if one is the wallpaper).
        function status(): string {
            return JSON.stringify({
                wallpaper: Wallpapers.shown,
                video: Wallpapers.isVideo(Wallpapers.shown),
                // ukiyo: the current scene, and whether ukiyo draws it (the
                // desktop's own layer has stepped aside).
                scene: Ukiyo.current,
                ukiyo: Ukiyo.shown,
                mode: Motion.mode,
                allowed: Motion.allowed,
                onBattery: Motion.onBattery,
                screensOff: Motion.screensOff,
                playing: Motion.playing
            });
        }
    }

    IpcHandler {
        target: "session"

        function lock(): void {
            Session.lock();
        }
    }

    IpcHandler {
        target: "recorder"

        function toggle(): void {
            Recorder.toggle();
        }

        function stop(): void {
            Recorder.stop();
        }
    }

    IpcHandler {
        target: "notifications"

        function toggleDnd(): void {
            Notifs.toggleDnd();
        }

        function clear(): void {
            Notifs.clear();
        }
    }
}
