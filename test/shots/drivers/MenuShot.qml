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
// STATE: typed ("fi" typed), actions (then the Action panel open), actions-long (the Action panel
// with more Actions than the panel has room for), pushed (a list View pushed, held where its slide
// starts, at full opacity), form-long (a form View with more fields than the body has room for).
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

    // Twenty Actions: more than the Action panel has room for on any screen.
    readonly property var manyActions: Array.from({
            length: 20
        }, (_, n) => ({
                title: n === 0 ? "Open" : `Action ${n + 1}`,
                group: n < 12 ? "" : "kubaos",
                shortcut: n === 1 ? "ctrl+enter" : ""
            }))

    // Twelve fields: more than the body has room for on a Full HD screen.
    readonly property var manyFields: Array.from({
            length: 12
        }, (_, n) => n % 4 === 3 ? {
                label: `Option ${n + 1}`,
                kind: "checkbox",
                value: n % 8 === 3
            } : {
                label: `Field ${n + 1}`,
                kind: "text",
                placeholder: "Type here",
                value: ""
            })

    property Timer typed: Timer {
        interval: 300
        onTriggered: {
            if (driver.state === "actions" || driver.state === "actions-long")
                driver.menu.openActions();
            if (driver.state === "actions-long")
                driver.menu.actions = driver.manyActions;
            if (driver.state === "pushed") {
                driver.menu.engine.push({
                    kind: "list",
                    filter: ""
                }, driver.menu.engine.results);
                holdSlide.start();
            }
            if (driver.state === "form-long")
                driver.menu.engine.push({
                    kind: "form",
                    fields: driver.manyFields,
                    submit: "Save"
                });
            driver.capture.start();
        }
    }

    // Once the push's slide is over, the body is put back where it starts: a View's first frame,
    // held for the shot.
    property Timer holdSlide: Timer {
        interval: K.Tokens.duration.medium * 2
        onTriggered: {
            driver.menu.slide = driver.menu.viewTravel;
            driver.menu.settle = 1;
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
