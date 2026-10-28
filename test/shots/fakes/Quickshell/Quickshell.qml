pragma Singleton
import QtQuick

// Stand-in for Quickshell's global: a fixed user and no real environment, so nothing the shell
// reads from the session reaches a shot.
QtObject {
    readonly property var screens: []

    function env(name: string): string {
        return ({
                USER: "kuba",
                HOME: "/home/kuba",
                XDG_RUNTIME_DIR: "/nonexistent/run"
            })[name] ?? "";
    }

    function execDetached(command: var): void {
        console.log(`exec: ${JSON.stringify(command)}`);
    }

    function iconPath(name: string, check: var): string {
        return name.startsWith("/") ? `file://${name}` : "";
    }
}
