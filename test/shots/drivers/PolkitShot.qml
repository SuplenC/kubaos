import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services
import qs.modules.polkit

// The polkit dialog over the wallpaper, asking for a password.
//
//   qml PolkitShot.qml -- MODE OUT [STATE]
//
// STATE: "" (asking), failed (the password was wrong).
QtObject {
    id: driver

    readonly property var args: Qt.application.arguments.slice(Qt.application.arguments.indexOf("--") + 1)
    readonly property ShellScreen screen: ShellScreen {}
    property var agent: null

    property Component polkit: Component {
        Polkit {}
    }

    property Component wallpaper: Component {
        Image {
            z: -1
            anchors.fill: parent
            source: "file://@WALLPAPERS@/shinjuku/kubaos-neon-sign.jpg"
            fillMode: Image.PreserveAspectCrop
            sourceSize.width: driver.screen.width * 2
        }
    }

    property Capture capture: Capture {
        path: driver.args[1]
    }

    property Timer shown: Timer {
        interval: 200
        onTriggered: {
            const window = driver.windowOf(driver.agent);
            if (!window) {
                console.error("shot: the polkit agent has no window");
                Qt.exit(1);
                return;
            }
            if (driver.args[2] === "failed")
                driver.agent.flow.failed = true;
            driver.wallpaper.createObject(window.contentItem);
            driver.capture.window = window;
            driver.capture.start();
        }
    }

    function windowOf(scope: QtObject): var {
        for (const o of scope.data)
            if (o && o.contentItem !== undefined)
                return o;
        return null;
    }

    Component.onCompleted: {
        K.Tokens.mode = driver.args[0] === "light" ? "light" : "dark";
        Hypr.focusedScreen = driver.screen;
        driver.agent = driver.polkit.createObject(null);
        driver.shown.start();
    }
}
