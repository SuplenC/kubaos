import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services
import qs.modules.menu

// The Command menu on a 1920×1080 screen over a plain backdrop, put in a state through its own
// calls (what `kotodama open` and Ctrl+K reach), not key events, which a headless window gets
// only once it has focus.
//
//   qml MenuShot.qml -- MODE OUT STATE
//
// STATE: typed ("fi" typed), actions (then the Action panel open).
QtObject {
    id: driver

    readonly property var args: Qt.application.arguments.slice(Qt.application.arguments.indexOf("--") + 1)
    readonly property string state: driver.args[2] ?? "typed"
    readonly property ShellScreen screen: ShellScreen {
        // design-allow(size): a Full HD screen's resolution, the screen the menu is laid out on.
        width: 1920
        // design-allow(size): a Full HD screen's resolution, the screen the menu is laid out on.
        height: 1080
    }
    property var menu: null

    property Component menuComponent: Component {
        Menu {}
    }

    property Component backdrop: Component {
        Rectangle {
            z: -1
            anchors.fill: parent
            color: K.Tokens.color.surface_container
        }
    }

    property Capture capture: Capture {
        path: driver.args[1]
    }

    property Timer typed: Timer {
        interval: 300
        onTriggered: {
            if (driver.state === "actions")
                driver.menu.openActions();
            driver.capture.start();
        }
    }

    Component.onCompleted: {
        K.Tokens.mode = driver.args[0] === "light" ? "light" : "dark";
        Hypr.focusedScreen = driver.screen;
        driver.menu = driver.menuComponent.createObject(null);
        driver.backdrop.createObject(driver.menu.contentItem);
        driver.capture.window = driver.menu;
        driver.menu.show("fi");
        driver.typed.start();
    }
}
