import QtQuick
import Quickshell.Bluetooth
import KubaOS.Components as K
import qs.services
import "../../services/radios.js" as Radios

// The Bluetooth tab: paired devices (Known), the connected first and checked with their battery,
// then the devices in range to pair (Nearby), nearest first by their last signal. A known device
// connects or disconnects on a press; a nearby one opens the connection form, which pairs it and
// shows what the pairing agent asks: a code to type on the device or to check, or a PIN or
// passkey to type here. A pairing the device starts opens the form too.
RadioTab {
    id: root

    // The device the form is for, or null while the list shows.
    property var pending: null
    readonly property var prompt: root.pending && Bt.prompt?.device === root.pending.dbusPath ? Bt.prompt : null
    readonly property string kind: root.prompt?.kind ?? ""
    readonly property bool pairing: root.pending !== null && Bt.isPairing(root.pending)
    readonly property string failure: root.pending ? (Bt.pairError(root.pending) || Bt.problem) : ""

    function line(kind: string, key: string, title: string, failed: bool): var {
        return {
            key: key,
            kind: kind,
            title: title,
            subtitle: "",
            subtitleIcon: "",
            icon: "",
            accent: false,
            checked: false,
            forgettable: false,
            failed: failed
        };
    }

    function deviceRow(device): var {
        const battery = device.connected && device.batteryAvailable;
        return {
            key: device.dbusPath,
            kind: "row",
            title: device.name,
            subtitle: root.statusText(device),
            subtitleIcon: battery ? Radios.batterySymbol(device.battery) : "",
            icon: Radios.deviceSymbol(device.icon),
            accent: device.connected,
            checked: device.connected,
            forgettable: device.paired && !device.connected,
            failed: false
        };
    }

    function statusText(device): string {
        if (Bt.isPairing(device))
            return "Pairing…";
        if (device.state === BluetoothDeviceState.Connecting || device.state === BluetoothDeviceState.Disconnecting)
            return `${BluetoothDeviceState.toString(device.state)}…`;
        if (device.connected)
            return device.batteryAvailable ? `Connected · ${Math.round(device.battery * 100)}%` : "Connected";
        if (device.paired)
            return "Not connected";
        return Radios.distance(Bt.rssi[device.dbusPath]) || "Not paired";
    }

    function find(path: string): var {
        return Bt.devices.find(device => device.dbusPath === path) ?? null;
    }

    // A six-digit passkey reads as two groups of three.
    function grouped(code: string): string {
        return code.length === 6 ? `${code.slice(0, 3)} ${code.slice(3)}` : code;
    }

    function formLine(): string {
        switch (root.kind) {
        case "display-passkey":
        case "display-pin-code":
            return "Type this code on it, then press its Enter";
        case "confirm":
            return "Check that it shows the same code";
        case "authorize":
            return "It asks to pair";
        case "passkey":
            return "Type the passkey it shows";
        case "pin-code":
            return "Type its PIN";
        }
        const distance = root.pending ? Radios.distance(Bt.rssi[root.pending.dbusPath]) : "";
        return distance === "" ? "Not paired" : `Not paired · ${distance}`;
    }

    function formStatus(): string {
        if (root.failure !== "")
            return root.failure;
        if ((root.prompt?.entered ?? null) !== null)
            return `${root.prompt.entered} of ${root.prompt.code?.length ?? 6} typed`;
        return root.pairing && root.prompt === null ? "Pairing…" : "";
    }

    function close(): void {
        root.pending = null;
    }

    rows: {
        const out = [];
        if (Bt.known.length > 0) {
            out.push(root.line("label", "label:known", "Known", false));
            for (const device of Bt.known)
                out.push(root.deviceRow(device));
        }
        out.push(root.line("label", "label:nearby", "Nearby", false));
        if (Bt.nearby.length > 0) {
            for (const device of Bt.nearby)
                out.push(root.deviceRow(device));
        } else if (Bt.scanning) {
            // A scan that hasn't found anything yet: the devices it finds take these places.
            for (const n of [0, 1])
                out.push(root.line("skeleton", `skeleton:${n}`, "", false));
        } else {
            out.push(root.line("note", "note:nearby", Bt.problem || "Nothing in pairing mode nearby", Bt.problem !== ""));
        }
        return out;
    }
    formOpen: root.pending !== null
    off: !Bt.available || !Bt.enabled
    offIcon: "bluetooth_disabled"
    offTitle: !Bt.available ? "No Bluetooth adapter" : Bt.blocked ? "Bluetooth is blocked" : "Bluetooth is off"
    offBody: !Bt.available ? "Nothing to pair with on this machine." : Bt.blocked ? Bt.blockedText : Bt.problem || "Turn it on to find devices."
    offAction: Bt.available && !Bt.blocked ? turnOn : null

    form.icon: root.pending ? Radios.deviceSymbol(root.pending.icon) : ""
    form.title: root.pending?.name ?? ""
    form.subtitle: root.formLine()
    form.ask: root.kind === "passkey" || root.kind === "pin-code" ? "field" : (root.prompt?.code ?? "") !== "" ? "code" : ""
    form.placeholder: root.kind === "passkey" ? "Six digits" : "The PIN"
    form.code: root.grouped(root.prompt?.code ?? "")
    form.status: root.formStatus()
    form.failed: root.failure !== ""
    form.action: "Pair"
    // Pair starts the pairing, or says yes to what the agent asks; a code shown to type on the
    // device waits on the device.
    form.ready: root.prompt ? !root.kind.startsWith("display") : !root.pairing

    // A prompt that asks for a PIN or passkey puts the keyboard in its field.
    onKindChanged: if (root.formOpen)
        root.form.focusFirst()
    onPicked: key => {
        const device = root.find(key);
        if (!device)
            return;
        if (device.paired)
            Bt.activate(device);
        else
            root.pending = device;
    }
    onForgot: key => root.find(key)?.forget()

    Component {
        id: turnOn

        K.Button {
            text: "Turn on"
            kind: "Tonal"
            onActivated: Bt.powerOn()
        }
    }

    // A pairing the device starts, or one from elsewhere, opens the form for its device.
    Connections {
        target: Bt

        function onPromptIdChanged(): void {
            const device = Bt.prompt ? root.find(Bt.prompt.device) : null;
            if (device)
                root.pending = device;
        }
    }

    Connections {
        target: root.pending
        ignoreUnknownSignals: true

        function onPairedChanged(): void {
            if (root.pending.paired)
                root.close();
        }
    }

    Connections {
        target: root.form

        function onSubmitted(value: string): void {
            if (root.prompt)
                Bt.answer(true, value);
            else
                Bt.activate(root.pending);
        }

        function onCancelled(): void {
            if (root.prompt)
                Bt.answer(false, "");
            else if (root.pairing)
                Bt.activate(root.pending);
            root.close();
        }
    }
}
