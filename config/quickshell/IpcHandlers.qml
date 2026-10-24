import QtQuick
import Quickshell
import Quickshell.Io
import qs.services

// Entry points for `qs ipc call <target> <function>`, bound in Hyprland.
// The table in docs/CONTRACTS.md is the source of truth; keep them in sync.
Scope {
    IpcHandler {
        target: "controlcenter"

        function toggle(): void {
            Panels.toggle("controlcenter");
        }

        // The Bluetooth tab, searching for devices to pair.
        function bluetooth(): void {
            Bt.show(true);
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

        // The picker on its settings tab.
        function settings(): void {
            Wallpapers.openSettings = true;
            Panels.open("wallpaper");
        }

        // Wallpaper motion: always, ac or never. Remembered.
        function animate(mode: string): string {
            return Motion.setMode(mode) ? Motion.mode : `unknown mode '${mode}': always, ac or never`;
        }

        // JSON: what's shown, the motion mode, whether motion is allowed
        // right now, and which screens are playing which video.
        function status(): string {
            return JSON.stringify({
                wallpaper: Wallpapers.shown,
                video: Wallpapers.isVideo(Wallpapers.shown),
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

        // `scripts/dot-session` waits for false before it restarts the shell.
        function locked(): bool {
            return Session.locked;
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
        target: "clock"

        function toggle(): void {
            Panels.toggle("clock");
        }

        function settings(): void {
            Time.show("settings", "");
        }

        // The popup with `text` in its date search.
        function search(text: string): void {
            Time.show("", text);
        }

        // Starts a timer (25m, 1h30m, 90 seconds); a notification says
        // when it rings, or why it didn't start.
        function timer(duration: string): void {
            Time.startTimer(duration, error => {
                if (error !== "")
                    Time.notify("No timer", error);
                else
                    Time.notify("Timer", `${Time.left(Time.timerTotal)}, rings at ${Qt.formatDateTime(new Date(Time.timerEnd), "HH:mm")}`);
            });
        }

        // Sets an alarm (7:30, 6pm, friday 17:00), and says when.
        function alarm(time: string): void {
            Time.addAlarm(time, error => {
                if (error !== "")
                    Time.notify("No alarm", error);
                else
                    Time.notify("Alarm", Time.alarms.map(a => a.label).join(", "));
            });
        }

        // Starts or stops the stopwatch, with the popup open on it.
        function stopwatch(): void {
            Time.toggleStopwatch();
            Time.show("stopwatch", "");
        }

        // JSON: whether the engine runs, today, the zones, countdowns,
        // timer, stopwatch and alarms, and whether the popup is open.
        function status(): string {
            return JSON.stringify(Object.assign(Time.status(), {
                open: Panels.current === "clock"
            }));
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
