pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Bluetooth
import Quickshell.Io
import "radios.js" as Radios

// BlueZ adapter and devices through Quickshell.Bluetooth. Pairing and scanning go through the
// pairing agent (`kubaos-bluetooth`, crates/kubaos-bluetooth), which bluetoothd asks for every
// code a pairing needs: its state file is watched, its prompt answered with `kubaos-bluetooth
// answer`. Quickshell's own pair() can't end a pairing that hangs, and its discovery can't be
// restarted once bluetoothd suspends it, so neither is used.
Singleton {
    id: root

    readonly property BluetoothAdapter adapter: Bluetooth.defaultAdapter
    readonly property bool available: adapter !== null
    readonly property bool enabled: adapter?.enabled ?? false
    // Soft- or hard-blocked by rfkill: it can't be powered on until it's unblocked.
    readonly property bool blocked: adapter?.state === BluetoothAdapterState.Blocked
    readonly property bool scanning: adapter?.discovering ?? false
    // The agent's state file: {agent, prompt, pairing, errors}, empty while the agent is down.
    readonly property string stateFile: `${Quickshell.env("XDG_RUNTIME_DIR")}/kubaos-bluetooth.json`
    property var agentState: ({})
    readonly property bool agentRunning: agentState.agent === true
    // What the agent asks or shows: {id, kind, device, name, code, entered}, or null.
    readonly property var prompt: agentState.prompt ?? null
    // A new prompt opens the control center on this tab; the same one updated (digits typed) doesn't.
    readonly property int promptId: prompt?.id ?? 0
    readonly property string blockedText: "Blocked by rfkill (airplane mode). Unblock it: rfkill unblock bluetooth"
    // Why the last scan, pairing or power-on didn't happen, until the next attempt.
    property string problem: ""
    // How long powering on may take before the tab says it didn't.
    readonly property int powerOnWait: 4000
    readonly property var devices: Bluetooth.devices.values
    // How strongly each device was last heard while scanning, in dBm, by object path.
    readonly property var rssi: agentState.rssi ?? ({})
    // Paired devices, the connected first, named or not; then the ones in range to pair, nearest
    // first, those that send a name in `nearby` and those that don't in `unnamed`, which the tab
    // keeps in a drawer that is closed unless `showUnnamed` (radios.js). BlueZ forgets an unpaired
    // device a while after a scan stops hearing it.
    readonly property var known: Radios.btKnown(devices)
    readonly property var inRange: Radios.btSplit(Radios.btNearby(devices, rssi))
    readonly property var nearby: inRange.named
    readonly property var unnamed: inRange.unnamed
    readonly property bool showUnnamed: ShellState.bluetoothUnnamed
    readonly property int connectedCount: devices.filter(d => d.connected).length
    readonly property string icon: !enabled ? "bluetooth_disabled" : connectedCount > 0 ? "bluetooth_connected" : "bluetooth"

    // The control center goes to its Bluetooth tab.
    signal reveal

    function toggle(): void {
        if (adapter)
            adapter.enabled = !adapter.enabled;
    }

    function powerOn(): void {
        if (!adapter || blocked)
            return;
        problem = "";
        adapter.enabled = true;
        powerCheck.restart();
    }

    // Stopping always goes to the agent too: a discovery bluetoothd suspended for a pairing
    // reads as off, and would resume on its own once the pairing ends.
    function setScanning(on: bool): void {
        if (!adapter)
            return;
        if (on && !enabled) {
            problem = blocked ? root.blockedText : "Bluetooth is off";
            return;
        }
        problem = "";
        run(["scan", on ? "on" : "off", adapter.dbusPath], on);
    }

    // Remembered across restarts, for the Nearby line's Show and Hide and the Command menu's pair.
    function setShowUnnamed(on: bool): void {
        ShellState.bluetoothUnnamed = on;
    }

    // The control center on the Bluetooth tab, which scans while it shows.
    function show(): void {
        reveal();
        Panels.open("controlcenter");
    }

    function isPairing(device: BluetoothDevice): bool {
        return (agentState.pairing ?? []).includes(device.dbusPath);
    }

    function pairError(device: BluetoothDevice): string {
        return agentState.errors?.[device.dbusPath] ?? "";
    }

    // Enter on a device: cancel its pairing, disconnect, connect, or pair.
    function activate(device: BluetoothDevice): void {
        problem = "";
        if (isPairing(device))
            run(["cancel", device.dbusPath], true);
        else if (device.connected)
            device.disconnect();
        else if (device.paired)
            device.connect();
        else
            run(["pair", device.dbusPath], true);
    }

    // Pair or Cancel on the prompt, with the PIN or passkey typed.
    function answer(accept: bool, value: string): void {
        if (prompt)
            run(["answer", String(prompt.id), accept ? "yes" : "no", value], true);
    }

    // `kubaos-bluetooth ARGS`; its stderr becomes the problem when it fails and `report`.
    function run(args: var, report: bool): void {
        call.createObject(root, {
            command: ["kubaos-bluetooth", ...args],
            report: report
        });
    }

    onPromptIdChanged: if (promptId)
        show()

    Component {
        id: call

        Process {
            id: proc

            property bool report: true
            property int code: -1
            property bool streamed: false

            function finish(): void {
                if (code < 0 || !streamed)
                    return;
                if (code !== 0 && report)
                    root.problem = errors.text.trim() || "The pairing agent didn't answer";
                proc.destroy();
            }

            running: true
            stderr: StdioCollector {
                id: errors

                onStreamFinished: {
                    proc.streamed = true;
                    proc.finish();
                }
            }
            onExited: exitCode => {
                proc.code = exitCode;
                proc.finish();
            }
        }
    }

    Timer {
        id: powerCheck

        interval: root.powerOnWait
        onTriggered: if (root.available && !root.enabled)
            root.problem = root.blocked ? root.blockedText : "Bluetooth didn't turn on. Restart it: sudo systemctl restart bluetooth"
    }

    FileView {
        path: root.stateFile
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: {
            try {
                root.agentState = JSON.parse(text());
            } catch (e) {
                root.agentState = {};
            }
        }
        onLoadFailed: root.agentState = {}
    }
}
