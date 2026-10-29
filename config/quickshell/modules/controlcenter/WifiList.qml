import QtQuick
import Quickshell.Networking
import qs.services

// The Wi-Fi tab: saved networks in range (Known), the one in use first and checked, then the rest
// in range (Nearby) by signal, each on its strength's tile, a lock before a secured one's line.
// A known network connects at once; a nearby one opens the connection form, which asks a secured
// network's password and keeps the form open with the error when it is wrong. A saved password
// that stops working opens the form too.
//
// The form and a pending connect hold a network by name, not the network itself: NetworkManager
// drops a network that goes out of range, and the form then says so instead of calling into it.
RadioTab {
    id: root

    // The name of the network the form is for, or empty while the list shows.
    property string pendingName: ""
    readonly property var pending: root.pendingName === "" ? null : root.find(root.pendingName)
    readonly property bool secured: root.pending !== null && Net.isSecured(root.pending)
    // "connecting" while the form waits on NetworkManager, "failed" once it said no.
    property string phase: ""
    property string failure: ""
    // The name of the known network last pressed, whose saved password may turn out wrong.
    property string tryingName: ""
    readonly property var trying: root.tryingName === "" ? null : root.find(root.tryingName)

    function line(kind: string, key: string, title: string): var {
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
            failed: false,
            folded: false
        };
    }

    function networkRow(network): var {
        return {
            key: network.name,
            kind: "row",
            title: network.name,
            subtitle: root.statusText(network),
            subtitleIcon: Net.isSecured(network) ? "lock" : "",
            icon: Net.strengthIcon(network.signalStrength),
            accent: network.connected,
            checked: network.connected,
            forgettable: network.known && !network.connected,
            failed: false,
            folded: false
        };
    }

    function statusText(network): string {
        if (network.connected)
            return Net.band === "" ? "Connected" : `Connected · ${Net.band}`;
        if (network.stateChanging)
            return `${ConnectionState.toString(network.state)}…`;
        if (network.known)
            return "Saved";
        return Net.isSecured(network) ? "Secured" : "Open";
    }

    function find(name: string): var {
        return Net.networks.find(network => network.name === name) ?? null;
    }

    function open(name: string, failure: string): void {
        root.phase = failure === "" ? "" : "failed";
        root.failure = failure;
        root.pendingName = name;
    }

    function close(): void {
        root.pendingName = "";
        root.phase = "";
    }

    function failureText(reason: int): string {
        switch (reason) {
        case ConnectionFailReason.NoSecrets:
            return "Wrong password";
        case ConnectionFailReason.WifiAuthTimeout:
            return "Wrong password, or the network didn't answer";
        case ConnectionFailReason.WifiNetworkLost:
            return "The network went out of range";
        }
        return "Couldn't connect";
    }

    function formStatus(): string {
        if (root.pending === null)
            return "Out of range";
        if (root.phase === "connecting")
            return "Connecting…";
        return root.phase === "failed" ? root.failure : "";
    }

    // A WPA password is 8 characters or more, so a shorter one isn't sent; WEP keys are shorter.
    function longEnough(password: string): bool {
        return !root.secured || password.length >= 8 || Net.securityText(root.pending) === "WEP";
    }

    rows: {
        const out = [];
        if (Net.known.length > 0) {
            out.push(root.line("label", "label:known", "Known"));
            for (const network of Net.known)
                out.push(root.networkRow(network));
        }
        out.push(root.line("label", "label:nearby", "Nearby"));
        if (Net.nearby.length > 0) {
            for (const network of Net.nearby)
                out.push(root.networkRow(network));
        } else if (Net.networks.length === 0 && Net.scanning) {
            // A scan that hasn't found anything yet: the rows it finds take these places.
            for (const n of [0, 1, 2])
                out.push(root.line("skeleton", `skeleton:${n}`, ""));
        } else {
            out.push(root.line("note", "note:nearby", Net.networks.length === 0 ? "No networks in range" : "No other networks in range"));
        }
        return out;
    }
    formOpen: root.pendingName !== ""
    off: !Net.wifiDevice || !Net.wifiEnabled
    offIcon: "wifi_off"
    offTitle: Net.wifiDevice ? "Wi-Fi is off" : "No Wi-Fi adapter"
    offBody: Net.wifiDevice ? "The Wi-Fi tile turns it on." : "Nothing to connect with on this machine."

    form.icon: root.pending ? Net.strengthIcon(root.pending.signalStrength) : "signal_wifi_0_bar"
    form.title: root.pendingName
    form.subtitle: root.secured ? `Secured · ${Net.securityText(root.pending)}` : root.pending ? "Open network" : ""
    form.subtitleIcon: root.secured ? "lock" : ""
    form.ask: root.secured ? "field" : ""
    form.placeholder: "Password"
    form.secret: true
    form.status: root.formStatus()
    form.failed: root.pending === null || root.phase === "failed"
    form.action: "Connect"
    form.ready: root.pending !== null && root.phase !== "connecting" && root.longEnough(root.form.value)

    onPicked: key => {
        const network = root.find(key);
        if (!network || network.connected)
            return;
        if (network.known) {
            root.tryingName = network.name;
            network.connect();
        } else {
            root.open(network.name, "");
        }
    }
    onForgot: key => root.find(key)?.forget()

    Connections {
        target: root.form

        function onSubmitted(value: string): void {
            root.phase = "connecting";
            if (root.secured)
                root.pending.connectWithPsk(value);
            else
                root.pending.connect();
        }

        function onCancelled(): void {
            root.close();
        }
    }

    Connections {
        target: root.pending
        ignoreUnknownSignals: true

        function onConnectedChanged(): void {
            if (root.pending.connected)
                root.close();
        }

        function onConnectionFailed(reason: int): void {
            root.phase = "failed";
            root.failure = root.failureText(reason);
        }
    }

    Connections {
        target: root.trying
        ignoreUnknownSignals: true

        function onConnectedChanged(): void {
            if (root.trying.connected)
                root.tryingName = "";
        }

        function onConnectionFailed(reason: int): void {
            const name = root.tryingName;
            root.tryingName = "";
            if (reason === ConnectionFailReason.NoSecrets)
                root.open(name, "The saved password didn't work");
        }
    }
}
