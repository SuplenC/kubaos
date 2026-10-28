import QtQuick
import Quickshell
import KubaOS.Components as K
import qs.services
import qs.modules.frame

// The shell's frame for one screen over the wallpaper, as Frame.qml draws it (bar, border,
// shadow, scrim), with one panel open.
//
//   qml FrameShot.qml -- MODE OUT PANEL [STATE]
//
// PANEL is a name Panels opens (controlcenter, clock, sysmon, powermenu, wallpaper), or osd,
// notifications, or bar (nothing open). STATE puts the panel in a state, through the services
// and public properties only, so the driver works with any version of the panel:
//   controlcenter: "" (Wi-Fi tab, the history scrolled to its middle), bluetooth, bluetooth-code
//                  (with the history cleared), keyboard, scanning (a Wi-Fi scan that has found
//                  nothing yet, the history cleared)
//   clock: answer (a question typed into the search), settings
//   sysmon: process (a process's details)
//   wallpaper: settings
//   osd: volume, brightness
//   bar: traymenu (the network item's menu open)
QtObject {
    id: driver

    readonly property var args: Qt.application.arguments.slice(Qt.application.arguments.indexOf("--") + 1)
    readonly property string panel: driver.args[2]
    readonly property string state: driver.args[3] ?? ""
    readonly property ShellScreen screen: ShellScreen {}
    property var frame: null
    property var window: null

    property Component frameComponent: Component {
        Frame {
            screen: driver.screen
        }
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

    // The OSD hides itself a moment after a change; shown again and again, as a held volume key
    // does, it stays up for the shot.
    property var osd: null
    property Timer keepOsd: Timer {
        interval: 400
        repeat: true
        triggeredOnStart: true
        onTriggered: driver.osd.show(driver.state === "brightness" ? "brightness" : "volume")
    }

    property Timer open: Timer {
        interval: 100
        onTriggered: driver.openPanel()
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

    function windowOf(scope: QtObject): var {
        for (const o of scope.data)
            if (o && o.contentItem !== undefined)
                return o;
        return null;
    }

    function openPanel(): void {
        const root = driver.window.contentItem;
        if (driver.panel === "osd") {
            const osd = driver.find(root, it => it.armed !== undefined && typeof it.show === "function");
            if (!osd) {
                console.error("shot: no OSD in the frame");
                Qt.exit(1);
                return;
            }
            osd.armed = true;
            driver.osd = osd;
            driver.keepOsd.start();
        } else if (driver.panel === "notifications") {
            Notifs.showPopups();
        } else if (driver.panel !== "bar") {
            Panels.current = driver.panel;
        }
        if (driver.panel === "controlcenter")
            driver.controlCenter(root);
        if (driver.panel === "clock" && driver.state === "settings") {
            const clock = driver.find(root, it => it.page !== undefined && typeof it.land === "function");
            if (clock)
                clock.page = "settings";
            else
                console.warn("shot: no clock popup found");
        }
        if (driver.panel === "bar" && driver.state === "traymenu") {
            const tray = driver.find(root, it => typeof it.openMenu === "function");
            if (tray)
                tray.openMenu(1);
            else
                console.warn("shot: no tray found");
        }
        driver.capture.start();
    }

    function controlCenter(root: Item): void {
        const cc = driver.find(root, it => it.tab !== undefined && it.keyboardTab !== undefined);
        if (!cc) {
            console.warn("shot: no control center found");
            return;
        }
        if (driver.state === "bluetooth" || driver.state === "bluetooth-code")
            cc.tab = 1;
        if (driver.state === "bluetooth-code") {
            Bt.prompt = {
                id: 1,
                kind: "passkey",
                name: "MX Master 3S",
                passkey: "482913"
            };
        }
        if (driver.state === "keyboard")
            cc.tab = cc.keyboardTab;
        if (driver.state === "")
            scroll.start();
    }

    // The notification history, scrolled to its middle once the panel has opened.
    property Timer scroll: Timer {
        interval: 700
        onTriggered: {
            const history = driver.find(driver.window.contentItem, it => it.maxListHeight !== undefined);
            const list = history ? driver.find(history, it => it.contentY !== undefined && it.count !== undefined) : null;
            if (!list) {
                console.warn("shot: no notification history found");
                return;
            }
            list.contentY = Math.max(0, (list.contentHeight - list.height) / 2);
        }
    }

    Component.onCompleted: {
        K.Tokens.mode = driver.args[0] === "light" ? "light" : "dark";
        Hypr.focusedScreen = driver.screen;
        Panels.screen = driver.screen;
        if (driver.panel === "clock" && driver.state === "answer")
            Time.pendingSearch = "days until vacation";
        if (driver.panel === "sysmon" && driver.state === "process")
            Sysmon.pendingPid = 4211;
        if (driver.panel === "wallpaper" && driver.state === "settings")
            Wallpapers.openSettings = true;
        // A scan that has found nothing yet, its skeleton rows with the room to show: emptied
        // before the frame exists, so no row is made and torn down again.
        if (driver.panel === "controlcenter" && driver.state === "scanning")
            Net.networks = [];
        // The history cleared, so the tab has the room to show the passkey prompt and its keycaps
        // whole, or a scan's rows; cleared before the frame exists, so no card is made and torn
        // down again.
        if (driver.panel === "controlcenter" && (driver.state === "bluetooth-code" || driver.state === "scanning"))
            Notifs.clear();
        driver.frame = driver.frameComponent.createObject(null);
        driver.window = driver.windowOf(driver.frame);
        if (!driver.window) {
            console.error("shot: the frame has no window");
            Qt.exit(1);
            return;
        }
        driver.wallpaper.createObject(driver.window.contentItem);
        driver.capture.window = driver.window;
        driver.open.start();
    }
}
