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
// notifications (the popups, the peek or the panel), or bar (nothing open). STATE puts the panel
// in a state, through the services and public properties only, so the driver works with any
// version of the panel:
//   controlcenter: "" and wifi (the Wi-Fi tab),
//                  wifi-form (a secured network's form, a password typed), wifi-error (the
//                  same, the password turned down), wifi-leaving (a network gone out of range
//                  held half-way out), wifi-scan (a scan's update: a network gone, one new and two
//                  swapped, its frame times logged), bluetooth (the Bluetooth tab),
//                  bluetooth-unnamed (scrolled to its end, the line over the closed drawer of
//                  unnamed devices), bluetooth-unnamed-open (the drawer opened, scrolled to its
//                  end), bluetooth-scan (a
//                  scan that finds six more devices, nearer than the ones listed), bluetooth-form
//                  (a nearby device's form), bluetooth-code (its pairing code to type on it),
//                  keyboard, keyboard-tooltip (the Mac accents explained), keyboard-drag (the second layout dragged onto the first),
//                  keyboard-add ("germ" searched for a layout to add), each with the history
//                  cleared, scanning (a Wi-Fi scan that has found nothing yet, the history cleared),
//                  proxy (the Proxy tile's detail open, the local network let through),
//                  proxy-lan-off (the same, the local network blocked)
//   clock: answer (a question typed into the search), settings
//   sysmon: "" (a disk close to full: Breaking a sweat), fine (every reading fine), critical (the
//           CPU over its limit), loading (the processes still counted), bare (no GPU, battery
//           or fan), scrolled (scrolled down to the processes, as a short screen shows them),
//           process (a process's details, scrolled down to them), kill (its Kill asking first)
//   wallpaper: settings
//   osd: volume, brightness
//   notifications: "" (two popups, folded, one with actions and one without), unfolded (the
//                  one without actions unfolded), held (the stack held as the pointer on it holds
//                  it, a third notification waiting), over (the panel opened over the popups),
//                  fan (a burst from one app, its group's preview showing),
//                  peek (the bell's peek, every one unread, so "+N more"), panel (the
//                  notifications panel, a group of four and a pair at rest), panel-unfolded (the
//                  build, with actions, unfolded), panel-fan (the group of four's preview, "+1
//                  more" under it), panel-expanded (the group
//                  open, its second card selected), panel-keys (the keys button's tooltip),
//                  panel-empty (nothing in it), panel-read (every one read), panel-dnd (Do
//                  not disturb on while the screen is shared, its tooltip showing)
//   bar: traymenu (the network item's menu open), dnd (the bell crossed out), nobell (no
//        notification, so no bell), dnd-empty (no notification, the bell crossed out all the same),
//        preview (the home a live preview of a branch)
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
        } else if (driver.panel === "notifications" && driver.state.startsWith("panel")) {
            Panels.current = "notifications";
            if (["panel-expanded", "panel-fan", "panel-keys", "panel-dnd", "panel-unfolded"].includes(driver.state))
                notificationsPose.start();
        } else if (driver.panel === "notifications" && driver.state === "peek") {
            const popups = driver.find(root, it => it.peeking !== undefined && it.hasPeek !== undefined);
            if (popups)
                popups.peeking = true;
            else
                console.warn("shot: no peek found");
        } else if (driver.panel === "notifications" && driver.state === "fan") {
            Notifs.showBurst();
            fanPose.start();
        } else if (driver.panel === "notifications") {
            Notifs.showPopups();
            if (driver.state !== "")
                popupsPose.start();
        } else if (driver.panel !== "bar") {
            Panels.current = driver.panel;
        }
        if (driver.panel === "controlcenter" && driver.state === "wifi-scan")
            driver.measure("panel opening");
        if (driver.panel === "controlcenter")
            driver.controlCenter(root);
        if (driver.panel === "sysmon" && ["scrolled", "process", "kill"].includes(driver.state))
            sysmonPose.start();
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
        if (driver.state.startsWith("bluetooth"))
            cc.tab = 1;
        if (driver.state.startsWith("wifi-") || driver.state.startsWith("bluetooth-"))
            radioPose.start();
        if (driver.state.startsWith("keyboard")) {
            cc.tab = cc.keyboardTab;
            keyboardPose.start();
        }
        if (driver.state.startsWith("proxy")) {
            const toggles = driver.find(root, it => it.proxyDetail !== undefined);
            if (toggles)
                toggles.proxyDetail = true;
            else
                console.warn("shot: no Proxy tile found");
        }
    }

    // The system monitor's sample for the state, changed from the stand-in's before the frame
    // exists, so nothing is drawn twice.
    function sysmonSample(): void {
        const s = JSON.parse(JSON.stringify(Sysmon.sample));
        if (driver.state === "fine")
            s.disks[1] = Object.assign(s.disks[1], {
                free: 256055095296,
                percent: 50
            });
        if (driver.state === "critical")
            s.cpu.temp = 94;
        // A light sample, as the pill's pace leaves the panel with when it opens.
        if (driver.state === "loading") {
            s.detail = false;
            s.cpu.cores = [];
            s.cpu.mhz = null;
            s.fans = [];
            s.io = null;
            s.net.address = null;
            s.top = null;
        }
        if (driver.state === "bare") {
            s.gpus = [];
            s.battery = null;
            s.fans = [];
        }
        Sysmon.sample = s;
        if (driver.state === "process" || driver.state === "kill")
            Sysmon.pendingPid = 4211;
    }

    // The system monitor scrolled to its end once it has opened, through its `K.Scroll`, and for
    // `kill` the shown process's Kill asking first. It stays at the end as the body lengthens: a
    // process's details arrive from the stand-in engine after the panel opens.
    property Timer sysmonPose: Timer {
        interval: 700
        onTriggered: {
            const panel = driver.find(driver.window.contentItem, it => it.lastDetailed !== undefined);
            const processes = panel ? driver.find(panel, it => it.confirming !== undefined) : null;
            if (driver.state === "kill" && processes)
                processes.confirming = "kill";
            const scroll = panel ? driver.find(panel, it => it.travel !== undefined && it.contentY !== undefined) : null;
            if (!scroll) {
                console.warn("shot: no system monitor scroll found");
                return;
            }
            scroll.contentY = scroll.travel;
            scroll.travelChanged.connect(() => scroll.contentY = scroll.travel);
        }
    }

    // The panel's stack opened once the panel has, the keyboard on its second card, through what
    // NotificationPanel.qml exposes.
    property Timer notificationsPose: Timer {
        interval: 700
        onTriggered: {
            const panel = driver.find(driver.window.contentItem, it => it.expanded !== undefined && typeof it.setExpanded === "function");
            if (!panel) {
                console.warn("shot: no notifications panel found");
                return;
            }
            if (driver.state === "panel-unfolded") {
                panel.unfolded = Notifs.build;
                return;
            }
            if (driver.state === "panel-keys" || driver.state === "panel-dnd") {
                const title = driver.state === "panel-keys" ? "Keys" : "Do not disturb is on";
                const tooltip = driver.find(driver.window.contentItem, it => it.title === title && it.shown !== undefined && it.within !== undefined);
                if (tooltip)
                    tooltip.shown = true;
                else
                    console.warn(`shot: no tooltip "${title}" found`);
                return;
            }
            if (driver.state === "panel-fan") {
                driver.fanFirstGroup(panel);
                return;
            }
            driver.settleGroups();
            const stack = panel.rows.find(row => row.stacked);
            panel.setExpanded(stack.key, true);
            const list = driver.find(panel, it => typeof it.jump === "function");
            list.currentIndex = panel.rows.findIndex(row => row.key === stack.key);
            panel.inner = 1;
        }
    }

    // The popups posed once they have come in, through what ToastList.qml exposes: a card
    // unfolded, the stack held as the pointer holds it (a stand-in has no pointer) while one more
    // arrives, or the panel opened over them.
    property Timer popupsPose: Timer {
        interval: 700
        onTriggered: {
            const list = driver.find(driver.window.contentItem, it => it.hold !== undefined && it.unfolded !== undefined);
            if (!list) {
                console.warn("shot: no popups found");
                return;
            }
            if (driver.state === "unfolded")
                list.unfolded = Notifs.battery;
            if (driver.state === "held") {
                list.hold.pointed = true;
                Notifs.arrive();
            }
            if (driver.state === "over")
                Panels.current = "notifications";
        }
    }

    // A notification group's motion is over at once in a shot, so the capture can't catch it
    // part-way on a busy machine: the group's end state is the picture.
    function settleGroups(): void {
        K.Motion.reduced = true;
    }

    // The first notification group's preview, as if the pointer had rested on it: the shots have
    // no pointer.
    function fanFirstGroup(within: Item): void {
        driver.settleGroups();
        const group = driver.find(within, it => it.fanCount !== undefined && it.stacked === true);
        if (group)
            group.armed = true;
        else
            console.warn("shot: no notification group found");
    }

    // The popups' group fanned once the popups have slid in, and held as the pointer resting on it
    // holds them: their clocks stopped, so their lines don't keep the window drawing.
    property Timer fanPose: Timer {
        interval: 700
        onTriggered: {
            const list = driver.find(driver.window.contentItem, it => it.hold !== undefined && it.unfolded !== undefined);
            if (list)
                list.hold.pointed = true;
            fanLater.start();
        }
    }

    // Holding the popups rebuilds the stand-in model's rows, the groups with them, so the group
    // fans once they have settled.
    property Timer fanLater: Timer {
        interval: 300
        onTriggered: driver.fanFirstGroup(driver.find(driver.window.contentItem, it => it.peeking !== undefined && it.hasPeek !== undefined))
    }

    // A radio tab's form or a row leaving, once the panel has opened, through what WifiList.qml
    // and BluetoothList.qml expose.
    property Timer radioPose: Timer {
        interval: 900
        onTriggered: {
            const content = driver.window.contentItem;
            const wifi = driver.find(content, it => it.trying !== undefined && it.pending !== undefined);
            const bluetooth = driver.find(content, it => it.prompt !== undefined && it.pending !== undefined && it.trying === undefined);
            if (!wifi || !bluetooth) {
                console.warn("shot: no Wi-Fi or Bluetooth tab found");
                return;
            }
            const jbl = Bt.devices.find(device => device.name === "JBL Flip 6");
            if (driver.state === "wifi-form" || driver.state === "wifi-error") {
                wifi.open("Omoide Yokocho", driver.state === "wifi-error" ? "Wrong password" : "");
                typing.start();
            }
            if (driver.state === "wifi-scan") {
                const by = name => Net.networks.find(network => network.name === name);
                by("Kabukicho Free Wi-Fi").signalStrength = 0.9;
                Net.networks = [...Net.networks.filter(network => network.name !== "7SPOT"), Net.network("Tsutaya Wi-Fi", 0.6, false, false, true)];
                driver.measure("scan update");
            }
            if (driver.state === "wifi-leaving") {
                Net.networks = Net.networks.filter(network => network.name !== "7SPOT");
                holdLeave.start();
            }
            if (driver.state === "bluetooth-unnamed")
                bluetooth.list.positionViewAtEnd();
            if (driver.state === "bluetooth-unnamed-open") {
                Bt.setShowUnnamed(true);
                drawerPose.list = bluetooth.list;
                drawerPose.start();
            }
            if (driver.state === "bluetooth-scan") {
                const found = ["Bose QC45", "AirPods Pro", "Galaxy Buds2", "Sonos Roam", "Kindle", "Apple Watch"].map(name => Bt.device(name, "audio-headphones", false, false, -1));
                const rssi = Object.assign({}, Bt.rssi);
                found.forEach((device, n) => rssi[device.dbusPath] = -40 - n);
                Bt.rssi = rssi;
                Bt.devices = [...Bt.devices, ...found];
            }
            if (driver.state === "bluetooth-form")
                bluetooth.open(jbl);
            if (driver.state === "bluetooth-code") {
                Bt.pairing = [jbl.dbusPath];
                Bt.prompt = {
                    id: 1,
                    kind: "display-passkey",
                    device: jbl.dbusPath,
                    name: jbl.name,
                    code: "482913",
                    entered: null
                };
            }
        }
    }

    // The frame times while something animates, logged as `what`: each frame's distance from the
    // last, the first one's included. The scan pose measures the panel opening too, to set the
    // update's against.
    property var frames: []
    property string measuring: ""

    function measure(what: string): void {
        driver.measuring = what;
        driver.frames = [Date.now()];
        frameLog.restart();
    }

    property Connections frameTimes: Connections {
        target: driver.frames.length > 0 && frameLog.running ? driver.window : null

        function onFrameSwapped(): void {
            driver.frames.push(Date.now());
        }
    }
    property Timer frameLog: Timer {
        interval: K.Tokens.duration.slow
        onTriggered: {
            const gaps = driver.frames.slice(1).map((t, i) => t - driver.frames[i]);
            const mean = gaps.reduce((a, b) => a + b, 0) / Math.max(1, gaps.length);
            console.log(`shot: ${driver.measuring}, ${gaps.length} frames, mean ${mean.toFixed(1)} ms, longest ${Math.max(0, ...gaps)} ms`);
        }
    }

    // The leaving row held half-way out, watched for as the view removes it on its next frame,
    // which llvmpipe can take a while to draw.
    property int leaveWatch: 0
    // The open drawer scrolled into view once it has opened, as the person would scroll to it.
    property Timer drawerPose: Timer {
        property var list: null

        interval: 600
        onTriggered: list.positionViewAtEnd()
    }

    property Timer holdLeave: Timer {
        interval: 16
        repeat: true
        onTriggered: {
            const row = driver.find(driver.window.contentItem, it => it.leaving === true);
            if (row) {
                // Held where the leave is half done, whenever the watch first saw it.
                row.leaveAnimation.pause();
                row.grow = 0.5;
                row.opacity = 0.5;
                stop();
            } else if (++driver.leaveWatch > 200) {
                console.warn("shot: no leaving row found");
                stop();
            }
        }
    }

    // The Keyboard tab in its state once it has read its layouts, through what KeyboardList.qml
    // exposes.
    property Timer keyboardPose: Timer {
        interval: 300
        onTriggered: {
            const tab = driver.find(driver.window.contentItem, it => it.dragFrom !== undefined && it.explaining !== undefined);
            if (!tab) {
                console.warn("shot: no keyboard tab found");
                return;
            }
            if (driver.state === "keyboard-tooltip")
                tab.explaining = true;
            if (driver.state === "keyboard-drag") {
                tab.dragFrom = 1;
                tab.dropAt = 0;
                tab.dragOffset = -K.Tokens.space.s32;
            }
            if (driver.state === "keyboard-add") {
                tab.adding = true;
                typing.start();
            }
        }
    }

    // A query typed into the add view's search once it has entered the layouts app, or a password
    // into the Wi-Fi form, its caret stopped: llvmpipe takes about 400 ms to draw a frame of the
    // add view at 1.5×, so a blinking caret leaves the window no quiet 300 ms to be shot in.
    property Timer typing: Timer {
        interval: 300
        onTriggered: {
            const wifi = driver.state.startsWith("wifi");
            const placeholder = wifi ? "Password" : "Search languages and layouts";
            const field = driver.find(driver.window.contentItem, it => it.placeholder === placeholder && it.cursorPosition !== undefined && it.visible);
            if (!field) {
                console.warn(`shot: no field "${placeholder}" found`);
                return;
            }
            field.text = wifi ? "hunter22" : "germ";
            const input = driver.find(field, it => it.cursorVisible !== undefined);
            if (input)
                input.cursorVisible = false;
        }
    }

    Component.onCompleted: {
        K.Tokens.mode = driver.args[0] === "light" ? "light" : "dark";
        Hypr.focusedScreen = driver.screen;
        Panels.screen = driver.screen;
        if (driver.panel === "clock" && driver.state === "answer")
            Time.pendingSearch = "days until vacation";
        if (driver.panel === "sysmon")
            driver.sysmonSample();
        if (driver.panel === "wallpaper" && driver.state === "settings")
            Wallpapers.openSettings = true;
        // A scan that has found nothing yet, its skeleton rows with the room to show: emptied
        // before the frame exists, so no row is made and torn down again.
        if (driver.panel === "controlcenter" && driver.state === "scanning")
            Net.networks = [];
        if (driver.panel === "controlcenter" && driver.state === "proxy-lan-off")
            Proxy.allowLocalNetwork = false;
        // Every one unread, so the peek has more than its three cards.
        if (driver.panel === "notifications" && driver.state === "peek")
            Notifs.unread = Notifs.history.slice();
        // Emptied before the frame exists, so no card is made and torn down again.
        if ((driver.panel === "notifications" && driver.state === "panel-empty") || (driver.panel === "bar" && (driver.state === "nobell" || driver.state === "dnd-empty")))
            Notifs.clear();
        if (driver.panel === "notifications" && driver.state === "panel-read")
            Notifs.unread = [];
        if ((driver.panel === "notifications" && driver.state === "panel-dnd") || (driver.panel === "bar" && (driver.state === "dnd" || driver.state === "dnd-empty")))
            Notifs.dndReason = "sharing";
        if (driver.panel === "bar" && driver.state === "preview") {
            Preview.branch = "feature/issue-228-live-preview";
            Preview.commit = "1a2b3c4";
            Preview.since = new Date(2026, 9, 27, 20, 15).getTime();
            Preview.active = true;
        }
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
