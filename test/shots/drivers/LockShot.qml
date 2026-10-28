import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services
import qs.modules.lock

// The lock screen: dark over the neon sign, light over the busy, dark lanterns, which the light
// veil has to carry, or either over the wallpaper named.
//
//   qml LockShot.qml -- MODE OUT [STATE [WALLPAPER]]
//
// STATE: idle (or none), error (a wrong password), waiting (the security key asked for a touch).
// WALLPAPER: a picture of the wallpaper sets, as `set/name.jpg`.
QtObject {
    id: driver

    readonly property var args: Qt.application.arguments.slice(Qt.application.arguments.indexOf("--") + 1)
    readonly property bool light: driver.args[0] === "light"
    readonly property string state: driver.args[2] ?? "idle"
    readonly property string wallpaper: driver.args[3] ?? (driver.light ? "shinjuku/omoide-yokocho-lanterns.jpg" : "shinjuku/kubaos-neon-sign.jpg")
    readonly property ShellScreen screen: ShellScreen {}
    property var surface: null

    property Component lock: Component {
        LockSurface {
            width: driver.screen.width
            height: driver.screen.height
            auth: Auth {
                message: driver.state === "error" ? "Wrong password. Fat fingers? Try again." : ""
                keyState: driver.state === "waiting" ? "waiting" : ""
            }

            // The window's own colour, which a grab of its content leaves out.
            Rectangle {
                z: -1
                anchors.fill: parent
                color: parent.Window.window ? parent.Window.window.color : K.Tokens.color.surface_container_lowest
            }
        }
    }

    property Capture capture: Capture {
        interval: 2000
        path: driver.args[1]
    }

    Component.onCompleted: {
        K.Tokens.mode = driver.light ? "light" : "dark";
        Wallpapers.current = `@WALLPAPERS@/${driver.wallpaper}`;
        driver.surface = driver.lock.createObject(null);
        driver.capture.window = driver.surface;
        driver.capture.start();
    }
}
