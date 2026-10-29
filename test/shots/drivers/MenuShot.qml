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
// starts, at full opacity), form-long (a form View with more fields than the body has room for),
// actions-end (the same, the keys on its last Action), form-scrolled (the long form, the keys on
// its last field), file-recent ("Import VPN config…" asking for its file, the newest it takes
// listed), file-typed (`~/Down` typed in it, completed by Downloads), folder ("Unpin a Suite
// version…", the keys on its folder field, the projects listed).
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
            if (driver.state.startsWith("actions"))
                driver.menu.openActions();
            if (driver.state === "actions-long" || driver.state === "actions-end")
                driver.menu.actions = driver.manyActions;
            if (driver.state === "pushed") {
                driver.menu.engine.push({
                    kind: "list",
                    filter: ""
                }, driver.menu.engine.results);
                holdSlide.start();
            }
            if (driver.state === "form-long" || driver.state === "form-scrolled")
                driver.menu.engine.push({
                    kind: "form",
                    fields: driver.manyFields,
                    submit: "Save"
                });
            if (driver.state.startsWith("file") || driver.state === "folder") {
                driver.menu.engine.push(driver.state === "folder" ? driver.unpinForm : driver.importForm);
                toPath.start();
            }
            if (driver.state === "form-scrolled" || driver.state === "actions-end")
                toLast.start();
            driver.capture.start();
        }
    }

    // "Import VPN config…" as the Daemon asks for its file, and "Unpin a Suite version…" for its
    // suite and folder.
    readonly property var importForm: ({
            kind: "form",
            id: "argument-form",
            title: "Import VPN config…",
            submit: "Import VPN config…",
            fields: [
                {
                    id: "path",
                    label: "path",
                    kind: "file",
                    accept: ["conf", "ovpn"],
                    placeholder: "~/Downloads/work.conf",
                    value: ""
                }
            ]
        })
    readonly property var unpinForm: ({
            kind: "form",
            id: "argument-form",
            title: "Unpin a Suite version…",
            submit: "Unpin a Suite version…",
            fields: [
                {
                    id: "suite",
                    label: "suite",
                    kind: "text",
                    placeholder: "node",
                    value: "node"
                },
                {
                    id: "folder",
                    label: "folder",
                    kind: "folder",
                    accept: [],
                    placeholder: "~/Projects/acme",
                    value: ""
                }
            ]
        })
    // Seconds since the epoch, `hours` before the harness's frozen clock.
    function before(hours: real): int {
        return Math.round(Time.now.getTime() / 1000 - hours * 3600);
    }
    // What the Daemon answers for each state: the newest files the field takes, what completes
    // `~/Down`, and the folders in the projects folder.
    readonly property var paths: ({
            "file-recent": [
                {
                    path: "/home/kuba/Downloads/work-kuba.conf",
                    title: "work-kuba.conf",
                    folder: false,
                    modified: driver.before(2),
                    highlights: []
                },
                {
                    path: "/home/kuba/Documents/vpn/home-router.ovpn",
                    title: "home-router.ovpn",
                    folder: false,
                    modified: driver.before(30),
                    highlights: []
                },
                {
                    path: "/home/kuba/Desktop/work-old.conf",
                    title: "work-old.conf",
                    folder: false,
                    modified: driver.before(24 * 16),
                    highlights: []
                }
            ],
            "file-typed": [
                {
                    path: "/home/kuba/Downloads",
                    title: "Downloads",
                    folder: true,
                    modified: -1,
                    highlights: [0, 1, 2, 3]
                }
            ],
            "folder": [
                {
                    path: "/home/kuba/Projects/acme",
                    title: "acme",
                    folder: true,
                    modified: -1,
                    highlights: []
                },
                {
                    path: "/home/kuba/Projects/kubaos",
                    title: "kubaos",
                    folder: true,
                    modified: -1,
                    highlights: []
                },
                {
                    path: "/home/kuba/Projects/kotodama",
                    title: "kotodama",
                    folder: true,
                    modified: -1,
                    highlights: []
                }
            ]
        })

    // The keys on the form's path field, what was typed in it, and the Daemon's paths for it,
    // once the View has laid out.
    property Timer toPath: Timer {
        interval: K.Tokens.duration.medium * 2
        onTriggered: {
            const content = driver.menu.contentItem;
            const field = driver.find(content, it => typeof it.focusField === "function" && it.visible);
            if (!field) {
                console.warn("shot: no file field found");
                return;
            }
            field.focusField();
            if (driver.state === "file-typed")
                field.text = "~/Down";
            driver.menu.engine.completions = driver.paths[driver.state];
            driver.capture.start();
        }
    }

    // The first item under `item` for which `test` holds, depth first.
    function find(item: Item, test: var): Item {
        if (!item)
            return null;
        if (test(item))
            return item;
        for (const kid of item.children) {
            const hit = driver.find(kid, test);
            if (hit)
                return hit;
        }
        return null;
    }

    // The keys on the last field of the form, as Shift+Tab from the first takes them there, or on
    // the last Action, once the View or the Action panel has laid out. The shot waits from here
    // for the scroll that follows.
    property Timer toLast: Timer {
        interval: K.Tokens.duration.medium * 2
        onTriggered: {
            const content = driver.menu.contentItem;
            if (driver.state === "actions-end") {
                const panel = driver.find(content, it => it.maxHeight !== undefined && typeof it.reveal === "function");
                if (panel)
                    panel.current = driver.manyActions.length - 1;
                else
                    console.warn("shot: no Action panel found");
            } else {
                const form = driver.find(content, it => typeof it.takeFirst === "function" && it.fields !== undefined);
                if (form)
                    form.move(-1);
                else
                    console.warn("shot: no form found");
            }
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
